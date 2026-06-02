use super::*;
use atlas_domain::RecordKind;
use atlas_index::SearchCandidateRecord;

fn test_record_key(value: &str) -> RecordKey {
    RecordKey::parse(value).expect("fixture record key should parse")
}

fn semantic_hit(record: &str, distance: f64) -> SemanticSearchHit {
    SemanticSearchHit::new(
        test_record_key(record),
        "parent".to_string(),
        None,
        distance,
        distance,
    )
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
        foundry_record_type: "spell".to_string(),
        traits: traits.iter().map(|value| (*value).to_string()).collect(),
        kind: RecordKind::Spell,
        taxonomy_families: Vec::new(),
        system_category: None,
        system_group: None,
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
    let candidates_by_key = BTreeMap::from([
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
        candidates_by_key: &candidates_by_key,
        fts_tokens: &["battle".to_string(), "medicine".to_string()],
        identity_keys: &identity_keys,
        excluded_keys: &excluded_keys,
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        fts_policy: DEFAULT_FTS_FUSION_POLICY,
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
    assert_eq!(fused[0].explain.as_ref().unwrap().rank, 2);
    assert_eq!(fused[0].explain.as_ref().unwrap().fts_rank, Some(2));
    assert_eq!(fused[0].explain.as_ref().unwrap().vector_rank, Some(1));
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
    let candidates_by_key = BTreeMap::from([
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
        candidates_by_key: &candidates_by_key,
        fts_tokens: &["attack".to_string(), "opportunity".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        fts_policy: DEFAULT_FTS_FUSION_POLICY,
        explain: true,
        identity_count: 0,
    });

    assert_eq!(fused[0].record_key.to_string(), "records:reactive");
    assert_eq!(
        fused[0].explain.as_ref().unwrap().fts_confidence,
        Some(FtsMatchConfidence::DirectTitle)
    );
    assert_eq!(fused[0].explain.as_ref().unwrap().fts_rank, Some(1));
}

#[test]
fn default_policy_demotes_weak_fts_hits_below_vector_hits() {
    let fts_hits = vec![fts_hit("records:weak", 1.0, FtsSearchLane::Facet, 1)];
    let vector_hits = vec![semantic_hit("records:semantic", 0.1)];
    let candidates_by_key = BTreeMap::from([
        (
            test_record_key("records:weak"),
            test_record("records:weak", "Shielded Arm", &["metal"]),
        ),
        (
            test_record_key("records:semantic"),
            test_record("records:semantic", "Semantic Fear Result", &["fear"]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        candidates_by_key: &candidates_by_key,
        fts_tokens: &[
            "low".to_string(),
            "level".to_string(),
            "fear".to_string(),
            "spell".to_string(),
        ],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions::default(),
        fts_policy: DEFAULT_FTS_FUSION_POLICY,
        explain: true,
        identity_count: 0,
    });

    assert_eq!(fused[0].record_key.to_string(), "records:semantic");
    let weak = fused
        .iter()
        .find(|hit| hit.record_key.to_string() == "records:weak")
        .expect("weak hit should still be present under demote-weak policy");
    assert_eq!(
        weak.explain.as_ref().unwrap().fts_confidence,
        Some(FtsMatchConfidence::WeakLexical)
    );

    let all_policy = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        candidates_by_key: &candidates_by_key,
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
            method: FusionMethod::WeightedRrf,
            fts_weight: 10.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        },
        fts_policy: FtsFusionPolicy::All,
        explain: true,
        identity_count: 0,
    });
    let demote_weak = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        candidates_by_key: &candidates_by_key,
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
            method: FusionMethod::WeightedRrf,
            fts_weight: 10.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        },
        fts_policy: FtsFusionPolicy::DemoteWeak,
        explain: true,
        identity_count: 0,
    });

    assert_eq!(all_policy[0].record_key.to_string(), "records:weak");
    assert_eq!(demote_weak[0].record_key.to_string(), "records:semantic");
}

#[test]
fn fts_fusion_policy_can_zero_weak_hits() {
    let fts_hits = vec![
        fts_hit("records:weak", 10.0, FtsSearchLane::Facet, 1),
        fts_hit("records:strong", 8.0, FtsSearchLane::Facet, 2),
    ];
    let vector_hits = vec![semantic_hit("records:semantic", 0.1)];
    let candidates_by_key = BTreeMap::from([
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

    let strong_only = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &vector_hits,
        candidates_by_key: &candidates_by_key,
        fts_tokens: &["fear".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Hybrid,
        fusion: FusionOptions {
            method: FusionMethod::WeightedRrf,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        },
        fts_policy: FtsFusionPolicy::StrongOnly,
        explain: true,
        identity_count: 0,
    });

    assert_eq!(strong_only[0].record_key.to_string(), "records:semantic");
    assert!(
        strong_only
            .iter()
            .any(|hit| hit.record_key.to_string() == "records:strong")
    );
    assert!(
        !strong_only
            .iter()
            .any(|hit| hit.record_key.to_string() == "records:weak")
    );
}

#[test]
fn best_fts_explain_prefers_higher_confidence_then_better_rank() {
    let mut accumulator = FusionAccumulator::new(test_record_key("records:a"));
    accumulator.record_best_fts_explain(
        1,
        1.0,
        FtsSearchLane::Facet,
        FtsMatchConfidence::WeakLexical,
    );
    accumulator.record_best_fts_explain(
        3,
        2.0,
        FtsSearchLane::TitleAlias,
        FtsMatchConfidence::StrongLexical,
    );
    assert_eq!(
        accumulator.fts_confidence,
        Some(FtsMatchConfidence::StrongLexical)
    );
    assert_eq!(accumulator.fts_rank, Some(3));
    assert_eq!(accumulator.fts_score, Some(2.0));

    accumulator.record_best_fts_explain(
        10,
        3.0,
        FtsSearchLane::TitleAlias,
        FtsMatchConfidence::StrongLexical,
    );
    assert_eq!(accumulator.fts_rank, Some(3));
    assert_eq!(accumulator.fts_score, Some(2.0));

    accumulator.record_best_fts_explain(
        2,
        4.0,
        FtsSearchLane::TitleAlias,
        FtsMatchConfidence::StrongLexical,
    );
    assert_eq!(accumulator.fts_rank, Some(2));
    assert_eq!(accumulator.fts_score, Some(4.0));
}

#[test]
fn zero_weight_fts_drops_fts_only_hits() {
    let fts_hits = vec![
        title_fts_hit("records:strong", 1.0, 1, &["Fear"]),
        fts_hit("records:weak", 2.0, FtsSearchLane::Facet, 2),
    ];
    let candidates_by_key = BTreeMap::from([
        (
            test_record_key("records:strong"),
            test_record("records:strong", "Fear", &["emotion"]),
        ),
        (
            test_record_key("records:weak"),
            test_record("records:weak", "Shielded Arm", &["metal"]),
        ),
    ]);

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &[],
        candidates_by_key: &candidates_by_key,
        fts_tokens: &["fear".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Fts,
        fusion: FusionOptions {
            method: FusionMethod::WeightedRrf,
            fts_weight: 0.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        },
        fts_policy: DEFAULT_FTS_FUSION_POLICY,
        explain: true,
        identity_count: 0,
    });

    assert!(fused.is_empty());
}

#[test]
fn lane_and_confidence_weights_affect_fused_order() {
    let candidates_by_key = BTreeMap::from([
        (
            test_record_key("records:title"),
            test_record("records:title", "Fear Strike", &[]),
        ),
        (
            test_record_key("records:facet"),
            test_record("records:facet", "Frightened Option", &["fear"]),
        ),
        (
            test_record_key("records:strong"),
            test_record("records:strong", "Fear Strike Shield Ward", &[]),
        ),
        (
            test_record_key("records:medium"),
            test_record("records:medium", "Fear Strike Shield Ward Guard", &[]),
        ),
    ]);
    let fts_hits = vec![
        fts_hit("records:facet", 1.0, FtsSearchLane::Facet, 1),
        title_fts_hit("records:title", 2.0, 2, &["Fear Strike"]),
        title_fts_hit("records:medium", 3.0, 3, &["Fear Strike Shield Ward Guard"]),
        title_fts_hit("records:strong", 4.0, 4, &["Fear Strike Shield Ward"]),
    ];

    let fused = fuse_ranked_hits(FusionInput {
        fts_hits: &fts_hits,
        vector_hits: &[],
        candidates_by_key: &candidates_by_key,
        fts_tokens: &["fear".to_string(), "strike".to_string()],
        identity_keys: &BTreeSet::new(),
        excluded_keys: &BTreeSet::new(),
        retrieval: RetrievalMode::Fts,
        fusion: FusionOptions::default(),
        fts_policy: FtsFusionPolicy::All,
        explain: true,
        identity_count: 0,
    });

    assert_eq!(
        fused
            .iter()
            .map(|hit| hit.record_key.to_string())
            .collect::<Vec<_>>(),
        vec![
            "records:title",
            "records:strong",
            "records:medium",
            "records:facet"
        ]
    );
    assert_eq!(
        fused[0].explain.as_ref().unwrap().fts_confidence,
        Some(FtsMatchConfidence::DirectTitle)
    );
    assert_eq!(
        fused[1].explain.as_ref().unwrap().fts_confidence,
        Some(FtsMatchConfidence::StrongLexical)
    );
    assert_eq!(
        fused[2].explain.as_ref().unwrap().fts_confidence,
        Some(FtsMatchConfidence::MediumLexical)
    );
    assert_eq!(
        fused[3].explain.as_ref().unwrap().fts_lane,
        Some(FtsLane::Facet)
    );
}
