//! Desktop setup jobs and the narrow product document attachment.
use std::{
    collections::HashMap,
    sync::{Mutex, atomic::Ordering, mpsc},
};

use atlas_desktop_protocol::{
    LauncherReady, ReferenceRefreshOutcome, ReferenceRefreshResult, ReferenceStatus,
    SetupPublication, SetupStage,
};
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
    menu_requests: std::sync::atomic::AtomicUsize,
}

#[derive(Clone)]
struct Document {
    instance: String,
    origin: url::Origin,
    token: Option<String>,
    open_request: Option<usize>,
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
    reference_job: bool,
    #[serde(skip)]
    reference: Option<ReferenceStatus>,
    #[serde(skip)]
    reference_loading: bool,
    #[serde(skip)]
    reference_request: usize,
    #[serde(skip)]
    pending_refresh: Option<ReferenceStatus>,
    #[serde(skip)]
    reference_retry_offer: Option<String>,
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
                open_request: None,
            },
        );
        Ok(())
    }

    pub fn clear_documents(&self) {
        lock(&self.documents).clear();
        let mut state = lock(&self.state);
        state.reference = None;
        state.reference_loading = false;
        state.reference_request += 1;
        state.sequence += 1;
    }

    pub fn attached_to(&self, label: &str, instance: &str) -> bool {
        lock(&self.documents)
            .get(label)
            .is_some_and(|document| document.instance == instance)
    }

    pub fn has_document(&self, label: &str, instance: &str) -> bool {
        lock(&self.documents)
            .get(label)
            .is_some_and(|document| document.instance == instance && document.token.is_some())
    }

    pub fn request_open(&self, window: &WebviewWindow, instance: &str, initial: bool) -> bool {
        let origin = window.url().ok().map(|url| url.origin());
        let mut documents = lock(&self.documents);
        let Some(doc) = documents.get_mut(window.label()) else {
            return false;
        };
        if doc.instance != instance
            || origin.as_ref() != Some(&doc.origin) && !initial
            || doc.token.is_none() && !initial
        {
            return false;
        }
        doc.open_request = Some(self.menu_requests.fetch_add(1, Ordering::Relaxed) + 1);
        drop(documents);
        broadcast(window.app_handle());
        true
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
    let reference = state.reference.as_ref().map(|reference| {
        serde_json::json!({
            "revision": reference.revision,
            "target": reference.target,
            "targetVersion": reference.target_version,
            "managed": reference.managed,
            "available": reference.offer.is_some(),
        })
    });
    let detail = serde_json::json!({
        "token": token, "setup": state, "reference": reference,
        "referenceLoading": state.reference_loading, "openRequest": document.open_request,
    });
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
        if doc.token.is_some() {
            doc.open_request = None;
        }
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
    load_reference(window.app_handle(), false);
    broadcast(window.app_handle());
}

fn current_launcher(app: &AppHandle) -> Option<LauncherReady> {
    lock(&app.state::<Desktop>().launcher)
        .as_ref()
        .map(|launcher| launcher.ready.clone())
}

fn load_reference(app: &AppHandle, force: bool) {
    let desktop = app.state::<Desktop>();
    let (launcher, request) = {
        let current = lock(&desktop.launcher);
        let Some(launcher) = current.as_ref().map(|launcher| launcher.ready.clone()) else {
            return;
        };
        let mut state = lock(&desktop.staged_setup.state);
        if !force && (state.reference_loading || state.reference.is_some()) {
            return;
        }
        state.reference_loading = true;
        state.reference_request += 1;
        state.sequence += 1;
        (launcher, state.reference_request)
    };
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let status = process::reference_status().ok();
        store_reference(&app, &launcher, request, status);
    });
}

fn same_workspace(before: &ReferenceStatus, after: &ReferenceStatus) -> bool {
    before.workspace == after.workspace
        && before.generation.rsplit_once(':').map(|(key, _)| key)
            == after.generation.rsplit_once(':').map(|(key, _)| key)
}

fn store_reference(
    app: &AppHandle,
    launcher: &LauncherReady,
    request: usize,
    status: Option<ReferenceStatus>,
) {
    let desktop = app.state::<Desktop>();
    let current = lock(&desktop.launcher);
    if !current
        .as_ref()
        .is_some_and(|value| value.ready.instance == launcher.instance)
    {
        return;
    }
    let status = status.filter(|status| status.workspace == launcher.workspace);
    let changed = {
        let mut state = lock(&desktop.staged_setup.state);
        if state.reference_request != request {
            return;
        }
        let changed = status.as_ref().and_then(|status| {
            state.pending_refresh.take().map(|before| {
                if !same_workspace(&before, status) {
                    state.message = None;
                    state.local_build = false;
                    state.reference_job = false;
                    false
                } else {
                    before.generation != status.generation
                }
            })
        });
        if changed == Some(true) {
            state.local_build = false;
        }
        if state.reference_job {
            state.local_build = state.reference_retry_offer.as_ref().is_some_and(|offer| {
                status.as_ref().and_then(|status| status.offer.as_ref()) == Some(offer)
            });
            if !state.local_build
                && status.is_some()
                && state.reference_retry_offer.take().is_some()
            {
                state.message = Some(
                    "Reference setup changed. Check the current reference status and try again."
                        .into(),
                );
            }
        }
        state.reference = status;
        state.reference_loading = false;
        state.sequence += 1;
        changed
    };
    drop(current);
    broadcast(app);
    if let Some(changed) = changed {
        let instance = launcher.instance.clone();
        let _ = run_on_main(app, move |app| {
            if current_launcher(app).is_some_and(|launcher| launcher.instance == instance) {
                refresh_product(app, !changed);
            }
        });
    }
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

enum Start {
    Bootstrap,
    Resume {
        job: usize,
        local: bool,
    },
    Reference {
        job: usize,
        sequence: usize,
        action: SetupAction,
    },
}

fn begin(app: &AppHandle, start: Start) -> Result<Option<ReferenceStatus>, String> {
    let desktop = app.state::<Desktop>();
    let _operation = desktop
        .operation
        .try_lock()
        .map_err(|_| "Atlas is already working.")?;
    let mut state = lock(&desktop.staged_setup.state);
    let stale = || "Atlas setup state changed. Try again.".to_owned();
    if matches!(state.status, Status::Running | Status::Cancelling) {
        return Err(stale());
    }
    let reference = match start {
        Start::Bootstrap => None,
        Start::Resume { job, local } => {
            if job != state.job
                || !matches!(state.status, Status::Available | Status::Failed)
                || !state.can_resume
                || (local && !state.local_build)
            {
                return Err(stale());
            }
            None
        }
        Start::Reference {
            job,
            sequence,
            action,
        } => {
            if job != state.job
                || sequence != state.sequence
                || state.reference_loading
                || state.can_resume
            {
                return Err(stale());
            }
            let reference = state
                .reference
                .as_ref()
                .filter(|reference| reference.offer.is_some())
                .ok_or_else(stale)?;
            let pending = reference.revision.as_deref() != Some(&reference.target);
            match action {
                SetupAction::ApplyReference if pending => {}
                SetupAction::CheckReference if !pending => {}
                SetupAction::BuildLocally
                    if state.reference_job
                        && state.local_build
                        && state.reference_retry_offer == reference.offer => {}
                _ => return Err(stale()),
            }
            Some(reference.clone())
        }
    };
    if desktop.quitting.load(Ordering::Acquire) {
        return Err("Atlas setup state changed. Try again.".into());
    }
    desktop.cancel_requested.store(false, Ordering::Release);
    let job = state.job + 1;
    *state = SetupState {
        job,
        sequence: state.sequence + 1,
        status: Status::Running,
        reference_job: reference.is_some(),
        reference: state.reference.take(),
        reference_loading: state.reference_loading,
        reference_request: state.reference_request,
        pending_refresh: state.pending_refresh.take(),
        ..Default::default()
    };
    Ok(reference)
}

pub fn from_bootstrap(
    app: AppHandle,
    window: WebviewWindow,
    stage: SetupStage,
    local: bool,
) -> Result<String, Value> {
    begin(&app, Start::Bootstrap)?;
    run(app, Some(window), Job::Staged(stage), local)?;
    Ok("Your workspace is ready.".into())
}

enum Job {
    Staged(SetupStage),
    Reference(ReferenceStatus),
}

fn run(
    app: AppHandle,
    bootstrap: Option<WebviewWindow>,
    job: Job,
    local: bool,
) -> Result<(), Value> {
    broadcast(&app);
    let (published, receipts) = mpsc::channel();
    let (completed, completion) = mpsc::channel();
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
    let mode = match &job {
        Job::Staged(stage) => process::SetupMode::Staged(*stage),
        Job::Reference(reference) => process::SetupMode::Reference(
            reference
                .offer
                .as_deref()
                .ok_or("Reference setup is unavailable.")?,
        ),
    };
    let result = process::setup(
        mode,
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
        move |event| match event {
            process::SetupEvent::Published(receipt) => {
                let _ = published.send(receipt);
            }
            process::SetupEvent::ReferenceFinished(receipt) => {
                let _ = completed.send(receipt);
            }
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
    if let Job::Reference(reference) = job {
        finish_reference(&app, reference, &result, completion.try_recv().ok());
        return result;
    }
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
    load_reference(&app, true);
    let _ = run_on_main(&app, |app| refresh_product(app, true));
    if opening_result.map_err(Value::from)? == Some(false) && result.is_ok() {
        return Err(Value::from(
            "Atlas could not verify the published workspace. Try opening Atlas again.",
        ));
    }
    result
}

fn finish_reference(
    app: &AppHandle,
    before: ReferenceStatus,
    result: &Result<(), Value>,
    receipt: Option<ReferenceRefreshResult>,
) {
    let desktop = app.state::<Desktop>();
    let (launcher, request) = {
        let current = lock(&desktop.launcher);
        let launcher = current.as_ref().map(|launcher| launcher.ready.clone());
        let mut state = lock(&desktop.staged_setup.state);
        let cancelled = state.status == Status::Cancelling
            || result
                .as_ref()
                .err()
                .and_then(Value::as_str)
                .is_some_and(|message| message.starts_with("Setup was interrupted."));
        let outcome = receipt.map(|receipt| receipt.outcome).filter(|outcome| {
            if let ReferenceRefreshOutcome::Ready {
                generation,
                updated,
                ..
            } = outcome
            {
                generation.rsplit_once(':').map(|(key, _)| key)
                    == before.generation.rsplit_once(':').map(|(key, _)| key)
                    && *updated == (generation != &before.generation)
            } else {
                !cancelled
            }
        });
        let message = match &outcome {
            Some(ReferenceRefreshOutcome::Ready { updated: true, .. }) => "Reference data updated.",
            Some(ReferenceRefreshOutcome::Ready {
                model_repaired: true,
                ..
            }) => "Search model repaired. Reference data is current.",
            Some(ReferenceRefreshOutcome::Ready { .. }) => "Reference data is already up to date.",
            None if cancelled => "Setup was cancelled.",
            Some(ReferenceRefreshOutcome::SeedUnavailable { .. }) => {
                "Precomputed embeddings are unavailable. Retry or build embeddings locally."
            }
            Some(ReferenceRefreshOutcome::Busy) => {
                "Another Atlas session is using this workspace. Try again in a moment."
            }
            None if result.is_ok() => {
                "Atlas could not confirm the result. Check the current reference status."
            }
            None => {
                "Reference setup could not finish. Check the current reference status and try again."
            }
        };
        state.status =
            if matches!(outcome, Some(ReferenceRefreshOutcome::Ready { .. })) || cancelled {
                Status::Idle
            } else {
                Status::Failed
            };
        state.progress = None;
        state.message = Some(message.into());
        state.local_build = false;
        state.reference_retry_offer = match outcome {
            Some(ReferenceRefreshOutcome::SeedUnavailable { retry_offer }) => Some(retry_offer),
            _ => None,
        };
        state.reference = None;
        state.reference_loading = launcher.is_some();
        state.reference_request += 1;
        state.pending_refresh = Some(before);
        state.sequence += 1;
        (launcher, state.reference_request)
    };
    broadcast(app);
    if let Some(launcher) = launcher {
        store_reference(app, &launcher, request, process::reference_status().ok());
    }
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
    ApplyReference,
    CheckReference,
    ReadReferenceStatus,
    AcknowledgeOpen,
}

#[tauri::command]
pub fn desktop_setup_action(
    window: WebviewWindow,
    token: String,
    job: usize,
    action: SetupAction,
    sequence: Option<usize>,
    open_request: Option<usize>,
) -> Result<(), String> {
    let app = window.app_handle().clone();
    let desktop = app.state::<Desktop>();
    let instance = lock(&desktop.launcher)
        .as_ref()
        .map(|launcher| launcher.ready.instance.clone())
        .ok_or("The Atlas service stopped.")?;
    let origin = window.url().map_err(display)?.origin();
    {
        let mut documents = lock(&desktop.staged_setup.documents);
        let doc = documents
            .get_mut(window.label())
            .filter(|doc| {
                doc.instance == instance
                    && doc.token.as_deref() == Some(token.as_str())
                    && origin == doc.origin
            })
            .ok_or("This setup action belongs to an earlier Atlas view.")?;
        if matches!(action, SetupAction::AcknowledgeOpen) {
            if open_request.is_some() && doc.open_request == open_request {
                doc.open_request = None;
                drop(documents);
                load_reference(&app, true);
                broadcast(&app);
            }
            return Ok(());
        }
    }
    if matches!(action, SetupAction::Cancel) {
        return cancel(&app, Some(job));
    }
    if matches!(action, SetupAction::ReadReferenceStatus) {
        load_reference(&app, true);
        broadcast(&app);
        return Ok(());
    }
    let local = matches!(action, SetupAction::BuildLocally);
    let reference = matches!(
        action,
        SetupAction::ApplyReference | SetupAction::CheckReference
    ) || (local && lock(&desktop.staged_setup.state).reference_job);
    let start = if reference {
        Start::Reference {
            job,
            sequence: sequence.ok_or("Reference setup state changed.")?,
            action,
        }
    } else {
        Start::Resume { job, local }
    };
    let job = begin(&app, start)?.map_or(Job::Staged(SetupStage::Resume), Job::Reference);
    tauri::async_runtime::spawn_blocking(move || {
        let _ = run(app, None, job, local);
    });
    Ok(())
}
