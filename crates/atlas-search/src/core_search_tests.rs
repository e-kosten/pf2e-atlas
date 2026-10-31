//! Core policy is already independently tested; these fixtures protect retrieval composition.
use super::*;
use crate::test_support::{input, input_with_both_profiles, open_input_fixture, record};
use atlas_domain::{
    CoreFallbackReason, CoreSectionRepresentation, SearchContentScope, SimilarityCandidateParts,
    SimilarityOptions, SimilaritySeed, SimilaritySourceSections, SourcePassageAddress,
};
use serde_json::json;

fn request<'a>(
    seed: &'a SimilaritySeed,
    options: &'a SimilarityOptions,
) -> ConfigurableSimilarRequest<'a> {
    ConfigurableSimilarRequest {
        seed,
        options,
        source_sections: None,
        content_scope: SearchContentScope::All,
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::first(50).unwrap(),
        sort: atlas_domain::ResultSort::Relevance {},
        prefer_remaster: false,
        cancelled: None,
    }
}
fn fixture() -> atlas_index::IndexBuildInput {
    let mut data = input(
        vec![
            record(
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Seed","type":"npc","system":{"details":{"publicNotes":"<p>Seed description</p>"}},"items":[{"_id":"seedAbility00001","name":"Extra ability","type":"action","system":{"description":{"value":"<p>Extra prose</p>"}}}]}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Carrier","type":"npc","system":{"details":{"publicNotes":"<p>Unrelated description</p>"}},"items":[{"_id":"ownedItem0000001","name":"Fireball","type":"consumable","system":{"description":{"value":"<p>Matching carried prose</p>"}}}]}),
            ),
            record(
                "spells",
                "Item",
                json!({"_id":"cccccccccccccccc","name":"Fireball","type":"spell","system":{"description":{"value":"<p>Matching standalone prose</p>"}}}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"dddddddddddddddd","name":"Sparse","type":"npc","system":{}}),
            ),
            record(
                "journals",
                "JournalEntry",
                json!({"_id":"eeeeeeeeeeeeeeee","name":"Reference book","pages":[]}),
            ),
        ],
        true,
    );
    for unit in &mut data.semantic_units {
        let axis = match unit.record.id().as_str() {
            "aaaaaaaaaaaaaaaa" => usize::from(!unit.owners.is_empty()),
            "bbbbbbbbbbbbbbbb" => usize::from(unit.owners.is_empty()),
            "cccccccccccccccc" => {
                usize::from(matches!(unit.address, SourcePassageAddress::Identity {}))
            }
            _ => 0,
        };
        unit.vector.fill(0.);
        unit.vector[axis] = 1.;
    }
    data
}

#[test]
fn core_candidate_scope_excludes_carried_matches_and_preserves_standalone_prose() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::default();
    let mut query = request(&seed, &options);
    query.source_sections = Some(SimilaritySourceSections::Core);
    let all = service.configurable_similar(query).unwrap().unwrap();
    assert!(
        all.results
            .records
            .iter()
            .any(|row| row.record.key.to_string() == "actors:bbbbbbbbbbbbbbbb")
    );
    let mut query = request(&seed, &options);
    query.source_sections = Some(SimilaritySourceSections::Core);
    query.content_scope = SearchContentScope::Core;
    let core = service.configurable_similar(query).unwrap().unwrap();
    assert!(
        !core
            .results
            .records
            .iter()
            .any(|row| row.record.key.to_string() == "actors:bbbbbbbbbbbbbbbb")
    );
    assert!(
        core.results
            .records
            .iter()
            .any(|row| row.record.key.to_string() == "spells:cccccccccccccccc")
    );
    assert!(
        core.evidence
            .iter()
            .flat_map(|(_, evidence)| &evidence.pairs)
            .all(|pair| pair.candidate.location.owners.is_empty())
    );
    assert_eq!(core.source_sections, Some(SimilaritySourceSections::Core));
    assert_eq!(core.content_scope, SearchContentScope::Core);
    assert!(core.candidate_identity);
    assert_eq!(core.resolved_selectors.len(), 2);
    assert_eq!(core.recipe, atlas_domain::SIMILARITY_COMPONENT_RECIPE);
}

#[test]
fn custom_prose_only_keeps_its_recipe_and_core_enables_identity_in_all_candidates() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(
            service
                .similarity_parts(&seed, None)
                .unwrap()
                .parts
                .iter()
                .filter(|component| !component.is_identity())
                .map(|component| component.selector().clone())
                .collect(),
        ),
        candidate_parts: SimilarityCandidateParts::Corresponding,
    };
    let all = service
        .configurable_similar(request(&seed, &options))
        .unwrap()
        .unwrap();
    assert!(!all.candidate_identity);
    assert_eq!(all.recipe, atlas_domain::SIMILARITY_RECIPE);
    assert_eq!(all.resolved_selectors.len(), 2);
    assert!(
        all.resolved_selectors
            .iter()
            .all(|selector| selector.0.starts_with("part:"))
    );
    let core_options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: None,
        candidate_parts: SimilarityCandidateParts::Corresponding,
    };
    let mut query = request(&seed, &core_options);
    query.source_sections = Some(SimilaritySourceSections::Core);
    let core = service.configurable_similar(query).unwrap().unwrap();
    let sparse = core
        .evidence
        .iter()
        .find(|(key, _)| key.to_string() == "actors:dddddddddddddddd")
        .unwrap();
    assert!(core.candidate_identity);
    assert_eq!(core.content_scope, SearchContentScope::All);
    assert!(
        sparse
            .1
            .parts
            .iter()
            .any(|part| part.part.is_identity() && part.comparable && part.score > 0.)
    );
    assert!(sparse.1.pairs.iter().any(|pair| matches!(
        pair.candidate.location.address,
        Some(SourcePassageAddress::Identity {})
    ) && pair.candidate.location.field.is_none()));
}

#[test]
fn all_sources_include_identity_once_and_match_exact_custom_components_for_both_profiles() {
    let mut data = input_with_both_profiles(
        ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"]
            .into_iter()
            .map(|id| {
                record(
                    "actors",
                    "Actor",
                    json!({
                        "_id":id,"name":id,"type":"npc",
                        "system":{"details":{"publicNotes":"<p>Root description</p>"}},
                        "items":[{"_id":"ownedAction00001","type":"action","name":"Action",
                            "system":{"description":{"value":"<p>Owned ability</p>"}}}]
                    }),
                )
            })
            .collect(),
    );
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        let axis = if matches!(unit.address, SourcePassageAddress::Identity {}) {
            0
        } else if unit.owners.is_empty() {
            1
        } else {
            2
        };
        unit.vector[axis] = 1.;
    }
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    for profile in ["bge-base", "embeddinggemma"] {
        let profile = atlas_domain::SearchProfileId::new(profile);
        let inventory = service.similarity_parts(&seed, Some(&profile)).unwrap();
        let selectors = inventory
            .parts
            .iter()
            .map(|part| part.selector().clone())
            .collect::<Vec<_>>();
        assert_eq!(selectors.len(), 3);
        for mode in [
            SimilarityCandidateParts::Any,
            SimilarityCandidateParts::Corresponding,
        ] {
            for average in [false, true] {
                let make_options = |parts| {
                    if average {
                        SimilarityOptions::ContentAverage {
                            parts,
                            candidate_parts: mode,
                        }
                    } else {
                        SimilarityOptions::Passages {
                            parts,
                            candidate_parts: mode,
                            decay: 0.5,
                        }
                    }
                };
                let custom_options = make_options(Some(selectors.clone()));
                let mut custom = request(&seed, &custom_options);
                custom.search_profile = Some(&profile);
                let custom = service.configurable_similar(custom).unwrap().unwrap();
                for preset in [None, Some(SimilaritySourceSections::All)] {
                    let options = make_options(None);
                    let mut query = request(&seed, &options);
                    query.source_sections = preset;
                    query.search_profile = Some(&profile);
                    let all = service.configurable_similar(query).unwrap().unwrap();
                    assert_eq!(all.source_sections, Some(SimilaritySourceSections::All));
                    assert_eq!(all.resolved_selectors, custom.resolved_selectors);
                    assert_eq!(all.diagnostics.seed_units, 3);
                    assert!(all.candidate_identity);
                    assert_eq!(all.recipe, atlas_domain::SIMILARITY_COMPONENT_RECIPE);
                    assert_eq!(all.results.records.len(), 1);
                    assert_eq!(
                        all.results.records[0].score,
                        custom.results.records[0].score
                    );
                    assert!((all.results.records[0].score - 1.).abs() < 1e-6);
                    let evidence = &all.evidence[0].1;
                    assert_eq!(evidence.selected_parts, 3);
                    if !average {
                        assert_eq!(
                            evidence
                                .parts
                                .iter()
                                .filter(|part| part.part.is_identity())
                                .count(),
                            1
                        );
                    } else if mode == SimilarityCandidateParts::Corresponding {
                        assert_eq!(
                            evidence
                                .roles
                                .iter()
                                .filter(|role| role.key
                                    == atlas_domain::SimilarityComparisonKey::Identity)
                                .count(),
                            1
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn identity_only_fallback_keeps_each_content_method_and_reports_why() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    for (key, reason) in [
        ("actors:dddddddddddddddd", CoreFallbackReason::NoCoreProse),
        (
            "journals:eeeeeeeeeeeeeeee",
            CoreFallbackReason::UnmodelledKind,
        ),
    ] {
        let seed = SimilaritySeed::Record {
            record_key: key.parse().unwrap(),
        };
        for options in [
            SimilarityOptions::default(),
            SimilarityOptions::ContentAverage {
                parts: None,
                candidate_parts: SimilarityCandidateParts::Corresponding,
            },
        ] {
            let mut query = request(&seed, &options);
            query.source_sections = Some(SimilaritySourceSections::Core);
            let result = service.configurable_similar(query).unwrap().unwrap();
            assert_eq!(result.effective, options);
            assert_eq!(result.diagnostics.seed_units, 1);
            assert_eq!(result.resolved_selectors.len(), 1);
            let selection = result.core.unwrap();
            assert_eq!(
                selection.representation,
                CoreSectionRepresentation::IdentityOnly
            );
            assert_eq!(selection.fallback_reason, Some(reason));
            let all = service
                .configurable_similar(request(&seed, &options))
                .unwrap()
                .unwrap();
            assert_eq!(all.source_sections, Some(SimilaritySourceSections::All));
            assert_eq!(all.effective, options);
            assert_eq!(all.diagnostics.seed_units, 1);
            assert_eq!(all.resolved_selectors.len(), 1);
            assert!(all.candidate_identity);
            assert!(all.core.is_none());
            assert_eq!(all.recipe, atlas_domain::SIMILARITY_COMPONENT_RECIPE);
        }
    }
}

#[test]
fn explicit_identity_custom_is_preserved_and_conflicting_source_authorities_are_rejected() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let inventory = service.similarity_parts(&seed, None).unwrap();
    let identity = inventory
        .parts
        .iter()
        .find(|component| component.is_identity())
        .unwrap()
        .selector()
        .clone();
    let options = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(vec![identity.clone()]),
        candidate_parts: SimilarityCandidateParts::Corresponding,
    };
    let result = service
        .configurable_similar(request(&seed, &options))
        .unwrap()
        .unwrap();
    assert_eq!(
        result.source_sections,
        Some(SimilaritySourceSections::Custom)
    );
    assert_eq!(result.resolved_selectors, vec![identity]);
    assert!(result.candidate_identity);
    for mode in [
        SimilaritySourceSections::All,
        SimilaritySourceSections::Core,
    ] {
        let mut query = request(&seed, &options);
        query.source_sections = Some(mode);
        assert_eq!(
            service.configurable_similar(query).unwrap_err().kind(),
            SearchErrorKind::InvalidOptions
        );
    }
    let default = SimilarityOptions::default();
    let mut query = request(&seed, &default);
    query.source_sections = Some(SimilaritySourceSections::Custom);
    assert_eq!(
        service.configurable_similar(query).unwrap_err().kind(),
        SearchErrorKind::InvalidOptions
    );
}

#[test]
fn baseline_core_masks_candidates_but_rejects_every_source_preset() {
    let (service, _artifact) = open_input_fixture(fixture()).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::Baseline {};
    let mut query = request(&seed, &options);
    query.content_scope = SearchContentScope::Core;
    let result = service.configurable_similar(query).unwrap().unwrap();
    assert!(result.candidate_identity);
    assert_eq!(result.source_sections, None);
    assert!(result.resolved_selectors.is_empty());
    assert_eq!(
        result.core_recipe.as_deref(),
        Some(atlas_domain::CORE_SECTIONS_RECIPE)
    );
    let carrier = result
        .results
        .records
        .iter()
        .find(|row| row.record.key.to_string() == "actors:bbbbbbbbbbbbbbbb")
        .unwrap();
    assert_eq!(carrier.score, 0.);
    assert!(
        carrier
            .matches
            .iter()
            .all(|witness| witness.location.owners.is_empty())
    );
    let all = service
        .configurable_similar(request(&seed, &options))
        .unwrap()
        .unwrap();
    assert_eq!(
        all.results
            .records
            .iter()
            .find(|row| row.record.key.to_string() == "actors:bbbbbbbbbbbbbbbb")
            .unwrap()
            .score,
        1.
    );
    for mode in [
        SimilaritySourceSections::All,
        SimilaritySourceSections::Core,
        SimilaritySourceSections::Custom,
    ] {
        let mut query = request(&seed, &options);
        query.source_sections = Some(mode);
        assert_eq!(
            service.configurable_similar(query).unwrap_err().kind(),
            SearchErrorKind::InvalidOptions
        );
    }
}

#[test]
fn ordinary_core_fts_filters_owned_names_without_expanding_prose_coverage() {
    let (mut service, _artifact) = open_input_fixture(fixture()).unwrap();
    let mut query = TextSearchRequest::new("Fireball", SearchPage::first(50).unwrap());
    query.mode = RetrievalMode::Fts;
    let all = service.search_text(query).unwrap();
    assert_eq!(all.records.len(), 2);
    let mut query = TextSearchRequest::new("Fireball", SearchPage::first(50).unwrap());
    query.mode = RetrievalMode::Fts;
    query.content_scope = SearchContentScope::Core;
    let core = service.search_text(query).unwrap();
    assert_eq!(core.records.len(), 1);
    assert_eq!(
        core.records[0].record.key.to_string(),
        "spells:cccccccccccccccc"
    );
    assert_eq!(core.coverage.candidate_roots, 1);
    let mut query = TextSearchRequest::new("standalone", SearchPage::first(50).unwrap());
    query.mode = RetrievalMode::Fts;
    query.content_scope = SearchContentScope::Core;
    assert!(service.search_text(query).unwrap().records.is_empty());
}

#[test]
fn record_text_context_is_unavailable_for_all_facets_without_inference() {
    let (mut service, _artifact) = open_input_fixture(fixture()).unwrap();
    for content_scope in [SearchContentScope::Core, SearchContentScope::All] {
        let context = FilterDiscoveryContext {
            content_scope,
            search_profile: None,
            text: Some("Fireball"),
            scope: RecordScope::All,
            prefer_remaster: false,
            filter: None,
        };
        service.reset_read_metrics();
        assert!(matches!(
            service
                .discover_filter_counts(DiscoverFilterCountsRequest {
                    field: "level",
                    clause_id: None,
                    context,
                })
                .unwrap(),
            atlas_domain::QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::TextContext
            }
        ));
        assert!(matches!(
            service
                .discover_filter_values(DiscoverFilterValuesRequest {
                    field: "rarity",
                    clause_id: None,
                    context,
                    only_matching: true,
                    text: None,
                    offset: 0,
                    limit: 10,
                })
                .unwrap(),
            atlas_domain::QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::TextContext
            }
        ));
        assert!(matches!(
            service
                .discover_filter_value_counts(DiscoverFilterValueCountsRequest {
                    field: "rarity",
                    clause_id: None,
                    context,
                    values: &[atlas_domain::QueryLiteral::String("common".into())],
                })
                .unwrap(),
            atlas_domain::QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::TextContext
            }
        ));
        assert_eq!(service.read_metrics().knn_queries, 0);
    }
    let error = service
        .discover_filter_counts(DiscoverFilterCountsRequest {
            field: "level",
            clause_id: None,
            context: FilterDiscoveryContext {
                content_scope: SearchContentScope::Core,
                search_profile: None,
                text: None,
                scope: RecordScope::All,
                prefer_remaster: false,
                filter: None,
            },
        })
        .unwrap_err();
    assert_eq!(error.kind(), SearchErrorKind::InvalidOptions);
}

#[test]
fn identity_consumes_one_of_the_existing_seed_vector_slots() {
    for sections in [127, 128] {
        let prose = (0..sections)
            .map(|n| format!("<h2>Section {n}</h2><p>Distinct passage {n}.</p>"))
            .collect::<String>();
        let data = input(
            vec![
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"aaaaaaaaaaaaaaaa","name":"Large seed","type":"npc","system":{"details":{"publicNotes":prose}}}),
                ),
                record(
                    "spells",
                    "Item",
                    json!({"_id":"bbbbbbbbbbbbbbbb","name":"Candidate","type":"spell","system":{"description":{"value":"<p>Candidate description</p>"}}}),
                ),
            ],
            true,
        );
        let (service, _artifact) = open_input_fixture(data).unwrap();
        let seed = SimilaritySeed::Record {
            record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
        };
        let options = SimilarityOptions::default();
        let prose_only = SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(
                service
                    .similarity_parts(&seed, None)
                    .unwrap()
                    .parts
                    .iter()
                    .filter(|component| !component.is_identity())
                    .map(|component| component.selector().clone())
                    .collect(),
            ),
            candidate_parts: SimilarityCandidateParts::Any,
        };
        let custom = service
            .configurable_similar(request(&seed, &prose_only))
            .unwrap()
            .unwrap();
        assert_eq!(custom.diagnostics.seed_units, sections);
        for mode in [
            SimilaritySourceSections::All,
            SimilaritySourceSections::Core,
        ] {
            let mut query = request(&seed, &options);
            query.source_sections = Some(mode);
            if sections == 127 {
                assert_eq!(
                    service
                        .configurable_similar(query)
                        .unwrap()
                        .unwrap()
                        .diagnostics
                        .seed_units,
                    128
                );
            } else {
                let error = service.configurable_similar(query).unwrap_err();
                assert_eq!(error.kind(), SearchErrorKind::InvalidOptions);
                assert!(error.to_string().contains("128"));
                assert!(
                    error
                        .to_string()
                        .contains("Deselect some sections under Sections to compare")
                );
            }
        }
    }
}

#[test]
fn core_without_stored_profile_vectors_is_an_explicit_readiness_failure() {
    let mut data = fixture();
    data.context.search_profiles.clear();
    data.semantic_units.clear();
    let (service, _artifact) = open_input_fixture(data).unwrap();
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::default();
    let mut query = request(&seed, &options);
    query.source_sections = Some(SimilaritySourceSections::Core);
    let error = service.configurable_similar(query).unwrap_err();
    assert_eq!(error.kind(), SearchErrorKind::VectorReadinessRequired);
}

#[test]
fn ordinary_core_vector_and_hybrid_search_use_eligible_root_maxima_for_both_profiles() {
    let mut data = crate::test_support::input_with_both_profiles(vec![
        record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Carrier","type":"npc","system":{"details":{"publicNotes":"<h2>Ember physiology</h2><p>Root prose</p>"}},"items":[{"_id":"ownedItem0000001","name":"Ember weapon","type":"consumable","system":{"description":{"value":"<p>Carried prose</p>"}}}]}),
        ),
        record(
            "actors",
            "Actor",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Sparse","type":"npc","system":{}}),
        ),
        record(
            "spells",
            "Item",
            json!({"_id":"cccccccccccccccc","name":"Prose winner","type":"spell","system":{"description":{"value":"<h2>Ember spell</h2><p>Standalone prose</p>"}}}),
        ),
    ]);
    for unit in &mut data.semantic_units {
        let cosine: f32 = match unit.record.id().as_str() {
            "aaaaaaaaaaaaaaaa" if !unit.owners.is_empty() => 1.,
            "aaaaaaaaaaaaaaaa" if matches!(unit.address, SourcePassageAddress::Identity {}) => 0.6,
            "aaaaaaaaaaaaaaaa" => 0.3,
            "bbbbbbbbbbbbbbbb" => 0.8,
            "cccccccccccccccc" if matches!(unit.address, SourcePassageAddress::Identity {}) => 0.,
            "cccccccccccccccc" => 0.9,
            _ => unreachable!(),
        };
        unit.vector.fill(0.);
        unit.vector[0] = cosine;
        unit.vector[1] = (1. - cosine * cosine).sqrt();
    }
    let (fixture_service, artifact) = open_input_fixture(data).unwrap();
    for profile in ["bge-base", "embeddinggemma"] {
        let profile = atlas_domain::SearchProfileId::new(profile);
        let model = fixture_service
            .artifact_context()
            .search_profile(&profile)
            .unwrap()
            .clone();
        let mut query_vector = vec![0.; model.dimensions];
        query_vector[0] = 1.;
        let mut service = AtlasRetrievalService::from_index(
            atlas_index::test_support::open_reader(artifact.path()).unwrap(),
            profile.clone(),
            Some(PreparedQueryEmbedding::for_fixture(
                model,
                "Ember",
                query_vector,
            )),
        )
        .unwrap();
        for mode in [RetrievalMode::Vector, RetrievalMode::Hybrid] {
            for ranking in [
                atlas_domain::SearchRanking::Score,
                atlas_domain::SearchRanking::Rrf,
            ] {
                let mut query = TextSearchRequest::new("Ember", SearchPage::first(50).unwrap());
                query.search_profile = Some(&profile);
                query.mode = mode;
                query.ranking = ranking;
                let all = service.search_text(query).unwrap();
                let carrier = all
                    .records
                    .iter()
                    .find(|row| row.record.key.id().as_str() == "aaaaaaaaaaaaaaaa")
                    .unwrap();
                assert!((carrier.semantic_similarity.unwrap() - 1.).abs() < 1e-6);
                assert!(carrier.matches.iter().any(|witness| {
                    !witness.location.owners.is_empty()
                        && witness
                            .semantic_similarity
                            .is_some_and(|cosine| (cosine - 1.).abs() < 1e-6)
                }));
                let mut query = TextSearchRequest::new("Ember", SearchPage::first(50).unwrap());
                query.search_profile = Some(&profile);
                query.mode = mode;
                query.ranking = ranking;
                query.content_scope = SearchContentScope::Core;
                let core = service.search_text(query).unwrap();
                assert_eq!(core.search_profile, profile);
                assert_eq!(core.page.total, 3);
                assert_eq!(core.coverage.candidate_roots, 3);
                assert!(
                    core.records
                        .iter()
                        .flat_map(|row| &row.matches)
                        .all(|witness| witness.location.owners.is_empty())
                );
                let carrier = core
                    .records
                    .iter()
                    .find(|row| row.record.key.id().as_str() == "aaaaaaaaaaaaaaaa")
                    .unwrap();
                assert!((carrier.semantic_similarity.unwrap() - 0.6).abs() < 1e-6);
                assert!(carrier.matches.iter().any(|witness| {
                    matches!(
                        witness.location.address,
                        Some(SourcePassageAddress::Identity {})
                    ) && witness.location.field.is_none()
                        && witness
                            .semantic_similarity
                            .is_some_and(|cosine| (cosine - 0.6).abs() < 1e-6)
                }));
                let sparse = core
                    .records
                    .iter()
                    .find(|row| row.record.key.id().as_str() == "bbbbbbbbbbbbbbbb")
                    .unwrap();
                assert!((sparse.semantic_similarity.unwrap() - 0.8).abs() < 1e-6);
                assert!(sparse.matches.iter().all(|witness| matches!(
                    witness.location.address,
                    Some(SourcePassageAddress::Identity {})
                ) && witness.location.field.is_none()));
                let prose = core
                    .records
                    .iter()
                    .find(|row| row.record.key.id().as_str() == "cccccccccccccccc")
                    .unwrap();
                assert!((prose.semantic_similarity.unwrap() - 0.9).abs() < 1e-6);
                assert!(prose.matches.iter().any(|witness| {
                    witness.location.field.as_deref() == Some("/system/description/value")
                        && witness
                            .semantic_similarity
                            .is_some_and(|cosine| (cosine - 0.9).abs() < 1e-6)
                }));
                assert!(prose.score > sparse.score);
                if mode == RetrievalMode::Vector {
                    assert_eq!(
                        core.records
                            .iter()
                            .map(|row| row.record.key.id().as_str())
                            .collect::<Vec<_>>(),
                        ["cccccccccccccccc", "bbbbbbbbbbbbbbbb", "aaaaaaaaaaaaaaaa"]
                    );
                } else {
                    assert!(prose.lexical_rank.is_some());
                    assert!(sparse.lexical_rank.is_none());
                }
            }
        }
    }
}

#[test]
fn core_lexical_edition_preference_uses_scoped_matches_before_sorting_and_paging() {
    let mut data = input(
        vec![
            record(
                "actors",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Legacy sentinel","type":"npc","system":{"details":{"publicNotes":"<h2>Moonlit echo</h2><p>Root description</p>"}}}),
            ),
            record(
                "actors",
                "Actor",
                json!({"_id":"bbbbbbbbbbbbbbbb","name":"Modern sentinel","type":"npc","system":{"details":{"publicNotes":"<p>Unrelated description</p>"}},"items":[{"_id":"ownedAction00001","name":"Moonlit echo","type":"action","system":{"description":{"value":"<p>Unrelated ability prose</p>"}}}]}),
            ),
            record(
                "spells",
                "Item",
                json!({"_id":"cccccccccccccccc","name":"Zebra","type":"spell","system":{"description":{"value":"<h2>Moonlit spell</h2><p>Root prose</p>"}}}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"dddddddddddddddd","name":"Alpha","type":"effect","system":{"description":{"value":"<h2>Moonlit effect</h2><p>Root prose</p>"}}}),
            ),
        ],
        false,
    );
    let legacy: atlas_domain::RecordKey = "actors:aaaaaaaaaaaaaaaa".parse().unwrap();
    let remaster: atlas_domain::RecordKey = "actors:bbbbbbbbbbbbbbbb".parse().unwrap();
    let alpha: atlas_domain::RecordKey = "items:dddddddddddddddd".parse().unwrap();
    let zebra: atlas_domain::RecordKey = "spells:cccccccccccccccc".parse().unwrap();
    data.remaster_pairs
        .push(atlas_index::SourceRemasterPairInput {
            legacy: legacy.clone(),
            remaster: remaster.clone(),
            provenance: atlas_record::source_content::SourceContentLocator {
                record: legacy.clone(),
                owners: vec![],
                field: "/system/details/publicNotes".into(),
            },
        });
    let (mut service, _artifact) = open_input_fixture(data).unwrap();
    assert!(
        service
            .index
            .lookup_name_or_alias("Moonlit")
            .unwrap()
            .is_empty()
    );
    let mut query = TextSearchRequest::new("Moonlit", SearchPage::first(50).unwrap());
    query.mode = RetrievalMode::Fts;
    let all = service.search_text(query).unwrap();
    assert_eq!(all.page.total, 3);
    assert!(!all.records.iter().any(|row| row.record.key == legacy));
    let modern = all
        .records
        .iter()
        .find(|row| row.record.key == remaster)
        .unwrap();
    assert!(
        modern
            .matches
            .iter()
            .any(|witness| !witness.location.owners.is_empty())
    );
    let mut query = TextSearchRequest::new("Moonlit", SearchPage::first(50).unwrap());
    query.mode = RetrievalMode::Fts;
    query.content_scope = SearchContentScope::Core;
    let core = service.search_text(query).unwrap();
    assert_eq!(core.page.total, 3);
    assert!(!core.records.iter().any(|row| row.record.key == remaster));
    let retained = core
        .records
        .iter()
        .find(|row| row.record.key == legacy)
        .unwrap();
    assert!(
        retained
            .matches
            .iter()
            .all(|witness| witness.location.owners.is_empty()
                && witness.location.field.as_deref() == Some("/system/details/publicNotes"))
    );
    let expected_members = [legacy.clone(), alpha.clone(), zebra.clone()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    for sort in [
        atlas_domain::ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Asc,
        ),
        atlas_domain::ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Desc,
        ),
        atlas_domain::ResultSort::Random { seed: 17 },
    ] {
        let mut query = TextSearchRequest::new("Moonlit", SearchPage::first(50).unwrap());
        query.mode = RetrievalMode::Fts;
        query.content_scope = SearchContentScope::Core;
        query.sort = sort.clone();
        let ordered = service
            .search_text(query)
            .unwrap()
            .records
            .into_iter()
            .map(|row| row.record.key)
            .collect::<Vec<_>>();
        assert_eq!(
            ordered
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            expected_members
        );
        match &sort {
            atlas_domain::ResultSort::Attributes { rules }
                if rules[0].direction == atlas_domain::SortDirection::Asc =>
            {
                assert_eq!(ordered, [alpha.clone(), legacy.clone(), zebra.clone()])
            }
            atlas_domain::ResultSort::Attributes { .. } => {
                assert_eq!(ordered, [zebra.clone(), legacy.clone(), alpha.clone()])
            }
            _ => {}
        }
        for (index, expected) in ordered.iter().enumerate() {
            let mut query =
                TextSearchRequest::new("Moonlit", SearchPage::new(index as u32 + 1, 1).unwrap());
            query.mode = RetrievalMode::Fts;
            query.content_scope = SearchContentScope::Core;
            query.sort = sort.clone();
            let page = service.search_text(query).unwrap();
            assert_eq!(page.page.total, 3);
            assert_eq!(page.records.len(), 1);
            assert_eq!(&page.records[0].record.key, expected);
            assert_eq!(page.page.has_more, index < 2);
        }
        let mut query = TextSearchRequest::new("Moonlit", SearchPage::first(50).unwrap());
        query.mode = RetrievalMode::Fts;
        query.content_scope = SearchContentScope::Core;
        query.sort = sort;
        assert_eq!(
            service
                .search_text(query)
                .unwrap()
                .records
                .into_iter()
                .map(|row| row.record.key)
                .collect::<Vec<_>>(),
            ordered
        );
    }
}
