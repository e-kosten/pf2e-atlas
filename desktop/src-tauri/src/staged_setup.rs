//! Desktop-owned staged setup and the narrow product document attachment.
use std::{
    collections::HashMap,
    sync::{Mutex, atomic::Ordering, mpsc},
};

use atlas_desktop_protocol::{SetupPublication, SetupStage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{
    AppHandle, Emitter, Manager, WebviewWindow, ipc::CapabilityBuilder, webview::PageLoadEvent,
};
use url::Url;

use crate::{Desktop, display, lock, opening, process, refresh_product, run_on_main, verified_url};

#[derive(Default)]
pub struct StagedSetup {
    state: Mutex<SetupState>,
    documents: Mutex<HashMap<String, Document>>,
}

#[derive(Clone)]
struct Document {
    instance: String,
    origin: url::Origin,
    token: Option<String>,
}

#[derive(Clone, Default, Serialize)]
struct SetupState {
    job: usize,
    sequence: usize,
    status: Status,
    progress: Option<Value>,
    message: Option<String>,
    local_build: bool,
    can_resume: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    #[default]
    Idle,
    Available,
    Running,
    Cancelling,
    Failed,
}

impl StagedSetup {
    pub fn active(&self) -> bool {
        matches!(
            lock(&self.state).status,
            Status::Running | Status::Cancelling
        )
    }

    pub fn offer_resume(&self) {
        let mut state = lock(&self.state);
        if state.status == Status::Idle {
            state.status = Status::Available;
            state.can_resume = true;
            state.sequence += 1;
        }
    }

    pub fn attach_window(
        &self,
        app: &AppHandle,
        label: &str,
        url: &Url,
        instance: &str,
    ) -> Result<(), String> {
        let origin = url.origin();
        if lock(&self.documents)
            .get(label)
            .is_some_and(|doc| doc.instance == instance && doc.origin == origin)
        {
            return Ok(());
        }
        let permission = CapabilityBuilder::new(format!("atlas-setup-{label}-{instance}"))
            .local(false)
            .remote(format!("{}/*", origin.ascii_serialization()))
            .webview(label)
            .permission("allow-desktop-setup-action");
        app.add_capability(permission).map_err(display)?;
        lock(&self.documents).insert(
            label.into(),
            Document {
                instance: instance.into(),
                origin,
                token: None,
            },
        );
        Ok(())
    }

    pub fn clear_documents(&self) {
        lock(&self.documents).clear();
    }

    pub fn attached_to(&self, label: &str, instance: &str) -> bool {
        lock(&self.documents)
            .get(label)
            .is_some_and(|document| document.instance == instance)
    }
}

fn deliver(window: &WebviewWindow, document: &Document, state: &SetupState) {
    let Some(token) = &document.token else {
        return;
    };
    if !window
        .url()
        .is_ok_and(|url| url.origin() == document.origin)
    {
        return;
    }
    let detail = serde_json::json!({ "token": token, "setup": state });
    // Check the current document again inside the queued WebView callback.
    let origin = serde_json::json!(document.origin.ascii_serialization());
    let script = format!(
        "if (window === window.top && location.origin === {origin}) {{ const detail = {detail}; const previous = window.__ATLAS_DESKTOP_SETUP__; if (!previous || previous.token !== detail.token || previous.setup.sequence <= detail.setup.sequence) {{ window.__ATLAS_DESKTOP_SETUP__ = detail; window.dispatchEvent(new Event('atlas-desktop-setup')); }} }}"
    );
    let _ = window.eval(script);
}

fn broadcast(app: &AppHandle) {
    let _ = run_on_main(app, |app| {
        let desktop = app.state::<Desktop>();
        let snapshot = lock(&desktop.staged_setup.state).clone();
        let documents = lock(&desktop.staged_setup.documents).clone();
        for (label, document) in documents {
            if let Some(window) = app.get_webview_window(&label) {
                deliver(&window, &document, &snapshot);
            }
        }
    });
}

pub fn page_load(window: WebviewWindow, payload: tauri::webview::PageLoadPayload<'_>) {
    let desktop = window.state::<Desktop>();
    let instance = lock(&desktop.launcher)
        .as_ref()
        .map(|launcher| launcher.ready.instance.clone());
    let mut documents = lock(&desktop.staged_setup.documents);
    let Some(doc) = documents.get_mut(window.label()) else {
        return;
    };
    if payload.event() == PageLoadEvent::Started {
        doc.token = None;
        return;
    }
    if instance.as_deref() != Some(&doc.instance) || payload.url().origin() != doc.origin {
        return;
    }
    let mut token = [0; 32];
    if getrandom::fill(&mut token).is_err() {
        return;
    }
    doc.token = Some(token.iter().map(|byte| format!("{byte:02x}")).collect());
    drop(documents);
    broadcast(window.app_handle());
}

pub fn cancel(app: &AppHandle, expected_job: Option<usize>) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    {
        let mut state = lock(&desktop.staged_setup.state);
        if expected_job.is_some_and(|job| job != state.job) {
            return Err("This setup action belongs to an earlier job.".into());
        }
        if state.status != Status::Running {
            return Ok(());
        }
        state.status = Status::Cancelling;
        state.sequence += 1;
        desktop.cancel_requested.store(true, Ordering::Release);
        lock(&desktop.setup_pipe).take();
    }
    broadcast(app);
    Ok(())
}

fn begin(app: &AppHandle, previous_job: Option<usize>, local: bool) -> Result<usize, String> {
    let desktop = app.state::<Desktop>();
    let _operation = desktop
        .operation
        .try_lock()
        .map_err(|_| "Atlas is already working.")?;
    let mut state = lock(&desktop.staged_setup.state);
    if matches!(state.status, Status::Running | Status::Cancelling)
        || previous_job.is_some_and(|job| {
            job != state.job
                || !matches!(state.status, Status::Available | Status::Failed)
                || !state.can_resume
                || (local && !state.local_build)
        })
    {
        return Err("Atlas setup state changed. Try again.".into());
    }
    desktop.cancel_requested.store(false, Ordering::Release);
    let job = state.job + 1;
    *state = SetupState {
        job,
        sequence: state.sequence + 1,
        status: Status::Running,
        ..Default::default()
    };
    Ok(job)
}

pub fn from_bootstrap(
    app: AppHandle,
    window: WebviewWindow,
    stage: SetupStage,
    local: bool,
) -> Result<String, Value> {
    begin(&app, None, local)?;
    run(app, Some(window), stage, local)?;
    Ok("Your workspace is ready.".into())
}

fn run(
    app: AppHandle,
    bootstrap: Option<WebviewWindow>,
    stage: SetupStage,
    local: bool,
) -> Result<(), Value> {
    broadcast(&app);
    let (published, receipts) = mpsc::channel();
    let opening_task = bootstrap
        .as_ref()
        .filter(|window| window.label() == opening::PRIMARY)
        .map(|window| {
            let app = app.clone();
            let window = window.clone();
            std::thread::spawn(move || {
                for receipt in receipts {
                    match open_committed(&app, &window, &receipt) {
                        Ok(true) => return Ok(true),
                        Ok(false) => continue, // A later receipt can overtake this commit.
                        Err(error) => {
                            opening::retry_before_navigation(&app, &window, &error);
                            return Err(error);
                        }
                    }
                }
                Ok(false)
            })
        });

    let progress_app = app.clone();
    let progress_window = bootstrap.clone();
    let result = process::setup(
        Some(stage),
        local,
        move |event| {
            let desktop = progress_app.state::<Desktop>();
            if event["phase"] == "saved_writes_paused" {
                let _ = run_on_main(&progress_app, |app| refresh_product(app, true));
            }
            {
                let mut state = lock(&desktop.staged_setup.state);
                if state.status == Status::Running {
                    state.progress = Some(event.clone());
                    state.sequence += 1;
                }
            }
            if let Some(window) = &progress_window {
                let _ = window.emit("atlas-progress", event);
            }
            broadcast(&progress_app);
        },
        move |receipt| {
            let _ = published.send(receipt);
        },
        |pipe| {
            let desktop = app.state::<Desktop>();
            let mut held = lock(&desktop.setup_pipe);
            if !desktop.cancel_requested.load(Ordering::Acquire)
                && !desktop.quitting.load(Ordering::Acquire)
            {
                *held = Some(pipe);
            }
        },
    );
    let desktop = app.state::<Desktop>();
    lock(&desktop.setup_pipe).take();
    let opening_result = opening_task
        .map(|thread| {
            thread
                .join()
                .map_err(|_| "Atlas could not open the workspace.".to_owned())?
        })
        .transpose();
    let probe = process::probe().ok();
    let can_resume = probe
        .as_ref()
        .is_some_and(|probe| probe.stage == Some(SetupStage::Resume));
    let semantic_ready =
        probe.is_some_and(|probe| probe.status == atlas_desktop_protocol::WorkspaceStatus::Ready);
    {
        let mut state = lock(&desktop.staged_setup.state);
        state.sequence += 1;
        state.progress = None;
        state.status = match &result {
            Ok(()) => Status::Idle,
            Err(_) if semantic_ready => Status::Idle,
            Err(_) if can_resume && state.status == Status::Cancelling => Status::Available,
            Err(_) if state.status == Status::Cancelling => Status::Idle,
            Err(_) => Status::Failed,
        };
        state.can_resume = can_resume;
        state.local_build = result
            .as_ref()
            .is_err_and(|error| error["code"] == "embedding_seed_unavailable");
        state.message = result
            .as_ref()
            .err()
            .filter(|_| !semantic_ready)
            .and_then(|error| {
                error["message"]
                    .as_str()
                    .or_else(|| error.as_str())
                    .map(str::to_owned)
            });
    }
    broadcast(&app);
    let _ = run_on_main(&app, |app| refresh_product(app, true));
    if opening_result.map_err(Value::from)? == Some(false) && result.is_ok() {
        return Err(Value::from(
            "Atlas could not verify the published workspace. Try opening Atlas again.",
        ));
    }
    result
}

fn open_committed(
    app: &AppHandle,
    window: &WebviewWindow,
    receipt: &SetupPublication,
) -> Result<bool, String> {
    let desktop = app.state::<Desktop>();
    let _operation = lock(&desktop.operation);
    let probe = process::probe()?;
    if receipt.workspace != probe.workspace || desktop.quitting.load(Ordering::Acquire) {
        return Ok(false);
    }
    crate::start_launcher(app)?;
    let launcher = lock(&desktop.launcher)
        .as_ref()
        .ok_or("The Atlas service stopped.")?
        .ready
        .clone();
    let url = verified_url(&launcher.endpoint)?
        .join("/api/readiness")
        .map_err(display)?;
    let ready: Value = crate::local_http()
        .get(url.as_str())
        .header("x-atlas-api-contract", &launcher.api_contract)
        .call()
        .map_err(display)?
        .body_mut()
        .read_json()
        .map_err(display)?;
    #[cfg(debug_assertions)]
    let ready = if crate::smoke::forces_preflight_retry() {
        serde_json::json!({ "status": "blocked" })
    } else {
        ready
    };
    let generation = ready["artifact_generation"]
        .as_str()
        .filter(|_| ready["status"] == "ready")
        .ok_or("The local service readiness check failed.")?;
    if generation != receipt.generation {
        return Ok(false);
    }
    if !lock(&desktop.launcher)
        .as_ref()
        .is_some_and(|current| current.ready.instance == launcher.instance)
    {
        return Err("The Atlas service stopped.".into());
    }
    let _ = window.emit("atlas-opening", ());
    opening::open(app, window)?;
    Ok(true)
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupAction {
    Cancel,
    Resume,
    BuildLocally,
}

#[tauri::command]
pub fn desktop_setup_action(
    window: WebviewWindow,
    token: String,
    job: usize,
    action: SetupAction,
) -> Result<(), String> {
    let app = window.app_handle().clone();
    let desktop = app.state::<Desktop>();
    let instance = lock(&desktop.launcher)
        .as_ref()
        .map(|launcher| launcher.ready.instance.clone())
        .ok_or("The Atlas service stopped.")?;
    let origin = window.url().map_err(display)?.origin();
    if !lock(&desktop.staged_setup.documents)
        .get(window.label())
        .is_some_and(|doc| {
            doc.instance == instance
                && doc.token.as_deref() == Some(token.as_str())
                && origin == doc.origin
        })
    {
        return Err("This setup action belongs to an earlier Atlas view.".into());
    }
    if matches!(action, SetupAction::Cancel) {
        return cancel(&app, Some(job));
    }
    let local = matches!(action, SetupAction::BuildLocally);
    begin(&app, Some(job), local)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _ = run(app, None, SetupStage::Resume, local);
    });
    Ok(())
}
