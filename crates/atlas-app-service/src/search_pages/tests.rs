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
        criteria,
        page: SearchPageRequest { number, size: 1 },
        expected_artifact_generation: None,
    }
}

fn levels_at_least_two() -> atlas_domain::QueryPredicate {
    serde_json::from_value(json!({"kind":"compare","field":"item.level","op":"gte","value":2}))
        .unwrap()
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
        atlas_record::source_content::OwnedContentIdentity::Stable("owned00000000000".into())
    );
    assert_eq!(witness.label.as_deref(), Some("Owned flare"));
}

#[test]
fn similar_pages_preserve_seed_exclusion_filtering_and_bounded_coverage() {
    let (f, _) = fixture(true);
    let mut criteria = request(
        SearchCriteria::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
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
                    record.preparation_context_hash = built.context.sha256()?;
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
