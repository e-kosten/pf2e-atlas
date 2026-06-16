use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::RecordKey;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::{AddSavedListItemOutcome, NewSavedList, NewSavedListItem};
use crate::{
    LOCAL_STATE_CONTRACT_VERSION, LOCAL_STATE_SCHEMA_VERSION, LocalStateError, LocalStateResult,
};
use crate::{SavedList, SavedListItem, SavedListWithItems};

const METADATA_TABLE: &str = "local_state_metadata";
const SAVED_LISTS_TABLE: &str = "saved_lists";
const SAVED_LIST_ITEMS_TABLE: &str = "saved_list_items";
const METADATA_CONTRACT_VERSION: &str = "local_state_contract_version";
const METADATA_SCHEMA_VERSION: &str = "schema_version";

#[derive(Debug, Clone)]
pub struct LocalStateStore {
    path: PathBuf,
}

impl LocalStateStore {
    pub fn open(path: impl Into<PathBuf>) -> LocalStateResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let store = Self { path };
        store.initialize()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn create_list(&self, list: NewSavedList) -> LocalStateResult<SavedList> {
        validate_slug(&list.slug)?;
        let connection = self.connection()?;
        let now = now_rfc3339()?;
        let result = connection.execute(
            "INSERT INTO saved_lists (slug, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![list.slug, list.name, list.description, now],
        );
        match result {
            Ok(_) => self
                .get_list(&list.slug)?
                .ok_or(LocalStateError::ListNotFound(list.slug)),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(LocalStateError::ListAlreadyExists(list.slug))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn lists(&self) -> LocalStateResult<Vec<SavedList>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT slug, name, description, created_at, updated_at
             FROM saved_lists
             ORDER BY slug",
        )?;
        let rows = statement.query_map([], saved_list_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_list(&self, slug: &str) -> LocalStateResult<Option<SavedList>> {
        validate_slug(slug)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT slug, name, description, created_at, updated_at
                 FROM saved_lists
                 WHERE slug = ?1",
                params![slug],
                saved_list_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_list_with_items(&self, slug: &str) -> LocalStateResult<Option<SavedListWithItems>> {
        let Some(list) = self.get_list(slug)? else {
            return Ok(None);
        };
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT item.record_key, item.position, item.note,
                    item.record_title_snapshot, item.record_kind_snapshot,
                    item.added_at, item.updated_at
             FROM saved_list_items item
             JOIN saved_lists list ON list.id = item.list_id
             WHERE list.slug = ?1
             ORDER BY item.position ASC, item.record_key ASC",
        )?;
        let rows = statement.query_map(params![slug], saved_list_item_from_row)?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(Some(SavedListWithItems { list, items }))
    }

    pub fn delete_list(&self, slug: &str) -> LocalStateResult<bool> {
        validate_slug(slug)?;
        let connection = self.connection()?;
        let removed =
            connection.execute("DELETE FROM saved_lists WHERE slug = ?1", params![slug])?;
        Ok(removed > 0)
    }

    pub fn add_item(
        &self,
        slug: &str,
        item: NewSavedListItem,
    ) -> LocalStateResult<AddSavedListItemOutcome> {
        validate_slug(slug)?;
        validate_record_key(&item.record_key)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let Some(list_id) = list_id(&transaction, slug)? else {
            return Err(LocalStateError::ListNotFound(slug.to_string()));
        };
        let existing: Option<i64> = transaction
            .query_row(
                "SELECT 1 FROM saved_list_items WHERE list_id = ?1 AND record_key = ?2",
                params![list_id, item.record_key],
                |row| row.get(0),
            )
            .optional()?;
        if existing.is_some() {
            return Ok(AddSavedListItemOutcome::AlreadyPresent);
        }
        let position = next_position(&transaction, list_id)?;
        let now = now_rfc3339()?;
        transaction.execute(
            "INSERT INTO saved_list_items (
                list_id, record_key, position, note,
                record_title_snapshot, record_kind_snapshot,
                added_at, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                list_id,
                item.record_key,
                position,
                item.note,
                item.record_title_snapshot,
                item.record_kind_snapshot,
                now
            ],
        )?;
        transaction.execute(
            "UPDATE saved_lists SET updated_at = ?1 WHERE id = ?2",
            params![now, list_id],
        )?;
        transaction.commit()?;
        Ok(AddSavedListItemOutcome::Added)
    }

    pub fn remove_item(&self, slug: &str, record_key: &str) -> LocalStateResult<bool> {
        validate_slug(slug)?;
        validate_record_key(record_key)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let Some(list_id) = list_id(&transaction, slug)? else {
            return Err(LocalStateError::ListNotFound(slug.to_string()));
        };
        let removed = transaction.execute(
            "DELETE FROM saved_list_items WHERE list_id = ?1 AND record_key = ?2",
            params![list_id, record_key],
        )?;
        if removed > 0 {
            compact_positions(&transaction, list_id)?;
            transaction.execute(
                "UPDATE saved_lists SET updated_at = ?1 WHERE id = ?2",
                params![now_rfc3339()?, list_id],
            )?;
        }
        transaction.commit()?;
        Ok(removed > 0)
    }

    fn initialize(&self) -> LocalStateResult<()> {
        let connection = self.connection()?;
        if table_exists(&connection, METADATA_TABLE)? {
            validate_metadata(&connection)?;
            validate_v1_tables(&connection)?;
            return Ok(());
        }
        if table_exists(&connection, SAVED_LISTS_TABLE)?
            || table_exists(&connection, SAVED_LIST_ITEMS_TABLE)?
        {
            return Err(LocalStateError::IncompatibleSchema(
                "saved-list tables exist without local-state metadata".to_string(),
            ));
        }
        create_v1_schema(&connection)?;
        write_current_metadata(&connection)?;
        Ok(())
    }

    fn connection(&self) -> LocalStateResult<Connection> {
        let connection = Connection::open(&self.path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(connection)
    }
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

fn saved_list_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedList> {
    Ok(SavedList {
        slug: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn saved_list_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedListItem> {
    Ok(SavedListItem {
        record_key: row.get(0)?,
        position: row.get(1)?,
        note: row.get(2)?,
        record_title_snapshot: row.get(3)?,
        record_kind_snapshot: row.get(4)?,
        added_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn list_id(connection: &Connection, slug: &str) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM saved_lists WHERE slug = ?1",
            params![slug],
            |row| row.get(0),
        )
        .optional()
}

fn next_position(connection: &Connection, list_id: i64) -> rusqlite::Result<i64> {
    let max_position: Option<i64> = connection.query_row(
        "SELECT MAX(position) FROM saved_list_items WHERE list_id = ?1",
        params![list_id],
        |row| row.get(0),
    )?;
    Ok(max_position.map_or(1, |position| position + 1))
}

fn compact_positions(connection: &Connection, list_id: i64) -> rusqlite::Result<()> {
    let mut statement = connection.prepare(
        "SELECT record_key FROM saved_list_items WHERE list_id = ?1 ORDER BY position, record_key",
    )?;
    let keys = statement
        .query_map(params![list_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for (index, key) in keys.iter().enumerate() {
        connection.execute(
            "UPDATE saved_list_items SET position = ?1 WHERE list_id = ?2 AND record_key = ?3",
            params![i64::try_from(index + 1).unwrap_or(i64::MAX), list_id, key],
        )?;
    }
    Ok(())
}

fn now_rfc3339() -> LocalStateResult<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(Into::into)
}

fn validate_slug(slug: &str) -> LocalStateResult<()> {
    if slug.is_empty() {
        return Err(LocalStateError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug must not be empty",
        });
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(LocalStateError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug must not start or end with '-'",
        });
    }
    let mut previous_dash = false;
    for byte in slug.bytes() {
        let valid = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
        if !valid {
            return Err(LocalStateError::InvalidSlug {
                slug: slug.to_string(),
                reason: "use lowercase ASCII letters, digits, and '-'",
            });
        }
        if byte == b'-' && previous_dash {
            return Err(LocalStateError::InvalidSlug {
                slug: slug.to_string(),
                reason: "slug must not contain consecutive '-'",
            });
        }
        previous_dash = byte == b'-';
    }
    Ok(())
}

fn validate_record_key(record_key: &str) -> LocalStateResult<()> {
    RecordKey::parse(record_key)
        .map(|_| ())
        .map_err(|error| LocalStateError::InvalidRecordKey {
            record_key: record_key.to_string(),
            reason: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_lists_preserve_order_and_noop_duplicate_adds() -> Result<(), Box<dyn std::error::Error>>
    {
        let store = LocalStateStore::open(temp_path("saved-lists-order"))?;
        store.create_list(NewSavedList {
            slug: "undead-research".to_string(),
            name: "Undead Research".to_string(),
            description: Some("Campaign notes".to_string()),
        })?;

        assert_eq!(
            store.add_item(
                "undead-research",
                NewSavedListItem {
                    record_key: "actions:first".to_string(),
                    note: None,
                    record_title_snapshot: "First".to_string(),
                    record_kind_snapshot: Some("rule".to_string()),
                },
            )?,
            AddSavedListItemOutcome::Added
        );
        store.add_item(
            "undead-research",
            NewSavedListItem {
                record_key: "actions:second".to_string(),
                note: Some("important".to_string()),
                record_title_snapshot: "Second".to_string(),
                record_kind_snapshot: Some("rule".to_string()),
            },
        )?;
        assert_eq!(
            store.add_item(
                "undead-research",
                NewSavedListItem {
                    record_key: "actions:first".to_string(),
                    note: Some("ignored".to_string()),
                    record_title_snapshot: "First duplicate".to_string(),
                    record_kind_snapshot: Some("rule".to_string()),
                },
            )?,
            AddSavedListItemOutcome::AlreadyPresent
        );

        let list = store
            .get_list_with_items("undead-research")?
            .expect("list should exist");
        assert_eq!(list.list.name, "Undead Research");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].record_key, "actions:first");
        assert_eq!(list.items[0].position, 1);
        assert_eq!(list.items[1].record_key, "actions:second");
        assert_eq!(list.items[1].position, 2);
        Ok(())
    }

    #[test]
    fn remove_item_compacts_positions() -> Result<(), Box<dyn std::error::Error>> {
        let store = LocalStateStore::open(temp_path("saved-lists-remove"))?;
        store.create_list(NewSavedList {
            slug: "test-list".to_string(),
            name: "Test List".to_string(),
            description: None,
        })?;
        for key in ["actions:first", "actions:second", "actions:third"] {
            store.add_item(
                "test-list",
                NewSavedListItem {
                    record_key: key.to_string(),
                    note: None,
                    record_title_snapshot: key.to_string(),
                    record_kind_snapshot: None,
                },
            )?;
        }

        assert!(store.remove_item("test-list", "actions:second")?);
        let list = store
            .get_list_with_items("test-list")?
            .expect("list should exist");
        assert_eq!(list.items[0].record_key, "actions:first");
        assert_eq!(list.items[0].position, 1);
        assert_eq!(list.items[1].record_key, "actions:third");
        assert_eq!(list.items[1].position, 2);
        Ok(())
    }

    #[test]
    fn invalid_slugs_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let store = LocalStateStore::open(temp_path("saved-lists-slug"))?;
        let result = store.create_list(NewSavedList {
            slug: "Bad Slug".to_string(),
            name: "Bad".to_string(),
            description: None,
        });
        assert!(matches!(result, Err(LocalStateError::InvalidSlug { .. })));
        Ok(())
    }

    #[test]
    fn invalid_record_keys_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let store = LocalStateStore::open(temp_path("saved-lists-record-key"))?;
        store.create_list(NewSavedList {
            slug: "test-list".to_string(),
            name: "Test List".to_string(),
            description: None,
        })?;
        let result = store.add_item(
            "test-list",
            NewSavedListItem {
                record_key: "not a key".to_string(),
                note: None,
                record_title_snapshot: "Invalid".to_string(),
                record_kind_snapshot: None,
            },
        );
        assert!(matches!(
            result,
            Err(LocalStateError::InvalidRecordKey { .. })
        ));
        Ok(())
    }

    #[test]
    fn fresh_database_initializes_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("saved-lists-metadata");
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
        let path = temp_path("saved-lists-unsupported-metadata");
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
        let path = temp_path("saved-lists-no-metadata");
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
