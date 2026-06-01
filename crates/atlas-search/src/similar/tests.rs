use super::*;
use atlas_domain::{PackName, PublicationFamily, RecordFamily, SearchFilterNode};
use atlas_index::{
    FilterCompileError, FilterReadIndex, FilteredRecordKeyPage, FilteredRecordSort, FtsQuery,
    FtsReadIndex, FtsSearchHit, GraphReferenceEdge, IdentityReadIndex, IndexRemasterLinks,
    IndexVariantGroup, RecordIdentityMatch, RecordLoadError, RecordReadIndex, ReferenceReadIndex,
    RemasterReadIndex, SearchCandidateRecord, VariantReadIndex, VectorQueryError, VectorReadIndex,
    VectorSearchHit,
};
use atlas_record::{ContentSourceKind, ContentVisibility, PersistedRecordSet};

#[test]
fn similar_score_rewards_reference_overlap_when_semantic_distance_is_close() {
    let weights = SimilarScoreWeights::default();
    let plain = similar_score(0.20, 0, 0, weights);
    let connected = similar_score(0.21, 3, 2, weights);

    assert!(connected > plain);
}

#[test]
fn similar_score_can_disable_reference_and_trait_weights() {
    let weights = SimilarScoreWeights {
        semantic: 1.0,
        reference: 0.0,
        traits: 0.0,
    };

    let closer = similar_score(0.20, 0, 0, weights);
    let connected = similar_score(0.21, 5, 5, weights);

    assert!(closer > connected);
}

#[test]
fn similar_weights_require_semantic_dominance() {
    let error = SimilarScoreWeights {
        semantic: 0.10,
        reference: 0.20,
        traits: 0.0,
    }
    .validate()
    .expect_err("reference-heavy weights should be rejected");

    assert!(
        error.contains("semantic weight must be greater than the combined reference and trait")
    );
}

#[test]
fn similar_weights_reject_equal_semantic_and_graph_weights() {
    SimilarScoreWeights {
        semantic: 0.5,
        reference: 0.4,
        traits: 0.1,
    }
    .validate()
    .expect_err("equal semantic and graph weights should be rejected");
}

#[test]
fn similar_weights_reject_zero_negative_and_non_finite_values() {
    for weights in [
        SimilarScoreWeights {
            semantic: 0.0,
            reference: 0.0,
            traits: 0.0,
        },
        SimilarScoreWeights {
            semantic: -1.0,
            reference: 0.0,
            traits: 0.0,
        },
        SimilarScoreWeights {
            semantic: 1.0,
            reference: f64::NAN,
            traits: 0.0,
        },
        SimilarScoreWeights {
            semantic: 1.0,
            reference: 0.0,
            traits: f64::INFINITY,
        },
    ] {
        weights
            .validate()
            .expect_err("invalid weights should be rejected");
    }
}

#[test]
fn similar_records_candidate_window_never_drops_below_result_limit() {
    let seed = RecordKey::parse("actions:seed").expect("fixture key should parse");
    let service =
        AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeSimilarIndex::new()));

    let result = service
        .similar_records(SimilarRecordRequest {
            seed: &seed,
            filter: None,
            limit: 2,
            candidate_limit: 0,
            weights: SimilarScoreWeights {
                semantic: 1.0,
                reference: 0.0,
                traits: 0.0,
            },
        })
        .expect("similar search should run")
        .expect("seed should exist");

    assert_eq!(result.records.len(), 2);
    assert_eq!(result.records[0].record.key.to_string(), "actions:plain");
    assert_eq!(result.records[1].record.key.to_string(), "actions:graph");
}

#[test]
fn similar_records_passes_filter_and_parent_only_scope_to_vector_index() {
    let seed = RecordKey::parse("actions:seed").expect("fixture key should parse");
    let filter = SearchFilterNode::record_family(RecordFamily::Rule);
    let service = AtlasRetrievalService::without_embeddings_with_index(Box::new(
        FakeSimilarIndex::new_expecting_filter(),
    ));

    let result = service
        .similar_records(SimilarRecordRequest {
            seed: &seed,
            filter: Some(&filter),
            limit: 1,
            candidate_limit: 1,
            weights: SimilarScoreWeights {
                semantic: 1.0,
                reference: 0.0,
                traits: 0.0,
            },
        })
        .expect("similar search should run")
        .expect("seed should exist");

    assert_eq!(result.records.len(), 1);
}

#[test]
fn select_seed_embedding_prefers_parent_unit() {
    let units = vec![unit("heading_section"), unit("parent")];

    let selected = select_seed_embedding_unit(&units).expect("unit should be selected");
    assert_eq!(selected.unit_kind, "parent");
}

#[test]
fn select_seed_embedding_requires_parent_unit() {
    let units = vec![unit("heading_section")];

    let error = select_seed_embedding_unit(&units).expect_err("child-only seed should be rejected");

    assert!(error.is_vector_readiness_required());
    assert_eq!(
        error.to_string(),
        "retrieval pattern is not implemented yet: similar records require a stored parent seed embedding"
    );
}

#[test]
fn similar_records_uses_seed_embedding_and_reranks_with_graph_evidence() {
    let seed = RecordKey::parse("actions:seed").expect("fixture key should parse");
    let service =
        AtlasRetrievalService::without_embeddings_with_index(Box::new(FakeSimilarIndex::new()));

    let result = service
        .similar_records(SimilarRecordRequest {
            seed: &seed,
            filter: None,
            limit: 2,
            candidate_limit: 2,
            weights: SimilarScoreWeights::default(),
        })
        .expect("similar search should run")
        .expect("seed should exist");

    assert_eq!(result.records.len(), 2);
    assert_eq!(result.records[0].record.key.to_string(), "actions:graph");
    assert_eq!(result.records[0].graph.shared_references.len(), 1);
    assert_eq!(result.records[0].graph.shared_traits, vec!["auditory"]);
    assert_eq!(result.records[1].record.key.to_string(), "actions:plain");
}

fn unit(unit_kind: &str) -> RecordEmbeddingVector {
    RecordEmbeddingVector {
        unit_kind: unit_kind.to_string(),
        label: None,
        ordinal: 0,
        vector: vec![0.1, 0.2],
    }
}

struct FakeSimilarIndex {
    records: Vec<PersistedRecord>,
    expect_filter: bool,
}

impl FakeSimilarIndex {
    fn new() -> Self {
        Self {
            records: vec![
                fake_record("actions:seed", "Seed Action", &["auditory"]),
                fake_record("actions:plain", "Plain Action", &[]),
                fake_record("actions:graph", "Graph Action", &["auditory"]),
                fake_record("actions:reference", "Shared Reference", &[]),
            ],
            expect_filter: false,
        }
    }

    fn new_expecting_filter() -> Self {
        Self {
            expect_filter: true,
            ..Self::new()
        }
    }
}

impl RecordReadIndex for FakeSimilarIndex {
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
    ) -> Result<Vec<SearchCandidateRecord>, RecordLoadError> {
        Ok(self
            .load_records_by_key(keys)?
            .into_iter()
            .map(search_candidate_from_record)
            .collect())
    }
}

impl IdentityReadIndex for FakeSimilarIndex {
    fn resolve_record_identity_matches(
        &self,
        _query: &str,
        _normalized_query: &str,
        _filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordIdentityMatch>, FilterCompileError> {
        Ok(Vec::new())
    }
}

impl FilterReadIndex for FakeSimilarIndex {
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
}

impl FtsReadIndex for FakeSimilarIndex {
    fn query_precision_fts_index(
        &self,
        _fts_query: &FtsQuery,
        _filter: Option<&SearchFilterNode>,
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

impl VectorReadIndex for FakeSimilarIndex {
    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        _limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        assert_eq!(query_vector, [0.1, 0.2]);
        assert_eq!(
            filter,
            self.expect_filter
                .then(|| SearchFilterNode::record_family(RecordFamily::Rule))
                .as_ref()
        );
        assert!(!include_child_units);
        Ok(vec![
            vector_hit("actions:seed", 0.0),
            vector_hit("actions:plain", 0.20),
            vector_hit("actions:graph", 0.21),
        ])
    }

    fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
        if record_key.to_string() == "actions:seed" {
            Ok(vec![unit("parent")])
        } else {
            Ok(Vec::new())
        }
    }
}

impl ReferenceReadIndex for FakeSimilarIndex {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, RecordLoadError> {
        assert_eq!(direction, ReferenceEdgeDirection::Outgoing);
        let seed_text = seed.to_string();
        if seed_text == "actions:seed" || seed_text == "actions:graph" {
            Ok(vec![graph_edge(seed, "actions:reference")])
        } else {
            Ok(Vec::new())
        }
    }
}

impl VariantReadIndex for FakeSimilarIndex {
    fn variant_group_for_record(
        &self,
        _seed: &RecordKey,
    ) -> Result<Option<IndexVariantGroup>, RecordLoadError> {
        Ok(None)
    }

    fn variant_groups_by_base_name(
        &self,
        _normalized_base_name: &str,
    ) -> Result<Vec<IndexVariantGroup>, RecordLoadError> {
        Ok(Vec::new())
    }
}

impl RemasterReadIndex for FakeSimilarIndex {
    fn remaster_links_for_record(
        &self,
        _seed: &RecordKey,
    ) -> Result<Option<IndexRemasterLinks>, RecordLoadError> {
        Ok(None)
    }
}

fn vector_hit(record_key: &str, distance: f64) -> VectorSearchHit {
    VectorSearchHit {
        record_key: record_key.to_string(),
        unit_kind: "parent".to_string(),
        label: None,
        distance,
    }
}

fn graph_edge(from: &RecordKey, to: &str) -> GraphReferenceEdge {
    GraphReferenceEdge {
        from_record_key: from.clone(),
        to_record_key: RecordKey::parse(to).expect("fixture key should parse"),
        display_text: Some("Shared Reference".to_string()),
        reference_text: "fixture".to_string(),
        source_kind: ContentSourceKind::Description,
        visibility: ContentVisibility::Public,
    }
}

fn fake_record(key: &str, name: &str, traits: &[&str]) -> PersistedRecord {
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
        traits: traits.iter().map(|value| value.to_string()).collect(),
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
