use crate::locking::acquire;
use crate::{ReadSession, StorageError, StorageResult, UpdateOwner, WriteSession};
use rusqlite::{Connection, OpenFlags};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub(crate) const APPLICATION_ID: u32 = 0x41544c53;

#[derive(Clone, Debug)]
pub struct Workspace {
    pub(crate) path: PathBuf,
}

impl Workspace {
    /// Resolve existing symlinks so all cooperating callers use the same lock files.
    /// This does not create a database or silently initialize an unknown file.
    pub fn at(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref();
        let path = if path.exists() {
            path.canonicalize()?
        } else {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            std::fs::create_dir_all(parent)?;
            let name = path.file_name().ok_or_else(|| {
                StorageError::Unsupported("workspace path has no filename".into())
            })?;
            parent.canonicalize()?.join(name)
        };
        Ok(Self { path })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read(&self, schema: u32) -> StorageResult<ReadSession> {
        let admission = acquire(&self.path, ".access.lock", false)?;
        let connection = self.connect(false, false)?;
        // SQLITE_OPEN_READ_ONLY protects persistent state. Validation may still
        // create connection-local TEMP virtual tables (for FTS vocabulary).
        connection.execute_batch("BEGIN DEFERRED;")?;
        validate(&connection, schema, false)?;
        Ok(ReadSession {
            connection,
            _admission: admission,
            path: self.path.clone(),
        })
    }

    pub fn write(&self, schema: u32) -> StorageResult<WriteSession> {
        let admission = acquire(&self.path, ".access.lock", false)?;
        let connection = self.connect(true, false)?;
        connection.execute_batch("BEGIN IMMEDIATE;")?;
        validate(&connection, schema, false)?;
        Ok(WriteSession {
            connection,
            _admission: admission,
        })
    }

    /// Commit a complete composed action only after every domain operation succeeds.
    /// Errors and unwinding abandon the transaction, including earlier writes.
    pub fn write_action<T, E>(
        &self,
        schema: u32,
        action: impl FnOnce(&WriteSession) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<StorageError>,
    {
        let session = self.write(schema)?;
        let result = action(&session)?;
        session.commit()?;
        Ok(result)
    }

    /// Serializes source/config/model publishers without holding a SQLite lock.
    /// Ordinary reads and saved-state edits can continue until maintenance begins.
    pub fn update_owner(&self) -> StorageResult<UpdateOwner> {
        let lock = acquire(&self.path, ".update.lock", true)?;
        Ok(UpdateOwner {
            workspace: self.clone(),
            _lock: lock,
        })
    }

    pub(crate) fn connect(&self, writable: bool, create: bool) -> StorageResult<Connection> {
        let mut flags = if writable {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        };
        if create {
            flags |= OpenFlags::SQLITE_OPEN_CREATE;
        }
        let connection = Connection::open_with_flags(&self.path, flags)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        // A default 768-dimensional vec0 chunk exceeds SQLite's usual 2 MiB
        // cache. Keep a chunk and nearby metadata resident during sequential
        // publication/validation. This is a demand-filled per-session target,
        // not an eager allocation; sessions remain bounded by their callers.
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA cache_size=-8192;")?;
        if writable {
            connection.execute_batch("PRAGMA synchronous=FULL;")?;
        }
        Ok(connection)
    }
}

pub(crate) fn validate(
    connection: &Connection,
    schema: u32,
    maintenance: bool,
) -> StorageResult<()> {
    let app: u32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    if app != APPLICATION_ID {
        return Err(StorageError::Unsupported(
            "not an Atlas workspace; existing pre-release artifacts require explicit replacement"
                .into(),
        ));
    }
    let actual: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if actual != schema {
        return Err(StorageError::Schema {
            expected: schema,
            actual,
        });
    }
    let reason: Option<String> =
        connection.query_row("SELECT maintenance FROM workspace WHERE id=1", [], |r| {
            r.get(0)
        })?;
    if !maintenance && let Some(reason) = reason {
        return Err(StorageError::Maintenance(reason));
    }
    Ok(())
}
