use super::*;
use crate::test_support;
use atlas_domain::{QueryPredicate, SearchProfileId, SimilarityRole};
use serde_json::json;
use std::{
    cell::Cell,
    collections::BTreeSet,
    panic::{AssertUnwindSafe, catch_unwind},
};

fn vector() -> Vec<f32> {
    let mut v = vec![0.; 768];
    v[0] = 1.;
    v
}
fn request<'a>(
    profile: &'a SearchProfileId,
    query: &'a crate::ValidatedQuery,
    vector: &'a [f32],
) -> SourceSimilarityCandidateRequest<'a> {
    SourceSimilarityCandidateRequest {
        profile,
        vector,
        query,
        keys: None,
        exclude: None,
        comparison_key: None,
        content_scope: SearchContentScope::All,
        include_identity: true,
        limit: 256,
    }
}
fn locations(hits: &[SourceVectorHit]) -> Vec<(&crate::SourceUnitLocation, u64)> {
    hits.iter()
        .map(|hit| (&hit.location, hit.distance.to_bits()))
        .collect()
}
fn key(plan: &CandidatePlan) -> EligibilityKey<'_> {
    plan.key()
}

fn retained_ids(
    operation: &mut ConfiguredSimilarityCandidates<'_>,
    request: SourceSimilarityCandidateRequest<'_>,
) -> Vec<i64> {
    let plan = operation
        .index
        .configured_candidate_plan(request)
        .unwrap()
        .unwrap();
    let ids = operation
        .eligibility
        .get_or_try_insert::<_, IndexError>(&key(&plan), |_| {
            panic!("expected an already completed relation")
        })
        .unwrap()
        .unwrap();
    serde_json::from_str(ids).unwrap()
}
fn direct_ids(
    reader: &SqliteIndexReader,
    request: SourceSimilarityCandidateRequest<'_>,
) -> Vec<i64> {
    let plan = reader.configured_candidate_plan(request).unwrap().unwrap();
    reader
        .native()
        .unwrap()
        .prepare(&plan.setup_sql())
        .unwrap()
        .query_map(
            params_from_iter(plan.values[..plan.setup_parameters].iter()),
            |row| row.get(0),
        )
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}
fn mutate(path: &std::path::Path, sql: &str) {
    let workspace = atlas_storage::Workspace::at(path).unwrap();
    let write = workspace
        .write(test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    write
        .connection()
        .execute_batch("PRAGMA ignore_check_constraints=ON; PRAGMA defer_foreign_keys=ON;")
        .unwrap();
    write.connection().execute_batch(sql).unwrap();
    write.commit().unwrap();
}

#[test]
fn exact_unit_sets_and_native_bits_cover_profiles_masks_scopes_and_exclusions() {
    let input = crate::core_tests::nested_fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let queries = [
        crate::validate_query(&QueryPredicate::boolean(true)).unwrap(),
        crate::validate_query(&QueryPredicate::boolean(false)).unwrap(),
        crate::parse_where("source.type == 'npc'").unwrap(),
        crate::parse_where("links_to('items:bbbbbbbbbbbbbbbb')").unwrap(),
    ];
    let a = input.records[0].record.key().clone();
    let b = input.records[1].record.key().clone();
    let keys = [a.clone(), b.clone()];
    let duplicate = [b.clone(), a.clone(), b.clone()];
    let empty = [];
    let v = vector();
    for model in &input.context.search_profiles {
        for content in [SearchContentScope::All, SearchContentScope::Core] {
            for query in &queries {
                for scope in [
                    None,
                    Some(keys.as_slice()),
                    Some(duplicate.as_slice()),
                    Some(empty.as_slice()),
                ] {
                    for exclude in [None, Some(&a)] {
                        for comparison in [
                            None,
                            Some(SimilarityComparisonKey::Identity),
                            Some(SimilarityComparisonKey::Role(SimilarityRole::Description)),
                            Some(SimilarityComparisonKey::Role(SimilarityRole::Other)),
                        ] {
                            for include_identity in [false, true] {
                                let request = SourceSimilarityCandidateRequest {
                                    keys: scope,
                                    exclude,
                                    comparison_key: comparison,
                                    content_scope: content,
                                    include_identity,
                                    ..request(&model.profile_id, query, &v)
                                };
                                let expected = reader.similarity_candidates(request).unwrap();
                                let mut operation = reader.configured_similarity_candidates();
                                let actual = operation
                                    .candidates::<IndexError>(request, || Ok(()))
                                    .unwrap();
                                assert_eq!(locations(&actual), locations(&expected));
                                if comparison
                                    == Some(SimilarityComparisonKey::Role(SimilarityRole::Other))
                                {
                                    assert_eq!(operation.stats().materializations, 0);
                                    continue;
                                }
                                let raw = direct_ids(&reader, request);
                                assert_eq!(retained_ids(&mut operation, request), raw);
                                assert!(raw.len() <= input.semantic_units.len());
                                let set: BTreeSet<_> = raw.into_iter().collect();
                                if scope == Some(empty.as_slice())
                                    || query.predicate() == &QueryPredicate::boolean(false)
                                {
                                    assert!(set.is_empty());
                                }
                                if comparison == Some(SimilarityComparisonKey::Identity) {
                                    assert!(actual.iter().all(|hit| hit.location.field.is_none()));
                                }
                                assert!(
                                    actual
                                        .iter()
                                        .all(|hit| exclude != Some(&hit.location.record))
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // An independently named expected relation prevents shared compiler bugs from
    // passing solely because both execution paths use the same formula.
    let all = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let profile = &input.context.search_profiles[0].profile_id;
    let identity = SourceSimilarityCandidateRequest {
        comparison_key: Some(SimilarityComparisonKey::Identity),
        ..request(profile, &all, &v)
    };
    let hits = reader.similarity_candidates(identity).unwrap();
    assert_eq!(
        hits.iter()
            .map(|h| h.location.record.clone())
            .collect::<BTreeSet<_>>(),
        [a, b].into_iter().collect()
    );
}

#[test]
fn complete_setup_reuse_still_validates_each_vector_and_window() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let p = &input.context.search_profiles[0].profile_id;
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    let mut different = v.clone();
    different[0] = 0.8;
    different[1] = 0.6;
    let base = request(p, &q, &v);
    let mut operation = reader.configured_similarity_candidates();
    for r in [
        base,
        SourceSimilarityCandidateRequest {
            vector: &different,
            limit: 1,
            ..base
        },
        SourceSimilarityCandidateRequest {
            limit: 4096,
            ..base
        },
    ] {
        assert_eq!(
            locations(&operation.candidates::<IndexError>(r, || Ok(())).unwrap()),
            locations(&reader.similarity_candidates(r).unwrap())
        );
    }
    assert_eq!(operation.stats().materializations, 1);
    assert_eq!(operation.stats().hits, 2);
    let invalid = vec![f32::NAN; 768];
    let unavailable = SearchProfileId::new("not-built");
    for r in [
        SourceSimilarityCandidateRequest { limit: 0, ..base },
        SourceSimilarityCandidateRequest {
            limit: 4097,
            ..base
        },
        SourceSimilarityCandidateRequest {
            vector: &invalid,
            ..base
        },
        SourceSimilarityCandidateRequest {
            vector: &v[..5],
            ..base
        },
        SourceSimilarityCandidateRequest {
            profile: &unavailable,
            ..base
        },
    ] {
        assert_eq!(
            operation
                .candidates::<IndexError>(r, || panic!("guards must precede setup"))
                .unwrap_err()
                .to_string(),
            reader.similarity_candidates(r).unwrap_err().to_string()
        );
    }
    assert_eq!(operation.stats().materializations, 1);
    let short = SourceSimilarityCandidateRequest {
        profile: &unavailable,
        vector: &invalid,
        comparison_key: Some(SimilarityComparisonKey::Role(SimilarityRole::Other)),
        ..base
    };
    assert!(
        operation
            .candidates::<IndexError>(short, || panic!("role short-circuit"))
            .unwrap()
            .is_empty()
    );
    let core = SourceSimilarityCandidateRequest {
        content_scope: SearchContentScope::Core,
        ..base
    };
    operation.candidates::<IndexError>(core, || Ok(())).unwrap();
    assert_eq!(operation.stats().materializations, 2);
}

#[test]
fn capacity_bypass_is_once_exact_and_ordinary_selection_is_unchanged() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let p = &input.context.search_profiles[0].profile_id;
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    let r = request(p, &q, &v);
    let ordinary = crate::SourceVectorCandidateRequest {
        profile: p,
        vector: &v,
        query: &q,
        keys: None,
        exclude: None,
        content_scope: SearchContentScope::All,
        limit: 256,
    };
    let before = reader.vector_candidates(ordinary).unwrap();
    let mut operation = reader.configured_similarity_candidates();
    operation.eligibility =
        EligibilityStore::with_limits(crate::eligibility_buffer::EligibilityLimits {
            rows: 1,
            ..Default::default()
        });
    let expected = reader.similarity_candidates(r).unwrap();
    for _ in 0..3 {
        assert_eq!(
            locations(&operation.candidates::<IndexError>(r, || Ok(())).unwrap()),
            locations(&expected)
        );
    }
    assert_eq!(operation.stats().materializations, 1);
    assert_eq!(operation.stats().ready, 0);
    assert_eq!(operation.stats().bypasses, 3);
    assert_eq!(
        locations(&reader.vector_candidates(ordinary).unwrap()),
        locations(&before)
    );
}

#[test]
fn original_prepare_error_wins_before_malformed_selected_metadata_and_cached_reuse() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let p = &input.context.search_profiles[0].profile_id;
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    mutate(&path, "UPDATE selected_sections SET owners_json='broken'");
    let reader = test_support::open_reader(&path).unwrap();
    let r = request(p, &q, &v);
    assert_eq!(
        reader
            .configured_similarity_candidates()
            .candidates::<IndexError>(r, || Ok(()))
            .unwrap_err()
            .to_string(),
        reader.similarity_candidates(r).unwrap_err().to_string()
    );
    let table = crate::schema::vector_table(&input.context.search_profiles[0]).unwrap();
    reader
        .native()
        .unwrap()
        .execute_batch(&format!("CREATE TEMP TABLE {table}(denied INTEGER)"))
        .unwrap();
    let mut operation = reader.configured_similarity_candidates();
    assert_eq!(
        operation
            .candidates::<IndexError>(r, || panic!("prepare must fail before setup"))
            .unwrap_err()
            .to_string(),
        reader.similarity_candidates(r).unwrap_err().to_string()
    );
    assert_eq!(operation.stats().materializations, 0);
}

#[test]
fn cancellation_and_unwind_never_publish_partial_or_completed_cancelled_setup() {
    // More than256 source units gives deterministic mid-materialization checks.
    let records=(0..140).map(|i|("items","Item",json!({"_id":format!("{i:016}"),"name":format!("Tied {i}"),"type":"equipment","system":{"description":{"value":"<p>Same body.</p>"}}}))).collect();
    let input = crate::core_tests::fixture(records, false);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let p = &input.context.search_profiles[0].profile_id;
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    let r = request(p, &q, &v);
    for after in [1, 2, 3] {
        let mut operation = reader.configured_similarity_candidates();
        let probe = operation.eligibility.accounting_probe();
        let base = probe.live();
        let checks = Cell::new(0);
        let error = operation
            .candidates::<IndexError>(r, || {
                checks.set(checks.get() + 1);
                if checks.get() >= after {
                    Err(IndexError::Operation("cancelled setup".into()))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "artifact operation failed: cancelled setup"
        );
        assert_eq!(operation.stats().ready, 0);
        assert_eq!(probe.live(), base);
        drop(operation);
        assert_eq!(probe.live(), 0);
    }
    let mut operation = reader.configured_similarity_candidates();
    let probe = operation.eligibility.accounting_probe();
    assert!(
        catch_unwind(AssertUnwindSafe(
            || operation.candidates::<IndexError>(r, || panic!("abandoned setup"))
        ))
        .is_err()
    );
    assert_eq!(operation.stats().ready, 0);
    drop(operation);
    assert_eq!(probe.live(), 0);
    assert_eq!(
        reader
            .native()
            .unwrap()
            .query_row("SELECT count(*) FROM sqlite_temp_master", [], |row| row
                .get::<_, usize>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn tied_native_membership_is_exact_for_each_k_and_setup_without_new_native_budget() {
    let records=(0..320).map(|i|("items","Item",json!({"_id":format!("{i:016}"),"name":format!("Tie {i}"),"type":"equipment","system":{"description":{"value":"<p>Same body.</p>"}}}))).collect();
    let mut input = crate::core_tests::fixture(records, true);
    for unit in &mut input.semantic_units {
        unit.vector = vector();
    }
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    for model in &input.context.search_profiles {
        for content_scope in [SearchContentScope::All, SearchContentScope::Core] {
            let mut operation = reader.configured_similarity_candidates();
            for limit in [1, 5, 256, 4096] {
                let r = SourceSimilarityCandidateRequest {
                    content_scope,
                    limit,
                    ..request(&model.profile_id, &q, &v)
                };
                let actual = operation.candidates::<IndexError>(r, || Ok(())).unwrap();
                let expected = reader.similarity_candidates(r).unwrap();
                assert_eq!(locations(&actual), locations(&expected));
                let plan = reader.configured_candidate_plan(r).unwrap().unwrap();
                let ids = operation
                    .eligibility
                    .get_or_try_insert::<_, IndexError>(&key(&plan), |_| panic!("retained tie set"))
                    .unwrap()
                    .unwrap();
                let read_bits = |statement: &mut Statement<'_>, parameters| {
                    statement
                        .query_map(parameters, |row| {
                            Ok((
                                row.get::<_, i64>(0)?,
                                row.get::<_, f64>("distance")?.to_bits(),
                            ))
                        })
                        .unwrap()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap()
                };
                let expected_ids = read_bits(
                    &mut reader.native().unwrap().prepare(&plan.native_sql).unwrap(),
                    params_from_iter(plan.values.iter()),
                );
                let optimized_sql =
                    selection_sql(&plan.table, "SELECT value FROM json_each(?3)", 1, 2);
                let actual_ids = reader
                    .native()
                    .unwrap()
                    .prepare(&optimized_sql)
                    .unwrap()
                    .query_map(
                        params![
                            &plan.values[plan.setup_parameters],
                            &plan.values[plan.setup_parameters + 1],
                            ids
                        ],
                        |row| {
                            Ok((
                                row.get::<_, i64>(0)?,
                                row.get::<_, f64>("distance")?.to_bits(),
                            ))
                        },
                    )
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                assert_eq!(actual_ids, expected_ids);
                assert_eq!(actual.len(), limit.min(640));
                assert!(actual.iter().all(|h| h.distance.to_bits() == 0));
            }
            assert_eq!(operation.stats().materializations, 1);
        }
    }
}

#[test]
fn complete_setup_keys_separate_every_eligibility_input_and_empty_hits_validate() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    let p = &input.context.search_profiles[0].profile_id;
    let second = &input.context.search_profiles[1].profile_id;
    let all = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let empty = crate::validate_query(&QueryPredicate::boolean(false)).unwrap();
    let v = vector();
    let keys = [input.records[0].record.key().clone()];
    let base = request(p, &all, &v);
    let mut op = reader.configured_similarity_candidates();
    let variants = [
        base,
        SourceSimilarityCandidateRequest {
            query: &empty,
            ..base
        },
        SourceSimilarityCandidateRequest {
            profile: second,
            ..base
        },
        SourceSimilarityCandidateRequest {
            keys: Some(&keys),
            ..base
        },
        SourceSimilarityCandidateRequest {
            exclude: Some(&keys[0]),
            ..base
        },
        SourceSimilarityCandidateRequest {
            content_scope: SearchContentScope::Core,
            ..base
        },
        SourceSimilarityCandidateRequest {
            comparison_key: Some(SimilarityComparisonKey::Role(SimilarityRole::Description)),
            ..base
        },
        SourceSimilarityCandidateRequest {
            include_identity: false,
            ..base
        },
    ];
    for (i, r) in variants.into_iter().enumerate() {
        op.candidates::<IndexError>(r, || Ok(())).unwrap();
        assert_eq!(op.stats().materializations, i + 1);
    }
    let bad = vec![f32::NAN; 768];
    assert!(
        op.candidates::<IndexError>(
            SourceSimilarityCandidateRequest {
                query: &empty,
                vector: &bad,
                ..base
            },
            || panic!("empty retained relation must validate vector")
        )
        .is_err()
    );
    for limit in [0, 4097] {
        let r = SourceSimilarityCandidateRequest {
            limit,
            comparison_key: Some(SimilarityComparisonKey::Role(SimilarityRole::Other)),
            ..base
        };
        assert_eq!(
            op.candidates::<IndexError>(r, || panic!("limit first"))
                .unwrap_err()
                .to_string(),
            reader.similarity_candidates(r).unwrap_err().to_string()
        );
    }
}

#[test]
fn independent_operations_and_publication_cannot_share_or_retain_a_snapshot() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let results = std::thread::scope(|threads| {
        let jobs = (0..2)
            .map(|which| {
                let barrier = barrier.clone();
                let path = &path;
                let input = &input;
                threads.spawn(move || {
                    let reader = test_support::open_reader(path).unwrap();
                    let v = vector();
                    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
                    let keys = [input.records[which].record.key().clone()];
                    let r = SourceSimilarityCandidateRequest {
                        keys: Some(&keys),
                        ..request(&input.context.search_profiles[which].profile_id, &q, &v)
                    };
                    let mut op = reader.configured_similarity_candidates();
                    let probe = op.eligibility.accounting_probe();
                    barrier.wait();
                    let first = op.candidates::<IndexError>(r, || Ok(())).unwrap();
                    let second = op.candidates::<IndexError>(r, || Ok(())).unwrap();
                    assert_eq!(locations(&first), locations(&second));
                    assert!(first.iter().all(|h| h.location.record == keys[0]));
                    let stats = op.stats();
                    drop(op);
                    assert_eq!(probe.live(), 0);
                    (stats, reader.artifact_instance().generation())
                })
            })
            .collect::<Vec<_>>();
        jobs.into_iter()
            .map(|job| job.join().unwrap())
            .collect::<Vec<_>>()
    });
    for (stats, _) in &results {
        assert_eq!(stats.materializations, 1);
        assert_eq!(stats.hits, 1);
    }
    // Cooperating publication begins only after every admitted old reader drops.
    let changed = crate::core_tests::nested_fixture(true);
    test_support::publish_input(&path, &changed).unwrap();
    let reader = test_support::open_reader(&path).unwrap();
    assert_ne!(reader.artifact_instance().generation(), results[0].1);
    let v = vector();
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let mut op = reader.configured_similarity_candidates();
    op.candidates::<IndexError>(
        request(&changed.context.search_profiles[0].profile_id, &q, &v),
        || Ok(()),
    )
    .unwrap();
    assert_eq!(op.stats().materializations, 1);
    assert_eq!(op.stats().hits, 0);
}

#[test]
fn duplicate_eligibility_rows_keep_sql_set_semantics_and_large_integer_json_exactness() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let reader = test_support::open_reader(&path).unwrap();
    // Duplicate-producing joins must still form exactly the same SQL IN-set.
    reader.native().unwrap().execute_batch("CREATE TEMP TABLE selected_sections AS SELECT * FROM main.selected_sections; INSERT INTO selected_sections SELECT * FROM main.selected_sections;").unwrap();
    let p = &input.context.search_profiles[0].profile_id;
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    let r = request(p, &q, &v);
    let mut operation = reader.configured_similarity_candidates();
    let hits = operation.candidates::<IndexError>(r, || Ok(())).unwrap();
    assert_eq!(
        locations(&hits),
        locations(&reader.similarity_candidates(r).unwrap())
    );
    let raw = retained_ids(&mut operation, r);
    assert!(raw.len() > raw.iter().collect::<BTreeSet<_>>().len());
    drop(operation);
    assert_eq!(
        reader
            .native()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM sqlite_temp_master WHERE name='selected_sections'",
                [],
                |row| row.get::<_, usize>(0)
            )
            .unwrap(),
        1
    );
    // Exercise the actual bundled JSON consumer at and beyond floating precision.
    let mut store = EligibilityStore::new();
    let values = [
        i64::MIN,
        9_007_199_254_740_993,
        i64::MAX,
        9_007_199_254_740_993,
    ];
    let json = store
        .get_or_try_insert::<_, IndexError>(&0, |writer| {
            for n in values {
                assert!(writer.push_id(n));
            }
            Ok(())
        })
        .unwrap()
        .unwrap();
    let exact = reader
        .native()
        .unwrap()
        .prepare("SELECT value,typeof(value) FROM json_each(?1)")
        .unwrap()
        .query_map([json], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(exact, values.map(|n| (n, "integer".to_owned())).to_vec());
    let count: usize = reader
        .native()
        .unwrap()
        .query_row(
            "SELECT count(DISTINCT value) FROM json_each(?1)",
            [json],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn native_execution_and_malformed_eligibility_failures_keep_the_original_error_order() {
    let input = crate::core_tests::nested_fixture(true);
    let (_dir, path) = crate::foundation_tests::build(&input);
    let model = &input.context.search_profiles[0];
    let table = crate::schema::vector_table(model).unwrap();
    mutate(
        &path,
        &format!("UPDATE {table}_vector_chunks00 SET vectors=x'00'"),
    );
    let q = crate::validate_query(&QueryPredicate::boolean(true)).unwrap();
    let v = vector();
    {
        let reader = test_support::open_reader(&path).unwrap();
        let r = request(&model.profile_id, &q, &v);
        let exact = reader.similarity_candidates(r).unwrap_err().to_string();
        assert!(
            exact.contains("vectors blob size"),
            "must exercise an executing native failure: {exact}"
        );
        let mut operation = reader.configured_similarity_candidates();
        assert_eq!(
            operation
                .candidates::<IndexError>(r, || Ok(()))
                .unwrap_err()
                .to_string(),
            exact
        );
    }
    let reader = test_support::open_reader(&path).unwrap();
    reader
        .native()
        .unwrap()
        .execute_batch(
            "CREATE TEMP TABLE selected_sections(eligibility_column_unavailable INTEGER)",
        )
        .unwrap();
    let r = request(&model.profile_id, &q, &v);
    let exact = reader.similarity_candidates(r).unwrap_err().to_string();
    assert!(
        exact.contains("no such column"),
        "eligibility preparation failure must win: {exact}"
    );
    let mut operation = reader.configured_similarity_candidates();
    assert_eq!(
        operation
            .candidates::<IndexError>(r, || Ok(()))
            .unwrap_err()
            .to_string(),
        exact
    );
    assert_eq!(operation.stats().ready, 0);
}
