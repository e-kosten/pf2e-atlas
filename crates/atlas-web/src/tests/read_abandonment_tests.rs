use std::future::Future;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use crate::service::call_read_service;

const DEADLINE: Duration = Duration::from_secs(5);

pub(super) struct ReadProbe {
    first: AtomicBool,
    entered: mpsc::Sender<ReadExecutionContext>,
    release: Mutex<mpsc::Receiver<()>>,
    completed: mpsc::Sender<bool>,
}

impl ReadProbe {
    pub(super) fn observe(&self, execution: &ReadExecutionContext) -> Result<(), AppServiceError> {
        if !self.first.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let _ = self.entered.send(execution.clone());
        let released = self.release.lock().unwrap().recv_timeout(DEADLINE);
        let result = if released.is_ok() {
            execution.check()
        } else {
            Err(AppServiceError::new(
                AppErrorCode::OperationTimeout,
                "probe release timed out",
            ))
        };
        let _ = self.completed.send(result.is_ok());
        result
    }
}

async fn await_live_slot(state: AtlasWebState) {
    let deadline = Instant::now() + DEADLINE;
    loop {
        match call_service(state.clone(), || Ok::<_, AppServiceError>(())).await {
            Ok(()) => return,
            Err(error)
                if error.0.code == AppErrorCode::ServiceBusy && Instant::now() < deadline =>
            {
                tokio::task::yield_now().await;
            }
            result => panic!("blocking slot did not recover: {result:?}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropped_read_future_abandons_context_but_running_work_retains_permit() {
    let state = AtlasWebState::from_service_with_blocking_capacity(MockService::default(), 1);
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let (finished, finish) = mpsc::channel();
    let task = tokio::spawn(call_read_service(state.clone(), move |execution| {
        let _ = entered.send(execution.clone());
        let released = released.recv_timeout(DEADLINE);
        let result = released
            .map_err(|_| {
                AppServiceError::new(AppErrorCode::OperationTimeout, "read release timed out")
            })
            .and_then(|()| execution.check());
        let _ = finished.send(result.is_ok());
        result
    }));
    let entered = entry.recv_timeout(DEADLINE);
    task.abort();
    let dropped = task.await;
    let busy = call_service(state.clone(), || Ok::<_, AppServiceError>(())).await;
    let _ = release.send(());
    let finished = finish.recv_timeout(DEADLINE);
    await_live_slot(state).await;
    let execution = entered.unwrap();
    assert!(dropped.unwrap_err().is_cancelled());
    assert_eq!(
        execution.check().unwrap_err().into_app_error().code,
        AppErrorCode::OperationCancelled
    );
    assert_eq!(busy.unwrap_err().0.code, AppErrorCode::ServiceBusy);
    assert!(!finished.unwrap(), "post-read projection should be skipped");
}

#[test]
fn dropped_read_before_blocking_callback_starts_skips_app_callback() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let state = AtlasWebState::from_service_with_blocking_capacity(MockService::default(), 1);
        let (entered, entry) = mpsc::channel();
        let (release, released) = mpsc::channel();
        let occupying = tokio::task::spawn_blocking(move || {
            let _ = entered.send(());
            let _ = released.recv_timeout(DEADLINE);
        });
        let entered = entry.recv_timeout(DEADLINE);
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_calls = Arc::clone(&calls);
        let mut awaiting = Box::pin(call_read_service(state.clone(), move |_| {
            callback_calls.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }));
        // One direct poll acquires the permit and queues spawn_blocking while
        // the only blocking thread is held. Dropping this exact future is deterministic.
        let polled = awaiting
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()));
        drop(awaiting);
        let busy = call_service(state.clone(), || Ok::<_, AppServiceError>(())).await;
        let _ = release.send(());
        occupying.await.unwrap();
        await_live_slot(state).await;
        entered.unwrap();
        assert!(matches!(polled, Poll::Pending));
        assert_eq!(busy.unwrap_err().0.code, AppErrorCode::ServiceBusy);
        assert_eq!(calls.load(Ordering::Acquire), 0);
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropped_mutation_future_keeps_one_write_and_canonical_receipt() {
    let state = AtlasWebState::from_service_with_blocking_capacity(MockService::default(), 1);
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let (receipt, receipts) = mpsc::channel();
    let writes = Arc::new(AtomicUsize::new(0));
    let callback_writes = Arc::clone(&writes);
    let task = tokio::spawn(call_service(state.clone(), move || {
        let _ = entered.send(());
        released.recv_timeout(DEADLINE).map_err(|_| {
            AppServiceError::new(AppErrorCode::OperationTimeout, "mutation release timed out")
        })?;
        callback_writes.fetch_add(1, Ordering::AcqRel);
        let result = EncounterMutationView {
            encounter_key: "canonical-encounter".into(),
        };
        let _ = receipt.send(result.clone());
        Ok(result)
    }));
    let entered = entry.recv_timeout(DEADLINE);
    task.abort();
    let dropped = task.await;
    let busy = call_service(state.clone(), || Ok::<_, AppServiceError>(())).await;
    let _ = release.send(());
    let receipt = receipts.recv_timeout(DEADLINE);
    await_live_slot(state).await;
    entered.unwrap();
    assert!(dropped.unwrap_err().is_cancelled());
    assert_eq!(busy.unwrap_err().0.code, AppErrorCode::ServiceBusy);
    assert_eq!(writes.load(Ordering::Acquire), 1);
    assert_eq!(receipt.unwrap().encounter_key, "canonical-encounter");
}

fn send_page(address: SocketAddr, keep_alive: bool) -> TcpStream {
    let mut client = TcpStream::connect_timeout(&address, DEADLINE).unwrap();
    client.set_read_timeout(Some(DEADLINE)).unwrap();
    client.set_write_timeout(Some(DEADLINE)).unwrap();
    let body = serde_json::to_string(&json!({
        "criteria": {"kind":"list_records", "filter":null},
        "page": {"number":1, "size":10},
        "expected_artifact_generation":null
    }))
    .unwrap();
    write!(client, "POST /api/search/page HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n{body}",
        body.len(), if keep_alive { "keep-alive" } else { "close" }).unwrap();
    client.flush().unwrap();
    client
}

fn page_response(address: SocketAddr) -> String {
    let mut client = send_page(address, false);
    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    response
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complete_http1_disconnect_abandons_handler_before_running_read_is_released() {
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let (completed, completion) = mpsc::channel();
    let probe = Arc::new(ReadProbe {
        first: AtomicBool::new(true),
        entered,
        release: Mutex::new(released),
        completed,
    });
    let state = AtlasWebState::from_service_with_blocking_capacity(
        MockService {
            read_probe: Some(probe),
        },
        1,
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, router_with_state(state))
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });

    let client = thread::spawn(move || send_page(address, true))
        .join()
        .unwrap();
    let entered = entry.recv_timeout(DEADLINE);
    let shutdown = client.shutdown(Shutdown::Both);
    drop(client);
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut observed_abandoned = false;
    if let Ok(execution) = &entered {
        loop {
            if execution.check().is_err() {
                observed_abandoned = true;
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    let busy = thread::spawn(move || page_response(address)).join();
    // Release before evaluating outcomes, including a negative disconnect observation.
    let _ = release.send(());
    let completed = completion.recv_timeout(DEADLINE);
    let fresh = thread::spawn(move || {
        let deadline = Instant::now() + DEADLINE;
        loop {
            let response = page_response(address);
            if !response.starts_with("HTTP/1.1 503") || Instant::now() >= deadline {
                return response;
            }
            thread::sleep(Duration::from_millis(5));
        }
    })
    .join();
    let _ = stop.send(());
    let join_deadline = Instant::now() + DEADLINE;
    while !server.is_finished() && Instant::now() < join_deadline {
        tokio::task::yield_now().await;
    }
    if !server.is_finished() {
        server.abort();
    }
    let server_finished = server.await;

    eprintln!(
        "HTTP/1.1 probe: complete request body sent; callback entered={}; full socket shutdown={}; abandonment observed before release={}; occupied permit returned 503={}; projection completed={:?}; fresh request returned 200={}; server joined={}",
        entered.is_ok(),
        shutdown.is_ok(),
        observed_abandoned,
        busy.as_ref()
            .is_ok_and(|response| response.starts_with("HTTP/1.1 503")
                && response.contains("service_busy")),
        completed,
        fresh
            .as_ref()
            .is_ok_and(|response| response.starts_with("HTTP/1.1 200")),
        server_finished.is_ok()
    );
    entered.unwrap();
    shutdown.unwrap();
    server_finished.unwrap().unwrap();
    assert!(
        observed_abandoned,
        "full HTTP/1 disconnect did not drop the pending read handler"
    );
    assert!(busy.unwrap().starts_with("HTTP/1.1 503"));
    assert!(
        !completed.unwrap(),
        "abandoned read should skip its next projection checkpoint"
    );
    assert!(fresh.unwrap().starts_with("HTTP/1.1 200"));
}
