//! Admission checks shared by desktop setup commands on every CLI platform.
use atlas_desktop_protocol::workspace_selection::Role;
use atlas_runtime::Workspace;
use std::{io, path::PathBuf};

#[cfg(unix)]
pub(crate) fn file_identity(path: &std::path::Path) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = path.metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
pub(crate) fn file_identity(path: &std::path::Path) -> io::Result<(u64, u64)> {
    let information = winapi_util::file::information(std::fs::File::open(path)?)?;
    Ok((information.volume_serial_number(), information.file_index()))
}

pub(crate) fn verify_expected_workspace(workspace: &Workspace) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct Expected {
        workspace: PathBuf,
        key: Option<String>,
        file_id: Option<(u64, u64)>,
    }
    let Ok(expected) = std::env::var("ATLAS_DESKTOP_EXPECTED_WORKSPACE") else {
        return Ok(());
    };
    let expected: Expected = serde_json::from_str(&expected).map_err(|error| error.to_string())?;
    let current = workspace.path();
    let changed = || {
        "The selected workspace changed after the switch. Choose which workspace to open."
            .to_owned()
    };
    if current != expected.workspace {
        return Err(changed());
    }
    match (expected.key, expected.file_id) {
        (Some(key), Some(identity)) => {
            let ledger = atlas_runtime::desktop_migration_ledger();
            let (actual_key, _) = workspace
                .check_update_history(&ledger)
                .map_err(|error| error.to_string())?;
            if actual_key != key
                || file_identity(current).map_err(|error| error.to_string())? != identity
            {
                return Err(changed());
            }
        }
        (None, None) => match std::fs::symlink_metadata(current) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(changed()),
        },
        _ => return Err(changed()),
    }
    Ok(())
}

pub(crate) fn is_managed_index(index: &std::path::Path) -> bool {
    [Role::Original, Role::Separate]
        .iter()
        .filter_map(|role| atlas_runtime::managed_workspace_path(*role).ok())
        .any(|expected| {
            expected == index || expected.canonicalize().is_ok_and(|path| path == index)
        })
}
