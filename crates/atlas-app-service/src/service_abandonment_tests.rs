use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
        std::env::temp_dir().join("unused-abandonment-state.sqlite"),
    )
    .unwrap();
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    let error = service
        .search_page(
            SearchResultsRequest {
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
    let RetrievalBackend::Command { retrieval, .. } = &service.retrieval else {
        panic!("expected command")
    };
    assert!(retrieval.lock().unwrap().is_none());
}

#[test]
fn abandoned_submission_never_enters_callback_or_opens_reader() {
    let service = AtlasAppService::new(
        RetrievalBackend::command(CommandRetrievalMode::NoEmbeddings),
        missing_artifact_options(),
        std::env::temp_dir().join("unused-abandonment-state.sqlite"),
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
        missing_artifact_options()
            .overrides
            .index_path
            .unwrap()
            .with_file_name(format!(
                "unused-abandoned-scope-{}.sqlite",
                std::process::id()
            )),
    )
    .unwrap();
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    let context = FilterDiscoveryContext::SavedList {
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
            },
            &execution,
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(counts.code, AppErrorCode::OperationCancelled);
    for request in [
        DiscoverFilterValuesRequest::Contextual {
            context,
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
        !service.local_state_path.exists(),
        "abandoned scope should not initialize local state"
    );
    let RetrievalBackend::Command { retrieval, .. } = &service.retrieval else {
        panic!("expected command")
    };
    assert!(retrieval.lock().unwrap().is_none());
}

#[test]
fn command_branch_checks_abandonment_after_retained_reader_lock() {
    let retained = Arc::new(Mutex::new(None));
    let held = retained.lock().unwrap();
    let execution = ReadExecutionContext::tracked();
    let waiting_execution = execution.clone();
    let waiting_retained = Arc::clone(&retained);
    let called = Arc::new(AtomicBool::new(false));
    let callback_called = Arc::clone(&called);
    let (entered, entry) = mpsc::channel();
    let worker = thread::spawn(move || {
        waiting_execution.check().unwrap();
        entered.send(()).unwrap();
        // This is the actual private command branch, whose first checkpoint is
        // after taking the lock. No timing assumption about lock admission.
        with_command_reader(
            CommandRetrievalMode::NoEmbeddings,
            &waiting_retained,
            &AtlasRuntime::resolve(missing_artifact_options()).unwrap(),
            &waiting_execution,
            |_| {
                callback_called.store(true, Ordering::Release);
                Ok(())
            },
        )
    });
    let entered = entry.recv_timeout(Duration::from_secs(2));
    execution.abandon();
    drop(held);
    let result = worker.join().unwrap();
    entered.unwrap();
    assert_eq!(
        result.unwrap_err().into_app_error().code,
        AppErrorCode::OperationCancelled
    );
    assert!(retained.lock().unwrap().is_none());
    assert!(!called.load(Ordering::Acquire));
}
