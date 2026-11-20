//! Shared selection for managed desktop and global CLI workspaces.
use crate::{RuntimeError, paths::platform_data_root};
use atlas_desktop_protocol::workspace_selection::{Config, Role};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub fn managed_workspace_path(role: Role) -> Result<PathBuf, RuntimeError> {
    Ok(platform_data_root()?.join("pf2e-atlas").join(match role {
        Role::Original => "workspace.sqlite",
        Role::Separate => "separate-workspace.sqlite",
    }))
}

fn selection_path() -> Result<PathBuf, String> {
    Ok(platform_data_root()
        .map_err(display)?
        .join("pf2e-atlas")
        .join("workspace-selection.json"))
}

pub(crate) fn selected_workspace(root: &Path) -> Result<PathBuf, RuntimeError> {
    let selection = root.join("workspace-selection.json");
    let existing = read_existing(&selection).map_err(RuntimeError::workspace_selection)?;
    let committed = existing.is_some();
    let config = existing.unwrap_or_default();
    let selected = root.join(match config.active {
        Role::Original => "workspace.sqlite",
        Role::Separate => "separate-workspace.sqlite",
    });
    // An absent configuration is the ordinary first-run path. A committed
    // selection must never silently recreate a deleted campaign.
    if committed && !selected.is_file() {
        return Err(RuntimeError::workspace_selection(
            "The selected workspace is missing. Open Atlas recovery or pass an explicit --index path.",
        ));
    }
    Ok(selected)
}

pub fn recover_original_workspace_selection() -> Result<Config, String> {
    let path = selection_path()?;
    let _guard = lock_file(&path)?;
    let mut config = read(&path).unwrap_or_default();
    config.active = Role::Original;
    config.pending = None;
    config.returning = None;
    config.revision = config
        .revision
        .checked_add(1)
        .ok_or("Atlas config revision exhausted.")?;
    persist(&path, &config)?;
    Ok(config)
}

fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn read(path: &Path) -> Result<Config, String> {
    Ok(read_existing(path)?.unwrap_or_default())
}

fn read_existing(path: &Path) -> Result<Option<Config>, String> {
    let mut bytes = Vec::new();
    match File::open(path) {
        Ok(file) => file.take(65_537).read_to_end(&mut bytes).map_err(display)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return match fs::symlink_metadata(path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
                _ => Err("The workspace selection file cannot be read.".into()),
            };
        }
        Err(err) => return Err(display(err)),
    };
    if bytes.len() > 65_536 {
        return Err("The workspace selection is too large.".into());
    }
    let config: Config = serde_json::from_slice(&bytes).map_err(display)?;
    if config.format != 2 {
        return Err("This Atlas build cannot read the workspace selection.".into());
    }
    Ok(Some(config))
}

pub fn load_workspace_selection() -> Result<Config, String> {
    read(&selection_path()?)
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

pub fn save_workspace_selection(config: &mut Config) -> Result<(), String> {
    save_at(&selection_path()?, config)
}

fn save_at(path: &Path, config: &mut Config) -> Result<(), String> {
    if config.format != 2 {
        return Err("This Atlas build cannot save that workspace selection.".into());
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_desktop_protocol::workspace_selection::{ReturnChoice, Waiting};

    #[test]
    fn shared_selection_survives_restart_and_refuses_stale_writes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace-selection.json");
        assert_eq!(
            selected_workspace(directory.path()).unwrap(),
            directory.path().join("workspace.sqlite")
        );
        let mut config = read(&path).unwrap();
        let mut stale = config.clone();
        let separate = directory.path().join("separate-workspace.sqlite");
        File::create(&separate).unwrap();
        config.active = Role::Separate;
        config.returning = Some(ReturnChoice {
            workspace: Role::Original,
            updates: Waiting::Hold,
        });
        save_at(&path, &mut config).unwrap();
        assert_eq!(read(&path).unwrap(), config);
        assert_eq!(selected_workspace(directory.path()).unwrap(), separate);
        assert!(save_at(&path, &mut stale).is_err());
        assert_eq!(read(&path).unwrap(), config);
        fs::remove_file(&separate).unwrap();
        assert!(selected_workspace(directory.path()).is_err());
        assert!(!directory.path().join("workspace.sqlite").exists());
    }

    #[test]
    fn unreadable_selection_never_falls_back_to_original() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace-selection.json");
        File::create(directory.path().join("workspace.sqlite")).unwrap();
        fs::write(&path, b"not json").unwrap();
        assert!(selected_workspace(directory.path()).is_err());
        let config = Config {
            format: 99,
            ..Config::default()
        };
        persist(&path, &config).unwrap();
        assert!(selected_workspace(directory.path()).is_err());
        #[cfg(unix)]
        {
            fs::remove_file(&path).unwrap();
            std::os::unix::fs::symlink(directory.path().join("deleted-config.json"), &path)
                .unwrap();
            assert!(selected_workspace(directory.path()).is_err());
        }
    }
}
