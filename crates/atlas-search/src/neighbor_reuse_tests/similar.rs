use super::*;
use atlas_domain::{RecordKey, SearchProfileId, SimilarityOptions, SimilaritySeed, TraitKey};

fn open_native(
    path: &std::path::Path,
    store: Option<&Arc<NativeNeighborReuse>>,
    profile: &str,
) -> AtlasRetrievalService {
    // Baseline Similar reads persisted seed vectors; it needs no query embedding.
    let index = atlas_index::test_support::open_reader(path).unwrap();
    let service =
        AtlasRetrievalService::from_index(index, SearchProfileId::new(profile), None).unwrap();
    match store {
        Some(store) => service.with_native_neighbor_reuse(Arc::clone(store)),
        None => service,
    }
}

fn record_request(key: &RecordKey) -> SimilarRecordRequest<'_> {
    SimilarRecordRequest {
        content_scope: SearchContentScope::All,
        sort: ResultSort::Relevance {},
        search_profile: None,
        record_key: key,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::first(10).unwrap(),
        prefer_remaster: true,
    }
}

fn trait_request(key: &TraitKey) -> SimilarTraitRequest<'_> {
    SimilarTraitRequest {
        content_scope: SearchContentScope::All,
        sort: ResultSort::Relevance {},
        trait_key: key,
        search_profile: None,
        filter: None,
        scope: RecordScope::All,
        page: SearchPage::first(10).unwrap(),
        prefer_remaster: true,
        ranking: SearchRanking::Score,
    }
}

fn evidence_signature(evidence: &[(RecordKey, SimilarityEvidence)]) -> Value {
    json!(evidence.iter().map(|(key, value)| json!({
        "key": key, "score": value.score.to_bits(), "aggregation": value.aggregation,
        "selected": value.selected_parts, "comparable": value.comparable_parts,
        "parts": value.parts.iter().map(|p| (&p.part, p.score.to_bits(), p.weight.to_bits(), p.comparable)).collect::<Vec<_>>(),
        "roles": value.roles.iter().map(|r| (&r.key, r.score.to_bits(), r.weight.to_bits(), r.comparable)).collect::<Vec<_>>(),
        "pairs": value.pairs.iter().map(|p| json!({
            "part": p.seed_part, "seed": p.seed, "seed_text": p.seed_text,
            "candidate": p.candidate, "cosine": p.cosine.to_bits()
        })).collect::<Vec<_>>(),
        "representative": value.representative, "truncated": value.truncated
    })).collect::<Vec<_>>())
}

fn with_traits() -> atlas_index::IndexBuildInput {
    let mut data = fixture();
    for label in ["Fire", "Flame"] {
        let key = TraitKey::new(label.to_lowercase()).unwrap();
        data.traits.push(TraitDefinition {
            key: key.clone(),
            label: label.into(),
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
        for profile in ["bge-base", "embeddinggemma"] {
            let input = format!("{label}\n");
            let chunk_hash = crate::test_support::hash(input.as_bytes());
            let mut vector = vec![0.; 768];
            vector[0] = 1.;
            data.trait_vectors.push(atlas_index::TraitVectorInput {
                key: key.clone(),
                profile_id: SearchProfileId::new(profile),
                input_hash: crate::test_support::hash(
                    format!("trait-definition-centroid/v1:{chunk_hash}").as_bytes(),
                ),
                vector: vector.clone(),
                chunks: vec![atlas_index::TraitSeedChunk {
                    input: input.clone(),
                    input_hash: chunk_hash.clone(),
                    token_count: 2,
                    vector,
                    address: atlas_domain::SourcePassageAddress::PlainSection {
                        source_text_sha256: chunk_hash,
                        selection_version: "trait-definition-centroid/v1".into(),
                        section_ordinal: 0,
                        label: Some(label.into()),
                        chunk_bytes: atlas_domain::SourceByteRange {
                            start: 0,
                            end: input.len(),
                        },
                    },
                }],
            });
        }
    }
    data
}

fn transitions() -> Vec<(SearchPage, ResultSort, bool)> {
    vec![
        (
            SearchPage::first(1).unwrap(),
            ResultSort::Relevance {},
            true,
        ),
        (
            SearchPage::first(10).unwrap(),
            ResultSort::Relevance {},
            true,
        ),
        (
            SearchPage::new(2, 1).unwrap(),
            ResultSort::Relevance {},
            true,
        ),
        (
            SearchPage::first(3).unwrap(),
            ResultSort::Random { seed: 42 },
            true,
        ),
        (
            SearchPage::first(2).unwrap(),
            ResultSort::attribute(
                atlas_domain::SortField::Level,
                atlas_domain::SortDirection::Desc,
            ),
            false,
        ),
    ]
}

#[test]
fn record_preview_full_page_and_sort_reuse_exact_windows_for_both_profiles_and_content_scopes() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let key = "items:0000000000000000".parse().unwrap();
    for profile in ["bge-base", "embeddinggemma"] {
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            for (i, (page, sort, prefer)) in transitions().into_iter().enumerate() {
                let cached = open_native(&artifact.path(), Some(&store), profile);
                let cold = open_native(&artifact.path(), None, profile);
                let request = || SimilarRecordRequest {
                    content_scope,
                    page,
                    sort: sort.clone(),
                    prefer_remaster: prefer,
                    ..record_request(&key)
                };
                let actual = cached.similar_records(request()).unwrap().unwrap();
                let expected = cold.similar_records(request()).unwrap().unwrap();
                assert_eq!(json!(actual.seed), json!(expected.seed));
                assert_eq!(signature(&actual.results), signature(&expected.results));
                assert!(actual.results.records.iter().all(|r| r.record.key != key));
                assert_eq!(cold.read_metrics().knn_queries, 3);
                assert_eq!(
                    cached.read_metrics().knn_queries,
                    if i == 0 { 3 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn equal_record_vectors_keep_distinct_exclusions_in_native_identity() {
    let mut data = fixture();
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    let (service, artifact) = crate::test_support::open_input_fixture(data).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let keys: [RecordKey; 2] = [
        "items:0000000000000000".parse().unwrap(),
        "items:0000000000000001".parse().unwrap(),
    ];
    for (i, seed) in [0, 1, 0, 1].into_iter().enumerate() {
        let service = open_native(&artifact.path(), Some(&store), "bge-base");
        let cold = open_native(&artifact.path(), None, "bge-base");
        let result = service
            .similar_records(record_request(&keys[seed]))
            .unwrap()
            .unwrap();
        let expected = cold
            .similar_records(record_request(&keys[seed]))
            .unwrap()
            .unwrap();
        assert_eq!(signature(&result.results), signature(&expected.results));
        assert_eq!(result.results.records.len(), 3);
        assert!(
            result
                .results
                .records
                .iter()
                .all(|r| r.record.key != keys[seed])
        );
        assert!(
            result
                .results
                .records
                .iter()
                .any(|r| r.record.key == keys[1 - seed])
        );
        assert_eq!(
            service.read_metrics().knn_queries,
            if i < 2 { 3 } else { 0 }
        );
    }
    assert_eq!(store.stats().ready_entries, 6);
}

#[test]
fn trait_preview_full_and_paging_reuse_neighbors_with_current_seed_presentation() {
    let (service, artifact) = crate::test_support::open_input_fixture(with_traits()).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    for profile in ["bge-base", "embeddinggemma"] {
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            for (i, (page, sort, prefer)) in transitions().into_iter().enumerate() {
                // Fire and Flame intentionally have identical native vectors.
                let key = TraitKey::new(if i == 0 { "fire" } else { "flame" }).unwrap();
                let cached = open_native(&artifact.path(), Some(&store), profile);
                let cold = open_native(&artifact.path(), None, profile);
                let request = || SimilarTraitRequest {
                    content_scope,
                    page,
                    sort: sort.clone(),
                    prefer_remaster: prefer,
                    ranking: if i == 2 {
                        SearchRanking::Rrf
                    } else {
                        SearchRanking::Score
                    },
                    ..trait_request(&key)
                };
                let actual = cached.similar_trait(request()).unwrap().unwrap();
                let expected = cold.similar_trait(request()).unwrap().unwrap();
                assert_eq!(actual.seed.key, key);
                assert_eq!(actual.seed.label, if i == 0 { "Fire" } else { "Flame" });
                assert_eq!(json!(actual.seed), json!(expected.seed));
                assert_eq!(signature(&actual.results), signature(&expected.results));
                assert_eq!(cold.read_metrics().knn_queries, 3);
                assert_eq!(
                    cached.read_metrics().knn_queries,
                    if i == 0 { 3 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn similar_predicate_changes_miss_and_successful_empty_windows_can_hit() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let key = "items:0000000000000000".parse().unwrap();
    let empty = atlas_domain::QueryPredicate::boolean(false);
    for (i, filter) in [None, Some(&empty), Some(&empty), None]
        .into_iter()
        .enumerate()
    {
        let service = open_native(&artifact.path(), Some(&store), "bge-base");
        let cold = open_native(&artifact.path(), None, "bge-base");
        let request = || SimilarRecordRequest {
            filter,
            ..record_request(&key)
        };
        let actual = service.similar_records(request()).unwrap().unwrap();
        let expected = cold.similar_records(request()).unwrap().unwrap();
        assert_eq!(signature(&actual.results), signature(&expected.results));
        assert_eq!(actual.results.records.is_empty(), filter.is_some());
        assert_eq!(
            service.read_metrics().knn_queries,
            if i < 2 { 3 } else { 0 }
        );
    }
}

#[test]
fn explicit_and_empty_similar_key_scopes_bypass_retention() {
    let (service, artifact) = crate::test_support::open_input_fixture(with_traits()).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let seed = "items:0000000000000000".parse().unwrap();
    let trait_key = TraitKey::new("fire").unwrap();
    let keys = ["items:0000000000000001".parse().unwrap()];
    for scope in [keys.as_slice(), &[]] {
        for _ in 0..2 {
            let service = open_native(&artifact.path(), Some(&store), "bge-base");
            let cold = open_native(&artifact.path(), None, "bge-base");
            let request = || SimilarRecordRequest {
                scope: RecordScope::Keys(scope),
                ..record_request(&seed)
            };
            let actual = service.similar_records(request()).unwrap().unwrap();
            let expected = cold.similar_records(request()).unwrap().unwrap();
            assert_eq!(signature(&actual.results), signature(&expected.results));
            assert_eq!(
                service.read_metrics().knn_queries,
                cold.read_metrics().knn_queries
            );
            assert_eq!(store.stats().ready_entries, 0);
            service.reset_read_metrics();
            cold.reset_read_metrics();
            let request = || SimilarTraitRequest {
                scope: RecordScope::Keys(scope),
                ..trait_request(&trait_key)
            };
            let actual = service.similar_trait(request()).unwrap().unwrap();
            let expected = cold.similar_trait(request()).unwrap().unwrap();
            assert_eq!(signature(&actual.results), signature(&expected.results));
            assert_eq!(
                service.read_metrics().knn_queries,
                cold.read_metrics().knn_queries
            );
            assert_eq!(store.stats().ready_entries, 0);
        }
    }
}

#[test]
fn warm_neighbors_do_not_bypass_missing_seeds_profiles_or_sort_validation() {
    let data = with_traits();
    let seed: RecordKey = "items:0000000000000000".parse().unwrap();
    let (service, artifact) = crate::test_support::open_input_fixture(data).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let service = open_native(&artifact.path(), Some(&store), "bge-base");
    service
        .similar_records(record_request(&seed))
        .unwrap()
        .unwrap();
    let fire = TraitKey::new("fire").unwrap();
    service
        .similar_trait(trait_request(&fire))
        .unwrap()
        .unwrap();
    let entries = store.stats().ready_entries;
    service.reset_read_metrics();

    let absent = "items:9999999999999999".parse().unwrap();
    assert!(
        service
            .similar_records(record_request(&absent))
            .unwrap()
            .is_none()
    );
    let absent_trait = TraitKey::new("absent").unwrap();
    assert!(
        service
            .similar_trait(trait_request(&absent_trait))
            .unwrap()
            .is_none()
    );
    let unsupported = SearchProfileId::new("unavailable");
    assert!(
        service
            .similar_records(SimilarRecordRequest {
                search_profile: Some(&unsupported),
                ..record_request(&seed)
            })
            .is_err()
    );
    assert!(
        service
            .similar_records(SimilarRecordRequest {
                sort: ResultSort::Attributes { rules: vec![] },
                ..record_request(&seed)
            })
            .is_err()
    );
    assert!(
        service
            .similar_trait(SimilarTraitRequest {
                sort: ResultSort::Attributes { rules: vec![] },
                ..trait_request(&fire)
            })
            .is_err()
    );
    assert_eq!(service.read_metrics().knn_queries, 0);
    assert_eq!(store.stats().ready_entries, entries);
}

#[test]
fn advanced_similar_keeps_its_distinct_candidate_path_with_an_empty_or_warm_store() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(NativeNeighborReuse::default());
    let seed_key = "items:0000000000000000".parse().unwrap();
    let seed = SimilaritySeed::Record {
        record_key: seed_key,
    };
    for warm in [false, true] {
        if warm {
            let SimilaritySeed::Record { record_key } = &seed else {
                unreachable!()
            };
            open_native(&artifact.path(), Some(&store), "bge-base")
                .similar_records(record_request(record_key))
                .unwrap()
                .unwrap();
        }
        let entries = store.stats().ready_entries;
        for options in [
            SimilarityOptions::default(),
            SimilarityOptions::ContentAverage {
                parts: None,
                candidate_parts: Default::default(),
            },
        ] {
            for _ in 0..2 {
                let cached = open_native(&artifact.path(), Some(&store), "bge-base");
                let cold = open_native(&artifact.path(), None, "bge-base");
                let request = || ConfigurableSimilarRequest {
                    seed: &seed,
                    options: &options,
                    source_sections: Some(atlas_domain::SimilaritySourceSections::All),
                    content_scope: SearchContentScope::All,
                    search_profile: None,
                    filter: None,
                    scope: RecordScope::All,
                    page: SearchPage::first(10).unwrap(),
                    sort: ResultSort::Relevance {},
                    prefer_remaster: true,
                    cancelled: None,
                };
                let actual = cached.configurable_similar(request()).unwrap().unwrap();
                let expected = cold.configurable_similar(request()).unwrap().unwrap();
                assert_eq!(signature(&actual.results), signature(&expected.results));
                assert_eq!(
                    evidence_signature(&actual.evidence),
                    evidence_signature(&expected.evidence)
                );
                assert_eq!(actual.candidate_keys, expected.candidate_keys);
                assert!(actual.diagnostics.knn_queries > 0);
                assert_eq!(
                    actual.diagnostics.knn_queries,
                    expected.diagnostics.knn_queries
                );
                assert_eq!(store.stats().ready_entries, entries);
            }
        }
    }
}
