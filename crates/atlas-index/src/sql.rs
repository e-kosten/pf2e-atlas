use std::collections::BTreeMap;

use rusqlite::Connection;

use crate::IndexValidationError;

pub(crate) fn table_exists(
    connection: &Connection,
    table: &str,
) -> Result<bool, IndexValidationError> {
    let mut statement = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([table])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    rows.next()
        .map(|row| row.is_some())
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

pub(crate) fn table_columns(
    connection: &Connection,
    table: &str,
) -> Result<BTreeMap<String, String>, IndexValidationError> {
    let sql = format!("PRAGMA table_xinfo({table})");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut columns = BTreeMap::new();
    for row in rows {
        let (name, column_type) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        columns.insert(name, column_type);
    }
    Ok(columns)
}

pub(crate) fn count_rows(
    connection: &Connection,
    table: &str,
) -> Result<usize, IndexValidationError> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    count_sql(connection, &sql)
}

pub(crate) fn count_sql(connection: &Connection, sql: &str) -> Result<usize, IndexValidationError> {
    connection
        .query_row(sql, [], |row| row.get::<_, usize>(0))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

pub(crate) fn metadata_value(
    connection: &Connection,
    table: &str,
    key: &str,
) -> Result<Option<String>, IndexValidationError> {
    let sql = format!("SELECT value FROM {table} WHERE key = ?1 LIMIT 1");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    match statement.query_row([key], |row| row.get::<_, String>(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(IndexValidationError::QueryFailed(error.to_string())),
    }
}
