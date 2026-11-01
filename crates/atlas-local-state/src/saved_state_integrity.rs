//! Atomic reconstruction of the saved-domain relationship and enum constraints.
use atlas_storage::{StorageError, StorageResult};
use rusqlite::{Connection, OptionalExtension};

const TABLES: [&str; 5] = [
    "encounters",
    "encounter_participants",
    "encounter_pending_consequences",
    "encounter_boundary_claims",
    "encounter_condition_applications",
];

/// Immutable saved-state-integrity/v1 backfill. Storage owns its reconstruction transaction.
pub fn backfill_saved_state_integrity(connection: &Connection) -> StorageResult<()> {
    if connection.is_autocommit()
        || connection.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, bool>(0))?
    {
        return Err(StorageError::Migration(
            "saved-state reconstruction requires a storage-owned transaction with foreign keys suspended before BEGIN".into(),
        ));
    }
    preflight(connection)?;
    let before = row_counts(connection)?;
    connection.execute_batch(include_str!("saved_state_integrity.sql"))?;
    if row_counts(connection)? != before {
        return Err(StorageError::Migration(
            "saved-state reconstruction changed retained row counts".into(),
        ));
    }
    Ok(())
}

fn row_counts(connection: &Connection) -> StorageResult<Vec<i64>> {
    TABLES
        .iter()
        .map(|table| {
            Ok(
                connection.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })?,
            )
        })
        .collect()
}

fn preflight(connection: &Connection) -> StorageResult<()> {
    for (table, key, column, allowed) in [
        (
            "encounters",
            "encounter_key",
            "status",
            "'draft','running','complete','archived'",
        ),
        (
            "encounter_participants",
            "participant_key",
            "participant_kind",
            "'creature','hazard','pc'",
        ),
        (
            "encounter_participants",
            "participant_key",
            "participant_variant",
            "'normal','elite','weak'",
        ),
        (
            "encounter_participants",
            "participant_key",
            "side",
            "'pc','ally','enemy','neutral','hazard'",
        ),
        (
            "encounter_participants",
            "participant_key",
            "hp_origin",
            "'derived_pristine','derived_edited','explicit','unknown'",
        ),
        (
            "encounter_participants",
            "participant_key",
            "variant_origin",
            "'default_unadjusted','inherited_known','inherited_unknown','explicit'",
        ),
    ] {
        let invalid: Option<String> = connection.query_row(
            &format!("SELECT {key} || ': ' || quote({column}) FROM {table} WHERE {column} NOT IN ({allowed}) LIMIT 1"),
            [], |row| row.get(0),
        ).optional()?;
        if let Some(invalid) = invalid {
            return Err(StorageError::Migration(format!(
                "{table}.{column} has an unsupported stored value ({invalid}); saved data was preserved; inspect and explicitly resolve this value before retrying"
            )));
        }
    }
    let invalid_turn: Option<String> = connection.query_row(
        "SELECT encounter_key || ': ' || quote(current_turn_participant_key) FROM encounters e WHERE current_turn_participant_key IS NOT NULL AND NOT EXISTS(SELECT 1 FROM encounter_participants p WHERE p.encounter_id=e.id AND p.participant_key=e.current_turn_participant_key) LIMIT 1",
        [], |row| row.get(0),
    ).optional()?;
    if let Some(invalid) = invalid_turn {
        return Err(StorageError::Migration(format!(
            "encounters.current_turn_participant_key has a missing or cross-encounter owner ({invalid}); saved data was preserved; explicitly resolve its owner before retrying"
        )));
    }
    for table in [
        "encounter_pending_consequences",
        "encounter_boundary_claims",
        "encounter_condition_applications",
    ] {
        let invalid: Option<String> = connection.query_row(
            &format!("SELECT CAST(c.encounter_id AS TEXT) || ': ' || quote(c.participant_key) FROM {table} c WHERE NOT EXISTS(SELECT 1 FROM encounter_participants p WHERE p.encounter_id=c.encounter_id AND p.participant_key=c.participant_key) LIMIT 1"),
            [], |row| row.get(0),
        ).optional()?;
        if let Some(invalid) = invalid {
            return Err(StorageError::Migration(format!(
                "{table} has a missing or cross-encounter participant ({invalid}); saved data was preserved; explicitly resolve its owner before retrying"
            )));
        }
    }
    let adjustments: i64 = connection.query_row(
        "SELECT count(*) FROM encounter_participant_adjustments",
        [],
        |row| row.get(0),
    )?;
    if adjustments != 0 {
        return Err(StorageError::Migration(format!(
            "encounter_participant_adjustments contains {adjustments} historical rows; saved data was preserved; choose a preservation policy before retrying"
        )));
    }
    Ok(())
}
