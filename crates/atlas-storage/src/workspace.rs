use crate::locking::acquire;
use crate::{ReadSession, StorageError, StorageResult, UpdateOwner, WriteSession};
use rusqlite::{Connection, OpenFlags};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

// Evaluation ceiling, not an eager/private allocation. Each process admits at
// most two mapped read sessions, including callers outside retrieval workers.
const MAX_READ_MAPPING_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_MAPPED_READERS: usize = 2;
static READ_MAPPINGS: MappingBudget = MappingBudget(AtomicUsize::new(0));

struct MappingBudget(AtomicUsize);
impl MappingBudget {
    fn acquire(&self) -> Option<MappingLease<'_>> {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |active| {
                (active < MAX_MAPPED_READERS).then_some(active + 1)
            })
            .ok()
            .map(|_| MappingLease(self))
    }
}

pub(crate) struct MappingLease<'a>(&'a MappingBudget);
impl Drop for MappingLease<'_> {
    fn drop(&mut self) {
        self.0.0.fetch_sub(1, Ordering::Relaxed);
    }
}

fn read_mapping_bytes(file_bytes: u64, pointer_bits: u32) -> u64 {
    if pointer_bits < 64 {
        0
    } else {
        file_bytes.min(MAX_READ_MAPPING_BYTES)
    }
}

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
        // Do not retain a reader or wait for mapping capacity. SQLite owns
        // platform clamping, xFetch fallback, WAL coherence and actual residency.
        let bytes = read_mapping_bytes(
            std::fs::metadata(&self.path).map_or(0, |file| file.len()),
            usize::BITS,
        );
        let mapping = if bytes > 0 {
            READ_MAPPINGS.acquire()
        } else {
            None
        };
        let session = ReadSession {
            connection,
            _mapping: mapping,
            _admission: admission,
            path: self.path.clone(),
        };
        // Construct the owner before any fallible mapping/admission operation:
        // failures must close the connection before returning mapping capacity.
        if session._mapping.is_some() {
            session
                .connection
                .pragma_update(Some("main"), "mmap_size", bytes)?;
        }
        // SQLITE_OPEN_READ_ONLY still permits connection-local validation TEMP tables.
        session.connection.execute_batch("BEGIN DEFERRED;")?;
        validate(&session.connection, schema, false)?;
        Ok(session)
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
        #[cfg(feature = "diagnostics")]
        crate::diagnostics::attach(&connection);
        connection.busy_timeout(Duration::from_secs(5))?;
        // A default 768-dimensional vec0 chunk exceeds SQLite's usual 2 MiB
        // cache. Keep a chunk and nearby metadata resident during sequential
        // publication/validation. This is a demand-filled per-session target,
        // not an eager allocation; sessions remain bounded by their callers.
        connection.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA cache_size=-8192; PRAGMA main.mmap_size=0;",
        )?;
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

#[cfg(test)]
mod mapping_tests {
    use super::*;

    #[test]
    fn address_limit_is_finite_and_disabled_on_small_address_spaces() {
        assert_eq!(read_mapping_bytes(u64::MAX, 32), 0);
        assert_eq!(read_mapping_bytes(0, 64), 0);
        assert_eq!(read_mapping_bytes(4096, 64), 4096);
        assert_eq!(read_mapping_bytes(u64::MAX, 64), MAX_READ_MAPPING_BYTES);
    }

    #[test]
    fn capacity_is_nonblocking_and_released_after_drop_and_unwind() {
        let budget = MappingBudget(AtomicUsize::new(0));
        let first = budget.acquire().unwrap();
        let second = budget.acquire().unwrap();
        assert!(budget.acquire().is_none());
        drop(first);
        let replacement = budget.acquire().unwrap();
        assert!(budget.acquire().is_none());
        drop((second, replacement));
        let result = std::panic::catch_unwind(|| {
            let _lease = budget.acquire().unwrap();
            panic!("abandoned read");
        });
        assert!(result.is_err());
        assert_eq!(budget.0.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn concurrent_callers_cannot_multiply_mapping_capacity() {
        let budget = MappingBudget(AtomicUsize::new(0));
        let barrier = std::sync::Barrier::new(16);
        let admitted = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..16 {
                scope.spawn(|| {
                    let lease = budget.acquire();
                    if lease.is_some() {
                        admitted.fetch_add(1, Ordering::Relaxed);
                    }
                    barrier.wait();
                    assert_eq!(budget.0.load(Ordering::Relaxed), MAX_MAPPED_READERS);
                    barrier.wait();
                    drop(lease);
                });
            }
        });
        assert_eq!(admitted.load(Ordering::Relaxed), MAX_MAPPED_READERS);
        assert_eq!(budget.0.load(Ordering::Relaxed), 0);
    }
}
