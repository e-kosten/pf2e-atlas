use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;

use atlas_app_model::AppErrorCode;
use atlas_runtime::AtlasRuntime;
#[cfg(test)]
use atlas_runtime::AtlasRuntimeOptions;
use atlas_runtime::WorkspaceRead;
#[cfg(test)]
use atlas_search::AtlasRetrievalService;
use atlas_search::PreparedQueryEmbedding;

use crate::ReadExecutionContext;
use crate::error::{AppServiceError, AppServiceResult};

const DEFAULT_RETRIEVAL_WORKERS: usize = 2;
const DEFAULT_RETRIEVAL_QUEUE_CAPACITY: usize = 64;

type RetrievalJob = Box<dyn FnOnce(&AtlasRuntime) + Send + 'static>;

#[derive(Clone)]
pub(super) struct RetrievalExecutor {
    sender: SyncSender<RetrievalJob>,
    #[cfg(test)]
    pub(super) last_read_metrics: Arc<Mutex<atlas_search::SourceReadMetrics>>,
    #[cfg(test)]
    pub(super) runtime: Option<Arc<AtlasRuntime>>,
    #[cfg(test)]
    worker_count: usize,
    #[cfg(test)]
    submission_budget: Arc<Mutex<Option<usize>>>,
    queue_capacity: usize,
}

impl RetrievalExecutor {
    pub(super) fn start(runtime: Arc<AtlasRuntime>) -> AppServiceResult<Self> {
        Self::start_workers(
            runtime,
            DEFAULT_RETRIEVAL_WORKERS,
            DEFAULT_RETRIEVAL_QUEUE_CAPACITY,
            |builder, task| builder.spawn(task),
        )
    }

    fn start_workers(
        runtime: Arc<AtlasRuntime>,
        worker_count: usize,
        queue_capacity: usize,
        mut spawn: impl FnMut(thread::Builder, WorkerTask) -> std::io::Result<thread::JoinHandle<()>>,
    ) -> AppServiceResult<Self> {
        let worker_count = worker_count.max(1);
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(worker_count);

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let runtime = Arc::clone(&runtime);
            let builder = thread::Builder::new().name(format!("atlas-app-retrieval-{index}"));
            match spawn(
                builder,
                Box::new(move || retrieval_worker(&runtime, receiver)),
            ) {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    return Err(close_started_workers(
                        sender,
                        workers,
                        AppServiceError::new(
                            AppErrorCode::InternalError,
                            format!("failed to start retrieval worker: {error}"),
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            sender,
            #[cfg(test)]
            last_read_metrics: Default::default(),
            #[cfg(test)]
            runtime: Some(runtime),
            #[cfg(test)]
            worker_count,
            #[cfg(test)]
            submission_budget: Arc::new(Mutex::new(None)),
            queue_capacity,
        })
    }

    #[cfg(test)]
    pub(super) fn submit<T>(
        &self,
        execution: ReadExecutionContext,
        task: impl FnOnce(&mut WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        self.submit_prepared(execution, None, task)
    }

    pub(super) fn submit_prepared<T>(
        &self,
        execution: ReadExecutionContext,
        prepared: Option<PreparedQueryEmbedding>,
        task: impl FnOnce(&mut WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        self.enqueue_prepared(execution, prepared, task)?
            .recv()
            .map_err(|error| {
                AppServiceError::new(
                    AppErrorCode::InternalError,
                    format!("retrieval worker dropped response: {error}"),
                )
            })?
    }

    #[cfg(test)]
    fn enqueue<T>(
        &self,
        execution: ReadExecutionContext,
        task: impl FnOnce(&mut WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<Receiver<AppServiceResult<T>>>
    where
        T: Send + 'static,
    {
        self.enqueue_prepared(execution, None, task)
    }

    fn enqueue_prepared<T>(
        &self,
        execution: ReadExecutionContext,
        prepared: Option<PreparedQueryEmbedding>,
        task: impl FnOnce(&mut WorkspaceRead) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<Receiver<AppServiceResult<T>>>
    where
        T: Send + 'static,
    {
        execution.check()?;
        #[cfg(test)]
        {
            let mut budget = self
                .submission_budget
                .lock()
                .expect("test submission budget");
            if let Some(remaining) = budget.as_mut() {
                if *remaining == 0 {
                    return Err(AppServiceError::new(
                        AppErrorCode::QueryFailed,
                        "injected retrieval failure",
                    ));
                }
                *remaining -= 1;
            }
        }
        let (reply, receiver) = mpsc::channel();
        #[cfg(test)]
        let metrics = Arc::clone(&self.last_read_metrics);
        let job = Box::new(move |runtime: &AtlasRuntime| {
            let result = execution.check().and_then(|()| {
                let mut workspace = runtime.read_workspace(prepared)?;
                execution.check()?;
                let result = task(&mut workspace);
                #[cfg(test)]
                {
                    *metrics.lock().expect("operation read metrics") =
                        workspace.retrieval().read_metrics();
                }
                result
            });
            let _ = reply.send(result);
        });

        self.sender.try_send(job).map_err(|error| match error {
            TrySendError::Full(_) => AppServiceError::service_busy(format!(
                "app-service retrieval queue is full; capacity is {}",
                self.queue_capacity
            )),
            TrySendError::Disconnected(_) => AppServiceError::new(
                AppErrorCode::InternalError,
                "app-service retrieval workers are unavailable",
            ),
        })?;

        Ok(receiver)
    }

    #[cfg(test)]
    pub(super) fn worker_count(&self) -> usize {
        self.worker_count
    }

    #[cfg(test)]
    pub(super) fn fail_after_submissions(&self, count: usize) {
        *self
            .submission_budget
            .lock()
            .expect("test submission budget") = Some(count);
    }

    #[cfg(test)]
    pub(super) fn remaining_submission_budget(&self) -> Option<usize> {
        *self
            .submission_budget
            .lock()
            .expect("test submission budget")
    }

    #[cfg(test)]
    pub(super) fn from_fixture_workers(worker_count: usize, queue_capacity: usize) -> Self {
        use crate::test_support::fixture_retrieval;

        Self::from_test_fixture_factory(worker_count, queue_capacity, fixture_retrieval)
    }

    #[cfg(test)]
    pub(super) fn from_test_fixture_factory(
        worker_count: usize,
        queue_capacity: usize,
        open_fixture: impl FnOnce() -> Result<
            (
                AtlasRetrievalService,
                atlas_search::test_support::FixtureArtifact,
            ),
            Box<dyn std::error::Error>,
        >,
    ) -> Self {
        let (retrieval, artifact) = open_fixture().expect("fixture artifact should be built once");
        drop(retrieval);
        atlas_runtime::initialize_workspace(artifact.path())
            .expect("fixture workspace should initialize through runtime migrations");
        let options = AtlasRuntimeOptions {
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(artifact.path()),
                ..Default::default()
            },
            ..Default::default()
        };
        let runtime =
            Arc::new(AtlasRuntime::resolve(options).expect("fixture runtime should resolve"));
        let artifact = Arc::new(artifact);
        Self::start_workers(
            runtime,
            worker_count,
            queue_capacity,
            move |builder, task| {
                let artifact = Arc::clone(&artifact);
                builder.spawn(move || {
                    // A worker can outlive its executor while accepted jobs finish.
                    let _artifact = artifact;
                    task();
                })
            },
        )
        .expect("fixture workers should share the retained runtime and artifact")
    }
}

type WorkerTask = Box<dyn FnOnce() + Send + 'static>;

fn close_started_workers(
    sender: SyncSender<RetrievalJob>,
    workers: Vec<thread::JoinHandle<()>>,
    error: AppServiceError,
) -> AppServiceError {
    drop(sender);
    for worker in workers {
        // Wait for resources held by every started worker before returning
        // the original thread-spawn error.
        let _ = worker.join();
    }
    error
}

fn retrieval_worker(runtime: &AtlasRuntime, receiver: Arc<Mutex<Receiver<RetrievalJob>>>) {
    loop {
        let job = {
            let Ok(guard) = receiver.lock() else {
                break;
            };
            match guard.recv() {
                Ok(job) => job,
                Err(_) => break,
            }
        };
        job(runtime);
    }
}

#[cfg(test)]
#[path = "executor_lifetime_tests.rs"]
mod lifetime_tests;

#[cfg(test)]
#[path = "executor_abandonment_tests.rs"]
mod abandonment_tests;

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn default_worker_count_uses_parallel_retrieval_lanes() {
        let runtime = Arc::new(AtlasRuntime::resolve(AtlasRuntimeOptions::default()).unwrap());
        let executor = RetrievalExecutor::start(runtime).unwrap();

        assert!(executor.worker_count() > 1);
    }

    #[test]
    fn submit_returns_busy_when_queue_is_full() {
        let (sender, _receiver) = mpsc::sync_channel(0);
        let executor = RetrievalExecutor {
            sender,
            runtime: None,
            last_read_metrics: Default::default(),
            submission_budget: Arc::new(Mutex::new(None)),
            worker_count: 1,
            queue_capacity: 0,
        };

        let error = executor
            .submit(crate::ReadExecutionContext::uncancellable(), |_| Ok(()))
            .expect_err("zero-capacity queue without waiting worker should be busy")
            .into_app_error();

        assert_eq!(error.code, AppErrorCode::ServiceBusy);
        assert_eq!(error.retryable, Some(true));
        assert!(error.message.contains("retrieval queue is full"));
    }

    #[test]
    fn custom_worker_count_is_bounded_to_at_least_one() {
        let executor = RetrievalExecutor::from_fixture_workers(0, 16);

        assert_eq!(executor.worker_count(), 1);
    }

    #[test]
    fn workers_execute_jobs_concurrently() {
        let executor = RetrievalExecutor::from_fixture_workers(2, 16);
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new((Mutex::new(false), Condvar::new()));

        let handles = (0..2)
            .map(|_| {
                let executor = executor.clone();
                let entered_sender = entered_sender.clone();
                let release = Arc::clone(&release);
                thread::spawn(move || {
                    executor.submit(crate::ReadExecutionContext::uncancellable(), move |_| {
                        let _ = entered_sender.send(());
                        let (lock, condvar) = &*release;
                        let mut released = lock.lock().map_err(|_| {
                            AppServiceError::new(
                                AppErrorCode::InternalError,
                                "test release lock was poisoned",
                            )
                        })?;
                        while !*released {
                            let (next_released, timeout) = condvar
                                .wait_timeout(released, Duration::from_secs(5))
                                .map_err(|_| {
                                    AppServiceError::new(
                                        AppErrorCode::InternalError,
                                        "test release condition was poisoned",
                                    )
                                })?;
                            released = next_released;
                            if timeout.timed_out() {
                                return Err(AppServiceError::new(
                                    AppErrorCode::OperationTimeout,
                                    "test release timed out",
                                ));
                            }
                        }
                        Ok(())
                    })
                })
            })
            .collect::<Vec<_>>();

        let first_entered = entered_receiver.recv_timeout(Duration::from_secs(2));
        let second_entered = entered_receiver.recv_timeout(Duration::from_secs(2));

        {
            let (lock, condvar) = &*release;
            let mut released = lock
                .lock()
                .expect("test release lock should remain available");
            *released = true;
            condvar.notify_all();
        }

        for handle in handles {
            handle
                .join()
                .expect("submit thread should not panic")
                .expect("submit should complete");
        }

        assert!(first_entered.is_ok());
        assert!(second_entered.is_ok());
    }
}
