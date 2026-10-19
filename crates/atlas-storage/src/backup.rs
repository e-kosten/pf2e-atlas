//! Shared SQLite capture and durable file publication. Callers own admission.
use crate::StorageResult;
use rusqlite::{Connection, backup::Backup};
use std::{path::Path, time::Duration};

pub(crate) enum Destination {
    CreateNew,
    Replace,
}

pub(crate) fn write(
    source: &Connection,
    destination: &Path,
    policy: Destination,
) -> StorageResult<()> {
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let staged = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut target = Connection::open(staged.path())?;
        Backup::new(source, &mut target)?.run_to_completion(
            4096,
            Duration::from_millis(1),
            None,
        )?;
        // Both migration backups and developer seeds must be standalone files.
        target.execute_batch("PRAGMA journal_mode=DELETE")?;
    }
    staged.as_file().sync_all()?;
    match policy {
        Destination::CreateNew => staged.persist_noclobber(destination),
        Destination::Replace => staged.persist(destination),
    }
    .map_err(|error| error.error)?;
    sync_directory(parent)
}

pub(crate) fn sync_directory(path: &Path) -> StorageResult<()> {
    #[cfg(unix)]
    std::fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
