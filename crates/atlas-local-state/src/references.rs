use crate::LocalStateResult;
use rusqlite::{Connection, params};

/// Retain imported/saved references even when their current source is unavailable.
/// Existing identities, including available records and their names, are never replaced.
pub(crate) fn retain(connection: &Connection, key: &str, title: &str) -> LocalStateResult<()> {
    connection.execute(
        "INSERT INTO record_identity(key,last_known_name,availability) VALUES(?1,?2,'unavailable') ON CONFLICT(key) DO NOTHING",
        params![key, title],
    )?;
    Ok(())
}
