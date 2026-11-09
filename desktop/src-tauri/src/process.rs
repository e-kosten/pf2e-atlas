use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use atlas_desktop_protocol::{LauncherReady, VERSION, WorkspaceProbe};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct Launcher {
    pub ready: LauncherReady,
    pub pid: u32,
    pipe: ChildStdin,
    stopped: Receiver<()>,
}

pub fn bundled_cli() -> Result<PathBuf, String> {
    let path = std::env::current_exe().map_err(display)?;
    let cli = path.with_file_name("atlas");
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
        .map(PathBuf::from)
        .ok_or_else(|| "Atlas could not locate your home folder.".into())
}

fn command() -> Result<Command, String> {
    let mut command = Command::new(bundled_cli()?);
    command.env("HOME", home()?);
    Ok(command)
}

pub fn probe() -> Result<WorkspaceProbe, String> {
    probe_with(&[])
}

pub fn wait_for_upgrade() -> Result<(), String> {
    let probe = probe_with(&["--wait-for-service"])?;
    if probe.status == atlas_desktop_protocol::WorkspaceStatus::Busy {
        return Err(probe.message);
    }
    Ok(())
}

fn probe_with(args: &[&str]) -> Result<WorkspaceProbe, String> {
    let output = command()?
        .arg("desktop-probe")
        .args(args)
        .output()
        .map_err(display)?;
    let probe: WorkspaceProbe = decode(&output.stdout)?;
    if !output.status.success() || probe.version != VERSION {
        return Err("The app and bundled Atlas command use different readiness protocols.".into());
    }
    Ok(probe)
}

pub fn start(on_exit: impl FnOnce(u32) + Send + 'static) -> Result<Launcher, String> {
    let mut child = command()?
        .args(["web", "--desktop-launcher"])
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

pub fn setup(
    progress: impl Fn(Value) + Send + 'static,
    hold: impl FnOnce(ChildStdin),
) -> Result<(), String> {
    let mut child = command()?
        .args([
            "setup",
            "--json",
            "--desktop-progress",
            "--desktop-cancel-on-eof",
        ])
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
    let reason = reasons
        .and_then(|reasons| {
            reasons
                .iter()
                .rev()
                .find(|reason| reason["status"] == "failed" || reason["status"] == "blocked")
                .or_else(|| reasons.first())
        })
        .and_then(|reason| reason["message"].as_str())
        .unwrap_or("Setup could not finish. Check the workspace and try again.");
    Err(reason.into())
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
