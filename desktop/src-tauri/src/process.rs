use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use atlas_desktop_protocol::{
    LauncherReady, ReferenceRefreshResult, ReferenceStatus, SetupPublication, SetupStage, VERSION,
    WorkspaceProbe,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    Desktop,
    workspace_switch::{Admission, Role},
};

pub struct Launcher {
    pub ready: LauncherReady,
    pub pid: u32,
    pipe: ChildStdin,
    stopped: Receiver<()>,
}

pub fn bundled_cli() -> Result<PathBuf, String> {
    let path = std::env::current_exe().map_err(display)?;
    let cli = path.with_file_name(format!("atlas{}", std::env::consts::EXE_SUFFIX));
    if !cli.is_file() {
        return Err("The matching Atlas command is missing from the app bundle.".into());
    }
    Ok(cli)
}

pub fn home() -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("ATLAS_PILOT_HOME") {
        return Ok(path.into());
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "Atlas could not locate your home folder.".into())
}

fn command() -> Result<Command, String> {
    let mut command = Command::new(bundled_cli()?);
    command.env("HOME", home()?);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

        command.creation_flags(CREATE_NO_WINDOW);
        #[cfg(debug_assertions)]
        if let Some(home) = std::env::var_os("ATLAS_PILOT_HOME") {
            command.env("LOCALAPPDATA", home);
        }
    }
    Ok(command)
}

fn routed(app: &AppHandle, subcommand: &str) -> Result<Command, String> {
    let mut command = command()?;
    command.arg(subcommand);
    if let Some(index) = app.state::<Desktop>().workspace_route.index()? {
        command.arg("--index").arg(index);
    }
    if let Some(pending) = crate::workspace_switch::load(app)?.pending
        && crate::updates::installed_build().as_ref() == Some(&pending.target)
    {
        command.env(
            "ATLAS_DESKTOP_EXPECTED_WORKSPACE",
            serde_json::to_string(&pending.admission).map_err(display)?,
        );
    }
    Ok(command)
}

pub(crate) fn migration_ledger() -> Result<Value, String> {
    let output = command()?
        .args(["desktop-probe", "--update-compatibility"])
        .output()
        .map_err(display)?;
    let facts: Value = decode(&output.stdout)?;
    facts
        .get("migrationLedger")
        .cloned()
        .ok_or("Missing Atlas migration ledger.".into())
}

pub(crate) struct TransitionReservation {
    pub(crate) admission: Admission,
    child: Child,
}

impl Drop for TransitionReservation {
    fn drop(&mut self) {
        self.child.stdin.take();
        let _ = self.child.wait();
    }
}

pub(crate) fn inspect_transition(
    role: Role,
    ledger: &Value,
    seed: Option<&Admission>,
) -> Result<Admission, String> {
    Ok(transition(role, ledger, seed, None, false)?
        .admission
        .clone())
}

pub(crate) fn reserve_transition(
    role: Role,
    ledger: &Value,
) -> Result<TransitionReservation, String> {
    transition(role, ledger, None, None, true)
}

pub(crate) fn claim_transition(
    admission: &Admission,
    ledger: &Value,
    key: &str,
) -> Result<Admission, String> {
    Ok(transition(
        admission.role,
        ledger,
        None,
        Some((&admission.workspace, key)),
        false,
    )?
    .admission
    .clone())
}

fn transition(
    role: Role,
    ledger: &Value,
    seed: Option<&Admission>,
    initialize: Option<(&PathBuf, &str)>,
    reserve: bool,
) -> Result<TransitionReservation, String> {
    let mut command = command()?;
    command.args(["desktop-probe", "--transition-workspace", role.as_arg()]);
    if reserve {
        command.arg("--reserve-transition");
    }
    let mut payload = serde_json::to_vec(&serde_json::json!({
        "ledger": ledger,
        "initialize": initialize.map(|(workspace, key)| serde_json::json!({"workspace": workspace, "key": key})),
        "seed": seed.map(|source| serde_json::json!({
            "workspace": source.workspace, "key": source.key, "schema": source.schema, "file_id": source.file_id,
        }))
    })).map_err(display)?;
    payload.push(b'\n');
    if payload.len() > 65_536 {
        return Err("Atlas migration ledger exceeds the supported size.".into());
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(display)?;
    let result = (|| {
        child
            .stdin
            .as_mut()
            .ok_or("Could not open the workspace check pipe.")?
            .write_all(&payload)
            .map_err(display)?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Could not read the workspace check.")?;
        let mut line = String::new();
        BufReader::new(stdout)
            .read_line(&mut line)
            .map_err(display)?;
        decode(line.as_bytes())
    })();
    match result {
        Ok(admission) => Ok(TransitionReservation { admission, child }),
        Err(error) => {
            child.stdin.take();
            if let Ok(output) = child.wait_with_output()
                && let Ok(message) = String::from_utf8(output.stderr)
                && !message.trim().is_empty()
            {
                return Err(message.trim().to_owned());
            }
            Err(error)
        }
    }
}

pub fn probe(app: &AppHandle) -> Result<WorkspaceProbe, String> {
    probe_with(app, &[])
}

pub fn reference_status(app: &AppHandle) -> Result<ReferenceStatus, String> {
    let output = routed(app, "desktop-probe")?
        .arg("--reference-status")
        .output()
        .map_err(display)?;
    let status: ReferenceStatus = decode(&output.stdout)?;
    if !output.status.success() || status.version != VERSION {
        return Err("Atlas could not verify the workspace reference status.".into());
    }
    Ok(status)
}

pub fn wait_for_upgrade(app: &AppHandle) -> Result<(), String> {
    let probe = probe_with(app, &["--wait-for-service"])?;
    if probe.status == atlas_desktop_protocol::WorkspaceStatus::Busy {
        return Err(probe.message);
    }
    Ok(())
}

fn probe_with(app: &AppHandle, args: &[&str]) -> Result<WorkspaceProbe, String> {
    let output = routed(app, "desktop-probe")?
        .args(args)
        .output()
        .map_err(display)?;
    let probe: WorkspaceProbe = decode(&output.stdout)?;
    if !output.status.success() || probe.version != VERSION {
        return Err("The app and bundled Atlas command use different readiness protocols.".into());
    }
    Ok(probe)
}

pub fn start(
    app: &AppHandle,
    on_exit: impl FnOnce(u32) + Send + 'static,
) -> Result<Launcher, String> {
    let mut child = routed(app, "web")?
        .arg("--desktop-launcher")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(display)?;
    let pid = child.id();
    let pipe = child
        .stdin
        .take()
        .ok_or("Atlas could not open its launcher pipe.")?;
    let output = child
        .stdout
        .take()
        .ok_or("Atlas could not read launcher readiness.")?;
    let (ready_tx, ready_rx) = mpsc::channel();
    let (stopped_tx, stopped) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(output);
        let mut line = String::new();
        let result = match reader.read_line(&mut line) {
            Ok(0) => Err("The Atlas launcher stopped before it was ready.".into()),
            Ok(_) => decode::<LauncherReady>(line.as_bytes()),
            Err(error) => Err(display(error)),
        };
        let _ = ready_tx.send(result);
        let _ = std::io::copy(&mut reader, &mut std::io::sink());
        let _ = child.wait();
        let _ = stopped_tx.send(());
        on_exit(pid);
    });
    let ready = ready_rx
        .recv_timeout(Duration::from_secs(20))
        .map_err(|_| "Timed out waiting for the local Atlas service.".to_owned())??;
    if ready.version != VERSION || ready.instance.is_empty() || ready.api_contract.is_empty() {
        return Err("The local service did not return a compatible Atlas identity.".into());
    }
    Ok(Launcher {
        ready,
        pid,
        pipe,
        stopped,
    })
}

impl Launcher {
    pub fn has_stopped(&self) -> bool {
        !matches!(self.stopped.try_recv(), Err(mpsc::TryRecvError::Empty))
    }

    pub fn release(self) -> Result<(), String> {
        drop(self.pipe);
        self.stopped
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "Atlas is still releasing its connection. Try again shortly.".into())
    }
}

pub enum SetupMode<'a> {
    Ordinary,
    UpgradeWorkspace,
    Staged(SetupStage),
    Reference(&'a str),
    Models(&'a str, &'a atlas_desktop_protocol::ModelSelection, bool),
}

pub enum SetupEvent {
    Published(SetupPublication),
    ReferenceFinished(ReferenceRefreshResult),
}

pub fn setup(
    app: &AppHandle,
    mode: SetupMode<'_>,
    build_embeddings_locally: bool,
    progress: impl Fn(Value) + Send + 'static,
    event_received: impl Fn(SetupEvent) + Send + 'static,
    hold: impl FnOnce(ChildStdin),
) -> Result<(), Value> {
    let mut command = routed(app, "setup")?;
    command.args(["--json", "--desktop-progress", "--desktop-cancel-on-eof"]);
    match mode {
        SetupMode::Staged(stage) => command.args([
            "--desktop-stage",
            match stage {
                SetupStage::FirstRun => "first-run",
                SetupStage::Resume => "resume",
            },
        ]),
        SetupMode::Reference(offer) => command.args(["--desktop-reference-offer", offer]),
        SetupMode::Models(offer, selection, default_only) => {
            if default_only {
                command.arg("--desktop-set-default");
            }
            command
                .args([
                    "--desktop-reference-offer",
                    offer,
                    "--desktop-model-selection",
                ])
                .arg(serde_json::to_string(selection).map_err(display)?)
        }
        SetupMode::UpgradeWorkspace => command.arg("--desktop-upgrade-workspace"),
        SetupMode::Ordinary => &mut command,
    };
    if build_embeddings_locally {
        command.arg("--build-embeddings-locally");
    }
    #[cfg(all(debug_assertions, target_os = "macos"))]
    if matches!(mode, SetupMode::Reference(_)) && crate::smoke::reference_mode() {
        crate::smoke::record_setup();
    }
    #[cfg(all(debug_assertions, target_os = "macos"))]
    if matches!(mode, SetupMode::Ordinary | SetupMode::Staged(_))
        && let Some(source) = crate::smoke::source()?
    {
        crate::smoke::record_setup();
        command.args(["--offline", "--source"]);
        command.arg(source);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(display)?;
    let pipe = child
        .stdin
        .take()
        .ok_or("Atlas could not open the setup pipe.")?;
    hold(pipe);
    let stderr = child
        .stderr
        .take()
        .ok_or("Atlas could not read setup progress.")?;
    let progress_reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<Value>(&line)
                && event["version"] == VERSION
            {
                progress(event);
            } else if let Ok(event) = serde_json::from_str::<SetupPublication>(&line)
                && event.setup_event_version == atlas_desktop_protocol::SETUP_EVENT_VERSION
            {
                event_received(SetupEvent::Published(event));
            } else if let Ok(event) = serde_json::from_str::<ReferenceRefreshResult>(&line)
                && event.reference_refresh_version
                    == atlas_desktop_protocol::REFERENCE_REFRESH_VERSION
            {
                event_received(SetupEvent::ReferenceFinished(event));
            }
        }
    });
    let output = child.wait_with_output().map_err(display)?;
    let _ = progress_reader.join();
    if output.status.code() == Some(130) {
        return Err("Setup was interrupted. Run setup again to continue.".into());
    }
    let report: Value = decode(&output.stdout)?;
    if output.status.success() && report["ready"] == true {
        return Ok(());
    }
    let reasons = report["not_ready_reasons"].as_array();
    let reason = reasons.and_then(|reasons| {
        reasons
            .iter()
            .rev()
            .find(|reason| reason["status"] == "failed")
            .or_else(|| {
                reasons
                    .iter()
                    .rev()
                    .find(|reason| reason["status"] == "blocked")
            })
            .or_else(|| reasons.first())
    });
    let message = reason
        .and_then(|reason| reason["message"].as_str())
        .unwrap_or("Setup could not finish. Check the workspace and try again.");
    if reason.is_some_and(|reason| reason["code"] == "embedding_seed_unavailable") {
        return Err(
            serde_json::json!({ "code": "embedding_seed_unavailable", "message": message }),
        );
    }
    Err(message.into())
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let envelope: Value = serde_json::from_slice(bytes)
        .map_err(|_| "The bundled Atlas command did not return a valid response.".to_owned())?;
    if envelope["status"] != "ok" {
        return Err(envelope["error"]["message"]
            .as_str()
            .unwrap_or("The bundled Atlas command could not complete the operation.")
            .into());
    }
    serde_json::from_value(envelope["data"].clone()).map_err(display)
}

fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}
