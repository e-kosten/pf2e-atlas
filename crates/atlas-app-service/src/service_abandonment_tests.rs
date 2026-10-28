use std::sync::atomic::{AtomicBool, Ordering};

use super::*;
use atlas_app_model::{
    DiscoverFilterCountsRequest, DiscoverFilterValuesRequest, FilterDiscoveryContext,
    FilterVocabularyScope, RetrievalModeView,
};
use atlas_app_model::{SearchCriteria, SearchPageRequest, SearchResultsRequest};

fn missing_artifact_options() -> AtlasRuntimeOptions {
    AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: Some(std::env::temp_dir().join(format!(
                "absent-abandoned-read-{}.sqlite",
                std::process::id()
            ))),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn abandoned_public_read_never_opens_lazy_command_reader() {
    let service = AtlasAppService::new(
        RetrievalBackend::command(CommandRetrievalMode::NoEmbeddings),
        missing_artifact_options(),
    )
    .unwrap();
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    let error = service
        .search_page(
            SearchResultsRequest {
                sort: None,

                ranking: None,
                search_profile: None,
                criteria: SearchCriteria::ListRecords { filter: None },
                page: SearchPageRequest {
                    number: 1,
                    size: 10,
                },
                expected_artifact_generation: None,
            },
            &execution,
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::OperationCancelled);
    assert!(!service.workspace_path().exists());
}

#[test]
fn abandoned_submission_never_enters_callback_or_opens_reader() {
    let service = AtlasAppService::new(
        RetrievalBackend::command(CommandRetrievalMode::NoEmbeddings),
        missing_artifact_options(),
    )
    .unwrap();
    let called = Arc::new(AtomicBool::new(false));
    let callback_called = Arc::clone(&called);
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    let error = service
        .with_retrieval(&execution, move |_| {
            callback_called.store(true, Ordering::Release);
            Ok(())
        })
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::OperationCancelled);
    assert!(!called.load(Ordering::Acquire));
}

#[test]
fn abandoned_discovery_checks_before_saved_list_scope_reads_or_lazy_open() {
    let service = AtlasAppService::new(
        RetrievalBackend::command(CommandRetrievalMode::NoEmbeddings),
        missing_artifact_options(),
    )
    .unwrap();
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    let context = FilterDiscoveryContext::SavedList {
        search_content: None,
        search_profile: None,
        expected_artifact_generation: None,
        list_ref: "absent-list".into(),
        filter: None,
        text: None,
        mode: RetrievalModeView::Lexical,
    };
    let counts = service
        .discover_filter_counts(
            DiscoverFilterCountsRequest {
                context: context.clone(),
                field_id: "record.traits".into(),
                clause_id: None,
                values: vec![],
            },
            &execution,
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(counts.code, AppErrorCode::OperationCancelled);
    for request in [
        DiscoverFilterValuesRequest::Contextual {
            only_matching: false,
            context: Box::new(context),
            field_id: "record.traits".into(),
            clause_id: None,
            text: None,
            offset: 0,
            limit: 10,
        },
        DiscoverFilterValuesRequest::Vocabulary {
            scope: FilterVocabularyScope::SavedList {
                list_ref: "absent-list".into(),
            },
            field_id: "record.traits".into(),
            text: None,
            offset: 0,
            limit: 10,
            selected_values: vec![],
        },
    ] {
        let values = service
            .discover_filter_values(request, &execution)
            .unwrap_err()
            .into_app_error();
        assert_eq!(values.code, AppErrorCode::OperationCancelled);
    }
    assert!(
        !service.workspace_path().exists(),
        "abandoned read must not initialize a workspace"
    );
}
