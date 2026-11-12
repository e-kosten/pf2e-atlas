use crate::{MigrationHistory, StorageError, StorageResult, Workspace};
use crate::{locking::acquire, workspace::validate};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::fs::File;
use std::sync::{Arc, atomic::AtomicBool};

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
    /// Authenticate the migration ledger on this read snapshot.
    pub fn authenticate_history(&self, history: MigrationHistory<'_>) -> StorageResult<()> {
        authenticate_history(&self.connection, history)
    }

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
    /// Authenticate the migration ledger on this write transaction.
    pub fn authenticate_history(&self, history: MigrationHistory<'_>) -> StorageResult<()> {
        authenticate_history(&self.connection, history)
    }

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

fn authenticate_history(
    connection: &Connection,
    history: MigrationHistory<'_>,
) -> StorageResult<()> {
    history.validate()?;
    let (_, actual) = crate::migration::inspect_history(connection, history)?;
    let expected = history.target();
    if actual != expected {
        return Err(StorageError::Schema { expected, actual });
    }
    Ok(())
}

pub struct UpdateOwner {
    pub(crate) workspace: Workspace,
    pub(crate) _lock: File,
}

struct SharedPublication<'a> {
    denied: Arc<AtomicBool>,
    vector_tables: &'a [(String, String)],
}

/// Authority to replace generated reference data at one authenticated revision.
/// The update owner stays held while preparation runs outside SQLite.
pub struct ReferenceRepair<'a> {
    owner: &'a UpdateOwner,
    history: crate::MigrationHistory<'static>,
    workspace_key: String,
    reference_revision: u64,
}

impl<'a> ReferenceRepair<'a> {
    pub fn owner(&self) -> &'a UpdateOwner {
        self.owner
    }

    pub fn path(&self) -> &std::path::Path {
        self.owner.path()
    }

    pub fn schema(&self) -> u32 {
        self.history.target()
    }

    pub fn workspace_key(&self) -> &str {
        &self.workspace_key
    }

    pub fn reference_revision(&self) -> u64 {
        self.reference_revision
    }

    pub fn publish<T, E>(&self, apply: impl FnOnce(&Connection) -> Result<T, E>) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        self.owner
            .publish_transaction(self.schema(), None, Some(self), |_| Ok(()), apply)
    }

    fn check_expected(&self, connection: &Connection) -> StorageResult<()> {
        let (_, current) = crate::migration::inspect_history(connection, self.history)?;
        if current != self.schema() {
            return Err(crate::StorageError::Schema {
                expected: self.schema(),
                actual: current,
            });
        }
        let matches: bool = connection.query_row(
            "SELECT workspace_key=?1 AND reference_revision=?2 FROM workspace WHERE id=1",
            (&self.workspace_key, self.reference_revision),
            |row| row.get(0),
        )?;
        if !matches {
            return Err(crate::StorageError::Unsupported(
                "workspace identity or reference revision changed during repair".into(),
            ));
        }
        Ok(())
    }
}

fn vector_creation_action(table: &str, context: &rusqlite::hooks::AuthContext<'_>) -> bool {
    use rusqlite::hooks::AuthAction;
    if context.database_name != Some("main") {
        return false;
    }
    // sqlite-vec 0.1.9 creates these objects for one 768-dimensional vector
    // column. The authorizer is scoped to one statement from the index owner.
    match context.action {
        AuthAction::CreateVtable {
            table_name,
            module_name,
        } => table_name == table && module_name == "vec0",
        AuthAction::CreateTable { table_name } => {
            table_name == "sqlite_sequence"
                || matches!(
                    table_name.strip_prefix(table),
                    Some("_info" | "_chunks" | "_rowids" | "_vector_chunks00")
                )
        }
        AuthAction::CreateIndex {
            index_name,
            table_name,
        } => {
            matches!(
                table_name.strip_prefix(table),
                Some("_info" | "_vector_chunks00")
            ) && index_name
                .strip_prefix("sqlite_autoindex_")
                .and_then(|name| name.strip_suffix("_1"))
                == Some(table_name)
        }
        _ => false,
    }
}

impl UpdateOwner {
    /// Workspace whose update admission this owner holds.
    pub fn path(&self) -> &std::path::Path {
        self.workspace.path()
    }

    /// Authenticate a current workspace and allow repair configuration reads
    /// even when a maintenance marker blocks normal admission.
    pub fn begin_reference_repair<T, E>(
        &self,
        history: crate::MigrationHistory<'static>,
        inspect: impl FnOnce(&ReadSession) -> Result<T, E>,
    ) -> Result<(ReferenceRepair<'_>, T), E>
    where
        E: From<crate::StorageError>,
    {
        history.validate()?;
        let read = self.workspace.read_admitted(history.target(), true)?;
        crate::migration::inspect_history(read.connection(), history)?;
        let repair = ReferenceRepair {
            owner: self,
            history,
            workspace_key: read.workspace_key()?,
            reference_revision: read.reference_revision()?,
        };
        let result = inspect(&read)?;
        Ok((repair, result))
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
        self.publish_transaction(schema, None, None, |_| Ok(()), apply)
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
        let result = self.publish_transaction(
            schema,
            Some(SharedPublication {
                denied: Arc::clone(&structural),
                vector_tables: &[],
            }),
            None,
            |_| Ok(()),
            &apply,
        );
        if structural.load(Ordering::Relaxed) {
            self.publish_transaction(schema, None, None, |_| Ok(()), apply)
        } else {
            result
        }
    }

    /// Publish while readers remain admitted. Only the generated first-time
    /// vec0 creation statements and their extension-owned objects may change
    /// the main schema; unexpected DDL never triggers exclusive replay.
    pub fn publish_with_new_vector_tables<T, E>(
        &self,
        schema: u32,
        tables: &[(String, String)],
        validate_previous: impl FnOnce(&Connection) -> Result<(), E>,
        apply: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        self.publish_transaction(
            schema,
            Some(SharedPublication {
                denied: Arc::default(),
                vector_tables: tables,
            }),
            None,
            validate_previous,
            apply,
        )
    }

    fn publish_transaction<T, E>(
        &self,
        schema: u32,
        shared: Option<SharedPublication<'_>>,
        repair: Option<&ReferenceRepair<'_>>,
        validate_previous: impl FnOnce(&Connection) -> Result<(), E>,
        apply: impl FnOnce(&Connection) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<crate::StorageError>,
    {
        use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
        use std::sync::{Arc, Mutex, atomic::Ordering};

        let creating_vector = Arc::new(Mutex::new(None::<String>));
        let exclusive = shared.is_none();
        let admission = acquire(&self.workspace.path, ".access.lock", exclusive)?;
        if exclusive {
            tracing::info!(target: "atlas_progress", phase = "exclusive_publication", "Finalizing search index");
        }
        let mut connection = self.workspace.connect(true, false)?;
        if let Some(shared) = &shared {
            let denied = Arc::clone(&shared.denied);
            let creating_vector = Arc::clone(&creating_vector);
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
                    if structure
                        && !creating_vector
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .as_deref()
                            .is_some_and(|table| vector_creation_action(table, &context))
                    {
                        denied.store(true, Ordering::Relaxed);
                        Authorization::Deny
                    } else {
                        Authorization::Allow
                    }
                }))
                .map_err(crate::StorageError::from)?;
        }
        validate(&connection, schema, repair.is_some())?;
        connection
            .execute_batch("PRAGMA temp_store=FILE")
            .map_err(crate::StorageError::from)?;
        let transaction = Transaction::new_unchecked(&connection, TransactionBehavior::Immediate)
            .map_err(crate::StorageError::from)?;
        if let Some(repair) = repair {
            repair.check_expected(&transaction)?;
        }
        let result = (|| {
            validate_previous(&transaction)?;
            for (table, ddl) in shared
                .as_ref()
                .map_or(&[][..], |shared| shared.vector_tables)
            {
                let exists: bool = transaction
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?1)",
                        [table],
                        |row| row.get(0),
                    )
                    .map_err(crate::StorageError::from)?;
                if exists {
                    continue;
                }
                *creating_vector
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(table.clone());
                let created = transaction.execute(ddl, []);
                creating_vector
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                created.map_err(crate::StorageError::from)?;
            }
            apply(&transaction)
        })();
        if shared.is_some_and(|shared| shared.denied.load(Ordering::Relaxed)) {
            return Err(crate::StorageError::StructuralPublication.into());
        }
        let value = result?;
        let advance = if repair.is_some() {
            "UPDATE workspace SET reference_revision=reference_revision+1,maintenance=NULL WHERE id=1"
        } else {
            "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1"
        };
        transaction
            .execute(advance, [])
            .map_err(crate::StorageError::from)?;
        tracing::info!(target: "atlas_build_diagnostic", stage = "publication_before_commit", "Committing reference update");
        transaction.commit().map_err(crate::StorageError::from)?;
        drop(connection);
        drop(admission);
        if exclusive {
            tracing::info!(target: "atlas_progress", phase = "exclusive_publication_complete", "Search index saved");
        }
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
