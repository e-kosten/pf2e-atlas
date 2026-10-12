use super::*;
use atlas_search::test_support::{FixtureArtifact, input, open_input_fixture, record};
use serde_json::json;

fn artifact(name: &str, locale: &str) -> (AtlasRetrievalService, FixtureArtifact) {
    let mut input = input(
        vec![record(
            "actions",
            "Item",
            json!({"_id":"testAction100000","type":"action","name":name}),
        )],
        false,
    );
    input.context.indexing_locale = locale.into();
    let hash = input.context.sha256().unwrap();
    for record in &mut input.records {
        record.preparation_context_hash = hash.clone();
    }
    open_input_fixture(input).unwrap()
}

fn open(artifact: &FixtureArtifact) -> AppServiceResult<AtlasRetrievalService> {
    open_retrieval_service_no_embeddings(AtlasRuntimeOptions {
        overrides: atlas_runtime::AtlasPathOverrides {
            index_path: Some(artifact.path()),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[cfg(unix)]
fn publish(path: &std::path::Path, name: &str, locale: &str) {
    // Both artifacts are produced by the real validated writer; rename is the
    // same atomic replacement used for publication, without an index dependency.
    let (reader, replacement) = artifact(name, locale);
    drop(reader);
    std::fs::rename(replacement.path(), path).unwrap();
}

#[cfg(unix)]
#[test]
fn pool_startup_rejects_same_context_republication_and_joins_every_worker() {
    let (reader, artifact) = artifact("Original", "en");
    let generation = reader.artifact_generation().unwrap();
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let result = RetrievalExecutor::start_with_factory(2, 16, move |index| {
        if index == 1 {
            publish(&retained.path(), "Replacement", "en");
        }
        let reader = open(&retained)?;
        assert_eq!(reader.artifact_generation().unwrap(), generation);
        Ok(reader)
    });
    let error = result
        .err()
        .expect("mixed physical instances must fail")
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::IndexUnavailable);
    assert_eq!(
        Arc::strong_count(&artifact),
        1,
        "all worker factories must be dropped before failure returns"
    );
}

#[cfg(unix)]
#[test]
fn pool_startup_rejects_different_context_republication() {
    let (reader, artifact) = artifact("Original", "en");
    let generation = reader.artifact_generation().unwrap();
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let result = RetrievalExecutor::start_with_factory(2, 16, move |index| {
        if index == 1 {
            publish(&retained.path(), "Replacement", "fr");
        }
        let reader = open(&retained)?;
        if index == 1 {
            assert_ne!(reader.artifact_generation().unwrap(), generation);
        }
        Ok(reader)
    });
    assert_eq!(
        result.err().unwrap().into_app_error().code,
        AppErrorCode::IndexUnavailable
    );
    assert_eq!(Arc::strong_count(&artifact), 1);
}

#[test]
fn later_factory_failure_closes_and_joins_started_workers() {
    let (reader, artifact) = artifact("Original", "en");
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let result = RetrievalExecutor::start_with_factory(3, 16, move |index| {
        if index == 2 {
            return Err(AppServiceError::new(
                AppErrorCode::QueryFailed,
                "factory failure",
            ));
        }
        open(&retained)
    });
    assert_eq!(
        result.err().unwrap().into_app_error().code,
        AppErrorCode::QueryFailed
    );
    assert_eq!(Arc::strong_count(&artifact), 1);
}

#[test]
fn lost_startup_report_closes_and_joins_started_workers() {
    let (reader, artifact) = artifact("Original", "en");
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let result = RetrievalExecutor::start_with_factory(2, 16, move |index| {
        assert_ne!(index, 1, "injected factory panic before startup report");
        open(&retained)
    });
    let error = result.err().unwrap().into_app_error();
    assert_eq!(error.code, AppErrorCode::InternalError);
    assert!(error.message.contains("failed to report startup"));
    assert_eq!(Arc::strong_count(&artifact), 1);
}

#[cfg(unix)]
#[test]
fn successful_pool_keeps_parallel_lanes_on_original_artifact_after_publication() {
    let (reader, artifact) = artifact("Original", "en");
    let instance = reader.artifact_instance().clone();
    drop(reader);
    let artifact = Arc::new(artifact);
    let retained = Arc::clone(&artifact);
    let executor = RetrievalExecutor::start_with_factory(2, 16, move |_| open(&retained)).unwrap();
    publish(&artifact.path(), "Replacement", "fr");
    let (entered_sender, entered_receiver) = std::sync::mpsc::channel();
    let release = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let executor = executor.clone();
            let entered_sender = entered_sender.clone();
            let release = Arc::clone(&release);
            thread::spawn(move || {
                executor.submit(crate::ReadExecutionContext::uncancellable(), move |r| {
                    entered_sender.send(()).unwrap();
                    let (lock, condvar) = &*release;
                    let released = lock.lock().unwrap();
                    let (released, _) = condvar
                        .wait_timeout_while(
                            released,
                            std::time::Duration::from_secs(5),
                            |released| !*released,
                        )
                        .unwrap();
                    assert!(*released, "parallel lane test release timed out");
                    let key = atlas_domain::RecordKey::parse("actions:testAction100000").unwrap();
                    let records = r.get_records(atlas_search::GetRecordsRequest {
                        record_keys: &[key],
                    })?;
                    Ok((
                        r.artifact_instance().clone(),
                        records.records[0].name.as_value().unwrap().clone(),
                    ))
                })
            })
        })
        .collect();
    let first_entered = entered_receiver.recv_timeout(std::time::Duration::from_secs(2));
    let second_entered = entered_receiver.recv_timeout(std::time::Duration::from_secs(2));
    {
        let (lock, condvar) = &*release;
        *lock.lock().unwrap() = true;
        condvar.notify_all();
    }
    for handle in handles {
        let (actual, name) = handle.join().unwrap().unwrap();
        assert_eq!(actual, instance);
        assert_eq!(name, "Original");
    }
    assert!(first_entered.is_ok());
    assert!(
        second_entered.is_ok(),
        "both lanes must start before either completes"
    );
}
