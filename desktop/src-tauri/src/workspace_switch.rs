use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{Desktop, display, lock, process};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Role {
    #[default]
    Primary,
    Trial,
}

impl Role {
    pub(crate) fn as_arg(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Trial => "trial",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Waiting {
    Hold,
    Continue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReturnChoice {
    pub(crate) workspace: Role,
    pub(crate) updates: Waiting,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Build {
    pub(crate) version: String,
    pub(crate) tag: String,
    pub(crate) commit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Admission {
    pub(crate) role: Role,
    pub(crate) workspace: PathBuf,
    pub(crate) key: Option<String>,
    pub(crate) schema: Option<u32>,
    pub(crate) file_id: Option<(u64, u64)>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Transition {
    pub(crate) before: Build,
    pub(crate) target: Build,
    pub(crate) admission: Admission,
    pub(crate) returning_to_stable: bool,
    pub(crate) initialization_key: Option<String>,
    #[serde(default)]
    pub(crate) previous_resumed: bool,
}

impl Transition {
    pub(crate) fn new(
        before: Build,
        target: Build,
        admission: Admission,
        returning_to_stable: bool,
    ) -> Result<Self, String> {
        let initialization_key = if admission.key.is_none() {
            let mut id = [0u8; 16];
            getrandom::fill(&mut id).map_err(display)?;
            Some(format!("{:032x}", u128::from_be_bytes(id)))
        } else {
            None
        };
        Ok(Self {
            before,
            target,
            admission,
            returning_to_stable,
            initialization_key,
            previous_resumed: false,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    format: u8,
    revision: u64,
    pub(crate) active: Role,
    pub(crate) returning: Option<ReturnChoice>,
    pub(crate) pending: Option<Transition>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: 1,
            revision: 0,
            active: Role::Primary,
            returning: None,
            pending: None,
        }
    }
}

pub(crate) struct Route(Mutex<Result<Option<PathBuf>, String>>);

impl Default for Route {
    fn default() -> Self {
        Self(Mutex::new(Ok(None)))
    }
}

impl Route {
    pub(crate) fn index(&self) -> Result<Option<PathBuf>, String> {
        lock(&self.0).clone()
    }
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(home) = std::env::var_os("ATLAS_PILOT_HOME") {
        return Ok(PathBuf::from(home).join("desktop-workspace.json"));
    }
    Ok(app
        .path()
        .app_config_dir()
        .map_err(display)?
        .join("desktop-workspace.json"))
}

fn read(path: &Path) -> Result<Config, String> {
    let config: Config = match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(display)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(err) => return Err(display(err)),
    };
    if config.format != 1 {
        return Err("This Atlas build cannot read the workspace selection.".into());
    }
    Ok(config)
}

pub(crate) fn load(app: &AppHandle) -> Result<Config, String> {
    read(&path(app)?)
}

fn lock_file(path: &Path) -> Result<File, String> {
    let parent = path.parent().ok_or("Atlas config has no directory.")?;
    fs::create_dir_all(parent).map_err(display)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path.with_extension("lock"))
        .map_err(display)?;
    file.try_lock()
        .map_err(|_| "Another Atlas process is changing the workspace selection.")?;
    Ok(file)
}

fn persist(path: &Path, config: &Config) -> Result<(), String> {
    let parent = path.parent().ok_or("Atlas config has no directory.")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(display)?;
    serde_json::to_writer(&mut staged, config).map_err(display)?;
    staged.flush().map_err(display)?;
    staged.as_file().sync_all().map_err(display)?;
    staged.persist(path).map_err(display)?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(display)?;
    Ok(())
}

pub(crate) fn save(app: &AppHandle, config: &mut Config) -> Result<(), String> {
    save_at(&path(app)?, config)
}

fn save_at(path: &Path, config: &mut Config) -> Result<(), String> {
    let _guard = lock_file(path)?;
    if read(path)?.revision != config.revision {
        return Err("The workspace selection changed. Open Updates and try again.".into());
    }
    config.revision = config
        .revision
        .checked_add(1)
        .ok_or("Atlas config revision exhausted.")?;
    persist(path, config)
}

fn startup(app: &AppHandle, installed: Option<&Build>) -> Result<Option<PathBuf>, String> {
    let mut config = load(app)?;
    if config.pending.is_none()
        && config.returning.is_some()
        && installed.is_some_and(|build| build.tag.starts_with("release/"))
    {
        config.returning = None;
        save(app, &mut config)?;
    }
    if config.pending.is_none() && config.active == Role::Primary {
        return Ok(None);
    }
    let ledger: Value = process::migration_ledger()?;
    if let Some(mut pending) = config.pending.clone() {
        if installed == Some(&pending.target) {
            if let Some(key) = &pending.initialization_key {
                let claimed = process::claim_transition(&pending.admission, &ledger, key)?;
                if claimed.workspace != pending.admission.workspace {
                    return Err("The selected workspace path changed before initialization.".into());
                }
                pending.admission = claimed;
                pending.initialization_key = None;
                config.pending = Some(pending.clone());
                save(app, &mut config)?;
            }
            let reservation = process::reserve_transition(pending.admission.role, &ledger)?;
            if reservation.admission.workspace != pending.admission.workspace
                || reservation.admission.key != pending.admission.key
                || reservation.admission.file_id != pending.admission.file_id
            {
                return Err("The workspace changed while Atlas was installing. Choose which workspace to open.".into());
            }
            return Ok(Some(reservation.admission.workspace.clone()));
        }
        if installed != Some(&pending.before) {
            return Err("Atlas found an unfinished switch from a different build. Choose which workspace to open.".into());
        }
        if !pending.previous_resumed {
            return Err("A channel switch is still pending. Close the installer before resuming your previous workspace.".into());
        }
    }
    let admission = process::inspect_transition(config.active, &ledger, None)?;
    Ok((config.active == Role::Trial).then_some(admission.workspace))
}

pub(crate) struct PendingLaunch {
    config: Config,
    _reservation: process::TransitionReservation,
}

impl PendingLaunch {
    pub(crate) fn finish(mut self, app: &AppHandle) -> Result<(), String> {
        save(app, &mut self.config)
    }
}

pub(crate) fn reserve_pending_launch(
    app: &AppHandle,
    probe: &Path,
) -> Result<Option<PendingLaunch>, String> {
    let mut config = load(app)?;
    let Some(pending) = config.pending.clone() else {
        return Ok(None);
    };
    let installed = crate::updates::installed_build();
    if installed.as_ref() == Some(&pending.before) && pending.previous_resumed {
        return Ok(None);
    }
    if installed.as_ref() != Some(&pending.target) {
        return Err("This Atlas build does not match the pending switch.".into());
    }
    let ledger = process::migration_ledger()?;
    let reservation = process::reserve_transition(pending.admission.role, &ledger)?;
    let actual = &reservation.admission;
    if actual.workspace != probe
        || actual.workspace != pending.admission.workspace
        || (pending.admission.key.is_some()
            && (actual.key != pending.admission.key || actual.file_id != pending.admission.file_id))
    {
        return Err("The workspace changed before Atlas could finish switching.".into());
    }
    config.active = pending.admission.role;
    config.pending = None;
    if pending.returning_to_stable {
        config.returning = None;
    }
    Ok(Some(PendingLaunch {
        config,
        _reservation: reservation,
    }))
}

pub(crate) fn recovery_release(app: &AppHandle) -> Option<String> {
    let pending = load(app).ok()?.pending?;
    Some(format!(
        "If preparation cannot finish, reinstall Atlas {} from https://github.com/e-kosten/pf2e-atlas/releases/tag/{} to return to the previous selection.",
        pending.before.version, pending.before.tag
    ))
}

pub(crate) fn initialize(app: &AppHandle, installed: Option<&Build>) {
    *lock(&app.state::<Desktop>().workspace_route.0) = startup(app, installed);
}

pub(crate) fn recover(app: &AppHandle, previous: bool) -> Result<(), String> {
    let path = path(app)?;
    if previous {
        let mut config = read(&path)?;
        let installed = crate::updates::installed_build();
        if !config
            .pending
            .as_ref()
            .is_some_and(|pending| installed.as_ref() == Some(&pending.before))
        {
            return Err("Reinstall the previous Atlas build before resuming its selection.".into());
        }
        let ledger = process::migration_ledger()?;
        let reservation = process::reserve_transition(config.active, &ledger)?;
        if let Some(pending) = &mut config.pending {
            pending.previous_resumed = true;
        }
        save_at(&path, &mut config)?;
        *lock(&app.state::<Desktop>().workspace_route.0) =
            Ok((config.active == Role::Trial).then(|| reservation.admission.workspace.clone()));
        crate::updates::refresh(app);
        return Ok(());
    }
    if can_resume_previous(app) {
        return Err("Resume the previous selection after closing the installer. The pending selection is kept for the target build.".into());
    }
    let ledger = process::migration_ledger()?;
    let _reservation = process::reserve_transition(Role::Primary, &ledger)?;
    let _guard = lock_file(&path)?;
    let mut config = read(&path).unwrap_or_default();
    config.active = Role::Primary;
    config.pending = None;
    config.returning = None;
    config.revision = config
        .revision
        .checked_add(1)
        .ok_or("Atlas config revision exhausted.")?;
    persist(&path, &config)?;
    *lock(&app.state::<Desktop>().workspace_route.0) = Ok(None);
    crate::updates::refresh(app);
    Ok(())
}

pub(crate) fn can_resume_previous(app: &AppHandle) -> bool {
    let installed = crate::updates::installed_build();
    load(app).is_ok_and(|config| {
        config
            .pending
            .is_some_and(|pending| installed.as_ref() == Some(&pending.before))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_on_stable_preserves_independent_workspace_choices_and_rejects_stale_writes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("selection.json");
        let mut config = read(&path).unwrap();
        let mut stale = config.clone();
        config.active = Role::Trial;
        config.returning = Some(ReturnChoice {
            workspace: Role::Primary,
            updates: Waiting::Hold,
        });
        save_at(&path, &mut config).unwrap();
        assert_eq!(read(&path).unwrap(), config);
        assert!(save_at(&path, &mut stale).is_err());

        config.returning.as_mut().unwrap().updates = Waiting::Continue;
        save_at(&path, &mut config).unwrap();
        let restored = read(&path).unwrap();
        assert_eq!(restored.active, Role::Trial);
        assert_eq!(
            restored.returning,
            Some(ReturnChoice {
                workspace: Role::Primary,
                updates: Waiting::Continue
            })
        );
    }
}
