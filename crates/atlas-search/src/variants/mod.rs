use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_index::{IndexVariantGroup, VariantReadIndex};
use atlas_record::AtlasRecord;

use crate::query::normalize_record_query;
use crate::{AtlasRetrievalService, GetRecordsRequest, RecordRetrieval, SearchError};

#[derive(Debug, Clone, PartialEq)]
pub struct VariantGroupRequest<'a> {
    pub record_key: &'a RecordKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantBaseNameRequest<'a> {
    pub base_name: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantGroupResult {
    pub seed: Option<AtlasRecord>,
    pub variant_group_key: Option<String>,
    pub variants: Vec<AtlasRecord>,
}

pub trait VariantRetrieval {
    fn variant_group(
        &self,
        request: VariantGroupRequest<'_>,
    ) -> Result<Option<VariantGroupResult>, SearchError>;

    fn variant_groups_by_base_name(
        &self,
        request: VariantBaseNameRequest<'_>,
    ) -> Result<Vec<VariantGroupResult>, SearchError>;
}

impl VariantRetrieval for AtlasRetrievalService {
    fn variant_group(
        &self,
        request: VariantGroupRequest<'_>,
    ) -> Result<Option<VariantGroupResult>, SearchError> {
        let record_key = request.record_key;
        let Some(seed) = self
            .get_records(GetRecordsRequest {
                record_keys: std::slice::from_ref(record_key),
            })?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let group = variant_group_for_record(self.index.as_ref(), record_key)?;
        let (variant_group_key, variants) = group
            .map(|group| {
                self.load_records_preserving_order(&group.record_keys)
                    .map(|variants| (group.variant_group_key, variants))
            })
            .transpose()?
            .unwrap_or((None, Vec::new()));
        Ok(Some(VariantGroupResult {
            seed: Some(seed),
            variant_group_key,
            variants,
        }))
    }

    fn variant_groups_by_base_name(
        &self,
        request: VariantBaseNameRequest<'_>,
    ) -> Result<Vec<VariantGroupResult>, SearchError> {
        let normalized_base_name = normalize_record_query(request.base_name);
        variant_groups_by_base_name(self.index.as_ref(), &normalized_base_name)?
            .into_iter()
            .map(|group| {
                let variants = self.load_records_preserving_order(&group.record_keys)?;
                Ok(VariantGroupResult {
                    seed: None,
                    variant_group_key: group.variant_group_key,
                    variants,
                })
            })
            .collect()
    }
}

fn variant_group_for_record<I>(
    index: &I,
    record_key: &RecordKey,
) -> Result<Option<IndexVariantGroup>, SearchError>
where
    I: VariantReadIndex + ?Sized,
{
    index
        .variant_group_for_record(record_key)
        .map_err(SearchError::from_record_load)
}

fn variant_groups_by_base_name<I>(
    index: &I,
    normalized_base_name: &str,
) -> Result<Vec<IndexVariantGroup>, SearchError>
where
    I: VariantReadIndex + ?Sized,
{
    index
        .variant_groups_by_base_name(normalized_base_name)
        .map_err(SearchError::from_record_load)
}

impl AtlasRetrievalService {
    fn load_records_preserving_order(
        &self,
        record_keys: &[RecordKey],
    ) -> Result<Vec<AtlasRecord>, SearchError> {
        let mut by_key = self
            .get_records(GetRecordsRequest { record_keys })?
            .into_iter()
            .map(|record| (record.identity.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut records = Vec::with_capacity(record_keys.len());
        for key in record_keys {
            let Some(record) = by_key.remove(key) else {
                return Err(SearchError::from_record_load(
                    atlas_index::RecordLoadError::InvalidData(format!(
                        "graph relation target `{key}` was not found"
                    )),
                ));
            };
            records.push(record);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PublicationCategory, RecordKey, RecordKind};
    use atlas_index::{
        DiscoveryError, DiscoveryReadIndex, FilterCompileError, FilterReadIndex,
        FilterValueRequest, FilteredRecordKeyPage, FilteredRecordSort, FtsQuery, FtsReadIndex,
        FtsSearchHit, GraphReferenceEdge, IdentityReadIndex, IndexRemasterLinkRecord,
        IndexRemasterLinks, IndexVariantGroup, RecordIdentityMatch, RecordLoadError,
        RecordReadIndex, ReferenceEdgeDirection, ReferenceReadIndex, RemasterReadIndex,
        VariantReadIndex, VectorQueryError, VectorReadIndex, VectorSearchHit,
    };
    use atlas_record::{
        AtlasRecordSet, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        RecordClassification, RecordContent, RecordIdentity, RecordMechanics, RecordProvenance,
        RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility,
        RecordVisibilityReason,
    };

    use super::*;

    #[test]
    fn graph_product_service_uses_index_variant_group_seam()
    -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let result = service
            .variant_group(VariantGroupRequest {
                record_key: &RecordKey::parse("actions:testAction1")?,
            })?
            .expect("seed record should exist");

        assert_eq!(
            result
                .seed
                .as_ref()
                .map(|record| record.identity.key.to_string()),
            Some("actions:testAction1".to_string())
        );
        assert_eq!(result.variant_group_key.as_deref(), Some("test-action"));
        assert_eq!(
            result.variants[0].identity.key.to_string(),
            "actions:testAction1"
        );
        Ok(())
    }

    #[test]
    fn graph_product_service_uses_index_variant_base_name_seam()
    -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let results = service.variant_groups_by_base_name(VariantBaseNameRequest {
            base_name: "Test Action",
        })?;

        assert_eq!(results.len(), 1);
        assert!(results[0].seed.is_none());
        assert_eq!(results[0].variant_group_key.as_deref(), Some("test-action"));
        Ok(())
    }

    #[test]
    fn graph_product_service_propagates_index_read_errors() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = AtlasRetrievalService::without_embeddings_with_index(Box::new(
            FakeIndex::with_mode(FakeGraphMode::Error),
        ));

        let error = service
            .variant_group(VariantGroupRequest {
                record_key: &RecordKey::parse("actions:testAction1")?,
            })
            .expect_err("graph read error should propagate");

        assert_eq!(error.kind(), crate::SearchErrorKind::QueryFailed);
        assert_eq!(
            error.to_string(),
            "record query failed: fixture graph error"
        );
        Ok(())
    }

    #[test]
    fn graph_product_service_reports_missing_variant_relation_targets()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = AtlasRetrievalService::without_embeddings_with_index(Box::new(
            FakeIndex::with_mode(FakeGraphMode::MissingTarget),
        ));

        let error = service
            .variant_group(VariantGroupRequest {
                record_key: &RecordKey::parse("actions:testAction1")?,
            })
            .expect_err("missing variant target should be invalid artifact data");

        assert_eq!(error.kind(), crate::SearchErrorKind::QueryFailed);
        assert_eq!(
            error.to_string(),
            "record data is invalid: graph relation target `actions:missing` was not found"
        );
        Ok(())
    }

    struct FakeIndex {
        records: Vec<AtlasRecord>,
        mode: FakeGraphMode,
    }

    #[derive(Debug, Clone, Copy)]
    enum FakeGraphMode {
        Default,
        Error,
        MissingTarget,
    }

    impl FakeIndex {
        fn new() -> Self {
            Self::with_mode(FakeGraphMode::Default)
        }

        fn with_mode(mode: FakeGraphMode) -> Self {
            Self {
                records: vec![
                    fake_record("actions:testAction1", "Test Action 1"),
                    fake_record("actions:testAction2", "Test Action 2"),
                ],
                mode,
            }
        }
    }

    impl RecordReadIndex for FakeIndex {
        fn load_records_by_key(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<AtlasRecord>, RecordLoadError> {
            Ok(keys
                .iter()
                .filter_map(|key| {
                    self.records
                        .iter()
                        .find(|record| record.identity.key == *key)
                        .cloned()
                })
                .collect())
        }

        fn load_record_set(&self) -> Result<AtlasRecordSet, RecordLoadError> {
            Ok(AtlasRecordSet {
                records: self.records.clone(),
                ..AtlasRecordSet::default()
            })
        }

        fn load_search_candidate_records(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<atlas_index::SearchCandidateRecord>, RecordLoadError> {
            Ok(self
                .load_records_by_key(keys)?
                .into_iter()
                .map(|record| atlas_index::SearchCandidateRecord {
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
                })
                .collect())
        }
    }

    impl IdentityReadIndex for FakeIndex {
        fn resolve_record_identity_matches(
            &self,
            _query: &str,
            _normalized_query: &str,
            _filter: Option<&atlas_domain::SearchFilterNode>,
        ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
            Ok(Vec::new())
        }
    }

    impl FilterReadIndex for FakeIndex {
        fn resolve_metric_filters(
            &self,
            _filter: Option<&atlas_domain::SearchFilterNode>,
        ) -> Result<Option<atlas_domain::SearchFilterNode>, FilterCompileError> {
            Ok(None)
        }

        fn list_filtered_record_keys(
            &self,
            _filter: Option<&atlas_domain::SearchFilterNode>,
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

    impl FtsReadIndex for FakeIndex {
        fn query_precision_fts_index(
            &self,
            _fts_query: &FtsQuery,
            _filter: Option<&atlas_domain::SearchFilterNode>,
            _limit: u32,
        ) -> Result<Vec<FtsSearchHit>, FilterCompileError> {
            Ok(Vec::new())
        }

        fn query_fts_candidate_record_keys(
            &self,
            _fts_query: &FtsQuery,
            _candidate_keys: &[RecordKey],
        ) -> Result<Vec<RecordKey>, FilterCompileError> {
            Ok(Vec::new())
        }
    }

    impl VectorReadIndex for FakeIndex {
        fn query_vector_index(
            &self,
            _query_vector: &[f32],
            _filter: Option<&atlas_domain::SearchFilterNode>,
            _limit: u32,
            _include_child_units: bool,
        ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
            Ok(Vec::new())
        }

        fn load_record_embedding_vectors(
            &self,
            _record_key: &RecordKey,
        ) -> Result<Vec<atlas_index::RecordEmbeddingVector>, VectorQueryError> {
            Ok(Vec::new())
        }
    }

    impl ReferenceReadIndex for FakeIndex {
        fn reference_edges_for_seed(
            &self,
            _seed: &RecordKey,
            _direction: ReferenceEdgeDirection,
        ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
            Ok(Vec::new())
        }
    }

    impl VariantReadIndex for FakeIndex {
        fn variant_group_for_record(
            &self,
            _seed: &RecordKey,
        ) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
            if matches!(self.mode, FakeGraphMode::Error) {
                return Err(RecordLoadError::QueryFailed(
                    "fixture graph error".to_string(),
                ));
            }
            assert_eq!(self.records[0].identity.key, *_seed);
            let record_keys = if matches!(self.mode, FakeGraphMode::MissingTarget) {
                vec![RecordKey::parse("actions:missing").expect("fixture key should parse")]
            } else {
                self.records
                    .iter()
                    .map(|record| record.identity.key.clone())
                    .collect()
            };
            Ok(Some(IndexVariantGroup {
                variant_group_key: Some("test-action".to_string()),
                record_keys,
            }))
        }

        fn variant_groups_by_base_name(
            &self,
            normalized_base_name: &str,
        ) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
            if matches!(self.mode, FakeGraphMode::Error) {
                return Err(RecordLoadError::QueryFailed(
                    "fixture graph error".to_string(),
                ));
            }
            assert_eq!(normalized_base_name, "test action");
            Ok(vec![IndexVariantGroup {
                variant_group_key: Some("test-action".to_string()),
                record_keys: self
                    .records
                    .iter()
                    .map(|record| record.identity.key.clone())
                    .collect(),
            }])
        }
    }

    impl RemasterReadIndex for FakeIndex {
        fn remaster_links_for_record(
            &self,
            seed: &RecordKey,
        ) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
            if matches!(self.mode, FakeGraphMode::Error) {
                return Err(RecordLoadError::QueryFailed(
                    "fixture graph error".to_string(),
                ));
            }
            assert_eq!(self.records[0].identity.key, *seed);
            let remaster_record_key = if matches!(self.mode, FakeGraphMode::MissingTarget) {
                RecordKey::parse("actions:missing").expect("fixture key should parse")
            } else {
                self.records[1].identity.key.clone()
            };
            Ok(Some(IndexRemasterLinks {
                links: vec![IndexRemasterLinkRecord {
                    remaster_record_key,
                    legacy_record_key: self.records[0].identity.key.clone(),
                    source: atlas_domain::RemasterLinkSource::Migration,
                    source_ref: "fixture".to_string(),
                }],
            }))
        }
    }

    impl DiscoveryReadIndex for FakeIndex {
        fn list_filter_fields(
            &self,
            _filter: Option<&atlas_domain::SearchFilterNode>,
            _filter_json: Option<serde_json::Value>,
        ) -> Result<atlas_domain::FilterFieldDiscovery, DiscoveryError> {
            unreachable!("variant tests do not call filter discovery")
        }

        fn list_filter_values(
            &self,
            _filter: Option<&atlas_domain::SearchFilterNode>,
            _request: FilterValueRequest,
        ) -> Result<atlas_domain::FilterValueDiscovery, DiscoveryError> {
            unreachable!("variant tests do not call filter discovery")
        }
    }

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
}
