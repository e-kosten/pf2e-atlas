use crate::{
    StorageError, StorageResult, Workspace,
    locking::{acquire, sidecar},
    workspace::APPLICATION_ID,
};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Duration};

/// Ordered executable-owned SQL. Never load migration SQL from a workspace.
#[derive(Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}
impl Migration {
    fn checksum(&self) -> String {
        format!("{:x}", Sha256::digest(self.sql.as_bytes()))
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
                self.backup(&connection)?;
            }
        }
        connection.execute_batch("PRAGMA journal_mode=WAL;")?;
        let transaction = Transaction::new_unchecked(&connection, TransactionBehavior::Immediate)?;
        if fresh {
            transaction.execute_batch(HEADER)?;
            transaction.pragma_update(None, "application_id", APPLICATION_ID)?;
        }
        for migration in migrations.iter().skip(current as usize) {
            transaction.execute_batch(migration.sql)?;
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
    }

    pub fn upgrade_backup_path(&self) -> PathBuf {
        sidecar(&self.path, ".upgrade-backup.sqlite")
    }

    fn backup(&self, source: &Connection) -> StorageResult<()> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| StorageError::Unsupported("workspace has no parent".into()))?;
        let file = tempfile::NamedTempFile::new_in(parent)?;
        let mut destination = Connection::open(file.path())?;
        let backup = rusqlite::backup::Backup::new(source, &mut destination)?;
        backup.run_to_completion(256, Duration::from_millis(1), None)?;
        drop(backup);
        drop(destination);
        file.as_file().sync_all()?;
        file.persist(self.upgrade_backup_path())
            .map_err(|e| StorageError::Io(e.error))?;
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
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
