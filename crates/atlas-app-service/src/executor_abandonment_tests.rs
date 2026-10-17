use std::sync::Condvar;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::*;
use atlas_search::{ListRecordsRequest, SearchPage};

#[derive(Default)]
struct ReleaseGate {
    released: Mutex<bool>,
    ready: Condvar,
}

impl ReleaseGate {
    fn wait(&self) -> AppServiceResult<()> {
        let held = self.released.lock().unwrap();
        let (held, timeout) = self
            .ready
            .wait_timeout_while(held, Duration::from_secs(5), |released| !*released)
            .unwrap();
        if timeout.timed_out() && !*held {
            return Err(AppServiceError::new(
                AppErrorCode::OperationTimeout,
                "test gate timed out",
            ));
        }
        Ok(())
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.ready.notify_all();
    }
}

#[test]
fn admitted_abandoned_job_is_skipped_and_later_live_fixture_read_completes() {
    let executor = RetrievalExecutor::from_fixture_workers(2, 16);
    let gate = Arc::new(ReleaseGate::default());
    let (entered, entry) = mpsc::channel();
    let held_jobs = (0..2)
        .map(|_| {
            let gate = Arc::clone(&gate);
            let entered = entered.clone();
            executor
                .enqueue(ReadExecutionContext::uncancellable(), move |_| {
                    let _ = entered.send(());
                    gate.wait()
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    let entered_one = entry.recv_timeout(Duration::from_secs(2));
    let entered_two = entry.recv_timeout(Duration::from_secs(2));
    let calls = Arc::new(AtomicUsize::new(0));
    let cancelled_calls = Arc::clone(&calls);
    let execution = ReadExecutionContext::tracked();
    // enqueue returns only after actual bounded admission, before abandonment.
    let abandoned = executor.enqueue(execution.clone(), move |_| {
        cancelled_calls.fetch_add(1, Ordering::AcqRel);
        Ok(())
    });
    execution.abandon();
    let live = executor.enqueue(ReadExecutionContext::tracked(), |retrieval| {
        Ok(retrieval
            .list_records(ListRecordsRequest::new(None, SearchPage::new(1, 10)?))?
            .records
            .len())
    });
    gate.release();
    for held in held_jobs {
        held.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    }
    entered_one.unwrap();
    entered_two.unwrap();
    assert_eq!(
        abandoned
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::OperationCancelled
    );
    assert_eq!(calls.load(Ordering::Acquire), 0);
    assert_eq!(
        live.unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap(),
        4
    );
}

#[test]
fn preadmission_abandonment_does_not_consume_even_a_full_queue() {
    let (sender, _receiver) = mpsc::sync_channel(0);
    let executor = RetrievalExecutor {
        sender,
        worker_count: 1,
        submission_budget: Arc::new(Mutex::new(None)),
        queue_capacity: 0,
    };
    let execution = ReadExecutionContext::tracked();
    execution.abandon();
    assert_eq!(
        executor
            .enqueue(execution, |_| Ok(()))
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::OperationCancelled
    );
}

#[test]
fn completed_fixture_retrieval_can_skip_projection_at_safe_checkpoint() {
    let executor = RetrievalExecutor::from_fixture_workers(1, 16);
    let execution = ReadExecutionContext::tracked();
    let callback_execution = execution.clone();
    let gate = Arc::new(ReleaseGate::default());
    let callback_gate = Arc::clone(&gate);
    let projections = Arc::new(AtomicUsize::new(0));
    let callback_projections = Arc::clone(&projections);
    let (completed, completion) = mpsc::channel();
    let result = executor
        .enqueue(execution.clone(), move |retrieval| {
            let records =
                retrieval.list_records(ListRecordsRequest::new(None, SearchPage::new(1, 10)?))?;
            let _ = completed.send(records.records.len());
            callback_gate.wait()?;
            callback_execution.check()?;
            callback_projections.fetch_add(1, Ordering::AcqRel);
            Ok(records.records.len())
        })
        .unwrap();
    let read = completion.recv_timeout(Duration::from_secs(2));
    execution.abandon();
    gate.release();
    let result = result.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(read.unwrap(), 4);
    assert_eq!(
        result.unwrap_err().into_app_error().code,
        AppErrorCode::OperationCancelled
    );
    assert_eq!(projections.load(Ordering::Acquire), 0);
}
