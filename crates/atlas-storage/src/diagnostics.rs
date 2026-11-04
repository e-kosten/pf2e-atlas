//! Explicit synchronous developer observations. No SQL text is retained.
//! Workspace connections opened during an active scope receive a trace callback.
//! Only execution inside an explicit scope on the calling thread is counted.
//! Previously attached connections remain observable in a later scope; new
//! attachment counts describe that scope only. Other threads and independently
//! opened temporary connections are deliberately outside these counts.
use crate::{StorageError, StorageResult};
use rusqlite::{
    Connection,
    trace::{TraceEvent, TraceEventCodes},
};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SqlStatementMetrics {
    pub workspace_connections: u64,
    pub statement_start_events: u64,
    pub subprogram_start_events: u64,
    pub statement_completion_events: u64,
}
thread_local! {
    static ACTIVE: Cell<Option<SqlStatementMetrics>> = const { Cell::new(None) };
}
struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}

/// Observe synchronous workspace SQL in one explicit scope. Nested scopes are
/// rejected. Unwinding clears observation; the operation retains its own normal
/// transaction/error behavior. This does not initialize, lock or mutate storage.
pub fn capture_sql_work<T>(
    operation: impl FnOnce() -> T,
) -> StorageResult<(T, SqlStatementMetrics)> {
    if ACTIVE.get().is_some() {
        return Err(StorageError::Unsupported(
            "SQL diagnostic scopes cannot nest".into(),
        ));
    }
    ACTIVE.set(Some(SqlStatementMetrics::default()));
    let _scope = Scope;
    let result = operation();
    let metrics = ACTIVE
        .replace(None)
        .ok_or_else(|| StorageError::Unsupported("SQL diagnostic scope disappeared".into()))?;
    Ok((result, metrics))
}

pub(crate) fn attach(connection: &Connection) {
    let Some(mut metrics) = ACTIVE.get() else {
        return;
    };
    metrics.workspace_connections = metrics.workspace_connections.saturating_add(1);
    ACTIVE.set(Some(metrics));
    connection.trace_v2(
        TraceEventCodes::SQLITE_TRACE_STMT | TraceEventCodes::SQLITE_TRACE_PROFILE,
        Some(record),
    );
}
fn record(event: TraceEvent<'_>) {
    let Some(mut metrics) = ACTIVE.get() else {
        return;
    };
    match event {
        TraceEvent::Stmt(statement, sql)
            if sql.starts_with("--") && statement.sql().as_ref() != sql =>
        {
            metrics.subprogram_start_events = metrics.subprogram_start_events.saturating_add(1);
        }
        TraceEvent::Stmt(_, _) => {
            metrics.statement_start_events = metrics.statement_start_events.saturating_add(1);
        }
        TraceEvent::Profile(_, _) => {
            metrics.statement_completion_events =
                metrics.statement_completion_events.saturating_add(1);
        }
        _ => {}
    }
    ACTIVE.set(Some(metrics));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executed_work_includes_rolled_back_statements_and_trigger_events() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE item(value); CREATE TABLE audit(value); CREATE TRIGGER audit_item AFTER INSERT ON item BEGIN INSERT INTO audit VALUES(new.value); END;").unwrap();
        let (_, metrics) = capture_sql_work(|| {
            attach(&connection);
            connection
                .execute_batch("BEGIN; INSERT INTO item VALUES(1); ROLLBACK;")
                .unwrap();
        })
        .unwrap();
        assert_eq!(metrics.workspace_connections, 1);
        assert_eq!(metrics.statement_start_events, 3);
        assert_eq!(metrics.statement_completion_events, 3);
        assert!(metrics.subprogram_start_events >= 1);
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM item", [], |r| r.get::<_, u64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn no_implicit_connection_attachment_or_nested_collection() {
        let connection = Connection::open_in_memory().unwrap();
        attach(&connection);
        let (nested, metrics) = capture_sql_work(|| {
            connection
                .execute_batch("CREATE TABLE outside_scope(value)")
                .unwrap();
            capture_sql_work(|| ())
        })
        .unwrap();
        assert!(nested.is_err());
        assert_eq!(metrics, SqlStatementMetrics::default());
        let (_, next) = capture_sql_work(|| {
            attach(&connection);
            connection
                .execute_batch("INSERT INTO outside_scope VALUES(1)")
                .unwrap();
        })
        .unwrap();
        assert_eq!(next.statement_start_events, 1);
    }
    #[test]
    fn main_sql_comments_are_not_trigger_events_and_attached_connections_can_be_reused() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE item(value)")
            .unwrap();
        let (_, first) = capture_sql_work(|| {
            attach(&connection);
            connection
                .execute_batch("-- main statement comment\nINSERT INTO item VALUES(1)")
                .unwrap();
        })
        .unwrap();
        assert_eq!(first.statement_start_events, 1);
        assert_eq!(first.subprogram_start_events, 0);
        connection
            .execute_batch("INSERT INTO item VALUES(2)")
            .unwrap();
        let (_, next) = capture_sql_work(|| {
            connection
                .execute_batch("INSERT INTO item VALUES(3)")
                .unwrap()
        })
        .unwrap();
        assert_eq!(next.workspace_connections, 0);
        assert_eq!(next.statement_start_events, 1);
    }
    #[test]
    fn unwind_does_not_leave_a_diagnostic_scope_active() {
        let result =
            std::panic::catch_unwind(|| capture_sql_work(|| panic!("diagnostic operation failed")));
        assert!(result.is_err());
        assert!(capture_sql_work(|| ()).is_ok());
    }
}
