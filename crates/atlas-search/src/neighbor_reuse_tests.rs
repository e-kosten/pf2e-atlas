use crate::*;
use atlas_domain::{ResultSort, SearchContentScope, SearchRanking};
use serde_json::{Value, json};
use std::sync::Arc;

fn fixture() -> atlas_index::IndexBuildInput {
    crate::test_support::input_with_both_profiles((0..4).map(|i| {
        crate::test_support::record("items", "Item", json!({
            "_id": format!("{i:016}"), "name": format!("Ember {i}"), "type": "effect",
            "system":{"level":{"value":i},"description":{"value":"<h2>Flame</h2><p>Bright flame.</p>"},"traits":{"value":[]}}
        }))
    }).collect())
}
fn open(
    path: &std::path::Path,
    store: Option<&Arc<TextNeighborReuse>>,
    profile: &str,
    text: &str,
    axis: usize,
) -> AtlasRetrievalService {
    let index = atlas_index::test_support::open_reader(path).unwrap();
    let profile = atlas_domain::SearchProfileId::new(profile);
    let mut vector = vec![0.; 768];
    vector[axis] = 1.;
    let prepared = PreparedQueryEmbedding::for_fixture(
        index.context().search_profile(&profile).unwrap().clone(),
        text,
        vector,
    );
    let service = AtlasRetrievalService::from_index(index, profile, Some(prepared)).unwrap();
    match store {
        Some(store) => service.with_text_neighbor_reuse(Arc::clone(store)),
        None => service,
    }
}
fn signature(result: &TextSearchResult) -> Value {
    json!({
        "profile": result.search_profile, "generation":result.artifact_generation,
        "sort":result.sort, "page":(result.page.number,result.page.size,result.page.count,result.page.total,result.page.has_more,result.page.next_page), "coverage":result.coverage,
        "rows":result.records.iter().map(|r| json!({
            "record":r.record,"matches":r.matches,"lexical":r.lexical_rank,
            "score":r.score.to_bits(),"similarity":r.semantic_similarity.map(f64::to_bits),
            "witness_bits":r.matches.iter().map(|m|(m.lexical_rank.map(f64::to_bits),m.semantic_similarity.map(f64::to_bits))).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn reopened_pages_orders_fusion_and_profiles_match_exact_uncached_results() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    for profile in ["bge-base", "embeddinggemma"] {
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            for (i, page) in [1, 2, 1, 1, 1, 1].into_iter().enumerate() {
                let mut cached = open(&artifact.path(), Some(&store), profile, "Ember", 0);
                let mut cold = open(&artifact.path(), None, profile, "Ember", 0);
                let sort = if i == 3 {
                    ResultSort::Random { seed: 42 }
                } else if i == 4 {
                    ResultSort::attribute(
                        atlas_domain::SortField::Level,
                        atlas_domain::SortDirection::Desc,
                    )
                } else {
                    ResultSort::Relevance {}
                };
                let ranking = if i == 2 {
                    SearchRanking::Rrf
                } else {
                    SearchRanking::Score
                };
                let request = || TextSearchRequest {
                    query: "Ember",
                    mode: if i == 5 {
                        RetrievalMode::Vector
                    } else {
                        RetrievalMode::Hybrid
                    },
                    filter: None,
                    scope: RecordScope::All,
                    page: SearchPage::new(page, if i == 3 { 3 } else { 1 }).unwrap(),
                    content_scope,
                    sort: sort.clone(),
                    ranking,
                    search_profile: None,
                    prefer_remaster: true,
                };
                let actual = cached.search_text_with_neighbor_reuse(request()).unwrap();
                let expected = cold.search_text(request()).unwrap();
                assert_eq!(signature(&actual), signature(&expected));
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
fn text_vector_predicate_and_core_changes_miss_and_empty_results_hit() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let filters = [None, Some(atlas_domain::QueryPredicate::boolean(false))];
    for (text, axis, filter, core) in [
        ("Ember", 0, 0, false),
        ("Ember ", 0, 0, false),
        ("Ember", 1, 0, false),
        ("Ember", 0, 1, false),
        ("Ember", 0, 0, true),
    ] {
        for repeat in 0..2 {
            let mut service = open(&artifact.path(), Some(&store), "bge-base", text, axis);
            let mut request = TextSearchRequest::new(text, SearchPage::first(2).unwrap());
            request.filter = filters[filter].as_ref();
            request.content_scope = if core {
                SearchContentScope::Core
            } else {
                SearchContentScope::All
            };
            let result = service.search_text_with_neighbor_reuse(request).unwrap();
            assert_eq!(
                service.read_metrics().knn_queries,
                if repeat == 0 { 3 } else { 0 }
            );
            if filter == 1 {
                assert!(result.records.is_empty());
            }
        }
    }
}

#[test]
fn excluded_consumers_and_explicit_key_scopes_never_reuse() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let keys = ["items:0000000000000000".parse().unwrap()];
    for _ in 0..2 {
        let mut service = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
        service
            .search_text(TextSearchRequest::new(
                "Ember",
                SearchPage::first(2).unwrap(),
            ))
            .unwrap();
        assert_eq!(service.read_metrics().knn_queries, 3);
        service.reset_read_metrics();
        let mut request = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
        request.scope = RecordScope::Keys(&keys);
        service.search_text_with_neighbor_reuse(request).unwrap();
        assert_eq!(service.read_metrics().knn_queries, 3);
        service.reset_read_metrics();
        service
            .discover_filter_values(DiscoverFilterValuesRequest {
                only_matching: true,
                field: "rarity",
                clause_id: None,
                text: None,
                offset: 0,
                limit: 10,
                context: FilterDiscoveryContext {
                    content_scope: SearchContentScope::All,
                    search_profile: None,
                    text: Some("Ember"),
                    mode: RetrievalMode::Hybrid,
                    scope: RecordScope::All,
                    prefer_remaster: true,
                    filter: None,
                },
            })
            .unwrap();
        assert_eq!(service.read_metrics().knn_queries, 3);
    }
    assert_eq!(store.stats().ready_entries, 0);
}

#[test]
fn same_context_publication_invalidates_windows_across_reopened_services() {
    let fixture = fixture();
    let context = fixture.context.clone();
    let (service, artifact) = crate::test_support::open_input_fixture(fixture).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let mut old = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
    old.search_text_with_neighbor_reuse(TextSearchRequest::new(
        "Ember",
        SearchPage::first(2).unwrap(),
    ))
    .unwrap();
    let generation = old.artifact_generation().unwrap();
    drop(old);
    let mut replacement = self::fixture();
    assert_eq!(replacement.context, context);
    for unit in &mut replacement.semantic_units {
        unit.vector.fill(0.);
        unit.vector[1] = 1.;
    }
    crate::test_support::publish_input(artifact.path(), &replacement).unwrap();
    let mut new = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
    assert_ne!(new.artifact_generation().unwrap(), generation);
    let actual = new
        .search_text_with_neighbor_reuse(TextSearchRequest::new(
            "Ember",
            SearchPage::first(2).unwrap(),
        ))
        .unwrap();
    assert_eq!(new.read_metrics().knn_queries, 3);
    let mut cold = open(&artifact.path(), None, "bge-base", "Ember", 0);
    let expected = cold
        .search_text(TextSearchRequest::new(
            "Ember",
            SearchPage::first(2).unwrap(),
        ))
        .unwrap();
    assert_eq!(signature(&actual), signature(&expected));
}

#[test]
fn native_window_cutoff_ties_are_preserved_without_slicing_larger_windows() {
    let mut data = crate::test_support::input((0..1100).map(|i| {
        crate::test_support::record("items","Item",json!({
            "_id":format!("{i:016}"),"name":format!("Tie {i}"),"type":"effect","system":{}
        }))
    }).collect(),true);
    for unit in &mut data.semantic_units {
        unit.vector.fill(0.);
        unit.vector[0] = 1.;
    }
    let (service, _artifact) = crate::test_support::open_input_fixture(data).unwrap();
    let query = service.predicate(None).unwrap();
    let profile = service.default_search_profile();
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    for k in [1024, 2048, 4096] {
        let request = atlas_index::SourceVectorCandidateRequest {
            profile,
            vector: &vector,
            query: &query,
            keys: None,
            exclude: None,
            content_scope: SearchContentScope::All,
            limit: k,
        };
        let original = service.index.vector_candidates(request).unwrap();
        let (cold, neighbors) = service
            .index
            .vector_candidates_with_neighbors(request)
            .unwrap();
        let hit = service
            .index
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap();
        let signature = |hits: &[atlas_index::SourceVectorHit]| {
            hits.iter()
                .map(|h| (h.location.clone(), h.distance.to_bits()))
                .collect::<Vec<_>>()
        };
        assert_eq!(signature(&original), signature(&cold));
        assert_eq!(signature(&original), signature(&hit));
        assert_eq!(hit.len(), k.min(1100));
        assert!(
            hit.windows(2)
                .all(|h| h[0].location.unit_id < h[1].location.unit_id)
        );
    }
}

fn mutate(path: &std::path::Path, sql: &str) {
    let workspace = atlas_storage::Workspace::at(path).unwrap();
    let write = workspace
        .write(atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute_batch("PRAGMA ignore_check_constraints=ON; PRAGMA defer_foreign_keys=ON")
        .unwrap();
    write.connection().execute_batch(sql).unwrap();
    write.commit().unwrap();
}
#[test]
fn malformed_location_metadata_fails_on_hits_and_cold_reads_then_recovers() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let request = || TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
    open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
        .search_text_with_neighbor_reuse(request())
        .unwrap();
    // Deliberate unversioned fixture corruption probes existing read validation;
    // arbitrary external writes are not a supported invalidation protocol.
    mutate(
        &artifact.path(),
        "UPDATE semantic_locations SET owners_json='{}' WHERE owners_json='[]'",
    );
    let hit = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
        .search_text_with_neighbor_reuse(request())
        .unwrap_err();
    let cold = open(&artifact.path(), None, "bge-base", "Ember", 0)
        .search_text(request())
        .unwrap_err();
    assert_eq!(hit.kind(), cold.kind());
    assert_eq!(hit.to_string(), cold.to_string());
    mutate(
        &artifact.path(),
        "UPDATE semantic_locations SET owners_json='[]'",
    );
    open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
        .search_text_with_neighbor_reuse(request())
        .unwrap();
}

#[test]
fn malformed_global_remaster_evidence_is_not_hidden_by_nonempty_hits_or_empty_windows() {
    let mut data = fixture();
    data.remaster_pairs
        .push(atlas_index::SourceRemasterPairInput {
            legacy: "items:0000000000000002".parse().unwrap(),
            remaster: "items:0000000000000003".parse().unwrap(),
            evidence: "fixture".into(),
        });
    let (service, artifact) = crate::test_support::open_input_fixture(data).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let filter = atlas_index::parse_where("level < 2").unwrap();
    let empty = atlas_domain::QueryPredicate::boolean(false);
    for predicate in [filter.predicate(), &empty] {
        let request = || {
            let mut r = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
            r.filter = Some(predicate);
            r
        };
        open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
            .search_text_with_neighbor_reuse(request())
            .unwrap();
    }
    for bad in ["{}", "not-json"] {
        mutate(
            &artifact.path(),
            &format!("UPDATE remaster_pairs SET evidence_json='{bad}'"),
        );
        let request = || {
            let mut r = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
            r.filter = Some(filter.predicate());
            r
        };
        let mut hit = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
        let hit_error = hit.search_text_with_neighbor_reuse(request()).unwrap_err();
        let cold_error = open(&artifact.path(), None, "bge-base", "Ember", 0)
            .search_text(request())
            .unwrap_err();
        assert_eq!(hit_error.kind(), cold_error.kind());
        assert_eq!(hit_error.to_string(), cold_error.to_string());
        assert_eq!(hit.read_metrics().knn_queries, 0);
        let request = || {
            let mut r = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
            r.filter = Some(&empty);
            r
        };
        let actual = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
            .search_text_with_neighbor_reuse(request())
            .unwrap();
        let expected = open(&artifact.path(), None, "bge-base", "Ember", 0)
            .search_text(request())
            .unwrap();
        assert_eq!(signature(&actual), signature(&expected));
    }
    mutate(
        &artifact.path(),
        r#"UPDATE remaster_pairs SET evidence_json='"fixture"'; UPDATE record_identity SET key='invalid-key' WHERE key='items:0000000000000002'; UPDATE records SET key='invalid-key' WHERE key='items:0000000000000002'"#,
    );
    let request = || {
        let mut r = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
        r.filter = Some(filter.predicate());
        r
    };
    let hit = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
        .search_text_with_neighbor_reuse(request())
        .unwrap_err();
    let cold = open(&artifact.path(), None, "bge-base", "Ember", 0)
        .search_text(request())
        .unwrap_err();
    assert_eq!(
        (hit.kind(), hit.to_string()),
        (cold.kind(), cold.to_string())
    );
}

#[test]
fn invalid_sort_predicate_and_vectors_keep_their_validation_on_warm_reads() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
        .search_text_with_neighbor_reuse(TextSearchRequest::new(
            "Ember",
            SearchPage::first(2).unwrap(),
        ))
        .unwrap();
    let invalid: atlas_domain::QueryPredicate = serde_json::from_value(
        json!({"kind":"compare","field":"absent.field","op":"eq","value":3}),
    )
    .unwrap();
    for n in 0..2 {
        let request = || {
            let mut r = TextSearchRequest::new("Ember", SearchPage::first(2).unwrap());
            if n == 0 {
                r.sort = ResultSort::SavedOrder {};
            } else {
                r.filter = Some(&invalid);
            }
            r
        };
        let hit = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0)
            .search_text_with_neighbor_reuse(request())
            .unwrap_err();
        let cold = open(&artifact.path(), None, "bge-base", "Ember", 0)
            .search_text(request())
            .unwrap_err();
        assert_eq!(
            (hit.kind(), hit.to_string()),
            (cold.kind(), cold.to_string())
        );
    }
    let service = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
    let predicate = service.predicate(None).unwrap();
    let profile = service.default_search_profile();
    let mut valid = vec![0.; 768];
    valid[0] = 1.;
    let request = atlas_index::SourceVectorCandidateRequest {
        profile,
        vector: &valid,
        query: &predicate,
        keys: None,
        exclude: None,
        content_scope: SearchContentScope::All,
        limit: 1024,
    };
    let (_, neighbors) = service
        .index
        .vector_candidates_with_neighbors(request)
        .unwrap();
    for invalid in [vec![0.; 768], vec![f32::NAN; 768], vec![1.; 1]] {
        let request = atlas_index::SourceVectorCandidateRequest {
            vector: &invalid,
            ..request
        };
        let cold = service.index.vector_candidates(request).unwrap_err();
        let hit = service
            .index
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap_err();
        assert_eq!(cold.to_string(), hit.to_string());
    }
}

#[test]
fn duplicate_and_distinct_requests_execute_native_misses_on_independent_readers() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    for distinct in [false, true] {
        let store = Arc::new(TextNeighborReuse::default());
        *store.miss_barrier.lock().unwrap() = Some(Arc::new(std::sync::Barrier::new(2)));
        let workers = (0..2)
            .map(|n| {
                let path = artifact.path();
                let store = Arc::clone(&store);
                std::thread::spawn(move || {
                    let text = if distinct && n == 1 { "Flame" } else { "Ember" };
                    let mut service = open(&path, Some(&store), "bge-base", text, 0);
                    let result = service
                        .search_text_with_neighbor_reuse(TextSearchRequest::new(
                            text,
                            SearchPage::first(2).unwrap(),
                        ))
                        .unwrap();
                    assert!(!result.records.is_empty());
                    service.read_metrics().knn_queries
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), 3);
        }
        assert_eq!(store.stats().ready_entries, if distinct { 6 } else { 3 });
        assert!(store.stats().peak_bytes <= 16 * 1024 * 1024);
    }
}

#[test]
fn current_scope_hydration_rejects_missing_members_instead_of_dropping_them() {
    let (service, _artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    let query = service.predicate(None).unwrap();
    let empty = service
        .predicate(Some(&atlas_domain::QueryPredicate::boolean(false)))
        .unwrap();
    let profile = service.default_search_profile();
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let request = atlas_index::SourceVectorCandidateRequest {
        profile,
        vector: &vector,
        query: &query,
        keys: None,
        exclude: None,
        content_scope: SearchContentScope::All,
        limit: 1024,
    };
    let (_, neighbors) = service
        .index
        .vector_candidates_with_neighbors(request)
        .unwrap();
    let request = atlas_index::SourceVectorCandidateRequest {
        query: &empty,
        ..request
    };
    assert!(
        service
            .index
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap_err()
            .to_string()
            .contains("missing or unexpected")
    );
}

#[test]
fn failed_publication_blocks_admission_until_validated_recovery_preserves_cached_revision() {
    let (service, artifact) = crate::test_support::open_input_fixture(fixture()).unwrap();
    drop(service);
    let store = Arc::new(TextNeighborReuse::default());
    let mut warm = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
    let expected = warm
        .search_text_with_neighbor_reuse(TextSearchRequest::new(
            "Ember",
            SearchPage::first(2).unwrap(),
        ))
        .unwrap();
    drop(warm);
    let workspace = atlas_storage::Workspace::at(artifact.path()).unwrap();
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner
        .maintenance(
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            "failed update fixture",
        )
        .unwrap();
    assert!(
        maintenance
            .publish(|c| {
                c.execute("DELETE FROM artifact_context", [])?;
                atlas_index::validate_reference(c)
            })
            .is_err()
    );
    drop(maintenance);
    assert!(atlas_index::test_support::open_reader(artifact.path()).is_err());
    assert!(
        owner
            .recover_maintenance(
                atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
                |c| atlas_index::validate_reference(c).map(|_| ())
            )
            .unwrap()
    );
    let mut recovered = open(&artifact.path(), Some(&store), "bge-base", "Ember", 0);
    let actual = recovered
        .search_text_with_neighbor_reuse(TextSearchRequest::new(
            "Ember",
            SearchPage::first(2).unwrap(),
        ))
        .unwrap();
    assert_eq!(signature(&actual), signature(&expected));
    assert_eq!(recovered.read_metrics().knn_queries, 0);
}
