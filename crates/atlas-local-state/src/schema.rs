use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    LOCAL_STATE_CONTRACT_VERSION, LOCAL_STATE_SCHEMA_VERSION, LocalStateError, LocalStateResult,
};

const METADATA_TABLE: &str = "local_state_metadata";
const SAVED_LISTS_TABLE: &str = "saved_lists";
const SAVED_LIST_ITEMS_TABLE: &str = "saved_list_items";
const METADATA_CONTRACT_VERSION: &str = "local_state_contract_version";
const METADATA_SCHEMA_VERSION: &str = "schema_version";

pub(crate) fn initialize(connection: &Connection) -> LocalStateResult<()> {
    if table_exists(connection, METADATA_TABLE)? {
        validate_metadata(connection)?;
        validate_v1_tables(connection)?;
        return Ok(());
    }
    if table_exists(connection, SAVED_LISTS_TABLE)?
        || table_exists(connection, SAVED_LIST_ITEMS_TABLE)?
    {
        return Err(LocalStateError::IncompatibleSchema(
            "saved-list tables exist without local-state metadata".to_string(),
        ));
    }
    create_v1_schema(connection)?;
    write_current_metadata(connection)?;
    Ok(())
}

fn create_v1_schema(connection: &Connection) -> LocalStateResult<()> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE local_state_metadata (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL
        );
        CREATE TABLE saved_lists (
          id INTEGER PRIMARY KEY,
          slug TEXT NOT NULL UNIQUE,
          name TEXT NOT NULL,
          description TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE TABLE saved_list_items (
          list_id INTEGER NOT NULL,
          record_key TEXT NOT NULL,
          position INTEGER NOT NULL,
          note TEXT,
          record_title_snapshot TEXT NOT NULL,
          record_kind_snapshot TEXT,
          added_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (list_id, record_key),
          UNIQUE (list_id, position),
          FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
        );
        CREATE INDEX saved_list_items_position_idx
          ON saved_list_items(list_id, position);
        ",
    )?;
    Ok(())
}

fn write_current_metadata(connection: &Connection) -> LocalStateResult<()> {
    connection.execute(
        "INSERT INTO local_state_metadata (key, value) VALUES (?1, ?2)",
        params![METADATA_CONTRACT_VERSION, LOCAL_STATE_CONTRACT_VERSION],
    )?;
    connection.execute(
        "INSERT INTO local_state_metadata (key, value) VALUES (?1, ?2)",
        params![METADATA_SCHEMA_VERSION, LOCAL_STATE_SCHEMA_VERSION],
    )?;
    Ok(())
}

fn validate_metadata(connection: &Connection) -> LocalStateResult<()> {
    validate_metadata_value(
        connection,
        METADATA_CONTRACT_VERSION,
        LOCAL_STATE_CONTRACT_VERSION,
    )?;
    validate_metadata_value(
        connection,
        METADATA_SCHEMA_VERSION,
        LOCAL_STATE_SCHEMA_VERSION,
    )?;
    Ok(())
}

fn validate_metadata_value(
    connection: &Connection,
    key: &'static str,
    expected: &'static str,
) -> LocalStateResult<()> {
    let Some(actual) = metadata_value(connection, key)? else {
        return Err(LocalStateError::IncompatibleSchema(format!(
            "missing required local-state metadata `{key}`"
        )));
    };
    if actual == expected {
        return Ok(());
    }
    Err(LocalStateError::UnsupportedMetadata { key, value: actual })
}

fn validate_v1_tables(connection: &Connection) -> LocalStateResult<()> {
    for table in [SAVED_LISTS_TABLE, SAVED_LIST_ITEMS_TABLE] {
        if !table_exists(connection, table)? {
            return Err(LocalStateError::IncompatibleSchema(format!(
                "missing required local-state table `{table}`"
            )));
        }
    }
    Ok(())
}

fn table_exists(connection: &Connection, table: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table],
            |_row| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
}

fn metadata_value(connection: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    connection
        .query_row(
            "SELECT value FROM local_state_metadata WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use rusqlite::Connection;
    use time::OffsetDateTime;

    use super::*;
    use crate::LocalStateStore;

    #[test]
    fn fresh_database_initializes_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("metadata");
        let store = LocalStateStore::open(&path)?;
        drop(store);
        let connection = Connection::open(&path)?;
        assert_eq!(
            metadata_value(&connection, METADATA_CONTRACT_VERSION)?,
            Some(LOCAL_STATE_CONTRACT_VERSION.to_string())
        );
        assert_eq!(
            metadata_value(&connection, METADATA_SCHEMA_VERSION)?,
            Some(LOCAL_STATE_SCHEMA_VERSION.to_string())
        );
        Ok(())
    }

    #[test]
    fn unsupported_metadata_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("unsupported-metadata");
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch(
                "
                CREATE TABLE local_state_metadata (
                  key TEXT PRIMARY KEY,
                  value TEXT NOT NULL
                );
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('local_state_contract_version', 'pf2e-atlas-local-state/v2');
                INSERT INTO local_state_metadata (key, value)
                  VALUES ('schema_version', '2');
                ",
            )?;
        }

        let result = LocalStateStore::open(path);
        assert!(matches!(
            result,
            Err(LocalStateError::UnsupportedMetadata { .. })
        ));
        Ok(())
    }

    #[test]
    fn existing_tables_without_metadata_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("no-metadata");
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch("CREATE TABLE saved_lists (id INTEGER PRIMARY KEY);")?;
        }

        let result = LocalStateStore::open(path);
        assert!(matches!(
            result,
            Err(LocalStateError::IncompatibleSchema(_))
        ));
        Ok(())
    }

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "atlas-local-state-{name}-{}-{}.sqlite",
            std::process::id(),
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        ));
        let _ = fs::remove_file(&path);
        path
    }
}
