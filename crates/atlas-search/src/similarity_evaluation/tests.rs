use super::*;
use crate::test_support::{input_with_both_profiles, open_input_fixture, record};
use crate::{ConfigurableSimilarRequest, RecordScope, SearchPage};
use atlas_domain::{ResultSort, SimilarityRole};
use serde_json::json;

fn fixture() -> atlas_index::IndexBuildInput {
    let mut input = input_with_both_profiles(vec![
        record(
            "hazards",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Seed","type":"hazard","system":{"details":{"description":"<p>Fungal growth</p>","reset":"<p>Returns at dawn</p>"}}}),
        ),
        record(
            "hazards",
            "Actor",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Candidate","type":"hazard","system":{"details":{"description":"<p>Fungal growth</p>","reset":"<p>Returns at midnight</p>"}}}),
        ),
        record(
            "spells",
            "Item",
            json!({"_id":"cccccccccccccccc","name":"Spell","type":"spell","system":{"description":{"value":"<p>Shifting fog</p>"}}}),
        ),
        record(
            "spells",
            "Item",
            json!({"_id":"dddddddddddddddd","name":"Negative","type":"spell","system":{"description":{"value":"<p>Opposite direction</p>"}}}),
        ),
    ]);
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.0);
        unit.vector[0] = 1.0;
        if unit.record.id().as_str() == "bbbbbbbbbbbbbbbb"
            && unit.field.as_deref() == Some("/system/details/reset")
        {
            unit.vector[0] = 0.8;
            unit.vector[1] = 0.6;
        }
        if unit.record.id().as_str() == "cccccccccccccccc"
            && !matches!(
                unit.address,
                atlas_domain::SourcePassageAddress::Identity {}
            )
        {
            unit.vector[0] = -1.0;
        }
        if unit.record.id().as_str() == "dddddddddddddddd" {
            unit.vector[0] = -1.0;
        }
    }
    input
}
fn variants() -> Vec<SimilarityEvaluationVariant> {
    let mut variants = vec![SimilarityEvaluationVariant {
        id: "baseline".into(),
        options: SimilarityOptions::Baseline {},
    }];
    for mode in [
        SimilarityCandidateParts::Any,
        SimilarityCandidateParts::Corresponding,
    ] {
        for decay in [0.0, 0.25, 0.5, 0.75, 1.0] {
            variants.push(SimilarityEvaluationVariant {
                id: format!("{mode:?}-{decay}"),
                options: SimilarityOptions::Passages {
                    decay,
                    parts: None,
                    candidate_parts: mode,
                },
            });
        }
        variants.push(SimilarityEvaluationVariant {
            id: format!("average-{mode:?}"),
            options: SimilarityOptions::ContentAverage {
                parts: None,
                candidate_parts: mode,
            },
        });
    }
    variants
}
#[test]
fn exact_matrix_matches_product_scores_for_both_profiles_and_every_method() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let keys = [
        "hazards:aaaaaaaaaaaaaaaa",
        "hazards:bbbbbbbbbbbbbbbb",
        "spells:cccccccccccccccc",
        "spells:dddddddddddddddd",
        "items:eeeeeeeeeeeeeeee",
    ]
    .map(|key| key.parse().unwrap());
    let variants = variants();
    for profile in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(profile);
        service.reset_read_metrics();
        let exact = service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys,
                filter: None,
                cancelled: None,
            })
            .unwrap();
        assert_eq!(exact.eligible_candidates, 3);
        assert_eq!(exact.runs.len(), 13);
        assert!(exact.runs.iter().all(|run| run.rows.len() == keys.len()));
        assert_eq!(service.read_metrics().source_body_decodes, 0);
        // Both role masks perform one maxima pass each, not one per decay.
        assert!(exact.pairwise_comparisons < 5 * exact.candidate_vectors * 2 + 100);
        for (variant, run) in variants.iter().zip(&exact.runs) {
            let product = service
                .configurable_similar(ConfigurableSimilarRequest {
                    source_sections: None,
                    content_scope: atlas_domain::SearchContentScope::All,
                    seed: &seed,
                    options: &variant.options,
                    search_profile: Some(&profile),
                    filter: None,
                    scope: RecordScope::Keys(&keys),
                    page: SearchPage::default(),
                    sort: ResultSort::Relevance {},
                    prefer_remaster: false,
                    cancelled: None,
                })
                .unwrap()
                .unwrap();
            let actual = product
                .results
                .records
                .iter()
                .map(|row| row.record.key.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                service.evaluation_ranking(run, false).unwrap(),
                actual,
                "{}",
                variant.id
            );
            for row in &product.results.records {
                let measured = run
                    .rows
                    .iter()
                    .find(|measured| measured.record_key == row.record.key)
                    .unwrap();
                assert!(
                    (measured.score.unwrap() - row.score).abs() < 1e-6,
                    "{}: {:?} != {}",
                    variant.id,
                    measured.score,
                    row.score
                );
            }
        }
        // The spell's identity is positive, its prose negative: legacy baseline
        // must include identity units, while content methods do not use them.
        assert_eq!(
            exact.runs[0]
                .rows
                .iter()
                .find(|row| row.record_key == keys[2])
                .unwrap()
                .score,
            Some(1.0)
        );
        assert_eq!(
            exact.runs[0]
                .rows
                .iter()
                .find(|row| row.record_key == keys[3])
                .unwrap()
                .score,
            Some(-1.0)
        );
    }
}
#[test]
fn explicit_selection_has_exact_role_coverage_and_filtering_reports_each_key() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let profile = SearchProfileId::new("bge-base");
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let reset = service
        .similarity_parts(&seed, Some(&profile))
        .unwrap()
        .parts
        .into_iter()
        .filter_map(|component| match component {
            atlas_domain::SimilarityComponent::Section { part } => Some(part),
            atlas_domain::SimilarityComponent::Identity { .. } => None,
        })
        .find(|part| part.role == SimilarityRole::Reset)
        .unwrap();
    let variants = [SimilarityEvaluationVariant {
        id: "reset".into(),
        options: SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(vec![reset.selector]),
            candidate_parts: SimilarityCandidateParts::Corresponding,
        },
    }];
    let keys =
        ["hazards:bbbbbbbbbbbbbbbb", "spells:cccccccccccccccc"].map(|key| key.parse().unwrap());
    let result = service
        .evaluate_similarity_candidates(SimilarityEvaluationRequest {
            seed: &seed,
            profile: &profile,
            variants: &variants,
            candidate_keys: &keys,
            filter: None,
            cancelled: None,
        })
        .unwrap();
    assert_eq!(result.runs[0].rows[0].score, Some(0.800000011920929));
    assert_eq!(
        result.runs[0].rows[1].disposition,
        "no_corresponding_content"
    );
    let filter = QueryPredicate::boolean(false);
    let filtered = service
        .evaluate_similarity_candidates(SimilarityEvaluationRequest {
            seed: &seed,
            profile: &profile,
            variants: &variants,
            candidate_keys: &keys,
            filter: Some(&filter),
            cancelled: None,
        })
        .unwrap();
    assert_eq!(filtered.eligible_candidates, 0);
    assert!(
        filtered.runs[0]
            .rows
            .iter()
            .all(|row| row.disposition == "ineligible_or_seed")
    );
}
#[test]
fn direct_oracle_finds_a_winner_absent_from_every_bounded_knn_lane() {
    let mut records = vec![record(
        "hazards",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Seed","type":"hazard","system":{"details":{"description":"<p>One seed passage</p>"}}}),
    )];
    for i in 0..300 {
        records.push(record("hazards","Actor",json!({"_id":format!("b{i:015}"),"name":format!("Distractor {i}"),"type":"hazard","system":{"details":{"description":"<p>First section</p>","reset":"<p>Second section</p>"}}})));
    }
    records.push(record("hazards","Actor",json!({"_id":"cccccccccccccccc","name":"Centroid winner","type":"hazard","system":{"details":{"description":"<p>First section</p>","reset":"<p>Second section</p>"}}})));
    let mut input = crate::test_support::input(records, true);
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.0);
        unit.vector[0] = 1.0;
        if unit.record.id().as_str().starts_with('b')
            && unit.field.as_deref() == Some("/system/details/reset")
        {
            unit.vector[0] = -1.0;
        }
        if unit.record.id().as_str() == "cccccccccccccccc" {
            unit.vector[0] = 0.8;
            unit.vector[1] = if unit.field.as_deref() == Some("/system/details/reset") {
                -0.6
            } else {
                0.6
            };
        }
    }
    let (service, _artifact) = open_input_fixture(input).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let profile = SearchProfileId::new("bge-base");
    let options = SimilarityOptions::ContentAverage {
        parts: None,
        candidate_parts: SimilarityCandidateParts::Any,
    };
    let keys = (0..300)
        .map(|i| format!("hazards:b{i:015}").parse().unwrap())
        .chain(std::iter::once("hazards:cccccccccccccccc".parse().unwrap()))
        .collect::<Vec<_>>();
    let bounded = service
        .configurable_similar(ConfigurableSimilarRequest {
            source_sections: None,
            content_scope: atlas_domain::SearchContentScope::All,
            seed: &seed,
            options: &options,
            search_profile: Some(&profile),
            filter: None,
            scope: RecordScope::Keys(&keys),
            page: SearchPage::default(),
            sort: ResultSort::Relevance {},
            prefer_remaster: false,
            cancelled: None,
        })
        .unwrap()
        .unwrap();
    assert!(
        !bounded
            .candidate_keys
            .unwrap()
            .contains(keys.last().unwrap())
    );
    let variants = [SimilarityEvaluationVariant {
        id: "average".into(),
        options,
    }];
    let oracle = service
        .evaluate_similarity_candidates(SimilarityEvaluationRequest {
            seed: &seed,
            profile: &profile,
            variants: &variants,
            candidate_keys: &keys,
            filter: None,
            cancelled: None,
        })
        .unwrap();
    assert_eq!(oracle.eligible_candidates, 301);
    assert_eq!(oracle.runs[0].rows.len(), 301);
    assert_eq!(oracle.runs[0].rows[0].record_key, *keys.last().unwrap());
    assert!((oracle.runs[0].rows[0].score.unwrap() - 1.0).abs() < 1e-6);
}
#[test]
fn duplicate_keys_and_cancellation_are_explicit_failures() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let profile = SearchProfileId::new("bge-base");
    let variants = variants();
    let keys = [
        "hazards:bbbbbbbbbbbbbbbb".parse().unwrap(),
        "hazards:bbbbbbbbbbbbbbbb".parse().unwrap(),
    ];
    assert!(
        service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys,
                filter: None,
                cancelled: None
            })
            .is_err()
    );
    assert_eq!(
        service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys[..1],
                filter: None,
                cancelled: Some(&|| true)
            })
            .unwrap_err()
            .kind(),
        crate::SearchErrorKind::OperationCancelled
    );
}

#[test]
fn label_only_trait_baseline_and_content_oracles_match_both_product_profiles() {
    let mut data = fixture();
    let trait_key = atlas_domain::TraitKey::new("amp").unwrap();
    data.traits.push(atlas_index::TraitDefinition {
        key: trait_key.clone(),
        label: "Amp".into(),
        label_key: None,
        description_key: None,
        description_html: None,
        authored_markup: None,
        prepared: None,
        catalogs: vec![],
        source_locations: vec![],
        source_revision: data.context.source_revision.clone(),
        indexing_locale: data.context.indexing_locale.clone(),
    });
    let source = "Amp\n";
    let hash = crate::test_support::hash(source.as_bytes());
    for profile in ["bge-base", "embeddinggemma"] {
        let mut vector = vec![0.0; 768];
        vector[0] = 1.0;
        data.trait_vectors.push(atlas_index::TraitVectorInput {
            key: trait_key.clone(),
            profile_id: SearchProfileId::new(profile),
            input_hash: crate::test_support::hash(
                format!("trait-definition-centroid/v1:{hash}").as_bytes(),
            ),
            vector: vector.clone(),
            chunks: vec![atlas_index::TraitSeedChunk {
                input: source.into(),
                input_hash: hash.clone(),
                token_count: 2,
                vector,
                address: atlas_domain::SourcePassageAddress::PlainSection {
                    source_text_sha256: hash.clone(),
                    selection_version: "trait-definition-centroid/v1".into(),
                    section_ordinal: 0,
                    label: Some("Amp".into()),
                    chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 4 },
                },
            }],
        });
    }
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let seed = SimilaritySeed::Trait { trait_key };
    let keys = [
        "hazards:aaaaaaaaaaaaaaaa",
        "hazards:bbbbbbbbbbbbbbbb",
        "spells:cccccccccccccccc",
    ]
    .map(|key| key.parse().unwrap());
    let variants = variants()
        .into_iter()
        .filter(|variant| variant.options.candidate_parts() == SimilarityCandidateParts::Any)
        .collect::<Vec<_>>();
    for profile in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(profile);
        let exact = service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys,
                filter: None,
                cancelled: None,
            })
            .unwrap();
        assert_eq!(exact.eligible_candidates, 3);
        for (variant, run) in variants.iter().zip(&exact.runs) {
            let bounded = service
                .configurable_similar(ConfigurableSimilarRequest {
                    source_sections: None,
                    content_scope: atlas_domain::SearchContentScope::All,
                    seed: &seed,
                    options: &variant.options,
                    search_profile: Some(&profile),
                    filter: None,
                    scope: RecordScope::Keys(&keys),
                    page: SearchPage::default(),
                    sort: ResultSort::Relevance {},
                    prefer_remaster: false,
                    cancelled: None,
                })
                .unwrap()
                .unwrap();
            for row in bounded.results.records {
                assert!(
                    (row.score
                        - run
                            .rows
                            .iter()
                            .find(|scored| scored.record_key == row.record.key)
                            .unwrap()
                            .score
                            .unwrap())
                    .abs()
                        < 1e-6
                );
            }
        }
    }
}

#[test]
fn undefined_seed_and_candidate_centroids_are_distinguished_from_zero_matches() {
    for undefined_seed in [true, false] {
        let mut data = fixture();
        let broken = if undefined_seed {
            "aaaaaaaaaaaaaaaa"
        } else {
            "bbbbbbbbbbbbbbbb"
        };
        for unit in &mut data.semantic_units {
            if unit.record.id().as_str() == broken {
                unit.vector.fill(0.0);
                unit.vector[0] = if unit.field.as_deref() == Some("/system/details/reset") {
                    -1.0
                } else {
                    1.0
                };
            }
        }
        let (service, _artifact) = open_input_fixture(data).unwrap();
        let seed = SimilaritySeed::Record {
            record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
        };
        let profile = SearchProfileId::new("bge-base");
        let variants = [SimilarityEvaluationVariant {
            id: "average".into(),
            options: SimilarityOptions::ContentAverage {
                parts: None,
                candidate_parts: SimilarityCandidateParts::Any,
            },
        }];
        let keys = ["hazards:bbbbbbbbbbbbbbbb".parse().unwrap()];
        let result = service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys,
                filter: None,
                cancelled: None,
            })
            .unwrap();
        assert_eq!(result.runs[0].rows[0].score, None);
        assert_eq!(
            result.runs[0].rows[0].disposition,
            if undefined_seed {
                "undefined_seed_centroid"
            } else {
                "undefined_candidate_centroid"
            }
        );
    }
}
#[test]
fn cancellation_during_a_matrix_does_not_return_a_partial_success() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let profile = SearchProfileId::new("bge-base");
    let variants = variants();
    let keys = [
        "hazards:bbbbbbbbbbbbbbbb".parse().unwrap(),
        "spells:cccccccccccccccc".parse().unwrap(),
    ];
    let checks = std::cell::Cell::new(0usize);
    let cancelled = || {
        checks.set(checks.get() + 1);
        checks.get() > 6
    };
    let result = service.evaluate_similarity_candidates(SimilarityEvaluationRequest {
        seed: &seed,
        profile: &profile,
        variants: &variants,
        candidate_keys: &keys,
        filter: None,
        cancelled: Some(&cancelled),
    });
    assert_eq!(
        result.unwrap_err().kind(),
        crate::SearchErrorKind::OperationCancelled
    );
    assert!(checks.get() > 6);
}

#[test]
fn exact_custom_identity_is_scored_and_versioned_instead_of_silently_dropped() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let identity = service
        .similarity_parts(&seed, None)
        .unwrap()
        .parts
        .into_iter()
        .find(|component| component.is_identity())
        .unwrap()
        .selector()
        .clone();
    let variants = [SimilarityEvaluationVariant {
        id: "identity".into(),
        options: SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(vec![identity]),
            candidate_parts: SimilarityCandidateParts::Corresponding,
        },
    }];
    let keys = ["hazards:bbbbbbbbbbbbbbbb".parse().unwrap()];
    for profile in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(profile);
        let exact = service
            .evaluate_similarity_candidates(SimilarityEvaluationRequest {
                seed: &seed,
                profile: &profile,
                variants: &variants,
                candidate_keys: &keys,
                filter: None,
                cancelled: None,
            })
            .unwrap();
        assert!(exact.content_candidate_identity);
        assert_eq!(
            exact.runs[0].recipe,
            atlas_domain::SIMILARITY_COMPONENT_RECIPE
        );
        assert_eq!(exact.runs[0].rows[0].parts.len(), 1);
        assert!(exact.runs[0].rows[0].parts[0].part.is_identity());
        let product = service
            .configurable_similar(ConfigurableSimilarRequest {
                seed: &seed,
                options: &variants[0].options,
                source_sections: None,
                content_scope: atlas_domain::SearchContentScope::All,
                search_profile: Some(&profile),
                filter: None,
                scope: RecordScope::Keys(&keys),
                page: Default::default(),
                sort: ResultSort::Relevance {},
                prefer_remaster: false,
                cancelled: None,
            })
            .unwrap()
            .unwrap();
        assert_eq!(
            exact.runs[0].rows[0].score,
            Some(product.results.records[0].score)
        );
    }
}
