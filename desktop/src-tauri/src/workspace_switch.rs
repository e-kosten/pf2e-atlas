use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{Desktop, display, lock, process};

pub(crate) use atlas_desktop_protocol::workspace_selection::{
    Admission, Build, Config, ReturnChoice, Role, Transition, Waiting,
};

pub(crate) fn new_transition(
    before: Build,
    target: Build,
    admission: Admission,
    returning_to_stable: bool,
) -> Result<Transition, String> {
    let initialization_key = if admission.key.is_none() {
        let mut id = [0u8; 16];
        getrandom::fill(&mut id).map_err(display)?;
        Some(format!("{:032x}", u128::from_be_bytes(id)))
    } else {
        None
    };
    Ok(Transition {
        before,
        target,
        admission,
        returning_to_stable,
        initialization_key,
        previous_resumed: false,
    })
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

pub(crate) fn load(_app: &AppHandle) -> Result<Config, String> {
    process::load_workspace_selection()
}
pub(crate) fn save(_app: &AppHandle, config: &mut Config) -> Result<(), String> {
    process::save_workspace_selection(config)
}

pub(crate) const CHANGED_WORKSPACE: &str = "The default workspace changed in another Atlas app. Close and reopen this app before changing channels or installing updates.";

pub(crate) fn running_role(
    pinned: &Path,
    selection: &atlas_desktop_protocol::workspace_selection::SelectionSnapshot,
) -> Result<Role, String> {
    let mut matching = selection
        .workspaces
        .iter()
        .filter(|workspace| workspace.path == pinned);
    let role = matching
        .next()
        .ok_or("The running Atlas workspace is not a managed workspace.")?
        .role;
    if matching.next().is_some() {
        return Err("The two managed workspaces point to the same file.".into());
    }
    Ok(role)
}

pub(crate) fn load_current(app: &AppHandle) -> Result<Config, String> {
    let selection = process::workspace_selection_snapshot()?;
    let pinned = app
        .state::<Desktop>()
        .workspace_route
        .index()?
        .ok_or("Atlas has no running workspace.")?;
    if running_role(&pinned, &selection)? != selection.config.active {
        return Err(CHANGED_WORKSPACE.into());
    }
    Ok(selection.config)
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
    Ok(Some(admission.workspace))
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
    if previous {
        let mut config = load(app)?;
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
        save(app, &mut config)?;
        *lock(&app.state::<Desktop>().workspace_route.0) =
            Ok(Some(reservation.admission.workspace.clone()));
        crate::updates::refresh(app);
        return Ok(());
    }
    if can_resume_previous(app) {
        return Err("Resume the previous selection after closing the installer. The pending selection is kept for the target build.".into());
    }
    let ledger = process::migration_ledger()?;
    let reservation = process::reserve_transition(Role::Original, &ledger)?;
    process::recover_original_workspace_selection()?;
    *lock(&app.state::<Desktop>().workspace_route.0) =
        Ok(Some(reservation.admission.workspace.clone()));
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
    use atlas_desktop_protocol::workspace_selection::{ManagedWorkspaceInfo, SelectionSnapshot};

    #[test]
    fn running_workspace_is_pinned_when_another_app_changes_the_default() {
        let original = PathBuf::from("/Atlas/workspace.sqlite");
        let separate = PathBuf::from("/Atlas/separate-workspace.sqlite");
        let selection = SelectionSnapshot {
            config: Config {
                active: Role::Separate,
                ..Config::default()
            },
            workspaces: vec![
                ManagedWorkspaceInfo {
                    role: Role::Original,
                    path: original.clone(),
                    exists: true,
                },
                ManagedWorkspaceInfo {
                    role: Role::Separate,
                    path: separate,
                    exists: true,
                },
            ],
        };
        assert_eq!(running_role(&original, &selection).unwrap(), Role::Original);
        assert_ne!(
            running_role(&original, &selection).unwrap(),
            selection.config.active
        );
        assert!(running_role(Path::new("/Atlas/unknown.sqlite"), &selection).is_err());
    }
}
