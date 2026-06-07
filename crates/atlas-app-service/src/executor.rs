use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;

use atlas_app_model::AppErrorCode;
use atlas_runtime::{AtlasRuntime, AtlasRuntimeOptions};
use atlas_search::AtlasRetrievalService;

use crate::error::{AppServiceError, AppServiceResult};

const DEFAULT_RETRIEVAL_WORKERS: usize = 2;
const DEFAULT_RETRIEVAL_QUEUE_CAPACITY: usize = 64;

type RetrievalJob = Box<dyn FnOnce(&mut AtlasRetrievalService) + Send + 'static>;

#[derive(Clone)]
pub(super) struct RetrievalExecutor {
    sender: SyncSender<RetrievalJob>,
    #[cfg(test)]
    worker_count: usize,
    queue_capacity: usize,
}

impl RetrievalExecutor {
    pub(super) fn start(options: AtlasRuntimeOptions) -> AppServiceResult<Self> {
        Self::start_with_config(
            options,
            DEFAULT_RETRIEVAL_WORKERS,
            DEFAULT_RETRIEVAL_QUEUE_CAPACITY,
        )
    }

    fn start_with_config(
        options: AtlasRuntimeOptions,
        worker_count: usize,
        queue_capacity: usize,
    ) -> AppServiceResult<Self> {
        let worker_count = worker_count.max(1);
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let receiver = Arc::new(Mutex::new(receiver));

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let options = options.clone();
            spawn_retrieval_worker(
                format!("atlas-app-retrieval-{index}"),
                receiver,
                move || open_retrieval_service(options),
            )?;
        }

        Ok(Self {
            sender,
            #[cfg(test)]
            worker_count,
            queue_capacity,
        })
    }

    pub(super) fn submit<T>(
        &self,
        task: impl FnOnce(&mut AtlasRetrievalService) -> AppServiceResult<T> + Send + 'static,
    ) -> AppServiceResult<T>
    where
        T: Send + 'static,
    {
        let (reply, receiver) = mpsc::channel();
        let job = Box::new(move |retrieval: &mut AtlasRetrievalService| {
            let _ = reply.send(task(retrieval));
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

        receiver.recv().map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("retrieval worker dropped response: {error}"),
            )
        })?
    }

    #[cfg(test)]
    pub(super) fn worker_count(&self) -> usize {
        self.worker_count
    }

    #[cfg(test)]
    pub(super) fn from_fixture_workers(worker_count: usize, queue_capacity: usize) -> Self {
        use atlas_search::test_support::minimal_fixture_retrieval_service_without_embeddings;

        let worker_count = worker_count.max(1);
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let receiver = Arc::new(Mutex::new(receiver));

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let (startup_sender, startup_receiver) = mpsc::channel();
            thread::Builder::new()
                .name(format!("atlas-app-test-retrieval-{index}"))
                .spawn(move || {
                    let (mut retrieval, _artifact) =
                        minimal_fixture_retrieval_service_without_embeddings()
                            .expect("fixture retrieval service should build");
                    let _ = startup_sender.send(());
                    retrieval_worker(&mut retrieval, receiver);
                })
                .expect("test retrieval worker should start");
            startup_receiver
                .recv()
                .expect("test retrieval worker should report startup");
        }

        Self {
            sender,
            worker_count,
            queue_capacity,
        }
    }
}

fn spawn_retrieval_worker(
    name: String,
    receiver: Arc<Mutex<Receiver<RetrievalJob>>>,
    open: impl FnOnce() -> AppServiceResult<AtlasRetrievalService> + Send + 'static,
) -> AppServiceResult<()> {
    let (startup_sender, startup_receiver) = mpsc::channel();
    thread::Builder::new()
        .name(name)
        .spawn(move || match open() {
            Ok(mut retrieval) => {
                let _ = startup_sender.send(Ok(()));
                retrieval_worker(&mut retrieval, receiver);
            }
            Err(error) => {
                let _ = startup_sender.send(Err(error));
            }
        })
        .map_err(|error| {
            AppServiceError::new(
                AppErrorCode::InternalError,
                format!("failed to start retrieval worker: {error}"),
            )
        })?;

    startup_receiver.recv().map_err(|error| {
        AppServiceError::new(
            AppErrorCode::InternalError,
            format!("retrieval worker failed to report startup: {error}"),
        )
    })?
}

fn open_retrieval_service(options: AtlasRuntimeOptions) -> AppServiceResult<AtlasRetrievalService> {
    let runtime = AtlasRuntime::resolve(options)?;
    Ok(runtime.open_retrieval_service()?)
}

fn retrieval_worker(
    retrieval: &mut AtlasRetrievalService,
    receiver: Arc<Mutex<Receiver<RetrievalJob>>>,
) {
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
        job(retrieval);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn default_worker_count_uses_parallel_retrieval_lanes() {
        let (sender, _receiver) = mpsc::sync_channel(1);
        let executor = RetrievalExecutor {
            sender,
            worker_count: DEFAULT_RETRIEVAL_WORKERS,
            queue_capacity: DEFAULT_RETRIEVAL_QUEUE_CAPACITY,
        };

        assert!(executor.worker_count() > 1);
    }

    #[test]
    fn submit_returns_busy_when_queue_is_full() {
        let (sender, _receiver) = mpsc::sync_channel(0);
        let executor = RetrievalExecutor {
            sender,
            worker_count: 1,
            queue_capacity: 0,
        };

        let error = executor
            .submit(|_| Ok(()))
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
                    executor.submit(move |_| {
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
