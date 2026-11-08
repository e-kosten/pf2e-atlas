use crate::{StorageResult, Workspace};
use crate::{locking::acquire, workspace::validate};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::fs::File;

/// Read-only connection and snapshot shared by domain readers for one operation.
/// Transaction-local set of record keys populated by a domain owner.
pub struct RecordMembership<'a> {
    connection: &'a Connection,
}

impl<'a> RecordMembership<'a> {
    pub fn bind(
        connection: &'a Connection,
        select_keys: &str,
        params: impl rusqlite::Params,
    ) -> crate::StorageResult<Self> {
        connection.execute_batch(
            "CREATE TEMP TABLE atlas_record_membership(key TEXT PRIMARY KEY) WITHOUT ROWID",
        )?;
        connection.execute(
            &format!("INSERT INTO temp.atlas_record_membership(key) {select_keys}"),
            params,
        )?;
        Ok(Self { connection })
    }

    pub fn contains_sql(&self, connection: &Connection) -> crate::StorageResult<&'static str> {
        if !std::ptr::eq(self.connection, connection) {
            return Err(crate::StorageError::Unsupported(
                "record membership belongs to a different read snapshot".into(),
            ));
        }
        Ok("r.key IN (SELECT key FROM temp.atlas_record_membership)")
    }
}

pub struct ReadSession {
    pub(crate) connection: Connection,
    // Fields drop in declaration order: close/unmap before returning capacity,
    // then release admission so Windows maintenance can truncate the file.
    pub(crate) _mapping: Option<crate::workspace::MappingLease<'static>>,
    pub(crate) _admission: File,
    pub(crate) path: std::path::PathBuf,
}
impl ReadSession {
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    pub fn workspace_key(&self) -> StorageResult<String> {
        Ok(self.connection.query_row(
            "SELECT workspace_key FROM workspace WHERE id=1",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn reference_revision(&self) -> StorageResult<u64> {
        Ok(self.connection.query_row(
            "SELECT reference_revision FROM workspace WHERE id=1",
            [],
            |r| r.get(0),
        )?)
    }
}
impl Drop for ReadSession {
    fn drop(&mut self) {
        let _ = self.connection.execute_batch("ROLLBACK");
    }
}

/// One explicit atomic user action. Dropping without commit rolls it back.
pub struct WriteSession {
    pub(crate) connection: Connection,
    pub(crate) _admission: File,
}
impl WriteSession {
    pub fn reference_revision(&self) -> StorageResult<u64> {
        Ok(self.connection.query_row(
            "SELECT reference_revision FROM workspace WHERE id=1",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn commit(self) -> StorageResult<()> {
        self.connection.execute_batch("COMMIT")?;
        Ok(())
    }
}
impl Drop for WriteSession {
    fn drop(&mut self) {
        let _ = self.connection.execute_batch("ROLLBACK");
    }
}

pub struct UpdateOwner {
    pub(crate) workspace: Workspace,
    pub(crate) _lock: File,
}
impl UpdateOwner {
    /// Workspace whose update admission this owner holds.
    pub fn path(&self) -> &std::path::Path {
        self.workspace.path()
    }

    /// Recover an interrupted maintenance attempt without publishing a new
    /// revision. The caller must validate the previously committed domain data.
    /// Failed validation leaves admission closed and all progress intact.
    pub fn recover_maintenance<E>(
        &self,
        schema: u32,
        validate_committed: impl FnOnce(&Connection) -> Result<(), E>,
    ) -> Result<bool, E>
    where
        E: From<crate::StorageError>,
    {
        let _admission = acquire(&self.workspace.path, ".access.lock", true)?;
        let connection = self.workspace.connect(true, false)?;
        validate(&connection, schema, true)?;
        let transaction = Transaction::new_unchecked(&connection, TransactionBehavior::Immediate)
            .map_err(crate::StorageError::from)?;
        let reason: Option<String> = transaction
            .query_row("SELECT maintenance FROM workspace WHERE id=1", [], |row| {
                row.get(0)
            })
            .map_err(crate::StorageError::from)?;
        if reason.is_none() {
            return Ok(false);
        }
        validate_committed(&transaction)?;
        if transaction
            .prepare("PRAGMA foreign_key_check")
            .map_err(crate::StorageError::from)?
            .exists([])
            .map_err(crate::StorageError::from)?
        {
            return Err(crate::StorageError::Migration(
                "foreign-key check failed during maintenance recovery".into(),
            )
            .into());
        }
        transaction
            .execute("UPDATE workspace SET maintenance=NULL WHERE id=1", [])
            .map_err(crate::StorageError::from)?;
        transaction.commit().map_err(crate::StorageError::from)?;
        Ok(true)
    }

    /// Publish one atomic reference update under exclusive access. A failed or
    /// interrupted transaction leaves the prior committed state available.
    pub fn publish<T, E>(
        &self,
        schema: u32,
        apply: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        self.publish_transaction(schema, None, apply)
    }

    /// Allow old committed snapshots during data-only publication. SQLite rejects
    /// structural statements before execution; retry from a fresh transaction only
    /// after releasing shared admission.
    pub fn publish_incremental<T, E>(
        &self,
        schema: u32,
        apply: impl Fn(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let structural = Arc::new(AtomicBool::new(false));
        let result = self.publish_transaction(schema, Some(Arc::clone(&structural)), &apply);
        if structural.load(Ordering::Relaxed) {
            self.publish_transaction(schema, None, apply)
        } else {
            result
        }
    }

    fn publish_transaction<T, E>(
        &self,
        schema: u32,
        structural: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
        apply: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
        use std::sync::atomic::Ordering;

        let _admission = acquire(&self.workspace.path, ".access.lock", structural.is_none())?;
        let mut connection = self.workspace.connect(true, false)?;
        if let Some(denied) = structural.clone() {
            connection
                .authorizer(Some(move |context: AuthContext<'_>| {
                    let structure = match context.action {
                        AuthAction::CreateIndex { .. }
                        | AuthAction::CreateTable { .. }
                        | AuthAction::CreateTrigger { .. }
                        | AuthAction::CreateView { .. }
                        | AuthAction::CreateVtable { .. }
                        | AuthAction::DropIndex { .. }
                        | AuthAction::DropTable { .. }
                        | AuthAction::DropTrigger { .. }
                        | AuthAction::DropView { .. }
                        | AuthAction::DropVtable { .. } => context.database_name != Some("temp"),
                        AuthAction::AlterTable { database_name, .. } => database_name != "temp",
                        AuthAction::Attach { .. }
                        | AuthAction::Detach { .. }
                        | AuthAction::Unknown { .. } => true,
                        _ => false,
                    };
                    if structure {
                        denied.store(true, Ordering::Relaxed);
                        Authorization::Deny
                    } else {
                        Authorization::Allow
                    }
                }))
                .map_err(crate::StorageError::from)?;
        }
        validate(&connection, schema, false)?;
        connection
            .execute_batch("PRAGMA temp_store=FILE")
            .map_err(crate::StorageError::from)?;
        let transaction = Transaction::new_unchecked(&connection, TransactionBehavior::Immediate)
            .map_err(crate::StorageError::from)?;
        let value = apply(&transaction)?;
        if structural.is_some_and(|denied| denied.load(Ordering::Relaxed)) {
            return Err(crate::StorageError::StructuralPublication.into());
        }
        transaction
            .execute(
                "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1",
                [],
            )
            .map_err(crate::StorageError::from)?;
        tracing::info!(target: "atlas_build_diagnostic", stage = "publication_before_commit", "Committing reference update");
        transaction.commit().map_err(crate::StorageError::from)?;
        tracing::info!(target: "atlas_build_diagnostic", stage = "publication_after_commit", "Committed reference update");
        Ok(value)
    }

    /// Blocks new product sessions, then durably marks maintenance before work
    /// that may span transactions. An interrupted updater must explicitly resume.
    pub fn maintenance(&self, schema: u32, reason: &str) -> StorageResult<MaintenanceSession<'_>> {
        let admission = acquire(&self.workspace.path, ".access.lock", true)?;
        let connection = self.workspace.connect(true, false)?;
        validate(&connection, schema, true)?;
        connection.execute("UPDATE workspace SET maintenance=? WHERE id=1", [reason])?;
        Ok(MaintenanceSession {
            connection,
            _admission: admission,
            _owner: self,
        })
    }
}

pub struct MaintenanceSession<'a> {
    connection: Connection,
    _admission: File,
    _owner: &'a UpdateOwner,
}
impl MaintenanceSession<'_> {
    pub fn path(&self) -> &std::path::Path {
        self._owner.workspace.path()
    }
    /// Domain writers can checkpoint preparatory results here. They must not
    /// expose partially updated projections to normal readers.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Validate and publish the complete delta and clear maintenance atomically.
    /// Errors keep maintenance set; dropping the session does not claim success.
    pub fn publish<T, E>(
        &self,
        apply_and_validate: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<rusqlite::Error>,
    {
        let transaction =
            Transaction::new_unchecked(&self.connection, TransactionBehavior::Immediate)?;
        let value = apply_and_validate(&transaction)?;
        transaction.execute("UPDATE workspace SET maintenance=NULL,reference_revision=reference_revision+1 WHERE id=1", [])?;
        transaction.commit()?;
        Ok(value)
    }
}
