//! One foreground owner controls its live child; receipts never authorize PID kills.
mod control;
mod snapshot;

use super::{Manifest, Result, args::WebArgs, binary, files, web_arguments, workspace_baseline};
pub(super) use control::{Action, request, status};
use serde_json::{Value, json};
use snapshot::Snapshot;
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const POLL: Duration = Duration::from_millis(50);
const START_TIMEOUT: Duration = Duration::from_secs(20);
const CHECK_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration =
    Duration::from_secs(CHECK_TIMEOUT.as_secs() + 3 * START_TIMEOUT.as_secs() + 40);

struct Preview {
    checkout: PathBuf,
    directory: PathBuf,
    index: PathBuf,
    manifest: Manifest,
    port: u16,
    session: String,
    log: PathBuf,
    interrupted: Arc<AtomicBool>,
}

pub(super) fn run(checkout: &Path, store: &Path, options: &WebArgs) -> Result<ExitCode> {
    let port = options.candidate.port;
    if port == 0 {
        return Err("Managed previews require a fixed nonzero port".into());
    }
    let (index, baseline_directory, manifest) = workspace_baseline(
        checkout,
        store,
        options.candidate.selection.baseline.as_deref(),
    )?;
    if !index.is_file() {
        return Err("No private workspace; run just dev-workspace-init first".into());
    }
    if index.is_symlink() {
        return Err("Managed preview workspace must be a private file, not a symlink".into());
    }
    let directory = control::directory(checkout)?;
    let owner = control::lock_file(&directory.join("owner.lock"))?;
    owner
        .try_lock()
        .map_err(|error| format!("A managed preview already owns this checkout: {error}"))?;
    let lifetime = tempfile::Builder::new()
        .prefix("session-")
        .tempdir_in(&directory)?;
    let session = lifetime
        .path()
        .file_name()
        .ok_or("Missing preview session")?
        .to_string_lossy()
        .into_owned();
    let interrupted = Arc::new(AtomicBool::new(false));
    let signals = SignalGuard::new(Arc::clone(&interrupted))?;
    let preview = Preview {
        checkout: checkout.into(),
        index,
        manifest,
        port,
        session,
        log: directory.join("server.log"),
        directory,
        interrupted,
    };
    let binary = binary(checkout, &baseline_directory, &options.candidate)?;
    let frontend = checkout.join(
        options
            .frontend_index
            .as_ref()
            .ok_or("Managed preview needs --frontend-index")?,
    );
    let provenance = options
        .build_provenance
        .as_ref()
        .map(|path| checkout.join(path));
    let snapshot = Snapshot::prepare(&preview, &binary, &frontend, provenance.as_deref())?;
    preview.report("starting", &snapshot, Value::Null)?;
    let mut child = match preview.start(&snapshot) {
        Ok(child) => child,
        Err(error) => {
            preview.report("failed", &snapshot, json!({"error":error.to_string()}))?;
            return Err(error);
        }
    };
    preview.report("ready", &snapshot, Value::Null)?;
    eprintln!(
        "Managed preview: http://127.0.0.1:{port}\nWorkspace: {}\nLog: {}",
        preview.index.display(),
        preview.log.display()
    );
    let code = preview.serve(&mut child, snapshot)?;
    drop(signals);
    Ok(code)
}

impl Preview {
    fn report(&self, state: &str, snapshot: &Snapshot, outcome: Value) -> Result<Value> {
        let report = json!({
            "version":1, "session":self.session, "state":state,
            "url":format!("http://127.0.0.1:{}",self.port), "port":self.port,
            "workspace":self.index, "assets":self.manifest.assets,
            "baseline":self.manifest.id, "baseline_build":self.manifest.build,
            "selected_build":snapshot.report(), "log":self.log, "outcome":outcome,
        });
        files::replace(&self.directory.join("status.json"), &report)?;
        Ok(report)
    }

    fn start(&self, snapshot: &Snapshot) -> Result<OwnedChild> {
        self.start_at(snapshot, self.port, &self.log)
    }

    fn probe(&self, snapshot: &Snapshot) -> Result<()> {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        drop(listener);
        let mut child = self.start_at(snapshot, port, &self.directory.join("probe.log"))?;
        child.stop()
    }

    fn start_at(&self, snapshot: &Snapshot, port: u16, log_path: &Path) -> Result<OwnedChild> {
        // Never adopt or signal a listener discovered through the requested URL.
        drop(
            std::net::TcpListener::bind(("127.0.0.1", port))
                .map_err(|error| format!("Preview port {} is occupied: {error}", port))?,
        );
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        let child = Command::new(&snapshot.binary)
            .args(web_arguments(&self.index, &self.manifest, port))
            .current_dir(&self.checkout)
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .spawn()?;
        let mut child = OwnedChild(Some(child));
        self.wait_ready(&mut child, snapshot, port, log_path)?;
        Ok(child)
    }

    fn wait_ready(
        &self,
        child: &mut OwnedChild,
        snapshot: &Snapshot,
        port: u16,
        log_path: &Path,
    ) -> Result<()> {
        let deadline = Instant::now() + START_TIMEOUT;
        let mut last_error = "Waiting for HTTP readiness".to_owned();
        while Instant::now() < deadline {
            if self.interrupted.load(Ordering::Relaxed) {
                return Err("Preview interrupted during startup".into());
            }
            if let Some(status) = child.exited()? {
                return Err(format!(
                    "Preview exited before readiness ({status}); see {}",
                    log_path.display()
                )
                .into());
            }
            match snapshot::health(port, &snapshot.frontend_sha256) {
                Ok(snapshot::Health::Ready) => {
                    if child.exited()?.is_none() {
                        return Ok(());
                    }
                }
                Ok(snapshot::Health::Pending(reason)) => last_error = reason,
                Err(error) => return Err(error),
            }
            thread::sleep(POLL);
        }
        Err(format!(
            "Preview did not become ready: {last_error}; see {}",
            log_path.display()
        )
        .into())
    }

    fn serve(&self, child: &mut OwnedChild, mut snapshot: Snapshot) -> Result<ExitCode> {
        loop {
            if self.interrupted.load(Ordering::Relaxed) {
                child.stop()?;
                self.report("stopped", &snapshot, Value::Null)?;
                return Ok(ExitCode::SUCCESS);
            }
            if let Some(status) = child.exited()? {
                self.report("exited", &snapshot, json!({"exit_code":status.code()}))?;
                return Ok(ExitCode::from(status.code().unwrap_or(1) as u8));
            }
            let request_path = self.directory.join("request.json");
            if !request_path.is_file() {
                thread::sleep(POLL);
                continue;
            }
            let request: control::Request = files::read(&request_path)?;
            fs::remove_file(&request_path)?;
            let mut stopped = false;
            let response = if request.session != self.session {
                Err("Preview ownership changed; retry against the current managed preview".into())
            } else {
                match request.action {
                    Action::Stop => {
                        child.stop()?;
                        stopped = true;
                        self.report("stopped", &snapshot, Value::Null)
                    }
                    Action::Refresh {
                        binary,
                        frontend_index,
                        build_provenance,
                    } => self.refresh(
                        child,
                        &mut snapshot,
                        &binary,
                        &frontend_index,
                        build_provenance.as_deref(),
                    ),
                }
            };
            let response = match response {
                Ok(report) => {
                    json!({"id":request.id,"session":self.session,"ok":true,"report":report})
                }
                Err(error) => {
                    json!({"id":request.id,"session":self.session,"ok":false,"error":error.to_string()})
                }
            };
            files::replace(&self.directory.join("response.json"), &response)?;
            if stopped {
                return Ok(ExitCode::SUCCESS);
            }
        }
    }

    fn refresh(
        &self,
        child: &mut OwnedChild,
        current: &mut Snapshot,
        binary: &Path,
        frontend: &Path,
        provenance: Option<&Path>,
    ) -> Result<Value> {
        // Readiness and compatibility are checked while the old server remains usable.
        let replacement = Snapshot::prepare(self, binary, frontend, provenance)?;
        if replacement.binary_sha256 == current.binary_sha256
            && replacement.frontend_sha256 == current.frontend_sha256
        {
            self.wait_ready(child, current, self.port, &self.log)?;
            current.adopt_provenance(&replacement);
            return self.report(
                "ready",
                current,
                json!({"refreshed":false,"reason":"Same executable and UI already running"}),
            );
        }
        self.probe(&replacement)?;
        self.report("refreshing", current, Value::Null)?;
        child.stop()?;
        match self.start(&replacement) {
            Ok(replacement_child) => {
                *child = replacement_child;
                *current = replacement;
                self.report("ready", current, json!({"refreshed":true}))
            }
            Err(error) => match self.start(current) {
                Ok(previous_child) => {
                    *child = previous_child;
                    self.report("ready", current, json!({"refreshed":false,"replacement_error":error.to_string(),"previous_build_restored":true}))?;
                    Err(format!(
                        "Replacement failed; previous build restored at the same URL: {error}"
                    )
                    .into())
                }
                Err(restore_error) => {
                    self.report("failed", current, json!({"replacement_error":error.to_string(),"restore_error":restore_error.to_string()}))?;
                    Err(format!("Replacement failed ({error}); restoring the previous build also failed ({restore_error})").into())
                }
            },
        }
    }
}

struct OwnedChild(Option<Child>);
impl OwnedChild {
    fn exited(&mut self) -> Result<Option<ExitStatus>> {
        match self.0.as_mut() {
            Some(child) => Ok(child.try_wait()?),
            None => Ok(None),
        }
    }
    fn stop(&mut self) -> Result<()> {
        let Some(child) = self.0.as_mut() else {
            return Ok(());
        };
        if child.try_wait()?.is_some() {
            self.0 = None;
            return Ok(());
        }
        #[cfg(unix)]
        {
            // The live, unreaped Child owns this ID; no receipt/PID lookup is used.
            let _ = Command::new("kill")
                .args(["-INT", &child.id().to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if child.try_wait()?.is_some() {
                    self.0 = None;
                    return Ok(());
                }
                thread::sleep(POLL);
            }
        }
        child.kill()?;
        child.wait()?;
        self.0 = None;
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

struct SignalGuard(Vec<signal_hook::SigId>);
impl SignalGuard {
    fn new(interrupted: Arc<AtomicBool>) -> Result<Self> {
        let mut guard = Self(Vec::new());
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            guard.0.push(signal_hook::flag::register(
                signal,
                Arc::clone(&interrupted),
            )?);
        }
        Ok(guard)
    }
}
impl Drop for SignalGuard {
    fn drop(&mut self) {
        for id in self.0.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}
