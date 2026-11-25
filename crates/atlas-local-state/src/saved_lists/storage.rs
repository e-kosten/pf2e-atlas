use atlas_domain::RecordKey;
use atlas_domain::ReorderPlacement;
use rand::random;
use rusqlite::{Connection, OptionalExtension, params};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::model::{
    AddSavedListItemOutcome, ImportSavedListItem, NewSavedList, NewSavedListItem,
    ReorderSavedListItem, SavedList, SavedListItem, SavedListWithItems, UpdateSavedList,
};
use crate::{LocalStateError, LocalStateResult};

// The caller holds the workspace write transaction across this check and the mutation.
fn ensure_name_available(
    connection: &Connection,
    owner_id: i64,
    name: &str,
    excluded_list: Option<&str>,
) -> LocalStateResult<()> {
    let mut statement = connection.prepare(
        "SELECT name, list_key, slug, archived FROM saved_lists
         WHERE owner_id = ?1 AND list_key IS NOT ?2",
    )?;
    let mut candidates = statement.query(params![owner_id, excluded_list])?;
    while let Some(row) = candidates.next()? {
        let existing_name: String = row.get(0)?;
        if unicase::eq(name.trim(), existing_name.trim()) {
            return Err(LocalStateError::ListNameConflict {
                name: existing_name,
                list_key: row.get(1)?,
                slug: row.get(2)?,
                archived: row.get(3)?,
            });
        }
    }
    Ok(())
}

pub(crate) fn insert_list(
    connection: &Connection,
    owner_id: i64,
    list: NewSavedList,
) -> LocalStateResult<String> {
    ensure_name_available(connection, owner_id, &list.name, None)?;
    let slug = list.slug;
    let now = now_rfc3339()?;
    for _ in 0..5 {
        let list_key = new_list_key();
        let result = connection.execute(
            "INSERT INTO saved_lists (list_key, slug, name, description, created_at, updated_at, owner_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)",
            params![list_key, slug, list.name, list.description, now, owner_id],
        );
        match result {
            Ok(_) => {
                replace_tags(connection, owner_id, &list_key, list.tags)?;
                return Ok(list_key);
            }
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                if slug_exists(connection, owner_id, &slug)? {
                    return Err(LocalStateError::ListAlreadyExists(slug));
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Err(LocalStateError::ListKeyAllocationFailed)
}

pub(crate) fn list(
    connection: &Connection,
    owner_id: i64,
    include_archived: bool,
) -> LocalStateResult<Vec<SavedList>> {
    let mut statement = connection.prepare(
        "SELECT list.id, list.list_key, list.slug, list.name, list.description,
                COUNT(item.record_key), list.created_at, list.updated_at, list.revision, list.archived
         FROM saved_lists list
         LEFT JOIN saved_list_items item ON item.list_id = list.id
         WHERE list.owner_id = ?1 AND (?2 OR list.archived = 0)
         GROUP BY list.id
         ORDER BY list.updated_at DESC, list.slug ASC",
    )?;
    let rows = statement.query_map(params![owner_id, include_archived], raw_saved_list_from_row)?;
    let lists = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    if lists.is_empty() {
        return Ok(Vec::new());
    }
    let mut tags = std::collections::BTreeMap::<i64, Vec<String>>::new();
    let mut statement = connection.prepare(
        "SELECT t.list_id, t.tag FROM saved_list_tags t
         JOIN saved_lists l ON l.id = t.list_id
         WHERE l.owner_id = ?1 AND (?2 OR l.archived = 0)
         ORDER BY t.list_id, t.tag COLLATE NOCASE, t.tag",
    )?;
    let mut rows = statement.query(params![owner_id, include_archived])?;
    while let Some(row) = rows.next()? {
        tags.entry(row.get(0)?).or_default().push(row.get(1)?);
    }
    Ok(lists
        .into_iter()
        .map(|list| {
            let tags = tags.remove(&list.id).unwrap_or_default();
            saved_list_with_tags(list, tags)
        })
        .collect())
}

pub(crate) fn get(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
) -> LocalStateResult<Option<SavedList>> {
    let raw = connection
        .query_row(
            "SELECT list.id, list.list_key, list.slug, list.name, list.description,
                    COUNT(item.record_key), list.created_at, list.updated_at, list.revision, list.archived
             FROM saved_lists list
             LEFT JOIN saved_list_items item ON item.list_id = list.id
             WHERE list.owner_id = ?2 AND (list.list_key = ?1 OR list.slug = ?1)
             GROUP BY list.id",
            params![list_ref, owner_id],
            raw_saved_list_from_row,
        )
        .optional()?;
    raw.map(|row| {
        let tags = tags_for_list_id(connection, row.id)?;
        Ok(saved_list_with_tags(row, tags))
    })
    .transpose()
}

pub(super) fn item_note(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    record_key: &str,
) -> LocalStateResult<Option<crate::SavedListItemNote>> {
    connection.query_row(
        "SELECT l.list_key,l.slug,i.record_key,i.record_title_snapshot,i.note FROM saved_lists l
         LEFT JOIN saved_list_items i ON i.list_id=l.id AND i.record_key=?3
         WHERE l.owner_id=?1 AND (l.list_key=?2 OR l.slug=?2)", params![owner_id,list_ref,record_key], |row| {
            let Some(record_key) = row.get::<_,Option<String>>(2)? else { return Ok(None) };
            Ok(Some(crate::SavedListItemNote { list_key: row.get(0)?, slug: row.get(1)?, record_key, title: row.get(3)?, note: row.get(4)? }))
        }
    ).optional()?.ok_or_else(|| LocalStateError::ListNotFound(list_ref.into()))
}

pub(crate) fn get_with_items(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
) -> LocalStateResult<Option<SavedListWithItems>> {
    let Some(list) = get(connection, owner_id, list_ref)? else {
        return Ok(None);
    };
    let mut statement = connection.prepare(
        "SELECT item.record_key, item.position, item.note,
                item.record_title_snapshot, item.record_kind_snapshot,
                item.added_at, item.updated_at
         FROM saved_list_items item
         JOIN saved_lists list ON list.id = item.list_id
         WHERE list.list_key = ?1 AND list.owner_id = ?2
         ORDER BY item.position ASC, item.record_key ASC",
    )?;
    let rows = statement.query_map(params![list.list_key, owner_id], saved_list_item_from_row)?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(Some(SavedListWithItems { list, items }))
}

pub(crate) fn delete(
    connection: &Connection,
    owner_id: i64,
    list_key: &str,
) -> LocalStateResult<Option<String>> {
    Ok(connection
        .query_row(
            "DELETE FROM saved_lists WHERE owner_id = ?2 AND list_key = ?1 RETURNING slug",
            params![list_key, owner_id],
            |row| row.get(0),
        )
        .optional()?)
}

pub(crate) fn replace_items(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    items: Vec<ImportSavedListItem>,
) -> LocalStateResult<()> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    connection.execute(
        "DELETE FROM saved_list_items WHERE list_id = ?1",
        params![list_id],
    )?;
    insert_imported_items(connection, list_id, items)?;
    connection.execute(
        "UPDATE saved_lists SET revision = revision + 1, updated_at = ?1 WHERE id = ?2",
        params![now_rfc3339()?, list_id],
    )?;
    Ok(())
}

pub(crate) fn update_list(
    connection: &Connection,
    owner_id: i64,
    previous_name: &str,
    list: UpdateSavedList,
) -> LocalStateResult<bool> {
    if previous_name != list.name {
        ensure_name_available(connection, owner_id, &list.name, Some(&list.list_key))?;
    }
    let now = now_rfc3339()?;
    let result = connection.execute(
        "UPDATE saved_lists
         SET revision = revision + 1, slug = ?1, name = ?2, description = ?3, updated_at = ?4, archived = ?5
         WHERE list_key = ?6 AND owner_id = ?7",
        params![
            list.slug,
            list.name,
            list.description,
            now,
            list.archived,
            list.list_key,
            owner_id
        ],
    );
    match result {
        Ok(updated) => {
            if updated > 0
                && let Some(tags) = list.tags
            {
                replace_tags(connection, owner_id, &list.list_key, tags)?;
            }
            Ok(updated > 0)
        }
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
    owner_id: i64,
    list_ref: &str,
    item: NewSavedListItem,
) -> LocalStateResult<AddSavedListItemOutcome> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
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
    crate::references::retain(connection, &item.record_key, &item.record_title_snapshot)?;
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
        "UPDATE saved_lists SET revision = revision + 1, updated_at = ?1 WHERE id = ?2",
        params![now, list_id],
    )?;
    Ok(AddSavedListItemOutcome::Added)
}

fn insert_imported_items(
    connection: &Connection,
    list_id: i64,
    items: Vec<ImportSavedListItem>,
) -> LocalStateResult<()> {
    let now = now_rfc3339()?;
    for (index, item) in items.into_iter().enumerate() {
        crate::references::retain(connection, &item.record_key, &item.record_title_snapshot)?;
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
                (index + 1) as i64,
                item.note,
                item.record_title_snapshot,
                item.record_kind_snapshot,
                now
            ],
        )?;
    }
    Ok(())
}

pub(crate) fn remove_item(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    record_key: &str,
) -> LocalStateResult<bool> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    let removed = connection.execute(
        "DELETE FROM saved_list_items WHERE list_id = ?1 AND record_key = ?2",
        params![list_id, record_key],
    )?;
    if removed > 0 {
        compact_positions(connection, list_id)?;
        connection.execute(
            "UPDATE saved_lists SET revision = revision + 1, updated_at = ?1 WHERE id = ?2",
            params![now_rfc3339()?, list_id],
        )?;
    }
    Ok(removed > 0)
}

pub(crate) fn reorder_item(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    reorder: ReorderSavedListItem,
) -> LocalStateResult<bool> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    let mut statement = connection.prepare(
        "SELECT record_key FROM saved_list_items WHERE list_id = ?1 ORDER BY position, record_key",
    )?;
    let mut keys = statement
        .query_map(params![list_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let Some(from) = keys.iter().position(|key| key == &reorder.record_key) else {
        return Ok(false);
    };
    if reorder.record_key == reorder.target_record_key {
        return Ok(true);
    }
    keys.remove(from);
    let Some(target) = keys
        .iter()
        .position(|key| key == &reorder.target_record_key)
    else {
        return Ok(false);
    };
    let index = match reorder.placement {
        ReorderPlacement::Before => target,
        ReorderPlacement::After => target + 1,
    };
    keys.insert(index, reorder.record_key);
    // Move through a disjoint range so the unique (list_id, position) index
    // cannot collide while positions are rewritten.
    connection.execute(
        "UPDATE saved_list_items SET position = -position WHERE list_id = ?1",
        params![list_id],
    )?;
    let now = now_rfc3339()?;
    for (index, key) in keys.iter().enumerate() {
        connection.execute(
            "UPDATE saved_list_items SET position = ?1, updated_at = ?2 WHERE list_id = ?3 AND record_key = ?4",
            params![index as i64 + 1, now, list_id, key],
        )?;
    }
    connection.execute(
        "UPDATE saved_lists SET revision = revision + 1, updated_at = ?1 WHERE id = ?2",
        params![now, list_id],
    )?;
    Ok(true)
}

pub(crate) fn update_item_note(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    record_key: &str,
    note: Option<&str>,
) -> LocalStateResult<bool> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    let changed = connection.execute(
        "UPDATE saved_list_items SET note = ?3, updated_at = ?4 WHERE list_id = ?1 AND record_key = ?2",
        params![list_id, record_key, note, now_rfc3339()?],
    )?;
    if changed > 0 {
        connection.execute(
            "UPDATE saved_lists SET revision = revision + 1, updated_at = ?1 WHERE id = ?2",
            params![now_rfc3339()?, list_id],
        )?;
    }
    Ok(changed > 0)
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

#[derive(Debug)]
struct RawSavedList {
    revision: i64,
    id: i64,
    list_key: String,
    slug: String,
    name: String,
    description: Option<String>,
    archived: bool,
    item_count: u64,
    created_at: String,
    updated_at: String,
}

fn raw_saved_list_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawSavedList> {
    Ok(RawSavedList {
        revision: row.get(8)?,
        archived: row.get(9)?,
        id: row.get(0)?,
        list_key: row.get(1)?,
        slug: row.get(2)?,
        name: row.get(3)?,
        description: row.get(4)?,
        item_count: row.get::<_, i64>(5)? as u64,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn saved_list_with_tags(raw: RawSavedList, tags: Vec<String>) -> SavedList {
    SavedList {
        revision: raw.revision,
        archived: raw.archived,
        list_key: raw.list_key,
        slug: raw.slug,
        name: raw.name,
        description: raw.description,
        tags,
        item_count: raw.item_count,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
    }
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

fn list_id(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
) -> rusqlite::Result<Option<i64>> {
    connection
        .query_row(
            "SELECT id FROM saved_lists WHERE owner_id = ?2 AND (list_key = ?1 OR slug = ?1)",
            params![list_ref, owner_id],
            |row| row.get(0),
        )
        .optional()
}

fn tags_for_list_id(connection: &Connection, list_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT tag FROM saved_list_tags WHERE list_id = ?1 ORDER BY tag COLLATE NOCASE, tag",
    )?;
    statement
        .query_map(params![list_id], |row| row.get::<_, String>(0))?
        .collect()
}

fn replace_tags(
    connection: &Connection,
    owner_id: i64,
    list_ref: &str,
    tags: Vec<String>,
) -> LocalStateResult<()> {
    let Some(list_id) = list_id(connection, owner_id, list_ref)? else {
        return Err(LocalStateError::ListNotFound(list_ref.to_string()));
    };
    connection.execute(
        "DELETE FROM saved_list_tags WHERE list_id = ?1",
        params![list_id],
    )?;
    let now = now_rfc3339()?;
    for tag in tags {
        connection.execute(
            "INSERT INTO saved_list_tags (list_id, tag, created_at) VALUES (?1, ?2, ?3)",
            params![list_id, tag, now],
        )?;
    }
    Ok(())
}

pub(super) fn slug_exists(
    connection: &Connection,
    owner_id: i64,
    slug: &str,
) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM saved_lists WHERE owner_id = ?2 AND slug = ?1",
            params![slug, owner_id],
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
