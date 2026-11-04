use crate::{
    StorageError, StorageResult, Workspace,
    backup::{self, Destination},
    locking::{acquire, sidecar},
    workspace::APPLICATION_ID,
};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};
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
    fn checksum(&self) -> String {
        let mut hash = Sha256::new();
        hash.update(self.sql.as_bytes());
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
        for (index, migration) in migrations.iter().enumerate() {
            if migration.version as usize != index + 1 {
                return Err(StorageError::Migration(
                    "versions must be contiguous starting at 1".into(),
                ));
            }
        }
        // A publisher owns the update lock throughout expensive preparation.
        // Ordinary process startup must still authenticate an already-current
        // workspace without becoming another publisher. Shared admission and a
        // SQLite snapshot keep this check coherent with publication/migration.
        if self.path.exists() {
            let _admission = acquire(&self.path, ".access.lock", false)?;
            let connection = self.connect(false, false)?;
            let snapshot = Transaction::new_unchecked(&connection, TransactionBehavior::Deferred)?;
            let (fresh, current) = inspect_history(&snapshot, migrations)?;
            if !fresh && current == migrations.len() as u32 {
                let reason: Option<String> = snapshot.query_row(
                    "SELECT maintenance FROM workspace WHERE id=1",
                    [],
                    |r| r.get(0),
                )?;
                if reason.is_none() {
                    validate(&snapshot)?;
                    // Publication and actual migrations validate foreign keys;
                    // ordinary startup must not rescan the complete corpus.
                }
                // A maintenance marker remains closed to product admission;
                // only the domain coordinator may validate and recover it.
                return Ok(());
            }
        }
        // Drop shared admission before acquiring exclusive ownership. Recheck
        // history below because another process may have migrated in between.
        let _update = self.update_owner()?;
        let _admission = acquire(&self.path, ".access.lock", true)?;
        let connection = self.connect(true, true)?;
        let (fresh, current) = inspect_history(&connection, migrations)?;
        let target = migrations.len() as u32;
        if !fresh {
            let reason: Option<String> =
                connection.query_row("SELECT maintenance FROM workspace WHERE id=1", [], |r| {
                    r.get(0)
                })?;
            if reason.is_some() && current == target {
                // The history is authenticated, but product admission stays
                // closed until the domain coordinator validates recovery.
                return Ok(());
            }
            // Pending transactional migrations retain an interrupted maintenance
            // marker. Only post-migration domain recovery may reopen admission;
            // a failed migration rolls back to the old schema and marker.
            if current < target {
                backup::write(
                    &connection,
                    &self.upgrade_backup_path(),
                    Destination::Replace,
                )?;
            }
        }
        connection.execute_batch("PRAGMA journal_mode=WAL;")?;
        let reconstruction = migrations.iter().skip(current as usize).any(|migration| {
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
            if fresh {
                transaction.execute_batch(HEADER)?;
                transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
            }
            for migration in migrations.iter().skip(current as usize) {
                transaction.execute_batch(migration.sql)?;
                if let Some(backfill) = migration.backfill {
                    (backfill.apply)(&transaction)?;
                }
                transaction.execute(
                    "INSERT INTO schema_migrations(version,name,checksum) VALUES(?,?,?)",
                    rusqlite::params![migration.version, migration.name, migration.checksum()],
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

    pub fn upgrade_backup_path(&self) -> PathBuf {
        sidecar(&self.path, ".upgrade-backup.sqlite")
    }
}

/// Authenticate the header and complete applied history on one snapshot.
fn inspect_history(
    connection: &Connection,
    migrations: &[Migration],
) -> StorageResult<(bool, u32)> {
    let target = migrations.len() as u32;
    let app: u32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let table_count: u64 = connection.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    let fresh = app == 0 && table_count == 0;
    if !fresh && app != APPLICATION_ID {
        return Err(StorageError::Unsupported(
            "refusing to initialize a non-workspace database".into(),
        ));
    }
    let current: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if current > target {
        return Err(StorageError::Schema {
            expected: target,
            actual: current,
        });
    }
    if !fresh {
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
        if ledger.len() != current as usize {
            return Err(StorageError::Migration(
                "schema version and ledger disagree".into(),
            ));
        }
        for (index, (version, name, checksum)) in ledger.iter().enumerate() {
            let migration = &migrations[index];
            if *version != migration.version
                || name != migration.name
                || *checksum != migration.checksum()
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

fn validate_foreign_keys(connection: &Connection) -> StorageResult<()> {
    if connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(StorageError::Migration("foreign-key check failed".into()));
    }
    Ok(())
}
