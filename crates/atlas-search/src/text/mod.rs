use std::collections::{BTreeMap, BTreeSet};

use atlas_index::FtsQuery;

use crate::fusion::{DEFAULT_FTS_FUSION_POLICY, FusionInput, fuse_ranked_hits};
use crate::query::analyze_text_query;
use crate::semantic::{SemanticSearchMode, SemanticSearchRequest};
use crate::{AtlasRetrievalService, SearchError};

mod request;
mod results;
mod sources;

use request::validate_search_text_request;
pub use request::{RetrievalMode, TextSearchRequest};
pub use results::{TextSearchMatch, TextSearchRecord, TextSearchResult};
use results::{TextSearchResultItem, candidate_keys, identity_records};
use sources::{
    load_records_by_key, load_search_candidate_records, query_fts_candidate_record_keys,
    query_precision_fts_index, resolve_identity_tier, resolve_text_filter, search_vector_lane,
};

pub trait TextRetrieval {
    fn search_text(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchResult, SearchError>;
}

impl TextRetrieval for AtlasRetrievalService {
    fn search_text(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchResult, SearchError> {
        validate_search_text_request(&request)?;
        let resolved_filter = resolve_text_filter(self.index.as_ref(), request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let query = analyze_text_query(request.query, request.exclude);
        let fts_query = FtsQuery::from_tokens(query.fts_tokens.clone());
        let exclude_query = FtsQuery::from_tokens(query.exclude_tokens.clone());
        let identity_matches = resolve_identity_tier(self, request.query, filter)?;
        let fts_hits = if request.retrieval.uses_fts() {
            match fts_query.as_ref() {
                Some(fts_query) => query_precision_fts_index(
                    self.index.as_ref(),
                    fts_query,
                    filter,
                    request.fts_top_k,
                )?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let vector_hits = if request.retrieval.uses_vector() {
            search_vector_lane(
                self,
                SemanticSearchRequest {
                    query: request.query,
                    filter,
                    limit: request.vector_top_k,
                    mode: SemanticSearchMode::WeightedChunks,
                },
            )?
        } else {
            Vec::new()
        };
        let excluded_keys = match exclude_query.as_ref() {
            Some(exclude_query) => query_fts_candidate_record_keys(
                self.index.as_ref(),
                exclude_query,
                &candidate_keys(&identity_matches, &fts_hits, &vector_hits),
            )
            .map_err(SearchError::from_filter)?
            .into_iter()
            .collect::<BTreeSet<_>>(),
            None => BTreeSet::new(),
        };
        let identity_matches = identity_matches
            .into_iter()
            .filter(|identity| !excluded_keys.contains(&identity.record.identity.key))
            .collect::<Vec<_>>();
        let identity_keys = identity_matches
            .iter()
            .map(|identity| identity.record.identity.key.clone())
            .collect::<BTreeSet<_>>();
        let fusion_candidate_keys = candidate_keys(&identity_matches, &fts_hits, &vector_hits)
            .into_iter()
            .filter(|key| !excluded_keys.contains(key))
            .collect::<Vec<_>>();
        let candidate_records =
            load_search_candidate_records(self.index.as_ref(), &fusion_candidate_keys)?;
        let candidates_by_key = candidate_records
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            candidates_by_key: &candidates_by_key,
            fts_tokens: &query.fts_tokens,
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: request.retrieval,
            fusion: request.fusion,
            fts_policy: DEFAULT_FTS_FUSION_POLICY,
            explain: request.explain,
            identity_count: identity_matches.len(),
        });
        let total = identity_matches.len() + fused.len();
        let identity_records =
            identity_records(identity_matches, request.retrieval, request.explain);
        let mut page_items = identity_records
            .into_iter()
            .map(|record| TextSearchResultItem::Identity(Box::new(record)))
            .chain(fused.into_iter().map(TextSearchResultItem::Ranked))
            .skip(request.offset as usize)
            .take(request.limit as usize)
            .collect::<Vec<_>>();
        let ranked_page_keys = page_items
            .iter()
            .filter_map(|item| match item {
                TextSearchResultItem::Identity(_) => None,
                TextSearchResultItem::Ranked(ranked) => Some(ranked.record_key.clone()),
            })
            .collect::<Vec<_>>();
        let ranked_page_records = load_records_by_key(self.index.as_ref(), &ranked_page_keys)?
            .into_iter()
            .map(|record| (record.identity.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut page_records = page_items
            .drain(..)
            .filter_map(|item| match item {
                TextSearchResultItem::Identity(record) => Some(*record),
                TextSearchResultItem::Ranked(ranked) => ranked_page_records
                    .get(&ranked.record_key)
                    .map(|record| TextSearchRecord {
                        record: record.clone(),
                        match_info: TextSearchMatch::Ranked {
                            retrieval: request.retrieval,
                            explain: ranked.explain,
                        },
                    }),
            })
            .collect::<Vec<_>>();
        self.enrich_text_record_reference_labels(&mut page_records)?;

        Ok(TextSearchResult {
            query,
            retrieval: request.retrieval,
            fusion: request.fusion,
            records: page_records,
            total: total as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::fusion::{FusionMethod, FusionOptions};
    use atlas_domain::{
        MetricDomain, PublicationCategory, RecordKey, RecordKind, SearchFilterNode,
    };
    use atlas_index::{
        FilterCompileError, FilterReadIndex, FilteredRecordKeyPage, FilteredRecordSort,
        FtsReadIndex, FtsSearchHit, FtsSearchLane, GraphReferenceEdge, IdentityReadIndex,
        IndexRemasterLinks, IndexVariantGroup, RecordEmbeddingVector, RecordIdentityMatch,
        RecordReadIndex, ReferenceEdgeDirection, ReferenceReadIndex, RemasterReadIndex,
        SearchCandidateRecord, VariantReadIndex, VectorQueryError, VectorReadIndex,
        VectorSearchHit,
    };
    use atlas_record::{
        AtlasRecord, AtlasRecordSet, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        MetricRow, MetricValue, RecordClassification, RecordContent, RecordIdentity,
        RecordMechanics, RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy,
        RecordTiming, RecordVisibility, RecordVisibilityReason,
    };

    macro_rules! impl_unused_text_index_capabilities {
        ($type:ty) => {
            impl IdentityReadIndex for $type {
                fn resolve_record_identity_matches(
                    &self,
                    _query: &str,
                    _normalized_query: &str,
                    _filter: Option<&SearchFilterNode>,
                ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
                    Ok(Vec::new())
                }
            }

            impl VectorReadIndex for $type {
                fn query_vector_index(
                    &self,
                    _query_vector: &[f32],
                    _filter: Option<&SearchFilterNode>,
                    _limit: u32,
                    _include_child_units: bool,
                ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
                    Ok(Vec::new())
                }

                fn load_record_embedding_vectors(
                    &self,
                    _record_key: &RecordKey,
                ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
                    Ok(Vec::new())
                }
            }

            impl ReferenceReadIndex for $type {
                fn reference_edges_for_seed(
                    &self,
                    _seed: &RecordKey,
                    _direction: ReferenceEdgeDirection,
                ) -> Result<Vec<GraphReferenceEdge>, atlas_index::RecordLoadError> {
                    Ok(Vec::new())
                }
            }

            impl VariantReadIndex for $type {
                fn variant_group_for_record(
                    &self,
                    _seed: &RecordKey,
                ) -> Result<Option<IndexVariantGroup>, atlas_index::RecordLoadError> {
                    Ok(None)
                }

                fn variant_groups_by_base_name(
                    &self,
                    _normalized_base_name: &str,
                ) -> Result<Vec<IndexVariantGroup>, atlas_index::RecordLoadError> {
                    Ok(Vec::new())
                }
            }

            impl RemasterReadIndex for $type {
                fn remaster_links_for_record(
                    &self,
                    _seed: &RecordKey,
                ) -> Result<Option<IndexRemasterLinks>, atlas_index::RecordLoadError> {
                    Ok(None)
                }
            }
        };
    }

    #[test]
    fn search_text_uses_candidates_for_fusion_and_hydrates_ranked_page_only() {
        let identity = fake_record("actions:identity", "Identity Action");
        let mut ranked = fake_record("actions:ranked", "Ranked Action");
        ranked.mechanics.metrics.push(MetricRow {
            domain: MetricDomain::Actor,
            key: "hp.max".to_string(),
            value: MetricValue::Number(42.0),
        });
        let off_page = fake_record("actions:offPage", "Off Page Action");
        let index = FakeTextIndex::new(vec![identity.clone(), ranked.clone(), off_page]);
        let candidate_calls = Rc::clone(&index.candidate_calls);
        let load_by_key_calls = Rc::clone(&index.load_by_key_calls);
        let mut service = AtlasRetrievalService::without_embeddings_with_index(Box::new(index));

        let page = service
            .search_text(TextSearchRequest {
                query: "Identity Action",
                exclude: None,
                filter: None,
                limit: 1,
                offset: 1,
                retrieval: RetrievalMode::Fts,
                fusion: FusionOptions::default(),
                fts_top_k: 10,
                vector_top_k: 10,
                explain: true,
            })
            .expect("text search should succeed");

        assert_eq!(page.total, 3);
        assert_eq!(
            page.records
                .iter()
                .map(|record| record.record.identity.key.to_string())
                .collect::<Vec<_>>(),
            vec!["actions:ranked"]
        );
        assert_eq!(
            page.records[0].record.mechanics.metrics,
            ranked.mechanics.metrics
        );
        assert_eq!(
            candidate_calls.borrow().as_slice(),
            &[vec![
                RecordKey::parse("actions:identity").expect("fixture key should parse"),
                RecordKey::parse("actions:offPage").expect("fixture key should parse"),
                RecordKey::parse("actions:ranked").expect("fixture key should parse"),
            ]]
        );
        assert_eq!(
            load_by_key_calls.borrow().as_slice(),
            &[vec![ranked.identity.key]]
        );
    }

    #[test]
    fn unweighted_rrf_rejects_lane_weights_at_runtime_boundary() {
        let request = TextSearchRequest {
            query: "healing",
            exclude: None,
            filter: None,
            limit: 10,
            offset: 0,
            retrieval: RetrievalMode::Fts,
            fusion: FusionOptions {
                method: FusionMethod::Rrf,
                fts_weight: 2.0,
                vector_weight: 1.0,
                rank_constant: 60.0,
            },
            fts_top_k: 10,
            vector_top_k: 10,
            explain: false,
        };

        let error = validate_search_text_request(&request).expect_err("invalid fusion should fail");

        assert_eq!(error.kind(), crate::SearchErrorKind::InvalidOptions);
    }

    struct FakeTextIndex {
        records: Vec<AtlasRecord>,
        candidate_calls: Rc<RefCell<Vec<Vec<RecordKey>>>>,
        load_by_key_calls: Rc<RefCell<Vec<Vec<RecordKey>>>>,
    }

    impl FakeTextIndex {
        fn new(records: Vec<AtlasRecord>) -> Self {
            Self {
                records,
                candidate_calls: Rc::new(RefCell::new(Vec::new())),
                load_by_key_calls: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn records_for_keys(&self, keys: &[RecordKey]) -> Vec<AtlasRecord> {
            keys.iter()
                .filter_map(|key| {
                    self.records
                        .iter()
                        .find(|record| record.identity.key == *key)
                        .cloned()
                })
                .collect()
        }
    }

    impl RecordReadIndex for FakeTextIndex {
        fn load_records_by_key(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<AtlasRecord>, atlas_index::RecordLoadError> {
            if !keys.is_empty() {
                self.load_by_key_calls.borrow_mut().push(keys.to_vec());
            }
            Ok(self.records_for_keys(keys))
        }

        fn load_record_set(&self) -> Result<AtlasRecordSet, atlas_index::RecordLoadError> {
            Ok(AtlasRecordSet {
                records: self.records.clone(),
                ..AtlasRecordSet::default()
            })
        }

        fn load_search_candidate_records(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<SearchCandidateRecord>, atlas_index::RecordLoadError> {
            self.candidate_calls.borrow_mut().push(keys.to_vec());
            Ok(self
                .records_for_keys(keys)
                .into_iter()
                .map(search_candidate_from_record)
                .collect())
        }
    }

    impl FilterReadIndex for FakeTextIndex {
        fn resolve_metric_filters(
            &self,
            _filter: Option<&SearchFilterNode>,
        ) -> Result<Option<SearchFilterNode>, FilterCompileError> {
            Ok(None)
        }

        fn list_filtered_record_keys(
            &self,
            _filter: Option<&SearchFilterNode>,
            _sort: FilteredRecordSort,
            _limit: u32,
            _offset: u32,
        ) -> Result<FilteredRecordKeyPage, FilterCompileError> {
            Ok(FilteredRecordKeyPage {
                record_keys: self
                    .records
                    .iter()
                    .map(|record| record.identity.key.clone())
                    .collect(),
                total: self.records.len() as u64,
            })
        }
    }

    impl FtsReadIndex for FakeTextIndex {
        fn query_precision_fts_index(
            &self,
            _fts_query: &FtsQuery,
            _filter: Option<&SearchFilterNode>,
            _limit: u32,
        ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
            Ok(vec![
                FtsSearchHit {
                    record_key: RecordKey::parse("actions:ranked")
                        .expect("fixture key should parse"),
                    rank: 1.0,
                    lane: FtsSearchLane::TitleAlias,
                    lane_rank: 1,
                    title_alias_texts: vec!["Ranked Action".to_string()],
                },
                FtsSearchHit {
                    record_key: RecordKey::parse("actions:offPage")
                        .expect("fixture key should parse"),
                    rank: 2.0,
                    lane: FtsSearchLane::TitleAlias,
                    lane_rank: 2,
                    title_alias_texts: vec!["Off Page Action".to_string()],
                },
            ])
        }

        fn query_fts_candidate_record_keys(
            &self,
            _fts_query: &FtsQuery,
            _candidate_keys: &[RecordKey],
        ) -> Result<Vec<RecordKey>, FilterCompileError> {
            Ok(Vec::new())
        }
    }

    impl_unused_text_index_capabilities!(FakeTextIndex);

    fn fake_record(key: &str, name: &str) -> AtlasRecord {
        let key = RecordKey::parse(key).expect("fixture key should parse");
        AtlasRecord {
            identity: RecordIdentity {
                key,
                name: name.to_string(),
            },
            classification: RecordClassification {
                kind: RecordKind::Rule,
                level: None,
                rarity: None,
                traits: Vec::new(),
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Actions".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::Action,
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: format!("packs/actions/{name}.json"),
                raw_json: Some("{}".to_string()),
            },
            publication: RecordPublication {
                title: None,
                remaster: false,
                category: PublicationCategory::Unknown,
            },
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics::default(),
            content: RecordContent::default(),
            variant: None,
            visibility: RecordVisibility::visible(RecordVisibilityReason::SourceRecord),
        }
    }

    fn search_candidate_from_record(record: AtlasRecord) -> SearchCandidateRecord {
        SearchCandidateRecord {
            key: record.identity.key,
            name: record.identity.name,
            traits: record.classification.traits,
            kind: record.classification.kind,
            foundry_type: record.foundry.record_type,
            inferred_groups: record.classification.taxonomy.inferred_groups,
            item_category: record
                .mechanics
                .item()
                .and_then(|item| item.category.clone()),
            item_group: record.mechanics.item().and_then(|item| item.group.clone()),
        }
    }
}
