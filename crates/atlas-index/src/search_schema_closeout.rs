//! Immutable schema migration for catalog, profile and search ownership facts.
use atlas_storage::{StorageError, StorageResult};
use rusqlite::{Connection, OptionalExtension};

pub const SEARCH_SCHEMA_CLOSEOUT_VERSION: u32 = 40;

pub fn backfill_search_schema_closeout(connection: &Connection) -> StorageResult<()> {
    let invalid = |message: &str| {
        StorageError::Migration(format!(
            "search schema closeout: {message}; repair the reference index with the previous compatible Atlas version before upgrading"
        ))
    };
    let bad_profile: Option<String> = connection.query_row(
        "SELECT p.profile_id
         FROM search_profiles p LEFT JOIN embedding_models m ON m.identity_json=p.identity_json
         WHERE m.model_id IS NULL OR json_extract(m.identity_json,'$.profile_id') IS NOT p.profile_id
           OR json_extract(m.identity_json,'$.dimensions') IS NOT p.dimensions
         ORDER BY p.profile_id LIMIT 1",
        [], |row| row.get(0),
    ).optional()?;
    if let Some(profile) = bad_profile {
        return Err(invalid(&format!(
            "published profile {profile} lacks a matching model"
        )));
    }
    for (child, parent, key) in [
        ("core_lexical_units", "lexical_units", "unit_id"),
        (
            "core_semantic_locations",
            "semantic_locations",
            "location_id",
        ),
        ("semantic_units", "semantic_locations", "location_id"),
    ] {
        let invalid_owner: bool = connection.query_row(
            &format!(
                "SELECT EXISTS(
                SELECT 1 FROM {child} c LEFT JOIN {parent} p USING({key})
                WHERE p.record_id IS NOT c.record_id)"
            ),
            [],
            |row| row.get(0),
        )?;
        if invalid_owner {
            return Err(invalid(&format!(
                "{child} contains a mismatched record owner"
            )));
        }
    }

    let fingerprint =
        crate::catalog_fingerprint::fingerprint_v1(legacy_catalog_fields(connection)?);
    connection.execute_batch(include_str!("search_schema_closeout.sql"))?;
    connection.execute(
        "UPDATE artifact_context SET query_catalog_sha256=?1 WHERE singleton=1",
        [fingerprint.as_slice()],
    )?;
    connection.execute_batch("DROP TABLE query_field_catalog")?;
    Ok(())
}

pub(crate) fn legacy_catalog_fields(
    connection: &Connection,
) -> rusqlite::Result<Vec<(String, String)>> {
    let mut statement = connection
        .prepare("SELECT field,definition_json FROM query_field_catalog ORDER BY field")?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}
