use super::*;
use atlas_index::{FtsSearchLane, SearchCandidateRecord};

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

fn fts_hit(record: &str, rank: f64, lane: FtsSearchLane, lane_rank: u32) -> FtsSearchHit {
    FtsSearchHit {
        record_key: test_record_key(record),
        rank,
        lane,
        lane_rank,
        title_alias_texts: Vec::new(),
    }
}

fn title_fts_hit(record: &str, rank: f64, lane_rank: u32, texts: &[&str]) -> FtsSearchHit {
    FtsSearchHit {
        record_key: test_record_key(record),
        rank,
        lane: FtsSearchLane::TitleAlias,
        lane_rank,
        title_alias_texts: texts.iter().map(|text| (*text).to_string()).collect(),
    }
}

fn test_record(key: &str, name: &str, traits: &[&str]) -> SearchCandidateRecord {
    let key = test_record_key(key);
    SearchCandidateRecord {
        key,
        name: name.to_string(),
        traits: traits.iter().map(|value| value.to_string()).collect(),
        prerequisites: Vec::new(),
        system_category: None,
        system_group: None,
        taxonomy_families: Vec::new(),
    }
}

#[test]
fn weighted_rrf_combines_lanes_and_excludes_identity_matches() {
    let fts_hits = vec![
        fts_hit("records:a", -2.0, FtsSearchLane::Mixed, 1),
        fts_hit("records:b", -1.0, FtsSearchLane::Mixed, 2),
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
        fts_hit("records:a", -2.0, FtsSearchLane::Mixed, 1),
        fts_hit("records:b", -1.0, FtsSearchLane::Mixed, 2),
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
        classify_fts_texts(
            FtsSearchLane::Mixed,
            &direct,
            &["treat".to_string(), "wounds".to_string()],
            std::iter::empty::<&str>(),
        ),
        FtsMatchConfidence::DirectTitle
    );
    assert_eq!(
        classify_fts_texts(
            FtsSearchLane::Facet,
            &strong,
            &["healing".to_string(), "manipulate".to_string()],
            std::iter::empty::<&str>(),
        ),
        FtsMatchConfidence::StrongLexical
    );
    assert_eq!(
        classify_fts_texts(
            FtsSearchLane::Facet,
            &weak,
            &[
                "low".to_string(),
                "level".to_string(),
                "fear".to_string(),
                "spell".to_string()
            ],
            std::iter::empty::<&str>(),
        ),
        FtsMatchConfidence::WeakLexical
    );
}

#[test]
fn fts_fusion_policy_can_zero_weak_hits() {
    let fts_hits = vec![
        fts_hit("records:weak", 10.0, FtsSearchLane::Facet, 1),
        fts_hit("records:strong", 8.0, FtsSearchLane::Facet, 2),
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

#[test]
fn title_alias_rerank_promotes_missing_stopword_alias() {
    let fts_hits = vec![
        title_fts_hit(
            "records:reactive",
            1.0,
            90,
            &["Reactive Strike", "Attack of Opportunity"],
        ),
        title_fts_hit("records:target", 2.0, 1, &["Target of Opportunity"]),
    ];
    let vector_hits = vec![semantic_hit("records:target", 0.1)];
    let records_by_key = BTreeMap::from([
        (
            test_record_key("records:reactive"),
            test_record("records:reactive", "Reactive Strike", &[]),
        ),
        (
            test_record_key("records:target"),
            test_record("records:target", "Target of Opportunity", &[]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &["attack".to_string(), "opportunity".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        explain: true,
        identity_count: 0,
    });

    assert_eq!(fused[0].record_key.to_string(), "records:reactive");
    assert_eq!(
        explanation(&fused[0]).fts_confidence,
        Some(FtsMatchConfidence::DirectTitle)
    );
    assert_eq!(explanation(&fused[0]).fts_rank, Some(1));
}

#[test]
fn title_alias_rerank_promotes_prefix_title_match() {
    let fts_hits = vec![
        title_fts_hit("records:battle", 1.0, 1, &["Battle"]),
        title_fts_hit("records:medicine", 2.0, 12, &["Battle Medicine"]),
    ];
    let vector_hits = vec![semantic_hit("records:battle", 0.1)];
    let records_by_key = BTreeMap::from([
        (
            test_record_key("records:battle"),
            test_record("records:battle", "Battle", &[]),
        ),
        (
            test_record_key("records:medicine"),
            test_record("records:medicine", "Battle Medicine", &[]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        records_by_key: &records_by_key,
        fts_tokens: &["battle".to_string(), "med".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        explain: true,
        identity_count: 0,
    });

    assert_eq!(fused[0].record_key.to_string(), "records:medicine");
    assert_eq!(
        explanation(&fused[0]).fts_confidence,
        Some(FtsMatchConfidence::DirectTitle)
    );
}

#[test]
fn title_alias_rerank_keeps_single_token_overlap_weak() {
    let persistent_servant = test_record("records:servant", "Persistent Servant", &[]);

    assert_eq!(
        classify_fts_texts(
            FtsSearchLane::TitleAlias,
            &persistent_servant,
            &["persistent".to_string(), "damage".to_string()],
            ["Persistent Servant"].into_iter(),
        ),
        FtsMatchConfidence::WeakLexical
    );
}
