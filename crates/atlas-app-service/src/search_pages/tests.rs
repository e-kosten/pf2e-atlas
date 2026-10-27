use crate::executor::RetrievalExecutor;
use crate::test_support::{FixtureWorker, fixture_worker_with_executor};
use atlas_app_model::{
    AppErrorCode, AppRecoverableAction, RetrievalModeView, SearchCriteria, SearchModeSummary,
    SearchPageRequest, SearchResultsRequest,
};
use atlas_search::test_support::{input, open_input_fixture, open_source_fixture, record};
use serde_json::json;

fn fixture(semantic: bool) -> (FixtureWorker, RetrievalExecutor) {
    let executor = RetrievalExecutor::from_test_fixture_factory(2, 16, move || {
        Ok(open_source_fixture(
            (0..4)
                .map(|i| {
                    record(
                        "items",
                        "Item",
                        json!({"_id":format!("{i:016}"),"name":format!("Record {i}"),"type":"effect","system":{"level":{"value":i},"traits":{"value":[]},"description":{"value":"<p>Useful prose</p>"}}}),
                    )
                })
                .chain([record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000000","name":"Reader","type":"npc","items":[{"_id":"owned00000000000","type":"action","name":"Owned flare","system":{"description":{"value":"<p>Useful owned mechanics</p>"}}}]}),
                )])
                .collect(),
            semantic,
        )?)
    });
    (fixture_worker_with_executor(executor.clone()), executor)
}

fn request(criteria: SearchCriteria, number: u32) -> SearchResultsRequest {
    SearchResultsRequest {
        sort: None,

        ranking: None,
        search_profile: None,
        criteria,
        page: SearchPageRequest { number, size: 1 },
        expected_artifact_generation: None,
    }
}

#[test]
fn selected_profile_is_request_local_and_stored_similar_and_lexical_need_no_model_assets() {
    let executor = RetrievalExecutor::from_test_fixture_factory(2, 16, || {
        Ok(open_input_fixture(atlas_search::test_support::input_with_both_profiles((0..3).map(|i|record("items","Item",json!({"_id":format!("{i:016}"),"name":format!("Record {i}"),"type":"effect"}))).collect()))?)
    });
    let worker = fixture_worker_with_executor(executor).worker;
    let execution = crate::ReadExecutionContext::uncancellable();
    let mut similar = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
            similarity: Some(atlas_domain::SimilarityOptions::Baseline {}),
            filter: None,
        },
        1,
    );
    similar.search_profile = Some(atlas_domain::SearchProfileId::new("embeddinggemma"));
    let gemma = worker.search_page(similar.clone(), &execution).unwrap();
    assert_eq!(gemma.search_profile.as_str(), "embeddinggemma");
    assert!(!gemma.rows.is_empty());
    similar.search_profile = None;
    let base = worker.search_page(similar.clone(), &execution).unwrap();
    assert_eq!(base.search_profile.as_str(), "bge-base");
    assert_eq!(base.artifact_generation, gemma.artifact_generation);
    let mut lexical = request(
        SearchCriteria::TextSearch {
            query: "Record".into(),
            filter: None,
            mode: RetrievalModeView::Lexical,
        },
        1,
    );
    lexical.search_profile = Some(atlas_domain::SearchProfileId::new("embeddinggemma"));
    assert_eq!(
        worker
            .search_page(lexical.clone(), &execution)
            .unwrap()
            .search_profile
            .as_str(),
        "embeddinggemma"
    );
    lexical.search_profile = Some(atlas_domain::SearchProfileId::new("missing"));
    assert!(
        worker
            .search_page(lexical, &execution)
            .unwrap_err()
            .into_app_error()
            .message
            .contains("missing")
    );
    similar.search_profile = Some(atlas_domain::SearchProfileId::new("missing"));
    assert!(
        worker
            .search_page(similar, &execution)
            .unwrap_err()
            .into_app_error()
            .message
            .contains("missing")
    );
}

#[test]
fn selected_contextual_facets_reject_stale_generation_before_query_inference() {
    let (worker, _) = fixture(true);
    let error = worker
        .worker
        .discover_filter_counts(
            atlas_app_model::DiscoverFilterCountsRequest {
                context: atlas_app_model::FilterDiscoveryContext::Filtered {
                    search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
                    expected_artifact_generation: Some("stale-generation".into()),
                    filter: None,
                    text: Some("query needing unavailable native session".into()),
                    mode: RetrievalModeView::Semantic,
                },
                field_id: "kind".into(),
                clause_id: None,
                values: vec![],
            },
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap_err();
    assert_eq!(
        error.into_app_error().code,
        AppErrorCode::ArtifactGenerationChanged
    );
}

fn levels_at_least_two() -> atlas_domain::QueryPredicate {
    serde_json::from_value(json!({"kind":"compare","field":"level","op":"gte","value":2})).unwrap()
}

#[test]
fn browse_pages_keep_exact_filtered_totals_and_need_no_retained_handle() {
    let (f, _) = fixture(false);
    let mut criteria = request(
        SearchCriteria::ListRecords {
            filter: Some(levels_at_least_two()),
        },
        1,
    );
    let first = f
        .worker
        .search_page(
            criteria.clone(),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(first.mode, SearchModeSummary::ListRecords);
    assert_eq!(first.page.total, 2);
    assert!(first.coverage.is_none());
    assert_eq!(first.page.next_page, Some(2));
    assert!(first.rows[0].matches.is_empty());
    criteria.expected_artifact_generation = Some(first.artifact_generation.clone());
    criteria.page.number = 2;
    let second = f
        .worker
        .search_page(
            criteria.clone(),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_ne!(
        first.rows[0].record.record_key,
        second.rows[0].record.record_key
    );
    assert_eq!(second.artifact_generation, first.artifact_generation);
    assert!(!second.page.has_more);
    assert_eq!(second.page.total, 2);

    // Independent preview searches do not allocate or evict continuation state.
    for _ in 0..65 {
        f.worker
            .search_page(
                request(SearchCriteria::ListRecords { filter: None }, 1),
                &crate::ReadExecutionContext::uncancellable(),
            )
            .unwrap();
    }
    assert_eq!(
        f.worker
            .search_page(
                criteria.clone(),
                &crate::ReadExecutionContext::uncancellable()
            )
            .unwrap(),
        second
    );
    criteria.page.number = 3;
    let beyond_last = f
        .worker
        .search_page(criteria, &crate::ReadExecutionContext::uncancellable())
        .unwrap();
    assert!(beyond_last.rows.is_empty());
    assert_eq!(beyond_last.page.total, 2);
    assert!(!beyond_last.page.has_more);
}

#[test]
fn lexical_pages_preserve_filters_counts_and_exact_owned_witnesses() {
    let (f, _) = fixture(false);
    let mut criteria = request(
        SearchCriteria::TextSearch {
            query: "Record".into(),
            filter: Some(levels_at_least_two()),
            mode: RetrievalModeView::Lexical,
        },
        1,
    );
    let first = f
        .worker
        .search_page(
            criteria.clone(),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(first.page.total, 2);
    assert!(first.coverage.as_ref().unwrap().exhaustive);
    assert_eq!(first.coverage.as_ref().unwrap().semantic_unit_window, None);
    criteria.expected_artifact_generation = Some(first.artifact_generation.clone());
    criteria.page.number = 2;
    let second = f
        .worker
        .search_page(criteria, &crate::ReadExecutionContext::uncancellable())
        .unwrap();
    assert_eq!(second.page.total, first.page.total);
    assert_ne!(
        first.rows[0].record.record_key,
        second.rows[0].record.record_key
    );
    let owned = f
        .worker
        .search_page(
            request(
                SearchCriteria::TextSearch {
                    query: "Owned flare".into(),
                    filter: None,
                    mode: RetrievalModeView::Lexical,
                },
                1,
            ),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(owned.rows[0].record.record_key, "actors:actor00000000000");
    let witness = &owned.rows[0].matches[0];
    assert_eq!(witness.navigation.owners.len(), 1);
    assert_eq!(
        witness.navigation.owners[0].identity,
        atlas_domain::OwnedContentIdentity::Stable("owned00000000000".into())
    );
    assert_eq!(witness.label.as_deref(), Some("Owned flare"));
    let sections = owned.rows[0].matching_sections.as_ref().unwrap();
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].navigation, witness.navigation);
    assert!(owned.rows[0].similarity.is_none());
}

#[test]
fn similar_pages_preserve_seed_exclusion_filtering_and_bounded_coverage() {
    let (f, _) = fixture(true);
    let mut criteria = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
            similarity: Some(atlas_domain::SimilarityOptions::Baseline {}),
            filter: Some(levels_at_least_two()),
        },
        1,
    );
    let first = f
        .worker
        .search_page(
            criteria.clone(),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    let SearchModeSummary::SimilarRecords { seed } = &first.mode else {
        panic!("expected Similar summary");
    };
    assert_eq!(seed.title, "Record 0");
    assert_eq!(first.page.total, 2);
    assert_ne!(first.rows[0].record.record_key, seed.record_key);
    assert!(!first.coverage.as_ref().unwrap().exhaustive);
    assert!(
        serde_json::to_value(seed)
            .unwrap()
            .get("presentation")
            .is_none()
    );
    criteria.expected_artifact_generation = Some(first.artifact_generation.clone());
    criteria.page.number = 2;
    let second = f
        .worker
        .search_page(criteria, &crate::ReadExecutionContext::uncancellable())
        .unwrap();
    assert_eq!(second.coverage, first.coverage);
    assert_eq!(second.mode, first.mode);
    assert_ne!(
        second.rows[0].record.record_key,
        first.rows[0].record.record_key
    );
    assert!(!second.page.has_more);
}

#[test]
fn changed_generation_rejects_semantic_and_hybrid_pages_before_query_embedding() {
    let (f, _) = fixture(false);
    let generation = f
        .worker
        .search_page(
            request(SearchCriteria::ListRecords { filter: None }, 1),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap()
        .artifact_generation;
    for mode in [RetrievalModeView::Semantic, RetrievalModeView::Hybrid] {
        let mut query = request(
            SearchCriteria::TextSearch {
                query: "Record".into(),
                filter: None,
                mode,
            },
            2,
        );
        query.expected_artifact_generation = Some("previous-generation".into());
        let error = f
            .worker
            .search_page(query.clone(), &crate::ReadExecutionContext::uncancellable())
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
        assert_eq!(error.retryable, Some(false));
        assert_eq!(
            error.recoverable_action,
            Some(AppRecoverableAction::RefreshResults)
        );
        assert_eq!(
            error.details,
            Some(
                json!({"expected_artifact_generation":"previous-generation","actual_artifact_generation":generation})
            )
        );
        // Without the stale expectation, this same fixture reaches model readiness.
        query.expected_artifact_generation = Some(generation.clone());
        assert_eq!(
            f.worker
                .search_page(query, &crate::ReadExecutionContext::uncancellable())
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::VectorReadinessRequired
        );
    }
}

#[test]
fn anchored_similar_request_rejects_a_stale_detail_generation() {
    let (f, _) = fixture(true);
    let mut query = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
            similarity: Some(atlas_domain::SimilarityOptions::Baseline {}),
            filter: None,
        },
        1,
    );
    query.expected_artifact_generation = Some("old-detail-generation".into());
    assert_eq!(
        f.worker
            .search_page(query, &crate::ReadExecutionContext::uncancellable())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::ArtifactGenerationChanged
    );
}

#[test]
fn restarted_service_rejects_prior_pages_when_only_build_context_changes() {
    let service = |revision: &'static str| {
        fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                let mut built = input(
                    vec![record(
                        "items",
                        "Item",
                        json!({"_id":"record0000000000","type":"effect","name":"Stable record","system":{"description":{"value":"<p>Stable content</p>"}}}),
                    )],
                    false,
                );
                built.context.source_revision = Some(revision.into());
                for record in &mut built.records {
                    record.preparation_context_hash = built.context.preparation_sha256()?;
                }
                Ok(open_input_fixture(built)?)
            },
        ))
    };
    let a = service("revision-a");
    let b = service("revision-b");
    let first = a
        .worker
        .search_page(
            request(SearchCriteria::ListRecords { filter: None }, 1),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    let mut next = request(SearchCriteria::ListRecords { filter: None }, 2);
    next.expected_artifact_generation = Some(first.artifact_generation.clone());
    let error = b
        .worker
        .search_page(next, &crate::ReadExecutionContext::uncancellable())
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
    let refreshed = b
        .worker
        .search_page(
            request(SearchCriteria::ListRecords { filter: None }, 1),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_ne!(first.artifact_generation, refreshed.artifact_generation);
    assert_eq!(first.rows, refreshed.rows);
    assert_eq!(
        a.worker
            .record_detail("items:record0000000000")
            .unwrap()
            .selected
            .source_fingerprint,
        b.worker
            .record_detail("items:record0000000000")
            .unwrap()
            .selected
            .source_fingerprint,
        "source navigation identity is separate from complete build generation",
    );
}

#[test]
fn invalid_pages_and_missing_similar_seeds_remain_explicit_errors() {
    let (f, _) = fixture(true);
    let mut browse = request(SearchCriteria::ListRecords { filter: None }, 1);
    browse.page.size = 101;
    assert_eq!(
        f.worker
            .search_page(browse, &crate::ReadExecutionContext::uncancellable())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    let missing = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:missing000000000".into(),
            similarity: Some(atlas_domain::SimilarityOptions::Baseline {}),
            filter: None,
        },
        1,
    );
    assert_eq!(
        f.worker
            .search_page(missing, &crate::ReadExecutionContext::uncancellable())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::RecordNotFound
    );
}

#[test]
fn relationship_pages_label_exact_entries_and_page_evidence_with_generation_guards() {
    use atlas_app_model::RelationshipOccurrencesRequest;
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![
                record(
                    "items",
                    "Item",
                    json!({"_id":"target0000000000","name":"Leather Armor","type":"armor"}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000000","name":"Guard","type":"npc","items":[{"_id":"owned00000000000","name":"Guard's armor","type":"armor","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.target0000000000"},"system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.target0000000000]</p>"}}}]}),
                ),
            ],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let execution = crate::ReadExecutionContext::uncancellable();
    let target = f.worker.record_detail("items:target0000000000").unwrap();
    let constraint = atlas_domain::QueryRelationship {
        direction: atlas_domain::QueryRelationshipDirection::Incoming,
        selection: crate::relationship_search::query_selection(&target.selected).unwrap(),
        include_contents: true,
    };
    let mut criteria = request(SearchCriteria::ListRecords { filter: None }, 1);
    let filter = relationship_predicate(constraint.clone());
    criteria.criteria = SearchCriteria::ListRecords {
        filter: Some(filter.clone()),
    };
    let result = f.worker.search_page(criteria.clone(), &execution).unwrap();
    assert_eq!(result.relationship_clauses[0].title, "Leather Armor");
    assert_eq!(result.page.total, 1);
    assert_eq!(result.rows[0].relationship_evidence[0].page.total, 2);
    assert_eq!(
        result.rows[0].relationship_evidence[0].occurrences[0]
            .source_title
            .as_deref(),
        Some("Guard's armor")
    );
    assert!(
        result.rows[0].relationship_evidence[0]
            .occurrences
            .iter()
            .any(|o| o.origin == "structured" && o.kind == "CompendiumProvenance")
    );
    assert!(
        result.rows[0].relationship_evidence[0]
            .occurrences
            .iter()
            .any(|o| o.origin == "content")
    );
    let page = f
        .worker
        .relationship_occurrences(
            RelationshipOccurrencesRequest {
                filter: filter.clone(),
                clause_id: "link".into(),
                result_record_key: result.rows[0].record.record_key.clone(),
                page: SearchPageRequest { number: 2, size: 1 },
                expected_artifact_generation: result.artifact_generation.clone(),
            },
            &execution,
        )
        .unwrap();
    assert_eq!((page.page.total, page.occurrences.len()), (2, 1));
    assert!(!page.page.has_more);
    let mut main = constraint.clone();
    main.include_contents = false;
    criteria.criteria = SearchCriteria::ListRecords {
        filter: Some(relationship_predicate(main)),
    };
    assert_eq!(
        f.worker
            .search_page(criteria.clone(), &execution)
            .unwrap()
            .page
            .total,
        0
    );
    let mut invalid_constraint = constraint.clone();
    invalid_constraint.selection.owners = vec![atlas_domain::OwnedContentLocator {
        collection: "/items".into(),
        identity: atlas_domain::OwnedContentIdentity::SnapshotLocal { index: 0 },
    }];
    criteria.criteria = SearchCriteria::TextSearch {
        query: "needs unavailable inference".into(),
        filter: Some(relationship_predicate(invalid_constraint)),
        mode: RetrievalModeView::Semantic,
    };
    let invalid = f
        .worker
        .search_page(criteria.clone(), &execution)
        .unwrap_err()
        .into_app_error();
    assert_eq!(
        invalid.code,
        AppErrorCode::InvalidRequest,
        "selection fingerprint must fail before unavailable inference"
    );
    criteria.expected_artifact_generation = Some("stale".into());
    let invalid = f
        .worker
        .search_page(criteria, &execution)
        .unwrap_err()
        .into_app_error();
    assert_eq!(invalid.code, AppErrorCode::ArtifactGenerationChanged);
    let invalid = f
        .worker
        .relationship_occurrences(
            RelationshipOccurrencesRequest {
                filter,
                clause_id: "link".into(),
                result_record_key: "actors:actor00000000000".into(),
                page: SearchPageRequest { number: 1, size: 1 },
                expected_artifact_generation: "stale".into(),
            },
            &execution,
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(invalid.code, AppErrorCode::ArtifactGenerationChanged);
}

#[test]
fn relationship_source_navigation_opens_structured_owner_and_precise_prose_field() {
    use atlas_app_model::RecordDetailRequest;
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![
                record(
                    "items",
                    "Item",
                    json!({"_id":"target0000000000","name":"Leather Armor","type":"armor"}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000000","name":"Guard","type":"npc","items":[{"_id":"owned00000000000","name":"Guard's armor","type":"armor","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.target0000000000"},"system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.target0000000000]</p>"}}}]}),
                ),
            ],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let execution = crate::ReadExecutionContext::uncancellable();
    let target = f.worker.record_detail("items:target0000000000").unwrap();
    let mut criteria = request(SearchCriteria::ListRecords { filter: None }, 1);
    criteria.criteria = SearchCriteria::ListRecords {
        filter: Some(relationship_predicate(atlas_domain::QueryRelationship {
            direction: atlas_domain::QueryRelationshipDirection::Incoming,
            selection: crate::relationship_search::query_selection(&target.selected).unwrap(),
            include_contents: true,
        })),
    };
    let result = f.worker.search_page(criteria, &execution).unwrap();
    let occurrences = &result.rows[0].relationship_evidence[0].occurrences;
    for origin in ["structured", "content"] {
        let occurrence = occurrences.iter().find(|o| o.origin == origin).unwrap();
        let navigation = &occurrence.source_navigation;
        assert!(
            !navigation.owners.is_empty(),
            "must open the referring item, not its parent"
        );
        assert_eq!(navigation.owners, occurrence.source.owners);
        if origin == "structured" {
            assert_eq!(occurrence.kind, "CompendiumProvenance");
            assert_eq!(
                occurrence.source.field, "/_stats/compendiumSource",
                "raw provenance remains exact"
            );
            assert!(navigation.field.is_none());
        } else {
            assert_eq!(
                navigation.field.as_deref(),
                Some("/system/description/value")
            );
            assert_eq!(
                navigation.field.as_deref(),
                Some(occurrence.source.field.as_str())
            );
        }
        let opened = f
            .worker
            .record_detail_at(RecordDetailRequest {
                spell_rank: navigation.spell_rank,
                record_key: navigation.record_key.clone(),
                owners: navigation.owners.clone(),
                fields: navigation.field.iter().cloned().collect(),
                passage: navigation.passage.clone(),
                source_fingerprint: navigation.source_fingerprint.clone(),
            })
            .unwrap();
        assert_eq!(opened.selected.record_key, navigation.record_key);
        assert_eq!(opened.selected.owners, navigation.owners);
        assert_eq!(opened.selected.field, navigation.field);
        assert_eq!(opened.selected.passage, navigation.passage);
    }
}

fn relationship_predicate(
    relationship: atlas_domain::QueryRelationship,
) -> atlas_domain::QueryPredicate {
    atlas_domain::QueryPredicate {
        clause_id: Some("link".into()),
        expression: atlas_domain::QueryExpression::Relationship { relationship },
    }
}

fn trait_search_fixture(semantic: bool, reference_count: u32) -> FixtureWorker {
    let executor = RetrievalExecutor::from_test_fixture_factory(2, 16, move || {
        let mut fixture = input(
            (0..=reference_count).map(|i| record("items", "Item", json!({
                "_id": format!("{i:016}"), "name": format!("Record {i}"), "type": "effect",
                "system": {"level": {"value": i}, "description": {"value":
                    if i < reference_count { "@Localize[PF2E.TraitAgile] @Localize[PF2E.TraitAgile]" } else { "Unrelated" }
                }}
            }))).collect(), semantic,
        );
        let key = atlas_domain::TraitKey::new("agile").unwrap();
        fixture.traits.push(atlas_search::TraitDefinition {
            key: key.clone(),
            label: "Agile".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec![],
            source_locations: vec![],
            source_revision: fixture.context.source_revision.clone(),
            indexing_locale: fixture.context.indexing_locale.clone(),
        });
        if semantic {
            let mut vector = vec![0.; 768];
            vector[0] = 1.;
            let chunk_hash = atlas_search::test_support::hash(b"Agile\n");
            fixture
                .trait_vectors
                .push(atlas_search::test_support::TraitVectorInput {
                    key,
                    profile_id: atlas_domain::SearchProfileId::new("bge-base"),
                    input_hash: atlas_search::test_support::hash(
                        format!("trait-definition-centroid/v1:{chunk_hash}").as_bytes(),
                    ),
                    vector: vector.clone(),
                    chunks: vec![atlas_search::test_support::TraitSeedChunk {
                        input: "Agile\n".into(),
                        input_hash: chunk_hash.clone(),
                        token_count: 2,
                        vector,
                        address: atlas_domain::SourcePassageAddress::PlainSection {
                            source_text_sha256: chunk_hash,
                            selection_version: "trait-definition-centroid/v1".into(),
                            section_ordinal: 0,
                            label: Some("Agile".into()),
                            chunk_bytes: atlas_domain::SourceByteRange { start: 0, end: 6 },
                        },
                    }],
                });
        }
        Ok(open_input_fixture(fixture)?)
    });
    fixture_worker_with_executor(executor)
}

#[test]
fn trait_reference_search_deduplicates_roots_and_filters_before_paging() {
    let fixture = trait_search_fixture(false, 3);
    let execution = crate::ReadExecutionContext::uncancellable();
    let mut search = request(
        SearchCriteria::TraitReferences {
            trait_key: "trait:agile".into(),
            query: None,
            filter: None,
        },
        1,
    );
    let first = fixture
        .worker
        .search_page(search.clone(), &execution)
        .unwrap();
    assert_eq!(
        first.mode,
        SearchModeSummary::TraitReferences {
            trait_key: "trait:agile".into(),
            title: "Agile".into(),
        }
    );
    assert_eq!(first.page.total, 3);
    search.page.number = 3;
    let last = fixture
        .worker
        .search_page(search.clone(), &execution)
        .unwrap();
    assert_eq!(last.rows[0].record.record_key, "items:0000000000000002");
    assert!(!last.page.has_more);
    search.page.number = 1;
    search.criteria = SearchCriteria::TraitReferences {
        trait_key: "trait:agile".into(),
        query: None,
        filter: Some(levels_at_least_two()),
    };
    let filtered = fixture.worker.search_page(search, &execution).unwrap();
    assert_eq!(filtered.page.total, 1);
    assert_eq!(filtered.rows[0].record.record_key, "items:0000000000000002");
}

#[test]
fn similar_trait_search_uses_persisted_seed_and_normal_filters() {
    let fixture = trait_search_fixture(true, 3);
    let execution = crate::ReadExecutionContext::uncancellable();
    let mut search = request(
        SearchCriteria::SimilarTraits {
            seed_trait_key: "trait:agile".into(),
            similarity: None,
            filter: None,
        },
        1,
    );
    let first = fixture
        .worker
        .search_page(search.clone(), &execution)
        .unwrap();
    assert_eq!(
        first.mode,
        SearchModeSummary::SimilarTraits {
            trait_key: "trait:agile".into(),
            title: "Agile".into(),
        }
    );
    assert!(!first.rows.is_empty());
    assert!(first.coverage.is_some());
    assert_eq!(
        first.similarity.unwrap().options,
        atlas_domain::SimilarityOptions::Baseline {}
    );
    search.criteria = SearchCriteria::SimilarTraits {
        seed_trait_key: "trait:agile".into(),
        similarity: Some(atlas_domain::SimilarityOptions::Baseline {}),
        filter: Some(
            serde_json::from_value(json!({
                "kind": "boolean_constant", "value": false
            }))
            .unwrap(),
        ),
    };
    assert!(
        fixture
            .worker
            .search_page(search, &execution)
            .unwrap()
            .rows
            .is_empty()
    );
}

#[test]
fn trait_reference_search_reaches_records_beyond_association_preview_limits() {
    let fixture = trait_search_fixture(false, 131);
    let result = fixture
        .worker
        .search_page(
            request(
                SearchCriteria::TraitReferences {
                    trait_key: "trait:agile".into(),
                    query: None,
                    filter: None,
                },
                131,
            ),
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(result.page.total, 131);
    assert_eq!(result.rows[0].record.record_key, "items:0000000000000099");
    assert!(!result.page.has_more);
}

#[test]
fn trait_reference_text_search_rejects_stale_generation_before_query_inference() {
    let fixture = trait_search_fixture(true, 3);
    let mut search = request(
        SearchCriteria::TraitReferences {
            trait_key: "trait:agile".into(),
            query: Some("query needing unavailable model assets".into()),
            filter: None,
        },
        1,
    );
    search.expected_artifact_generation = Some("retired-generation".into());
    let error = fixture
        .worker
        .search_page(search, &crate::ReadExecutionContext::uncancellable())
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
}

#[test]
fn similarity_inventory_and_explicit_parts_share_generation_and_effective_configuration() {
    let executor = RetrievalExecutor::from_test_fixture_factory(2, 16, || {
        let mut data = input((0..2).map(|i| record("items", "Item", json!({
            "_id":format!("{i:016}"),"name":format!("Related record {i}"),"type":"effect",
            "system":{"description":{"value":"<p>Shared explanatory content</p>"}}
        }))).collect(), true);
        for unit in &mut data.semantic_units {
            unit.vector.fill(0.0);
            unit.vector[0] = 1.0;
        }
        Ok(open_input_fixture(data)?)
    });
    let fixture = fixture_worker_with_executor(executor);
    let execution = crate::ReadExecutionContext::uncancellable();
    let seed = atlas_domain::SimilaritySeed::Record {
        record_key: "items:0000000000000000".parse().unwrap(),
    };
    let inventory = fixture
        .worker
        .similarity_parts(
            atlas_app_model::SimilarityPartsRequest {
                seed,
                search_profile: None,
                expected_artifact_generation: None,
            },
            &execution,
        )
        .unwrap();
    assert!(!inventory.parts.is_empty());
    assert!(
        inventory
            .parts
            .iter()
            .all(|part| !part.role_label.is_empty())
    );
    let options = atlas_domain::SimilarityOptions::Passages {
        decay: 0.123456,
        parts: Some(vec![inventory.parts[0].selector.clone()]),
        candidate_parts: atlas_domain::SimilarityCandidateParts::Any,
    };
    let mut query = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
            filter: None,
            similarity: Some(options.clone()),
        },
        1,
    );
    assert_eq!(
        fixture
            .worker
            .search_page(query.clone(), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    query.expected_artifact_generation = Some(inventory.artifact_generation);
    let page = fixture
        .worker
        .search_page(query.clone(), &execution)
        .unwrap();
    assert!(!page.rows.is_empty());
    let configuration = page.similarity.as_ref().unwrap();
    assert_eq!(configuration.options, options);
    assert_eq!(configuration.role_recipe, inventory.role_recipe);
    let serialized = serde_json::to_value(&page).unwrap();
    assert_eq!(
        serialized["similarity"]["role_recipe"],
        inventory.role_recipe
    );
    assert!(page.rows.iter().all(|row| row.similarity.is_some()));
    for row in &page.rows {
        let destinations = row
            .matching_sections
            .as_ref()
            .expect("Similar passage destinations accompany comparison evidence");
        assert!(!destinations.is_empty());
        assert!(destinations.iter().all(|section| {
            row.matches
                .iter()
                .any(|witness| witness.navigation == section.navigation)
        }));
        for score in &row.similarity.as_ref().unwrap().parts {
            let source = inventory
                .parts
                .iter()
                .find(|part| part.selector == score.part.selector)
                .unwrap();
            assert_eq!(&score.part, source);
            assert!(score.part.navigation.is_some());
        }
    }
    query.expected_artifact_generation = Some("stale-inventory-generation".into());
    assert_eq!(
        fixture
            .worker
            .search_page(query, &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::ArtifactGenerationChanged
    );
}

#[test]
fn similarity_part_inventory_remains_available_without_vectors_or_model_assets() {
    let (fixture, _) = fixture(false);
    let inventory = fixture
        .worker
        .similarity_parts(
            atlas_app_model::SimilarityPartsRequest {
                seed: atlas_domain::SimilaritySeed::Record {
                    record_key: "items:0000000000000000".parse().unwrap(),
                },
                search_profile: None,
                expected_artifact_generation: None,
            },
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert!(!inventory.parts.is_empty());
    assert!(inventory.parts.iter().all(|part| !part.has_vectors));
}

#[test]
fn similarity_section_labels_preserve_authored_headings_and_navigate_checked_sources() {
    for semantic in [false, true] {
        let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
            let sources = vec![record(
                "actors",
                "Actor",
                json!({
                    "_id":"aaaaaaaaaaaaaaaa", "name":"Seed", "type":"npc",
                    "system":{"details":{"publicNotes":"<p>Unheaded introduction</p><h2>Public notes</h2><p>Authored heading content</p>","blurb":"Plain authored blurb"}},
                    "items":[{"name":"Local ability","type":"action","system":{"description":{"value":"<h2>Ability heading</h2><p>Owned content</p>"}}}, {"name":"Local unheaded ability","type":"action","system":{"description":{"value":"<p>Unheaded owned content</p>"}}}]
                }),
            )];
            let data = if semantic {
                atlas_search::test_support::input_with_both_profiles(sources)
            } else {
                input(sources, false)
            };
            Ok(open_input_fixture(data)?)
        });
        let fixture = fixture_worker_with_executor(executor);
        let execution = crate::ReadExecutionContext::uncancellable();
        let request = atlas_app_model::SimilarityPartsRequest {
            seed: atlas_domain::SimilaritySeed::Record {
                record_key: "actors:aaaaaaaaaaaaaaaa".parse().unwrap(),
            },
            search_profile: None,
            expected_artifact_generation: None,
        };
        let inventory = fixture
            .worker
            .similarity_parts(request.clone(), &execution)
            .unwrap();
        if semantic {
            let mut other_profile = request.clone();
            other_profile.search_profile =
                Some(atlas_domain::SearchProfileId::new("embeddinggemma"));
            other_profile.expected_artifact_generation =
                Some(inventory.artifact_generation.clone());
            let other = fixture
                .worker
                .similarity_parts(other_profile, &execution)
                .unwrap();
            assert_eq!(inventory.artifact_generation, other.artifact_generation);
            assert_eq!(inventory.parts, other.parts);
        }
        let introduction = inventory
            .parts
            .iter()
            .find(|part| {
                part.owners.is_empty()
                    && part.field == "/system/details/publicNotes"
                    && part.section_ordinal == 0
            })
            .unwrap();
        assert_eq!(introduction.label, "Description");
        assert_eq!(
            introduction.navigation.as_ref().unwrap().passage.is_some(),
            semantic
        );
        let heading = inventory
            .parts
            .iter()
            .find(|part| part.label == "Public notes")
            .unwrap();
        assert!(heading.navigation.is_some());
        let owned = inventory
            .parts
            .iter()
            .find(|part| part.owner_label == "Local ability")
            .unwrap();
        assert_eq!(owned.label, "Ability heading");
        assert!(
            owned
                .navigation
                .as_ref()
                .unwrap()
                .source_fingerprint
                .is_some()
        );
        let unheaded_owned = inventory
            .parts
            .iter()
            .find(|part| part.owner_label == "Local unheaded ability")
            .unwrap();
        assert_eq!(
            unheaded_owned
                .navigation
                .as_ref()
                .unwrap()
                .passage
                .is_some(),
            semantic
        );
        assert!(
            unheaded_owned
                .navigation
                .as_ref()
                .unwrap()
                .source_fingerprint
                .is_some()
        );
        let plain = inventory
            .parts
            .iter()
            .find(|part| part.field == "/system/details/blurb")
            .unwrap();
        assert_eq!(
            plain.navigation.as_ref().unwrap().passage.is_some(),
            semantic
        );
        for part in [introduction, heading, owned, unheaded_owned, plain] {
            let navigation = part.navigation.as_ref().unwrap();
            assert_eq!(navigation.field.as_deref(), Some(part.field.as_str()));
            assert_eq!(navigation.owners, part.owners);
            let detail = fixture
                .worker
                .record_detail_at(atlas_app_model::RecordDetailRequest {
                    spell_rank: None,
                    record_key: navigation.record_key.clone(),
                    owners: navigation.owners.clone(),
                    fields: vec![part.field.clone()],
                    passage: navigation.passage.clone(),
                    source_fingerprint: navigation.source_fingerprint.clone(),
                })
                .unwrap();
            assert_eq!(detail.artifact_generation, inventory.artifact_generation);
            assert_eq!(&detail.selected, navigation);
        }
        let mut stale = request;
        stale.expected_artifact_generation = Some("stale-source-generation".into());
        assert_eq!(
            fixture
                .worker
                .similarity_parts(stale, &execution)
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::ArtifactGenerationChanged
        );
    }
}

#[test]
fn similar_record_and_trait_responses_acknowledge_scoring_and_mapping_recipes() {
    let fixture = trait_search_fixture(true, 3);
    let execution = crate::ReadExecutionContext::uncancellable();
    for trait_seed in [false, true] {
        for options in [
            atlas_domain::SimilarityOptions::Baseline {},
            atlas_domain::SimilarityOptions::default(),
        ] {
            let criteria = if trait_seed {
                SearchCriteria::SimilarTraits {
                    seed_trait_key: "trait:agile".into(),
                    filter: None,
                    similarity: Some(options.clone()),
                }
            } else {
                SearchCriteria::SimilarRecords {
                    seed_record_key: "items:0000000000000000".into(),
                    filter: None,
                    similarity: Some(options.clone()),
                }
            };
            let page = fixture
                .worker
                .search_page(request(criteria, 1), &execution)
                .unwrap();
            let value = serde_json::to_value(&page).unwrap();
            assert_eq!(
                value["similarity"]["role_recipe"],
                atlas_domain::SIMILARITY_ROLE_RECIPE
            );
            assert_eq!(
                value["similarity"]["options"],
                serde_json::to_value(&options).unwrap()
            );
            assert_eq!(
                value["similarity"]["recipe"],
                if matches!(options, atlas_domain::SimilarityOptions::Baseline {}) {
                    "current-baseline/v1"
                } else {
                    atlas_domain::SIMILARITY_RECIPE
                }
            );
        }
    }
}
