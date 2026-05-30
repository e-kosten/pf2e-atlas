use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::PersistedRecord;

use crate::query::normalize_record_query;
use crate::{
    AtlasRetrievalService, GraphRemasterLinkResult, GraphRemasterLinksResult,
    GraphVariantGroupResult, SearchError,
};

impl AtlasRetrievalService {
    pub fn variant_group(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<GraphVariantGroupResult>, SearchError> {
        let Some(seed) = self
            .get_records(std::slice::from_ref(record_key))?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let group = self.index.variant_group_for_record(record_key)?;
        let (variant_group_key, variants) = group
            .map(|group| {
                self.load_records_preserving_order(&group.record_keys)
                    .map(|variants| (group.variant_group_key, variants))
            })
            .transpose()?
            .unwrap_or((None, Vec::new()));
        Ok(Some(GraphVariantGroupResult {
            seed: Some(seed),
            variant_group_key,
            variants,
        }))
    }

    pub fn variant_groups_by_base_name(
        &self,
        base_name: &str,
    ) -> Result<Vec<GraphVariantGroupResult>, SearchError> {
        let normalized_base_name = normalize_record_query(base_name);
        self.index
            .variant_groups_by_base_name(&normalized_base_name)?
            .into_iter()
            .map(|group| {
                let variants = self.load_records_preserving_order(&group.record_keys)?;
                Ok(GraphVariantGroupResult {
                    seed: None,
                    variant_group_key: group.variant_group_key,
                    variants,
                })
            })
            .collect()
    }

    pub fn remaster_links(
        &self,
        record_key: &RecordKey,
    ) -> Result<Option<GraphRemasterLinksResult>, SearchError> {
        let Some(seed) = self
            .get_records(std::slice::from_ref(record_key))?
            .into_iter()
            .next()
        else {
            return Ok(None);
        };
        let links = self
            .index
            .remaster_links_for_record(record_key)?
            .map(|links| -> Result<_, SearchError> {
                let record_keys = links
                    .links
                    .iter()
                    .flat_map(|link| {
                        [
                            link.remaster_record_key.clone(),
                            link.legacy_record_key.clone(),
                        ]
                    })
                    .collect::<Vec<_>>();
                let records_by_key = self
                    .get_records(&record_keys)?
                    .into_iter()
                    .map(|record| (record.key.clone(), record))
                    .collect::<BTreeMap<_, _>>();
                links
                    .links
                    .into_iter()
                    .map(|link| {
                        let remaster_record = records_by_key
                            .get(&link.remaster_record_key)
                            .cloned()
                            .ok_or_else(|| {
                                SearchError::RecordLoad(atlas_index::RecordLoadError::InvalidData(
                                    format!(
                                        "remaster link target `{}` was not found",
                                        link.remaster_record_key
                                    ),
                                ))
                            })?;
                        let legacy_record = records_by_key
                            .get(&link.legacy_record_key)
                            .cloned()
                            .ok_or_else(|| {
                                SearchError::RecordLoad(atlas_index::RecordLoadError::InvalidData(
                                    format!(
                                        "remaster link target `{}` was not found",
                                        link.legacy_record_key
                                    ),
                                ))
                            })?;
                        Ok(GraphRemasterLinkResult {
                            remaster_record,
                            legacy_record,
                            source: link.source,
                            source_ref: link.source_ref,
                        })
                    })
                    .collect()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Some(GraphRemasterLinksResult { seed, links }))
    }

    fn load_records_preserving_order(
        &self,
        record_keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        let mut by_key = self
            .get_records(record_keys)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut records = Vec::with_capacity(record_keys.len());
        for key in record_keys {
            let Some(record) = by_key.remove(key) else {
                return Err(SearchError::RecordLoad(
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
    use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordKey};
    use atlas_index::{
        FilterCompileError, FilteredRecordKeyPage, FilteredRecordSort, FtsQuery, FtsSearchHit,
        GraphReadIndex, GraphReferenceEdge, IndexRemasterLinkRecord, IndexRemasterLinks,
        IndexVariantGroup, RecordLoadError, ReferenceEdgeDirection, SearchIndex, VectorQueryError,
        VectorSearchHit,
    };
    use atlas_record::PersistedRecordSet;

    use super::*;

    #[test]
    fn graph_product_service_uses_index_variant_group_seam()
    -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let result = service
            .variant_group(&RecordKey::parse("actions:testAction1")?)?
            .expect("seed record should exist");

        assert_eq!(
            result.seed.as_ref().map(|record| record.key.to_string()),
            Some("actions:testAction1".to_string())
        );
        assert_eq!(result.variant_group_key.as_deref(), Some("test-action"));
        assert_eq!(result.variants[0].key.to_string(), "actions:testAction1");
        Ok(())
    }

    #[test]
    fn graph_product_service_uses_index_variant_base_name_seam()
    -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let results = service.variant_groups_by_base_name("Test Action")?;

        assert_eq!(results.len(), 1);
        assert!(results[0].seed.is_none());
        assert_eq!(results[0].variant_group_key.as_deref(), Some("test-action"));
        Ok(())
    }

    #[test]
    fn graph_product_service_uses_index_remaster_seam() -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let result = service
            .remaster_links(&RecordKey::parse("actions:testAction1")?)?
            .expect("seed record should exist");

        assert_eq!(result.seed.key.to_string(), "actions:testAction1");
        assert_eq!(result.links.len(), 1);
        assert_eq!(
            result.links[0].remaster_record.key.to_string(),
            "actions:testAction2"
        );
        Ok(())
    }

    #[test]
    fn graph_product_service_returns_none_for_missing_seed()
    -> Result<(), Box<dyn std::error::Error>> {
        let service =
            AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeIndex::new()));

        let result = service.remaster_links(&RecordKey::parse("actions:missing")?)?;

        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn graph_product_service_propagates_index_read_errors() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = AtlasRetrievalService::without_embeddings_with_index(Box::new(
            FakeIndex::with_mode(FakeGraphMode::Error),
        ));

        let error = service
            .variant_group(&RecordKey::parse("actions:testAction1")?)
            .expect_err("graph read error should propagate");

        assert!(matches!(error, SearchError::RecordLoad(_)));
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
            .variant_group(&RecordKey::parse("actions:testAction1")?)
            .expect_err("missing variant target should be invalid artifact data");

        assert!(matches!(error, SearchError::RecordLoad(_)));
        assert_eq!(
            error.to_string(),
            "record data is invalid: graph relation target `actions:missing` was not found"
        );
        Ok(())
    }

    #[test]
    fn graph_product_service_reports_missing_remaster_relation_targets()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = AtlasRetrievalService::without_embeddings_with_index(Box::new(
            FakeIndex::with_mode(FakeGraphMode::MissingTarget),
        ));

        let error = service
            .remaster_links(&RecordKey::parse("actions:testAction1")?)
            .expect_err("missing remaster target should be invalid artifact data");

        assert!(matches!(error, SearchError::RecordLoad(_)));
        assert_eq!(
            error.to_string(),
            "record data is invalid: remaster link target `actions:missing` was not found"
        );
        Ok(())
    }

    struct FakeIndex {
        records: Vec<PersistedRecord>,
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

    impl SearchIndex for FakeIndex {
        fn load_records_by_key(
            &self,
            keys: &[RecordKey],
        ) -> Result<Vec<PersistedRecord>, RecordLoadError> {
            Ok(keys
                .iter()
                .filter_map(|key| {
                    self.records
                        .iter()
                        .find(|record| record.key == *key)
                        .cloned()
                })
                .collect())
        }

        fn load_record_set(&self) -> Result<PersistedRecordSet, RecordLoadError> {
            Ok(PersistedRecordSet {
                records: self.records.clone(),
                ..PersistedRecordSet::default()
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
                    key: record.key,
                    name: record.name,
                    traits: record.traits,
                    record_family: record.record_family,
                    foundry_record_type: record.foundry_record_type,
                    taxonomy_families: record.taxonomy_families,
                    system_category: record.system_category,
                    system_group: record.system_group,
                })
                .collect())
        }

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
                    .map(|record| record.key.clone())
                    .collect(),
                total: self.records.len() as u64,
            })
        }

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

    impl GraphReadIndex for FakeIndex {
        fn reference_edges_for_seed(
            &self,
            _seed: &RecordKey,
            _direction: ReferenceEdgeDirection,
        ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
            Ok(Vec::new())
        }

        fn variant_group_for_record(
            &self,
            _seed: &RecordKey,
        ) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
            if matches!(self.mode, FakeGraphMode::Error) {
                return Err(RecordLoadError::QueryFailed(
                    "fixture graph error".to_string(),
                ));
            }
            assert_eq!(self.records[0].key, *_seed);
            let record_keys = if matches!(self.mode, FakeGraphMode::MissingTarget) {
                vec![RecordKey::parse("actions:missing").expect("fixture key should parse")]
            } else {
                self.records
                    .iter()
                    .map(|record| record.key.clone())
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
                    .map(|record| record.key.clone())
                    .collect(),
            }])
        }

        fn remaster_links_for_record(
            &self,
            seed: &RecordKey,
        ) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
            if matches!(self.mode, FakeGraphMode::Error) {
                return Err(RecordLoadError::QueryFailed(
                    "fixture graph error".to_string(),
                ));
            }
            assert_eq!(self.records[0].key, *seed);
            let remaster_record_key = if matches!(self.mode, FakeGraphMode::MissingTarget) {
                RecordKey::parse("actions:missing").expect("fixture key should parse")
            } else {
                self.records[1].key.clone()
            };
            Ok(Some(IndexRemasterLinks {
                links: vec![IndexRemasterLinkRecord {
                    remaster_record_key,
                    legacy_record_key: self.records[0].key.clone(),
                    source: atlas_domain::RemasterLinkSource::Migration,
                    source_ref: "fixture".to_string(),
                }],
            }))
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
}
