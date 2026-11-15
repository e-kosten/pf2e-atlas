use crate::{
    StorageError, StorageResult, Workspace,
    backup::{self, Destination},
    locking::{acquire, sidecar},
    workspace::APPLICATION_ID,
};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::path::PathBuf;

/// Typed domain backfill, executed within the migration transaction.
/// Change the immutable identifier whenever the executable backfill changes.
#[derive(Clone, Copy)]
pub struct DataBackfill {
    /// Suspend foreign keys before BEGIN for the entire pending migration batch.
    /// This includes SQL and callbacks that do not request reconstruction themselves.
    /// Every pending migration must be safe without foreign-key actions: final
    /// validation detects dangling keys but cannot reproduce skipped cascades.
    /// Enforcement is restored after the batch transaction closes.
    pub reconstruct_tables: bool,
    pub id: &'static str,
    pub apply: fn(&Connection) -> StorageResult<()>,
}

/// Ordered executable-owned SQL and optional typed backfill. Never load code or
/// migration SQL from a workspace.
#[derive(Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
    pub backfill: Option<DataBackfill>,
}
impl Migration {
    fn canonical_sql(&self) -> Cow<'_, str> {
        if self.sql.contains("\r\n") {
            Cow::Owned(self.sql.replace("\r\n", "\n"))
        } else {
            Cow::Borrowed(self.sql)
        }
    }

    fn checksum(&self) -> String {
        let mut hash = Sha256::new();
        hash.update(self.canonical_sql().as_bytes());
        if let Some(backfill) = self.backfill {
            hash.update(b"\0atlas-data-backfill/v1\0");
            hash.update(backfill.id.as_bytes());
            if backfill.reconstruct_tables {
                hash.update(b"\0atlas-table-reconstruction/v1\0");
            }
        }
        format!("{hash:x}", hash = hash.finalize())
    }
}

/// Immutable checksum inputs from a retired baseline cutover.
#[derive(Clone, Copy)]
pub struct BaselineProvenance {
    pub ledger_sha256: &'static str,
    pub reconstruct_tables: bool,
    pub bridge_id: &'static str,
}

/// Executable-owned baseline and appended migrations.
#[derive(Clone, Copy)]
pub struct MigrationHistory<'a> {
    pub migrations: &'a [Migration],
    pub baseline_provenance: Option<BaselineProvenance>,
}

impl<'a> MigrationHistory<'a> {
    pub const fn new(migrations: &'a [Migration]) -> Self {
        Self {
            migrations,
            baseline_provenance: None,
        }
    }

    pub const fn target(self) -> u32 {
        match self.migrations.last() {
            Some(migration) => migration.version,
            None => 0,
        }
    }

    fn first(self) -> u32 {
        self.migrations
            .first()
            .map_or(1, |migration| migration.version)
    }

    pub(crate) fn validate(self) -> StorageResult<()> {
        let first = self.first();
        if first == 0
            || (first > 1) != self.baseline_provenance.is_some()
            || self
                .migrations
                .iter()
                .enumerate()
                .any(|(index, migration)| {
                    migration.version.checked_sub(first) != u32::try_from(index).ok()
                })
        {
            return Err(StorageError::Migration(
                "invalid executable migration history".into(),
            ));
        }
        Ok(())
    }

    fn checksum(self, migration: &Migration) -> String {
        let checksum = migration.checksum();
        let Some(provenance) = self
            .baseline_provenance
            .filter(|_| migration.version == self.first())
        else {
            return checksum;
        };
        let mut hash = Sha256::new();
        hash.update(b"atlas-baseline-cutover/v1\0");
        hash.update(APPLICATION_ID.to_be_bytes());
        hash.update(HEADER.as_bytes());
        hash.update(checksum.as_bytes());
        hash.update(provenance.ledger_sha256.as_bytes());
        hash.update([u8::from(provenance.reconstruct_tables)]);
        hash.update(provenance.bridge_id.as_bytes());
        format!("{hash:x}", hash = hash.finalize())
    }
}

const HEADER: &str = "
CREATE TABLE workspace(id INTEGER PRIMARY KEY CHECK(id=1),workspace_key TEXT NOT NULL UNIQUE,
reference_revision INTEGER NOT NULL CHECK(reference_revision>=0),maintenance TEXT) STRICT;
INSERT INTO workspace VALUES(1,lower(hex(randomblob(16))),0,NULL);
CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY,name TEXT NOT NULL,checksum TEXT NOT NULL,
applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP) STRICT;";

impl Workspace {
    /// One transaction for pending schema changes/backfills and ledger stamps.
    /// The validator runs on the exact upgraded transaction, or a read-only
    /// snapshot when the schema is already current. It must not mutate data.
    /// Existing workspaces get a consistent backup before any schema mutation.
    pub fn migrate(
        &self,
        migrations: &[Migration],
        validate: impl FnOnce(&Connection) -> StorageResult<()>,
    ) -> StorageResult<()> {
        self.migrate_history(MigrationHistory::new(migrations), validate)
    }

    pub fn migrate_history(
        &self,
        history: MigrationHistory<'_>,
        validate: impl FnOnce(&Connection) -> StorageResult<()>,
    ) -> StorageResult<()> {
        self.migrate_history_with_capture(
            history,
            |snapshot| backup::write(snapshot, &self.upgrade_backup_path(), Destination::Replace),
            validate,
        )
    }

    /// Capture recovery data from the authenticated, read-only pre-upgrade
    /// snapshot while holding exclusive access and update admission.
    pub fn migrate_history_with_capture(
        &self,
        history: MigrationHistory<'_>,
        capture: impl FnOnce(&Connection) -> StorageResult<()>,
        validate: impl FnOnce(&Connection) -> StorageResult<()>,
    ) -> StorageResult<()> {
        history.validate()?;
        let target = history.target();
        // A publisher owns the update lock throughout expensive preparation.
        // Ordinary process startup must still authenticate an already-current
        // workspace without becoming another publisher. Shared admission and a
        // SQLite snapshot keep this check coherent with publication/migration.
        if self.path.exists() {
            let _admission = acquire(&self.path, ".access.lock", false)?;
            let connection = self.connect(false, false)?;
            let snapshot = Transaction::new_unchecked(&connection, TransactionBehavior::Deferred)?;
            let (fresh, current) = inspect_history(&snapshot, history)?;
            if !fresh && current == target {
                validate_current(&snapshot, validate)?;
                return Ok(());
            }
        }
        // Drop shared admission before acquiring exclusive ownership. Recheck
        // history below because another process may have migrated in between.
        let _update = self.update_owner()?;
        let _admission = acquire(&self.path, ".access.lock", true)?;
        let connection = self.connect(true, true)?;
        let (fresh, current) = inspect_history(&connection, history)?;
        if !fresh && current == target {
            let snapshot = Transaction::new_unchecked(&connection, TransactionBehavior::Deferred)?;
            validate_current(&snapshot, validate)?;
            return Ok(());
        }
        let workspace_key: Option<String> = if fresh {
            None
        } else {
            let reader = self.connect(false, false)?;
            let snapshot = Transaction::new_unchecked(&reader, TransactionBehavior::Deferred)?;
            if inspect_history(&snapshot, history)? != (false, current) {
                return Err(StorageError::Migration(
                    "workspace history changed before upgrade capture".into(),
                ));
            }
            let key =
                snapshot.query_row("SELECT workspace_key FROM workspace WHERE id=1", [], |r| {
                    r.get(0)
                })?;
            capture(&snapshot)?;
            Some(key)
        };
        connection.execute_batch("PRAGMA journal_mode=WAL;")?;
        let pending = history
            .migrations
            .iter()
            .filter(|migration| migration.version > current);
        let reconstruction = pending.clone().any(|migration| {
            migration
                .backfill
                .is_some_and(|backfill| backfill.reconstruct_tables)
        });
        if reconstruction {
            connection.pragma_update(None, "foreign_keys", false)?;
        }
        let result = (|| -> StorageResult<()> {
            let transaction =
                Transaction::new_unchecked(&connection, TransactionBehavior::Immediate)?;
            if inspect_history(&transaction, history)? != (fresh, current) {
                return Err(StorageError::Migration(
                    "workspace history changed during migration".into(),
                ));
            }
            if let Some(workspace_key) = workspace_key {
                let current_key: String = transaction.query_row(
                    "SELECT workspace_key FROM workspace WHERE id=1",
                    [],
                    |row| row.get(0),
                )?;
                if workspace_key != current_key {
                    return Err(StorageError::Migration(
                        "workspace identity changed during migration".into(),
                    ));
                }
            }
            if fresh {
                transaction.execute_batch(HEADER)?;
                transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
            }
            for migration in pending {
                transaction.execute_batch(&migration.canonical_sql())?;
                if let Some(backfill) = migration.backfill {
                    (backfill.apply)(&transaction)?;
                }
                transaction.execute(
                    "INSERT INTO schema_migrations(version,name,checksum) VALUES(?,?,?)",
                    rusqlite::params![
                        migration.version,
                        migration.name,
                        history.checksum(migration)
                    ],
                )?;
            }
            transaction.pragma_update(None, "user_version", target)?;
            validate(&transaction)?;
            validate_foreign_keys(&transaction)?;
            transaction.commit()?;
            Ok(())
        })();
        if reconstruction {
            connection.pragma_update(None, "foreign_keys", true)?;
        }
        result?;
        Ok(())
    }

    /// Inspect a workspace without taking write admission or running a migration.
    pub fn check_migration_history(&self, history: MigrationHistory<'_>) -> StorageResult<u32> {
        history.validate()?;
        let _admission = acquire(&self.path, ".access.lock", false)?;
        let connection = self.connect(false, false)?;
        let snapshot = Transaction::new_unchecked(&connection, TransactionBehavior::Deferred)?;
        let (_, current) = inspect_history(&snapshot, history)?;
        Ok(current)
    }

    pub fn upgrade_backup_path(&self) -> PathBuf {
        sidecar(&self.path, ".upgrade-backup.sqlite")
    }
}

/// Authenticate the header and complete applied history on one snapshot.
pub(crate) fn inspect_history(
    connection: &Connection,
    history: MigrationHistory<'_>,
) -> StorageResult<(bool, u32)> {
    let target = history.target();
    let app: u32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let table_count: u64 = connection.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    let current: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let fresh = app == 0 && current == 0 && table_count == 0;
    if !fresh && app != APPLICATION_ID {
        return Err(StorageError::Unsupported(
            "refusing to initialize a non-workspace database".into(),
        ));
    }
    if current > target {
        return Err(StorageError::Schema {
            expected: target,
            actual: current,
        });
    }
    if !fresh {
        if current < history.first() {
            return Err(StorageError::Unsupported(format!(
                "workspace schema {current} must be upgraded to schema {} with an Atlas build that supports that schema first",
                history.first()
            )));
        }
        let mut statement = connection
            .prepare("SELECT version,name,checksum FROM schema_migrations ORDER BY version")?;
        let ledger = statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let applied = history
            .migrations
            .iter()
            .take_while(|entry| entry.version <= current);
        if ledger.len() != applied.clone().count() {
            return Err(StorageError::Migration(
                "schema version and ledger disagree".into(),
            ));
        }
        for ((version, name, checksum), migration) in ledger.iter().zip(applied) {
            if *version != migration.version
                || name != migration.name
                || *checksum != history.checksum(migration)
            {
                return Err(StorageError::Migration(format!(
                    "applied migration {} differs from executable",
                    version
                )));
            }
        }
    }
    Ok((fresh, current))
}

fn validate_current(
    snapshot: &Connection,
    validate: impl FnOnce(&Connection) -> StorageResult<()>,
) -> StorageResult<()> {
    let reason: Option<String> =
        snapshot.query_row("SELECT maintenance FROM workspace WHERE id=1", [], |r| {
            r.get(0)
        })?;
    // Only domain recovery reopens a workspace with interrupted maintenance.
    // Publication and upgrades validate foreign keys before committing.
    if reason.is_none() {
        validate(snapshot)?;
    }
    Ok(())
}

fn validate_foreign_keys(connection: &Connection) -> StorageResult<()> {
    if connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(StorageError::Migration("foreign-key check failed".into()));
    }
    Ok(())
}
