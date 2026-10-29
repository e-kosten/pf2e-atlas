use std::future::Future;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

use atlas_app_model::{
    AppErrorCode, InitiativeParticipantRequest, RollInitiativeRequest, RollMode,
};
use atlas_app_service::{AppServiceError, ReadExecutionContext};
use axum::{Json, Router};
use serde_json::{Value, json};

use super::fixture::Fixture;
use crate::service::call_read_service;
use crate::service::{AtlasWebState, call_service};

const DEADLINE: Duration = Duration::from_secs(5);

struct ReadProbe {
    first: AtomicBool,
    entered: mpsc::Sender<ReadExecutionContext>,
    release: Mutex<mpsc::Receiver<()>>,
    completed: mpsc::Sender<bool>,
}

impl ReadProbe {
    fn observe(&self, execution: &ReadExecutionContext) -> Result<(), AppServiceError> {
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
    let fixture = Fixture::new();
    let state = fixture.state(1);
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
        let fixture = Fixture::new();
        let state = fixture.state(1);
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
    let fixture = Fixture::new();
    let state = fixture.state(1);
    let initial = fixture.service.encounter("ambush").unwrap();
    let participant = initial
        .participants
        .iter()
        .find(|participant| participant.participant_key == fixture.participant_key)
        .unwrap();
    let request = RollInitiativeRequest {
        artifact_generation: None,
        request_id: "abandoned-initiative".into(),
        expected_revision: initial.revision,
        participants: vec![InitiativeParticipantRequest {
            participant_key: participant.participant_key.clone(),
            expected_revision: participant.revision,
            statistic: "manual".into(),
            modifier_override: Some(7),
            mode: RollMode::Normal,
        }],
    };
    let service = fixture.service.clone();
    let submitted = request.clone();
    let (entered, entry) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let (receipt, receipts) = mpsc::channel();
    let task = tokio::spawn(call_service(state.clone(), move || {
        let _ = entered.send(());
        released.recv_timeout(DEADLINE).map_err(|_| {
            AppServiceError::new(AppErrorCode::OperationTimeout, "mutation release timed out")
        })?;
        // This is an actual accepted storage mutation, not a synthetic write counter.
        let result = service.roll_encounter_initiative("ambush", submitted)?;
        let _ = receipt.send(result.clone());
        Ok(result)
    }));
    let entered = entry.recv_timeout(DEADLINE);
    task.abort();
    let dropped = task.await;
    let busy = call_service(state.clone(), || Ok::<_, AppServiceError>(())).await;
    let _ = release.send(());
    let receipt = receipts.recv_timeout(DEADLINE);
    await_live_slot(state.clone()).await;
    entered.unwrap();
    assert!(dropped.unwrap_err().is_cancelled());
    assert_eq!(busy.unwrap_err().0.code, AppErrorCode::ServiceBusy);
    let receipt = receipt.unwrap();
    assert_eq!(receipt.encounter_key, fixture.encounter_key);
    let committed = fixture.service.encounter(&receipt.encounter_key).unwrap();
    assert_eq!(committed.revision, initial.revision + 1);
    let participant = committed
        .participants
        .iter()
        .find(|participant| participant.participant_key == fixture.participant_key)
        .unwrap();
    assert_eq!(
        participant.initiative,
        Some(i64::from(receipt.rolls[0].outcome.total))
    );
    assert_eq!(
        participant.revision,
        request.participants[0].expected_revision + 1
    );

    // Recover the persisted receipt with the now-stale original revisions. A retry
    // must preserve both the exact dice result and the one committed mutation.
    let service = fixture.service.clone();
    let retry = request.clone();
    let recovered = call_service(state.clone(), move || {
        service.roll_encounter_initiative("ambush", retry)
    })
    .await
    .unwrap();
    assert_eq!(recovered, receipt);
    let after_retry = fixture.service.encounter("ambush").unwrap();
    assert_eq!(after_retry.revision, committed.revision);
    assert_eq!(after_retry.participants, committed.participants);

    let service = fixture.service.clone();
    let mut conflicting = request;
    conflicting.participants[0].modifier_override = Some(8);
    let conflict = call_service(state, move || {
        service.roll_encounter_initiative("ambush", conflicting)
    })
    .await
    .unwrap_err();
    assert_eq!(conflict.0.code, AppErrorCode::InvalidRequest);
}

fn send_probe(address: SocketAddr, keep_alive: bool) -> TcpStream {
    let mut client = TcpStream::connect_timeout(&address, DEADLINE).unwrap();
    client.set_read_timeout(Some(DEADLINE)).unwrap();
    client.set_write_timeout(Some(DEADLINE)).unwrap();
    let body = "{}";
    write!(client, "POST /probe HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n{body}",
        body.len(), if keep_alive { "keep-alive" } else { "close" }).unwrap();
    client.flush().unwrap();
    client
}

fn probe_response(address: SocketAddr) -> String {
    let mut client = send_probe(address, false);
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
    let fixture = Fixture::new();
    let state = fixture.state(1);
    // The test-only route exercises the real HTTP handler future and production
    // execution adapters. The one-shot barrier belongs to this harness, not the
    // app-service API or any product endpoint.
    let route_state = state.clone();
    let app = Router::new().route(
        "/probe",
        axum::routing::post(move |_: Json<Value>| {
            let state = route_state.clone();
            let probe = Arc::clone(&probe);
            async move {
                call_read_service(state, move |execution| {
                    probe.observe(&execution)?;
                    Ok(Json(json!({"completed": true})))
                })
                .await
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });

    let client = thread::spawn(move || send_probe(address, true))
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
            tokio::task::yield_now().await;
        }
    }
    let busy = thread::spawn(move || probe_response(address)).join();
    // Release before evaluating outcomes, including a negative disconnect observation.
    let _ = release.send(());
    let completed = completion.recv_timeout(DEADLINE);
    await_live_slot(state).await;
    let fresh = thread::spawn(move || probe_response(address)).join();
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
