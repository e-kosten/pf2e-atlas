use atlas_domain::RecordKey;
use rand::random;
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::model::{
    AddSavedListItemOutcome, NewSavedList, NewSavedListItem, SavedList, SavedListItem,
    SavedListWithItems, UpdateSavedList,
};
use crate::{LocalStateError, LocalStateResult};

pub(crate) fn insert_list(connection: &Connection, list: NewSavedList) -> LocalStateResult<String> {
    let slug = list.slug;
    let now = now_rfc3339()?;
    for _ in 0..5 {
        let list_key = new_list_key();
        let result = connection.execute(
            "INSERT INTO saved_lists (list_key, slug, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![list_key, slug, list.name, list.description, now],
        );
        match result {
            Ok(_) => return Ok(list_key),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                if slug_exists(connection, &slug)? {
                    return Err(LocalStateError::ListAlreadyExists(slug));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(LocalStateError::ListKeyAllocationFailed)
}

pub(crate) fn list(connection: &Connection) -> LocalStateResult<Vec<SavedList>> {
    let mut statement = connection.prepare(
        "SELECT list_key, slug, name, description, created_at, updated_at
         FROM saved_lists
         ORDER BY slug",
    )?;
    let rows = statement.query_map([], saved_list_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub(crate) fn get(connection: &Connection, list_ref: &str) -> LocalStateResult<Option<SavedList>> {
    connection
        .query_row(
            "SELECT list_key, slug, name, description, created_at, updated_at
             FROM saved_lists
             WHERE list_key = ?1 OR slug = ?1",
            params![list_ref],
            saved_list_from_row,
        )
        .optional()
        .map_err(Into::into)
}

pub(crate) fn get_with_items(
    connection: &Connection,
    list_ref: &str,
) -> LocalStateResult<Option<SavedListWithItems>> {
    let Some(list) = get(connection, list_ref)? else {
        return Ok(None);
    };
    let mut statement = connection.prepare(
        "SELECT item.record_key, item.position, item.note,
                item.record_title_snapshot, item.record_kind_snapshot,
                item.added_at, item.updated_at
         FROM saved_list_items item
         JOIN saved_lists list ON list.id = item.list_id
         WHERE list.list_key = ?1
         ORDER BY item.position ASC, item.record_key ASC",
    )?;
    let rows = statement.query_map(params![list.list_key], saved_list_item_from_row)?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(Some(SavedListWithItems { list, items }))
}

pub(crate) fn delete(connection: &Connection, list_ref: &str) -> LocalStateResult<bool> {
    let removed = connection.execute(
        "DELETE FROM saved_lists WHERE list_key = ?1 OR slug = ?1",
        params![list_ref],
    )?;
    Ok(removed > 0)
}

pub(crate) fn update_list(
    connection: &Connection,
    list: UpdateSavedList,
) -> LocalStateResult<bool> {
    let now = now_rfc3339()?;
    let result = connection.execute(
        "UPDATE saved_lists
         SET slug = ?1, name = ?2, description = ?3, updated_at = ?4
         WHERE list_key = ?5",
        params![list.slug, list.name, list.description, now, list.list_key],
    );
    match result {
        Ok(updated) => Ok(updated > 0),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Err(LocalStateError::ListAlreadyExists(list.slug))
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn add_item(
    connection: &Connection,
    list_ref: &str,
    item: NewSavedListItem,
) -> LocalStateResult<AddSavedListItemOutcome> {
    let Some(list_id) = list_id(connection, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    let existing: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM saved_list_items WHERE list_id = ?1 AND record_key = ?2",
            params![list_id, item.record_key],
            |row| row.get(0),
        )
        .optional()?;
    if existing.is_some() {
        return Ok(AddSavedListItemOutcome::AlreadyPresent);
    }
    let position = next_position(connection, list_id)?;
    let now = now_rfc3339()?;
    connection.execute(
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
    connection.execute(
        "UPDATE saved_lists SET updated_at = ?1 WHERE id = ?2",
        params![now, list_id],
    )?;
    Ok(AddSavedListItemOutcome::Added)
}

pub(crate) fn remove_item(
    connection: &Connection,
    list_ref: &str,
    record_key: &str,
) -> LocalStateResult<bool> {
    let Some(list_id) = list_id(connection, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    let removed = connection.execute(
        "DELETE FROM saved_list_items WHERE list_id = ?1 AND record_key = ?2",
        params![list_id, record_key],
    )?;
    if removed > 0 {
        compact_positions(connection, list_id)?;
        connection.execute(
            "UPDATE saved_lists SET updated_at = ?1 WHERE id = ?2",
            params![now_rfc3339()?, list_id],
        )?;
    }
    Ok(removed > 0)
}

pub(crate) fn validate_list_ref(list_ref: &str) -> LocalStateResult<()> {
    if list_ref.trim().is_empty() {
        return Err(LocalStateError::InvalidListRef {
            list_ref: list_ref.to_string(),
            reason: "list ref must not be empty",
        });
    }
    Ok(())
}

pub(crate) fn validate_record_key(record_key: &str) -> LocalStateResult<()> {
    RecordKey::parse(record_key)
        .map(|_| ())
        .map_err(|error| LocalStateError::InvalidRecordKey {
            record_key: record_key.to_string(),
            reason: error.to_string(),
        })
}

fn saved_list_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedList> {
    Ok(SavedList {
        list_key: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
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

fn list_id(connection: &Connection, list_ref: &str) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM saved_lists WHERE list_key = ?1 OR slug = ?1",
            params![list_ref],
            |row| row.get(0),
        )
        .optional()
}

fn slug_exists(connection: &Connection, slug: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM saved_lists WHERE slug = ?1",
            params![slug],
            |_row| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
}

fn new_list_key() -> String {
    format!("list_{:016x}", random::<u64>())
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
