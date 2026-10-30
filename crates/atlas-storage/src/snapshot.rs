//! Consistent whole-workspace snapshots and missing-only seed installation.
use crate::backup::{self, Destination, sync_directory};
use crate::{StorageError, StorageResult, Workspace, locking, workspace::validate};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

/// Holds both admission locks until the caller finishes recording seed provenance.
pub struct WorkspaceInitialization {
    path: PathBuf,
    _update: File,
    _access: File,
}

impl Workspace {
    /// Capture committed WAL contents without blocking ordinary saved-state writes.
    /// The destination must be absent. No domain migrations are performed.
    pub fn snapshot_to(&self, destination: &Path) -> StorageResult<()> {
        let _update = locking::acquire(&self.path, ".update.lock", false)?;
        let _access = locking::acquire(&self.path, ".access.lock", false)?;
        let source = self.connect(false, false)?;
        source.execute_batch("BEGIN DEFERRED")?;
        let schema = source.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        validate(&source, schema, false)?;
        backup::write(&source, destination, Destination::CreateNew)
    }

    /// Reserve an absent destination. Existing files and dangling symlinks are
    /// preserved; orphan SQLite sidecars require explicit recovery.
    pub fn begin_initialization(&self) -> StorageResult<Option<WorkspaceInitialization>> {
        let update = locking::acquire(&self.path, ".update.lock", true)?;
        let access = locking::acquire(&self.path, ".access.lock", true)?;
        if self.path.symlink_metadata().is_ok() {
            return Ok(None);
        }
        for suffix in ["-wal", "-shm", "-journal"] {
            if locking::sidecar(&self.path, suffix)
                .symlink_metadata()
                .is_ok()
            {
                return Err(StorageError::Unsupported(
                    "SQLite sidecars exist; recover the workspace before initialization".into(),
                ));
            }
        }
        Ok(Some(WorkspaceInitialization {
            path: self.path.clone(),
            _update: update,
            _access: access,
        }))
    }
}

impl WorkspaceInitialization {
    /// Clone a closed, immutable seed when available, otherwise copy it. The
    /// callback verifies the staged bytes before the missing-only installation.
    pub fn install(
        &self,
        source: &Path,
        verify: impl FnOnce(&Path) -> StorageResult<()>,
    ) -> StorageResult<&'static str> {
        let parent = self.path.parent().unwrap_or(Path::new("."));
        let staging = tempfile::tempdir_in(parent)?;
        let staged = staging.path().join("workspace.sqlite");
        let method = copy_seed(source, &staged)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&staged, fs::Permissions::from_mode(0o600))?;
        }
        verify(&staged)?;
        File::options().write(true).open(&staged)?.sync_all()?;
        // Same filesystem; hard-link creation atomically refuses an existing name.
        // The staging name disappears when the temporary directory is dropped.
        fs::hard_link(&staged, &self.path)?;
        sync_directory(parent)?;
        Ok(method)
    }
}

fn copy_seed(source: &Path, destination: &Path) -> StorageResult<&'static str> {
    #[cfg(target_os = "macos")]
    {
        let result = std::process::Command::new("/bin/cp")
            .arg("-c")
            .arg(source)
            .arg(destination)
            .output()?;
        if result.status.success() {
            return Ok("filesystem-clone");
        }
        if destination.exists() {
            fs::remove_file(destination)?;
        }
    }
    fs::copy(source, destination)?;
    Ok("copy")
}
