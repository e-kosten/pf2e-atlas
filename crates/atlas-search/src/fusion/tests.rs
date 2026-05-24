use super::*;
use crate::normalize_record_query;
use atlas_domain::{PublicationFamily, RecordFamily};
use atlas_index::FtsSearchLane;

fn test_record_key(value: &str) -> RecordKey {
    RecordKey::parse(value).expect("fixture record key should parse")
}

fn explanation(hit: &FusedRankedHit) -> &TextSearchExplain {
    hit.explain
        .as_ref()
        .expect("fixture requested explanations")
}

fn semantic_hit(record: &str, distance: f64) -> SemanticSearchHit {
    SemanticSearchHit {
        record_key: record.to_string(),
        embedding_unit_key: format!("{record}#parent"),
        unit_kind: "parent".to_string(),
        label: None,
        distance,
        rank_distance: distance,
    }
}

fn test_record(key: &str, name: &str, traits: &[&str]) -> PersistedRecord {
    let key = test_record_key(key);
    PersistedRecord {
        id: key.id().clone(),
        pack_name: key.pack().clone(),
        key,
        name: name.to_string(),
        normalized_name: normalize_record_query(name),
        record_family: RecordFamily::Feat,
        pack_label: "Test Pack".to_string(),
        foundry_document_type: "Item".to_string(),
        foundry_record_type: "feat".to_string(),
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
        variant_source: "test".to_string(),
        source_path: "test.json".to_string(),
        is_default_visible: true,
        raw_json: "{}".to_string(),
    }
}

#[test]
fn weighted_rrf_combines_lanes_and_excludes_identity_matches() {
    let fts_hits = vec![
        FtsSearchHit {
            record_key: test_record_key("records:a"),
            rank: -2.0,
            lane: FtsSearchLane::Mixed,
            lane_rank: 1,
        },
        FtsSearchHit {
            record_key: test_record_key("records:b"),
            rank: -1.0,
            lane: FtsSearchLane::Mixed,
            lane_rank: 2,
        },
    ];
    let vector_hits = vec![
        semantic_hit("records:b", 0.1),
        semantic_hit("records:c", 0.2),
    ];
    let identity_keys = [test_record_key("records:a")]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let excluded_keys = BTreeSet::new();
    let records_by_key = BTreeMap::from([
        (
            test_record_key("records:b"),
            test_record("records:b", "Battle Medicine", &["healing"]),
        ),
        (
            test_record_key("records:c"),
            test_record("records:c", "Risky Surgery", &[]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &["battle".to_string(), "medicine".to_string()],
        identity_keys: &identity_keys,
        excluded_keys: &excluded_keys,
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        explain: true,
        identity_count: 1,
    });

    assert_eq!(
        fused
            .iter()
            .map(|hit| hit.record_key.to_string())
            .collect::<Vec<_>>(),
        vec!["records:b", "records:c"]
    );
    assert_eq!(explanation(&fused[0]).rank, 2);
    assert_eq!(explanation(&fused[0]).fts_rank, Some(2));
    assert_eq!(explanation(&fused[0]).vector_rank, Some(1));
}

#[test]
fn min_max_score_fusion_uses_lane_scores_and_weights() {
    let fts_hits = vec![
        FtsSearchHit {
            record_key: test_record_key("records:a"),
            rank: -2.0,
            lane: FtsSearchLane::Mixed,
            lane_rank: 1,
        },
        FtsSearchHit {
            record_key: test_record_key("records:b"),
            rank: -1.0,
            lane: FtsSearchLane::Mixed,
            lane_rank: 2,
        },
    ];
    let vector_hits = vec![
        semantic_hit("records:b", 0.1),
        semantic_hit("records:c", 0.2),
    ];
    let records_by_key = BTreeMap::from([
        (
            test_record_key("records:a"),
            test_record("records:a", "Direct Result", &[]),
        ),
        (
            test_record_key("records:b"),
            test_record("records:b", "Shared Result", &[]),
        ),
        (
            test_record_key("records:c"),
            test_record("records:c", "Semantic Result", &[]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &["result".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions {
            method: FusionMethod::MinMaxScore,
            fts_weight: 1.0,
            vector_weight: 2.0,
            rank_constant: 60.0,
            fts_policy: FtsFusionPolicy::All,
        },
        explain: true,
        identity_count: 0,
    });

    assert_eq!(
        fused
            .iter()
            .map(|hit| hit.record_key.to_string())
            .collect::<Vec<_>>(),
        vec!["records:b", "records:a", "records:c"]
    );
    assert_eq!(explanation(&fused[0]).fused_score, Some(2.0));
    assert_eq!(explanation(&fused[1]).fused_score, Some(1.0));
}

#[test]
fn fts_confidence_distinguishes_direct_and_weak_hits() {
    let direct = test_record("records:a", "Treat Wounds", &["healing"]);
    let strong = test_record("records:b", "Battle Medicine", &["healing", "manipulate"]);
    let weak = test_record("records:c", "Shielded Arm", &["metal"]);

    assert_eq!(
        classify_fts_match(
            FtsSearchLane::Mixed,
            &direct,
            &["treat".to_string(), "wounds".to_string()]
        ),
        FtsMatchConfidence::DirectTitle
    );
    assert_eq!(
        classify_fts_match(
            FtsSearchLane::Facet,
            &strong,
            &["healing".to_string(), "manipulate".to_string()]
        ),
        FtsMatchConfidence::StrongLexical
    );
    assert_eq!(
        classify_fts_match(
            FtsSearchLane::Facet,
            &weak,
            &[
                "low".to_string(),
                "level".to_string(),
                "fear".to_string(),
                "spell".to_string()
            ],
        ),
        FtsMatchConfidence::WeakLexical
    );
}

#[test]
fn fts_fusion_policy_can_zero_weak_hits() {
    let fts_hits = vec![
        FtsSearchHit {
            record_key: test_record_key("records:weak"),
            rank: 10.0,
            lane: FtsSearchLane::Facet,
            lane_rank: 1,
        },
        FtsSearchHit {
            record_key: test_record_key("records:strong"),
            rank: 8.0,
            lane: FtsSearchLane::Facet,
            lane_rank: 2,
        },
    ];
    let vector_hits = vec![semantic_hit("records:semantic", 0.1)];
    let records_by_key = BTreeMap::from([
        (
            test_record_key("records:weak"),
            test_record("records:weak", "Shielded Arm", &["metal"]),
        ),
        (
            test_record_key("records:strong"),
            test_record("records:strong", "Fear", &["fear"]),
        ),
        (
            test_record_key("records:semantic"),
            test_record("records:semantic", "Semantic Fear Result", &["fear"]),
        ),
    ]);

    let all = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &[
            "low".to_string(),
            "level".to_string(),
            "fear".to_string(),
            "spell".to_string(),
        ],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions {
            method: FusionMethod::MinMaxScore,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
            fts_policy: FtsFusionPolicy::All,
        },
        explain: true,
        identity_count: 0,
    });
    let strong_only = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &[
            "low".to_string(),
            "level".to_string(),
            "fear".to_string(),
            "spell".to_string(),
        ],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions {
            method: FusionMethod::MinMaxScore,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
            fts_policy: FtsFusionPolicy::StrongOnly,
        },
        explain: true,
        identity_count: 0,
    });

    assert!(
        all.iter()
            .any(|hit| hit.record_key.to_string() == "records:weak")
    );
    assert_eq!(strong_only[0].record_key.to_string(), "records:semantic");
    assert!(
        !strong_only
            .iter()
            .any(|hit| hit.record_key.to_string() == "records:weak")
    );
}
