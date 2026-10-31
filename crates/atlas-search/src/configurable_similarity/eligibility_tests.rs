//! Complete configured-search parity against the original exact candidate producer.
use super::*;
use crate::AtlasRetrievalService;
use crate::test_support::{input_with_both_profiles, open_input_fixture, record};
use atlas_domain::{QueryPredicate, SearchProfileId};
use serde_json::{Value, json};
use std::{cell::Cell, path::Path};

fn both_profiles(mut data: atlas_index::IndexBuildInput) -> atlas_index::IndexBuildInput {
    let gemma =
        atlas_index::SourceSearchProfileIdentity::from_spec(atlas_embedding::SEARCH_PROFILES[1]);
    let mut units = data.semantic_units.clone();
    for unit in &mut units {
        unit.profile_id = gemma.profile_id.clone();
    }
    data.context.search_profiles.push(gemma);
    data.semantic_units.extend(units);
    data
}
fn open(path: &Path, profile: &SearchProfileId) -> AtlasRetrievalService {
    AtlasRetrievalService::from_index(
        atlas_index::test_support::open_reader(path).unwrap(),
        profile.clone(),
        None,
    )
    .unwrap()
}
fn witness(value: &SearchWitness) -> Value {
    json!({"value":value,"lexical_bits":value.lexical_rank.map(f64::to_bits),
        "semantic_bits":value.semantic_similarity.map(f64::to_bits)})
}
fn result_signature(result: &TextSearchResult) -> Value {
    json!({"profile":result.search_profile,"generation":result.artifact_generation,"sort":result.sort,
        "page":(result.page.number,result.page.size,result.page.count,result.page.total,result.page.has_more,result.page.next_page),
        "coverage":result.coverage,"rows":result.records.iter().map(|row|json!({
            "record":row.record,"matches":row.matches.iter().map(witness).collect::<Vec<_>>(),
            "lexical":row.lexical_rank,"score_bits":row.score.to_bits(),
            "similarity_bits":row.semantic_similarity.map(f64::to_bits)
        })).collect::<Vec<_>>()})
}
pub(super) fn output(result: &Result<Option<ConfigurableSimilarResult>, SearchError>) -> Value {
    match result {
        Err(error) => json!({"status":"error","kind":format!("{:?}",error.kind()),
            "message":error.to_string(),"query_error":error.query_error}),
        Ok(None) => json!({"status":"missing"}),
        Ok(Some(result)) => json!({"status":"ok","seed":result.seed,
            "results":result_signature(&result.results),
            "evidence":result.evidence.iter().map(|(key,value)|json!({
                "key":key,"score_bits":value.score.to_bits(),"aggregation":value.aggregation,
                "selected":value.selected_parts,"comparable":value.comparable_parts,
                "parts":value.parts.iter().map(|p|(&p.part,p.score.to_bits(),p.weight.to_bits(),p.comparable)).collect::<Vec<_>>(),
                "roles":value.roles.iter().map(|r|(&r.key,r.score.to_bits(),r.weight.to_bits(),r.comparable)).collect::<Vec<_>>(),
                "pairs":value.pairs.iter().map(|p|json!({"part":p.seed_part,"seed":p.seed.as_ref().map(witness),
                    "seed_text":p.seed_text,"candidate":witness(&p.candidate),"cosine_bits":p.cosine.to_bits()})).collect::<Vec<_>>(),
                "representative":value.representative,"truncated":value.truncated
            })).collect::<Vec<_>>(),"candidate_keys":result.candidate_keys,"effective":result.effective,
            "source_sections":result.source_sections,"content_scope":result.content_scope,
            "candidate_identity":result.candidate_identity,"core_recipe":result.core_recipe,
            "resolved_selectors":result.resolved_selectors,"core":result.core,
            "recipe":result.recipe,"role_recipe":result.role_recipe}),
    }
}
fn compare<'a>(
    path: &Path,
    profile: &SearchProfileId,
    request: impl Fn() -> ConfigurableSimilarRequest<'a>,
) -> Value {
    let candidate = open(path, profile).configurable_similar(request());
    let exact = open(path, profile).configurable_similar_exact(request());
    let actual = output(&candidate);
    assert_eq!(actual, output(&exact));
    actual
}
fn request<'a>(
    seed: &'a SimilaritySeed,
    profile: &'a SearchProfileId,
    options: &'a SimilarityOptions,
) -> ConfigurableSimilarRequest<'a> {
    ConfigurableSimilarRequest {
        seed,
        options,
        source_sections: None,
        content_scope: SearchContentScope::All,
        search_profile: Some(profile),
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::first(5).unwrap(),
        sort: ResultSort::Relevance {},
        prefer_remaster: true,
        cancelled: None,
    }
}

#[test]
fn exact_complete_outputs_survive_profiles_roles_scopes_filters_paging_and_order() {
    let (service, artifact) = open_input_fixture(both_profiles(super::tests::fixture())).unwrap();
    drop(service);
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let keys: Vec<RecordKey> = ["spells:bbbbbbbbbbbbbbbb", "hazards:cccccccccccccccc"]
        .into_iter()
        .map(|key| key.parse().unwrap())
        .collect();
    let reordered = vec![keys[1].clone(), keys[0].clone(), keys[1].clone()];
    let structured: QueryPredicate = serde_json::from_value(json!({
        "kind":"compare","clause_id":"kind-filter","field":"source.type","op":"eq","value":"hazard"
    }))
    .unwrap();
    let empty = QueryPredicate::boolean(false);
    for name in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(name);
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            for options in [
                SimilarityOptions::Passages {
                    decay: 0.5,
                    parts: None,
                    candidate_parts: SimilarityCandidateParts::Any,
                },
                SimilarityOptions::Passages {
                    decay: 0.25,
                    parts: None,
                    candidate_parts: SimilarityCandidateParts::Corresponding,
                },
                SimilarityOptions::ContentAverage {
                    parts: None,
                    candidate_parts: SimilarityCandidateParts::Any,
                },
                SimilarityOptions::ContentAverage {
                    parts: None,
                    candidate_parts: SimilarityCandidateParts::Corresponding,
                },
            ] {
                for case in 0..8 {
                    let (scope, filter, page, sort) = match case {
                        0 => (
                            RecordScope::All,
                            None,
                            SearchPage::first(1).unwrap(),
                            ResultSort::Relevance {},
                        ),
                        1 => (
                            RecordScope::All,
                            None,
                            SearchPage::first(5).unwrap(),
                            ResultSort::Relevance {},
                        ),
                        2 => (
                            RecordScope::All,
                            None,
                            SearchPage::new(2, 1).unwrap(),
                            ResultSort::Relevance {},
                        ),
                        3 => (
                            RecordScope::Keys(&keys),
                            Some(&structured),
                            SearchPage::first(5).unwrap(),
                            ResultSort::Random { seed: 42 },
                        ),
                        4 => (
                            RecordScope::Keys(&reordered),
                            None,
                            SearchPage::first(2).unwrap(),
                            ResultSort::attribute(
                                atlas_domain::SortField::Name,
                                atlas_domain::SortDirection::Desc,
                            ),
                        ),
                        5 => (
                            RecordScope::Keys(&[]),
                            None,
                            SearchPage::first(2).unwrap(),
                            ResultSort::Relevance {},
                        ),
                        6 => (
                            RecordScope::All,
                            Some(&structured),
                            SearchPage::first(2).unwrap(),
                            ResultSort::Relevance {},
                        ),
                        _ => (
                            RecordScope::All,
                            Some(&empty),
                            SearchPage::first(2).unwrap(),
                            ResultSort::Relevance {},
                        ),
                    };
                    let actual =
                        compare(&artifact.path(), &profile, || ConfigurableSimilarRequest {
                            content_scope,
                            scope,
                            filter,
                            page,
                            sort: sort.clone(),
                            ..request(&seed, &profile, &options)
                        });
                    assert_eq!(
                        actual["status"], "ok",
                        "{name} {content_scope:?} {options:?} case {case}"
                    );
                    if case == 5 || case == 7 {
                        assert_eq!(actual["results"]["rows"], json!([]));
                    }
                }
            }
        }
    }
}

#[test]
fn relationship_filter_membership_and_evidence_are_equal_on_both_paths() {
    let target = "aaaaaaaaaaaaaaaa";
    let records = vec![
        record(
            "items",
            "Item",
            json!({"_id":target,"name":"Seed","type":"effect","system":{"description":{"value":"<p>Target</p>"}}}),
        ),
        record(
            "items",
            "Item",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Linked","type":"effect","system":{"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{target}] @UUID[Compendium.pf2e.items.Item.{target}]</p>")}}}),
        ),
        record(
            "items",
            "Item",
            json!({"_id":"cccccccccccccccc","name":"Unlinked","type":"effect","system":{"description":{"value":"<p>Other</p>"}}}),
        ),
    ];
    let mut data = input_with_both_profiles(records);
    // The generic fixture gives each root an orthogonal vector. These cases
    // require positive matches, which configured scoring deliberately retains.
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    let (service, artifact) = open_input_fixture(data).unwrap();
    drop(service);
    let seed_key: RecordKey = format!("items:{target}").parse().unwrap();
    let seed = SimilaritySeed::Record {
        record_key: seed_key.clone(),
    };
    let filter = crate::relationship_tests::link_filter(
        None,
        &[crate::relationship_tests::criterion(
            crate::SourceRelationshipDirection::Incoming,
            seed_key,
            vec![],
            false,
        )],
    );
    let members = open(&artifact.path(), &SearchProfileId::new("bge-base"))
        .list_records(crate::ListRecordsRequest::new(
            Some(&filter),
            SearchPage::first(5).unwrap(),
        ))
        .unwrap()
        .record_keys;
    assert_eq!(
        members,
        vec!["items:bbbbbbbbbbbbbbbb".parse::<RecordKey>().unwrap()]
    );
    let options = SimilarityOptions::default();
    for name in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(name);
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            let actual = compare(&artifact.path(), &profile, || ConfigurableSimilarRequest {
                content_scope,
                filter: Some(&filter),
                ..request(&seed, &profile, &options)
            });
            assert_eq!(actual["status"], "ok");
            let rows = actual["results"]["rows"].as_array().unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["record"]["key"], "items:bbbbbbbbbbbbbbbb");
            assert!(f64::from_bits(rows[0]["score_bits"].as_u64().unwrap()) > 0.0);
        }
    }
}

#[test]
fn stale_missing_and_invalid_requests_keep_complete_error_parity() {
    let (service, artifact) = open_input_fixture(both_profiles(super::tests::fixture())).unwrap();
    drop(service);
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let missing = SimilaritySeed::Record {
        record_key: "hazards:9999999999999999".parse().unwrap(),
    };
    let options = SimilarityOptions::default();
    let stale = SimilarityOptions::Passages {
        decay: 0.5,
        parts: Some(vec![SimilarityPartSelector("stale".into())]),
        candidate_parts: SimilarityCandidateParts::Any,
    };
    let invalid: QueryPredicate = serde_json::from_value(json!({
        "kind":"compare","clause_id":"invalid-field","field":"not_a_field","op":"eq","value":1
    }))
    .unwrap();
    let bad_profile = SearchProfileId::new("not-a-profile");
    for name in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(name);
        for case in 0..5 {
            let actual = compare(&artifact.path(), &profile, || {
                let mut value = request(&seed, &profile, &options);
                match case {
                    0 => value.seed = &missing,
                    1 => value.options = &stale,
                    2 => value.sort = ResultSort::SavedOrder {},
                    3 => value.filter = Some(&invalid),
                    _ => value.search_profile = Some(&bad_profile),
                }
                value
            });
            assert_eq!(
                actual["status"],
                if case == 0 { "missing" } else { "error" }
            );
        }
    }
}

#[test]
fn oversized_seed_guard_and_selected_only_recovery_keep_exact_behavior() {
    let source = (0..129)
        .map(|i| format!("<h2>Section {i}</h2><p>Seed description {i}</p>"))
        .collect::<String>();
    let mut data = input_with_both_profiles(vec![
        record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"Large seed","type":"npc","system":{"details":{"publicNotes":source}}}),
        ),
        record(
            "actors",
            "Actor",
            json!({"_id":"bbbbbbbbbbbbbbbb","name":"Candidate","type":"npc","system":{"details":{"publicNotes":"<p>Candidate description</p>"}}}),
        ),
    ]);
    // Keep the recovered selected section positively comparable to the
    // candidate instead of the generic fixture's orthogonal root vectors.
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    let (service, artifact) = open_input_fixture(data).unwrap();
    drop(service);
    let seed = SimilaritySeed::Record {
        record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let all = SimilarityOptions::default();
    for name in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(name);
        let failed = compare(&artifact.path(), &profile, || {
            request(&seed, &profile, &all)
        });
        assert_eq!(failed["status"], "error");
        assert_eq!(failed["kind"], "InvalidOptions");
        let inventory = open(&artifact.path(), &profile)
            .similarity_parts(&seed, Some(&profile))
            .unwrap();
        let selector = inventory
            .parts
            .iter()
            .find_map(|part| match part {
                SimilarityComponent::Section { part } => Some(part.selector.clone()),
                _ => None,
            })
            .unwrap();
        let selected = SimilarityOptions::Passages {
            decay: 0.5,
            parts: Some(vec![selector]),
            candidate_parts: SimilarityCandidateParts::Any,
        };
        let recovered = compare(&artifact.path(), &profile, || {
            request(&seed, &profile, &selected)
        });
        assert_eq!(recovered["status"], "ok");
        assert_eq!(recovered["results"]["rows"].as_array().unwrap().len(), 1);
        assert!(
            f64::from_bits(
                recovered["results"]["rows"][0]["score_bits"]
                    .as_u64()
                    .unwrap()
            ) > 0.0
        );
    }
}

#[test]
fn cancellation_keeps_typed_errors_and_stops_at_the_requested_check() {
    let (service, artifact) = open_input_fixture(both_profiles(super::tests::fixture())).unwrap();
    drop(service);
    let seed = SimilaritySeed::Record {
        record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
    };
    let options = SimilarityOptions::default();
    for name in ["bge-base", "embeddinggemma"] {
        let profile = SearchProfileId::new(name);
        for threshold in [1, 2, 3, 5] {
            let mut outcomes = Vec::new();
            for exact in [false, true] {
                let service = open(&artifact.path(), &profile);
                let checks = Cell::new(0);
                let cancelled = || {
                    checks.set(checks.get() + 1);
                    checks.get() >= threshold
                };
                let value = ConfigurableSimilarRequest {
                    cancelled: Some(&cancelled),
                    ..request(&seed, &profile, &options)
                };
                let result = if exact {
                    service.configurable_similar_exact(value)
                } else {
                    service.configurable_similar(value)
                };
                assert_eq!(
                    result.as_ref().unwrap_err().kind(),
                    crate::SearchErrorKind::OperationCancelled
                );
                assert_eq!(checks.get(), threshold);
                outcomes.push(output(&result));
            }
            assert_eq!(outcomes[0], outcomes[1]);
            // Checks1/2 precede native selection. Setup adds legitimate check
            // opportunities, so3/5 need not represent the same internal phase.
        }
    }
}
