//! Consistent whole-workspace snapshots and missing-only seed installation.
use crate::backup::{self, Destination, sync_directory};
use crate::{
    MigrationHistory, StorageError, StorageResult, Workspace, locking, workspace::validate,
};
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

    /// Publish an independent copy of committed workspace data. An existing
    /// destination always wins, even if it appeared during snapshot capture.
    pub fn seed_copy(
        &self,
        destination: &Self,
        history: MigrationHistory<'_>,
        expected_key: &str,
        expected_schema: u32,
        verify_source: impl FnOnce() -> StorageResult<()>,
    ) -> StorageResult<()> {
        if self.path == destination.path {
            return Err(StorageError::Unsupported(
                "copy resolves to its source workspace".into(),
            ));
        }
        history.validate()?;
        let parent = destination.path.parent().unwrap_or_else(|| Path::new("."));
        let scratch = tempfile::tempdir_in(parent)?;
        let snapshot = scratch.path().join("workspace.sqlite");
        self.snapshot_to(&snapshot)?;
        verify_source()?;

        let initialization = destination.begin_initialization()?.ok_or_else(|| {
            StorageError::Unsupported("destination workspace already exists".into())
        })?;
        initialization.install(&snapshot, |staged| {
            let mut connection = rusqlite::Connection::open(staged)?;
            connection.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")?;
            let transaction = connection.transaction()?;
            let (fresh, schema) = crate::migration::inspect_history(&transaction, history)?;
            if fresh {
                return Err(StorageError::Unsupported(
                    "source workspace is uninitialized".into(),
                ));
            }
            validate(&transaction, schema, false)?;
            let actual: String = transaction.query_row(
                "SELECT workspace_key FROM workspace WHERE id=1",
                [],
                |row| row.get(0),
            )?;
            if schema != expected_schema || actual != expected_key {
                return Err(StorageError::Migration(
                    "source workspace changed before copying".into(),
                ));
            }
            transaction.execute(
                "UPDATE workspace SET workspace_key=lower(hex(randomblob(16))) WHERE id=1",
                [],
            )?;
            transaction.commit()?;
            Ok(())
        })?;
        Ok(())
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
    /// Stamp a newly built, private database with the identity recorded before
    /// install. Publishing only to an absent path lets recovery distinguish the
    /// initializer from an unrelated database created at the same location.
    pub fn install_claimed(
        &self,
        source: &Path,
        history: MigrationHistory<'_>,
        key: &str,
    ) -> StorageResult<()> {
        self.install(source, |path| {
            let mut connection = rusqlite::Connection::open(path)?;
            connection.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")?;
            let transaction = connection.transaction()?;
            crate::migration::inspect_history(&transaction, history)?;
            validate(&transaction, history.target(), false)?;
            transaction.execute("UPDATE workspace SET workspace_key=?1 WHERE id=1", [key])?;
            transaction.commit()?;
            Ok(())
        })?;
        Ok(())
    }

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
