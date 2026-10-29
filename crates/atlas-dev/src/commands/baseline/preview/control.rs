use super::{POLL, Result, files};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Action {
    Refresh {
        binary: PathBuf,
        frontend_index: PathBuf,
        build_provenance: Option<PathBuf>,
    },
    Stop,
}
#[derive(Serialize, Deserialize)]
pub(super) struct Request {
    pub session: String,
    pub id: String,
    pub action: Action,
}

pub(super) fn directory(checkout: &Path) -> Result<PathBuf> {
    super::super::workspace_paths(checkout)?;
    let directory = checkout.join(".atlas/dev-preview");
    if directory.is_symlink() {
        return Err("Preview control directory must not be a symlink".into());
    }
    fs::create_dir_all(&directory)?;
    files::mode(&directory, 0o700)?;
    Ok(directory)
}

pub(super) fn lock_file(path: &Path) -> Result<File> {
    if path.is_symlink() {
        return Err("Preview locks must not be symlinks".into());
    }
    Ok(OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?)
}

fn active(directory: &Path) -> Result<bool> {
    if !directory.join("owner.lock").is_file() {
        return Ok(false);
    }
    let file = lock_file(&directory.join("owner.lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn status(checkout: &Path) -> Result<Value> {
    let directory = checkout.join(".atlas/dev-preview");
    if directory.is_symlink() {
        return Err("Preview control directory must not be a symlink".into());
    }
    if !directory.join("status.json").is_file() {
        return Ok(Value::Null);
    }
    let mut report: Value = files::read(&directory.join("status.json"))?;
    report["owner_running"] = json!(active(&directory)?);
    Ok(report)
}

pub(crate) fn request(checkout: &Path, action: Action) -> Result<Value> {
    let directory = directory(checkout)?;
    let serial = lock_file(&directory.join("request.lock"))?;
    serial.lock()?;
    if !active(&directory)? {
        return Err(
            "No managed preview owns this checkout; start dev-web with --managed first".into(),
        );
    }
    let report: Value = files::read(&directory.join("status.json"))?;
    let session = report["session"]
        .as_str()
        .ok_or("Preview report has no session")?;
    let unique = tempfile::Builder::new()
        .prefix("request-")
        .tempfile_in(&directory)?;
    let id = unique
        .path()
        .file_name()
        .ok_or("Missing request identity")?
        .to_string_lossy()
        .into_owned();
    files::replace(
        &directory.join("request.json"),
        &Request {
            session: session.into(),
            id: id.clone(),
            action,
        },
    )?;
    let deadline = Instant::now() + super::REQUEST_TIMEOUT;
    while Instant::now() < deadline {
        if directory.join("response.json").is_file() {
            let response: Value = files::read(&directory.join("response.json"))?;
            if response["id"] == id {
                if response["session"] != session {
                    return Err("Preview ownership changed while handling the request; inspect current status".into());
                }
                if response["ok"] == true {
                    return Ok(response["report"].clone());
                }
                return Err(response["error"]
                    .as_str()
                    .unwrap_or("Preview request failed")
                    .to_owned()
                    .into());
            }
        }
        if !active(&directory)? {
            // The owner may have acknowledged Stop between our response read and
            // lock probe. Read once more after observing that it has exited.
            if directory.join("response.json").is_file() {
                let response: Value = files::read(&directory.join("response.json"))?;
                if response["id"] == id && response["session"] == session && response["ok"] == true
                {
                    return Ok(response["report"].clone());
                }
            }
            return Err(
                "Managed preview exited before acknowledging the request; inspect its log".into(),
            );
        }
        thread::sleep(POLL);
    }
    Err("Preview request timed out; inspect status and log before retrying".into())
}
