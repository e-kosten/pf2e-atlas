use super::*;
use atlas_search::test_support::{input, open_input_fixture, publish_input, record};
use serde_json::json;

macro_rules! source { ($name:expr) => { input(vec![record("actions", "Item", json!({"_id":"testAction100000","type":"action","name":$name}))], false) }; }

#[test]
fn idle_workers_release_snapshots_and_next_job_sees_publication() {
    let (reader, artifact) = open_input_fixture(source!("Original")).unwrap();
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let executor = RetrievalExecutor::start_with_factory(2, 16, move |_| {
        Ok(Arc::new(AtlasRuntime::resolve(AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(retained.path()),
                ..Default::default()
            },
            ..Default::default()
        })?))
    })
    .unwrap();
    let read = |workspace: &mut WorkspaceRead| {
        let retrieval = workspace.retrieval();
        Ok((
            retrieval.artifact_generation()?,
            retrieval
                .list_records(atlas_search::ListRecordsRequest::new(
                    None,
                    atlas_search::SearchPage::default(),
                ))?
                .records[0]
                .name
                .as_value()
                .unwrap()
                .clone(),
        ))
    };
    let (before, name) = executor
        .submit(ReadExecutionContext::uncancellable(), read)
        .unwrap();
    assert_eq!(name, "Original");
    publish_input(artifact.path(), &source!("Replacement")).unwrap();
    let (after, name) = executor
        .submit(ReadExecutionContext::uncancellable(), read)
        .unwrap();
    assert_eq!(name, "Replacement");
    assert_ne!(before, after);
}

#[test]
fn later_factory_failure_joins_started_workers() {
    let calls = Arc::new(());
    let retained = Arc::clone(&calls);
    let result = RetrievalExecutor::start_with_factory(3, 16, move |index| {
        let _ = &retained;
        if index == 2 {
            return Err(AppServiceError::invalid_request("factory failure"));
        }
        Ok(Arc::new(AtlasRuntime::resolve(
            AtlasRuntimeOptions::default(),
        )?))
    });
    assert!(result.is_err());
    assert_eq!(Arc::strong_count(&calls), 1);
}

#[test]
fn lost_startup_report_joins_started_workers() {
    let calls = Arc::new(());
    let retained = Arc::clone(&calls);
    let result = RetrievalExecutor::start_with_factory(2, 16, move |index| {
        let _ = &retained;
        assert_ne!(index, 1, "injected startup panic");
        Ok(Arc::new(AtlasRuntime::resolve(
            AtlasRuntimeOptions::default(),
        )?))
    });
    assert_eq!(
        result.err().unwrap().into_app_error().code,
        AppErrorCode::InternalError
    );
    assert_eq!(Arc::strong_count(&calls), 1);
}
