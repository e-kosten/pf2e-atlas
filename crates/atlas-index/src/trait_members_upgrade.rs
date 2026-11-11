//! Offline consolidation of filter and lookup membership from the retained artifact.
use crate::IndexError;
use rusqlite::Connection;

pub const TRAIT_MEMBERS_SCHEMA_SQL: &str = include_str!("trait_members_schema.sql");

pub fn backfill_trait_members(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|error| atlas_storage::StorageError::Migration(error.to_string()))
}

fn backfill(c: &Connection) -> Result<(), IndexError> {
    let mut keys = c.prepare("SELECT DISTINCT trait_key FROM trait_memberships")?;
    for key in keys.query_map([], |r| r.get::<_, String>(0))? {
        key?.parse::<atlas_domain::TraitKey>()
            .map_err(|e| IndexError::Invalid(format!("invalid legacy trait key: {e}")))?;
    }
    drop(keys);
    c.execute_batch(
        "INSERT INTO trait_members
           SELECT substr(trait_key,7),record_key,owners_json,1,source_label FROM trait_memberships;
         INSERT INTO trait_members
           SELECT t.value,r.key,'[]',0,NULL FROM record_traits t JOIN records r USING(record_id)
           WHERE true ON CONFLICT(value,record_key,owners_json) DO NOTHING;
         INSERT INTO trait_members
           SELECT t.value,r.key,json_array(json(i.owner_selector_json)),0,NULL
           FROM actor_item_traits t JOIN actor_items i ON i.id=t.item_id JOIN records r USING(record_id)
           WHERE true ON CONFLICT(value,record_key,owners_json) DO NOTHING;",
    )?;
    check_parity(
        c,
        "lookup",
        "SELECT trait_key,record_key,owners_json,source_label FROM trait_memberships",
        "SELECT 'trait:'||value,record_key,owners_json,source_label FROM trait_members WHERE lookup_member=1",
    )?;
    check_parity(
        c,
        "root filters",
        "SELECT record_id,value FROM record_traits",
        "SELECT r.record_id,m.value FROM trait_members m JOIN records r ON r.key=m.record_key WHERE m.owners_json='[]'",
    )?;
    check_parity(
        c,
        "embedded filters",
        "SELECT item_id,value FROM actor_item_traits",
        "SELECT i.id,m.value FROM trait_members m JOIN records r ON r.key=m.record_key
         JOIN actor_items i ON i.record_id=r.record_id
          AND m.owners_json=json_array(json(i.owner_selector_json))",
    )?;
    crate::component_digests::refresh_trait_migration(c)?;
    c.execute_batch(
        "DROP TABLE actor_item_traits;
         DROP TABLE record_traits;
         DROP TABLE trait_memberships;",
    )?;
    Ok(())
}

fn check_parity(c: &Connection, label: &str, old: &str, new: &str) -> Result<(), IndexError> {
    let diff: bool = c.query_row(
        &format!("SELECT EXISTS({old} EXCEPT {new}) OR EXISTS({new} EXCEPT {old})"),
        [],
        |row| row.get(0),
    )?;
    if diff {
        return Err(IndexError::Invalid(format!(
            "trait membership migration changed {label}"
        )));
    }
    Ok(())
}
