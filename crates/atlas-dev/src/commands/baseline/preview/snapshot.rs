use super::{OwnedChild, POLL, Preview, Result, files};
use crate::commands::baseline::{checks, model::Assets};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

pub(super) struct Snapshot {
    _directory: tempfile::TempDir,
    pub binary: PathBuf,
    original: PathBuf,
    pub binary_sha256: String,
    pub frontend_sha256: String,
    provenance: Value,
}
impl Snapshot {
    pub fn prepare(
        preview: &Preview,
        binary: &Path,
        frontend: &Path,
        provenance: Option<&Path>,
    ) -> Result<Self> {
        let original = binary.canonicalize()?;
        let binary_sha256 = files::hash(&original)?;
        let frontend_sha256 = files::hash(frontend)?;
        let provenance: Value = match provenance {
            Some(path) => {
                let value: Value = files::read(path)?;
                if value["version"] != 1
                    || value["binary_sha256"] != binary_sha256
                    || value["frontend_sha256"] != frontend_sha256
                {
                    return Err(
                        "Build receipt does not match executable and frontend checksums".into(),
                    );
                }
                json!({"kind":"build_receipt","identity":"caller-recorded source provenance; artifact checksums verified","receipt":value})
            }
            None => {
                json!({"kind":"unrecorded","identity":"Executable and expected UI checksums only; source commit unknown"})
            }
        };
        let directory = tempfile::Builder::new()
            .prefix("build-")
            .tempdir_in(&preview.directory)?;
        let binary = directory
            .path()
            .join(if cfg!(windows) { "atlas.exe" } else { "atlas" });
        fs::copy(&original, &binary)?;
        if files::hash(&binary)? != binary_sha256 {
            return Err("Candidate executable changed while copying".into());
        }
        files::mode(&binary, 0o555)?;
        let readiness = checks::data(&check(
            preview,
            &binary,
            &checks::paths(
                &["setup", "--check", "--offline", "--json"],
                &preview.index,
                &preview.manifest.assets,
            ),
        )?)?;
        if readiness["ready"] != true
            || !readiness["search_profiles"]
                .as_array()
                .is_some_and(|profiles| {
                    !profiles.is_empty() && profiles.iter().all(|profile| profile["ready"] == true)
                })
        {
            return Err("Candidate requires setup; previous preview preserved. Run dev-status with the explicit candidate for details".into());
        }
        checks::data(&check(
            preview,
            &binary,
            &checks::paths(
                &["index", "check", "--json"],
                &preview.index,
                &Assets::default(),
            ),
        )?)?;
        Ok(Self {
            _directory: directory,
            binary,
            original,
            binary_sha256,
            frontend_sha256,
            provenance,
        })
    }
    pub fn report(&self) -> Value {
        json!({"original_binary":self.original,"running_binary":self.binary,"binary_sha256":self.binary_sha256,"frontend_sha256":self.frontend_sha256,"provenance":self.provenance})
    }
    pub fn adopt_provenance(&mut self, replacement: &Self) {
        self.original.clone_from(&replacement.original);
        self.provenance.clone_from(&replacement.provenance);
    }
}

fn check(preview: &Preview, binary: &Path, arguments: &[String]) -> Result<Value> {
    let stdout = tempfile::NamedTempFile::new_in(&preview.directory)?;
    let stderr = tempfile::NamedTempFile::new_in(&preview.directory)?;
    let child = Command::new(binary)
        .args(arguments)
        .current_dir(&preview.checkout)
        .stdin(Stdio::null())
        .stdout(stdout.reopen()?)
        .stderr(stderr.reopen()?)
        .spawn()?;
    let mut child = OwnedChild(Some(child));
    let deadline = Instant::now() + super::CHECK_TIMEOUT;
    while Instant::now() < deadline {
        if preview.interrupted.load(Ordering::Relaxed) {
            return Err("Preview interrupted during candidate preflight".into());
        }
        if let Some(status) = child.exited()? {
            let output = fs::read(stdout.path())?;
            let response = serde_json::from_slice::<Value>(&output).unwrap_or_else(|_|json!({"status":"error","message":fs::read_to_string(stderr.path()).unwrap_or_default()}));
            return Ok(json!({"exit_code":status.code().unwrap_or(1),"response":response}));
        }
        thread::sleep(POLL);
    }
    Err("Candidate preflight timed out; previous preview preserved".into())
}

pub(super) fn health(port: u16, expected_index: &str) -> Result<()> {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(1)))
            .build(),
    );
    let base = format!("http://127.0.0.1:{port}");
    let readiness = agent
        .get(format!("{base}/api/readiness"))
        .call()?
        .body_mut()
        .with_config()
        .limit(1024 * 1024)
        .read_to_vec()?;
    let readiness: Value = serde_json::from_slice(&readiness)?;
    if readiness["status"] != "ready"
        || !readiness["search_profiles"]
            .as_array()
            .is_some_and(|profiles| {
                !profiles.is_empty() && profiles.iter().all(|profile| profile["usable"] == true)
            })
    {
        return Err("Served preview has unavailable search profiles".into());
    }
    let html = agent
        .get(&base)
        .call()?
        .body_mut()
        .with_config()
        .limit(4 * 1024 * 1024)
        .read_to_vec()?;
    if format!("{:x}", Sha256::digest(&html)) != expected_index {
        return Err("Served UI does not match the selected candidate index".into());
    }
    Ok(())
}
