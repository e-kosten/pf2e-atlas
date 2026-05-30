use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::{FtsQuery, FtsSearchHit};
use atlas_record::PersistedRecord;
use serde::{Deserialize, Serialize};

use crate::fusion::{
    DEFAULT_FTS_FUSION_POLICY, FusionInput, FusionMethod, FusionOptions, TextSearchExplain,
    fuse_ranked_hits, identity_explain,
};
use crate::query::{TextQueryAnalysis, analyze_text_query};
use crate::resolution::{RecordResolutionMatchKind, RecordResolutionResult};
use crate::semantic::{SemanticSearchHit, SemanticSearchMode};
use crate::{AtlasRetrievalService, SearchError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

impl RetrievalMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts => "fts",
            Self::Vector => "vector",
            Self::Hybrid => "hybrid",
        }
    }

    pub(crate) const fn uses_fts(self) -> bool {
        matches!(self, Self::Fts | Self::Hybrid)
    }

    pub(crate) const fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRequest<'a> {
    pub query: &'a str,
    pub exclude: Option<&'a str>,
    pub filter: Option<&'a SearchFilterNode>,
    pub limit: u32,
    pub offset: u32,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub fts_top_k: u32,
    pub vector_top_k: u32,
    pub explain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchPage {
    pub query: TextQueryAnalysis,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub records: Vec<TextSearchRecord>,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRecord {
    pub record: PersistedRecord,
    pub match_info: TextSearchMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSearchMatch {
    Identity {
        retrieval: RetrievalMode,
        identity_match_kind: RecordResolutionMatchKind,
        explain: Option<TextSearchExplain>,
    },
    Ranked {
        retrieval: RetrievalMode,
        explain: Option<TextSearchExplain>,
    },
}

#[derive(Debug, Clone)]
pub enum AtlasSearchRequest<'a> {
    Semantic {
        query: &'a str,
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    },
    FilterOnly {
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
    },
    Text(TextSearchRequest<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtlasSearchResult {
    Semantic(crate::semantic::SemanticSearchResult),
    Text(TextSearchPage),
}

impl AtlasRetrievalService {
    pub fn text_search(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchPage, SearchError> {
        validate_text_search_request(&request)?;
        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let query = analyze_text_query(request.query, request.exclude);
        let fts_query = FtsQuery::from_tokens(query.fts_tokens.clone());
        let exclude_query = FtsQuery::from_tokens(query.exclude_tokens.clone());
        let identity_matches = self.resolve_record(request.query, filter)?;
        let fts_hits = if request.retrieval.uses_fts() {
            match fts_query.as_ref() {
                Some(fts_query) => {
                    self.index
                        .query_precision_fts_index(fts_query, filter, request.fts_top_k)?
                }
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let vector_hits = if request.retrieval.uses_vector() {
            self.semantic(
                request.query,
                filter,
                request.vector_top_k,
                SemanticSearchMode::WeightedChunks,
            )?
        } else {
            Vec::new()
        };
        let excluded_keys = match exclude_query.as_ref() {
            Some(exclude_query) => self
                .index
                .query_fts_candidate_record_keys(
                    exclude_query,
                    &candidate_keys(&identity_matches, &fts_hits, &vector_hits),
                )?
                .into_iter()
                .collect::<BTreeSet<_>>(),
            None => BTreeSet::new(),
        };
        let identity_matches = identity_matches
            .into_iter()
            .filter(|identity| !excluded_keys.contains(&identity.record.key))
            .collect::<Vec<_>>();
        let identity_keys = identity_matches
            .iter()
            .map(|identity| identity.record.key.clone())
            .collect::<BTreeSet<_>>();
        let fusion_candidate_keys = candidate_keys(&identity_matches, &fts_hits, &vector_hits)
            .into_iter()
            .filter(|key| !excluded_keys.contains(key))
            .collect::<Vec<_>>();
        let candidate_records = self
            .index
            .load_search_candidate_records(&fusion_candidate_keys)?;
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
        let identity_records = identity_matches
            .into_iter()
            .enumerate()
            .map(|(index, identity)| TextSearchRecord {
                record: identity.record,
                match_info: TextSearchMatch::Identity {
                    retrieval: request.retrieval,
                    identity_match_kind: identity.match_kind,
                    explain: request.explain.then(|| identity_explain(index)),
                },
            })
            .collect::<Vec<_>>();
        let mut page_items = identity_records
            .into_iter()
            .map(|record| TextSearchPageItem::Identity(Box::new(record)))
            .chain(fused.into_iter().map(TextSearchPageItem::Ranked))
            .skip(request.offset as usize)
            .take(request.limit as usize)
            .collect::<Vec<_>>();
        let ranked_page_keys = page_items
            .iter()
            .filter_map(|item| match item {
                TextSearchPageItem::Identity(_) => None,
                TextSearchPageItem::Ranked(ranked) => Some(ranked.record_key.clone()),
            })
            .collect::<Vec<_>>();
        let ranked_page_records = self
            .index
            .load_records_by_key(&ranked_page_keys)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut page_records = page_items
            .drain(..)
            .filter_map(|item| match item {
                TextSearchPageItem::Identity(record) => Some(*record),
                TextSearchPageItem::Ranked(ranked) => ranked_page_records
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

        Ok(TextSearchPage {
            query,
            retrieval: request.retrieval,
            fusion: request.fusion,
            records: page_records,
            total: total as u64,
        })
    }
}

enum TextSearchPageItem {
    Identity(Box<TextSearchRecord>),
    Ranked(crate::fusion::FusedRankedHit),
}

fn validate_text_search_request(request: &TextSearchRequest<'_>) -> Result<(), SearchError> {
    if let Some(filter) = request.filter {
        filter
            .validate()
            .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
    }
    if request.fusion.method == FusionMethod::Rrf
        && ((request.fusion.fts_weight - 1.0).abs() > f64::EPSILON
            || (request.fusion.vector_weight - 1.0).abs() > f64::EPSILON)
    {
        return Err(SearchError::InvalidSearchOptions(
            "unweighted rrf does not accept lane weights; use weighted-rrf".to_string(),
        ));
    }
    if request.fusion.rank_constant <= 0.0
        || request.fusion.fts_weight < 0.0
        || request.fusion.vector_weight < 0.0
    {
        return Err(SearchError::InvalidSearchOptions(
            "fusion weights must be non-negative and rank constant must be greater than zero"
                .to_string(),
        ));
    }
    Ok(())
}

fn candidate_keys(
    identity_matches: &[RecordResolutionResult],
    fts_hits: &[FtsSearchHit],
    vector_hits: &[SemanticSearchHit],
) -> Vec<RecordKey> {
    let mut keys = identity_matches
        .iter()
        .map(|identity| identity.record.key.clone())
        .collect::<BTreeSet<_>>();
    keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    keys.extend(
        vector_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok()),
    );
    keys.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use atlas_domain::{MetricDomain, PackName, PublicationFamily, RecordFamily};
    use atlas_index::{
        FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort, FtsSearchLane,
        GraphReadIndex, GraphReferenceEdge, IndexRemasterLinks, IndexVariantGroup,
        RecordEmbeddingVector, ReferenceEdgeDirection, SearchCandidateRecord, SearchIndex,
        VectorQueryError, VectorSearchHit,
    };
    use atlas_record::{MetricRow, MetricValue, PersistedRecord, PersistedRecordSet};

    #[test]
    fn text_search_uses_candidates_for_fusion_and_hydrates_ranked_page_only() {
        let identity = fake_record("actions:identity", "Identity Action");
        let mut ranked = fake_record("actions:ranked", "Ranked Action");
        ranked.metrics.push(MetricRow {
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
            .text_search(TextSearchRequest {
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
                .map(|record| record.record.key.to_string())
                .collect::<Vec<_>>(),
            vec!["actions:ranked"]
        );
        assert_eq!(page.records[0].record.metrics, ranked.metrics);
        assert_eq!(
            candidate_calls.borrow().as_slice(),
            &[vec![
                RecordKey::parse("actions:identity").expect("fixture key should parse"),
                RecordKey::parse("actions:offPage").expect("fixture key should parse"),
                RecordKey::parse("actions:ranked").expect("fixture key should parse"),
            ]]
        );
        assert_eq!(load_by_key_calls.borrow().as_slice(), &[vec![ranked.key]]);
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

        assert!(matches!(
            validate_text_search_request(&request),
            Err(SearchError::InvalidSearchOptions(_))
        ));
    }

    struct FakeTextIndex {
        records: Vec<PersistedRecord>,
        candidate_calls: Rc<RefCell<Vec<Vec<RecordKey>>>>,
        load_by_key_calls: Rc<RefCell<Vec<Vec<RecordKey>>>>,
    }

    impl FakeTextIndex {
        fn new(records: Vec<PersistedRecord>) -> Self {
            Self {
                records,
                candidate_calls: Rc::new(RefCell::new(Vec::new())),
                load_by_key_calls: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn records_for_keys(&self, keys: &[RecordKey]) -> Vec<PersistedRecord> {
            keys.iter()
                .filter_map(|key| {
                    self.records
                        .iter()
                        .find(|record| record.key == *key)
                        .cloned()
                })
                .collect()
        }
    }

    impl SearchIndex for FakeTextIndex {
        fn load_records_by_key(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<PersistedRecord>, atlas_index::RecordLoadError> {
            if !keys.is_empty() {
                self.load_by_key_calls.borrow_mut().push(keys.to_vec());
            }
            Ok(self.records_for_keys(keys))
        }

        fn load_record_set(&self) -> Result<PersistedRecordSet, atlas_index::RecordLoadError> {
            Ok(PersistedRecordSet {
                records: self.records.clone(),
                ..PersistedRecordSet::default()
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
                    .map(|record| record.key.clone())
                    .collect(),
                total: self.records.len() as u64,
            })
        }

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

    impl GraphReadIndex for FakeTextIndex {
        fn reference_edges_for_seed(
            &self,
            _seed: &RecordKey,
            _direction: ReferenceEdgeDirection,
        ) -> Result<Vec<GraphReferenceEdge>, atlas_index::RecordLoadError> {
            Ok(Vec::new())
        }

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

        fn remaster_links_for_record(
            &self,
            _seed: &RecordKey,
        ) -> Result<Option<IndexRemasterLinks>, atlas_index::RecordLoadError> {
            Ok(None)
        }
    }

    fn fake_record(key: &str, name: &str) -> PersistedRecord {
        let key = RecordKey::parse(key).expect("fixture key should parse");
        PersistedRecord {
            id: key.id().clone(),
            key,
            name: name.to_string(),
            normalized_name: name.to_lowercase(),
            record_family: RecordFamily::Rule,
            pack_name: PackName::new("actions").expect("fixture pack should parse"),
            pack_label: "Actions".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "action".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            prerequisites: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: format!("packs/actions/{name}.json"),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }

    fn search_candidate_from_record(record: PersistedRecord) -> SearchCandidateRecord {
        SearchCandidateRecord {
            key: record.key,
            name: record.name,
            traits: record.traits,
            record_family: record.record_family,
            foundry_record_type: record.foundry_record_type,
            taxonomy_families: record.taxonomy_families,
            system_category: record.system_category,
            system_group: record.system_group,
        }
    }
}
