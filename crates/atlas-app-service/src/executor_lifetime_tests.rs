use super::*;
use crate::test_support::publish_input;
use atlas_search::test_support::{input, open_input_fixture, record};
use serde_json::json;

macro_rules! source { ($name:expr) => { input(vec![record("actions", "Item", json!({"_id":"testAction100000","type":"action","name":$name}))], false) }; }

#[test]
fn idle_workers_release_snapshots_and_next_job_sees_publication() {
    let (reader, artifact) = open_input_fixture(source!("Original")).unwrap();
    drop(reader);
    atlas_runtime::initialize_workspace(artifact.path()).unwrap();
    let runtime = Arc::new(
        AtlasRuntime::resolve(AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(artifact.path()),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap(),
    );
    let executor = RetrievalExecutor::start(runtime).unwrap();
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
fn thread_spawn_failure_joins_started_workers_and_releases_runtime() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    for fail_at in [0, 2] {
        let runtime = Arc::new(AtlasRuntime::resolve(AtlasRuntimeOptions::default()).unwrap());
        let retained_runtime = Arc::downgrade(&runtime);
        let exited = Arc::new(AtomicUsize::new(0));
        let worker_exited = Arc::clone(&exited);
        let (finished, completion) = mpsc::channel();
        let starter = thread::spawn(move || {
            let mut index = 0;
            let result = RetrievalExecutor::start_workers(runtime, 3, 16, move |builder, task| {
                let current = index;
                index += 1;
                let exited = Arc::clone(&worker_exited);
                // Ask the real thread builder to reject an impossible stack size.
                // No runtime initialization failure or startup report is involved.
                let builder = if current == fail_at {
                    builder.stack_size(usize::MAX)
                } else {
                    builder
                };
                builder.spawn(move || {
                    task();
                    exited.fetch_add(1, Ordering::AcqRel);
                })
            });
            let _ = finished.send(result.map(|_| ()));
        });
        let error = completion
            .recv_timeout(Duration::from_secs(5))
            .expect("spawn failure must close the queue and join started workers")
            .unwrap_err()
            .into_app_error();
        starter.join().unwrap();
        assert_eq!(error.code, AppErrorCode::InternalError);
        assert!(error.message.contains("failed to start retrieval worker"));
        assert_eq!(exited.load(Ordering::Acquire), fail_at);
        assert!(retained_runtime.upgrade().is_none());
    }
}
