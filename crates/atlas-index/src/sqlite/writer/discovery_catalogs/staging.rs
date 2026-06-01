use rusqlite::{Connection, params};

use crate::IndexWriteError;

use super::field_seeds::FIELD_SEEDS;

const TEMP_DISCOVERY_VALUES: &str = "temp_discovery_values";

pub(super) fn stage_discovery_values(connection: &Connection) -> Result<(), IndexWriteError> {
    connection
        .execute_batch(
            "DROP TABLE IF EXISTS temp_discovery_values;
             CREATE TEMP TABLE temp_discovery_values (
               field TEXT NOT NULL,
               record_key TEXT NOT NULL,
               record_family TEXT NOT NULL,
               value TEXT,
               numeric_value REAL
             );",
        )
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    let total = FIELD_SEEDS.len() as u64;
    for (index, seed) in FIELD_SEEDS.iter().enumerate() {
        super::progress(
            "filter_discovery_values",
            index as u64,
            total,
            format!("Staging filter discovery values: {}", seed.field),
        );
        let sql = format!(
            "WITH field_values(record_key, value) AS ({value_sql})
             INSERT INTO {TEMP_DISCOVERY_VALUES}
                 (field, record_key, record_family, value, numeric_value)
             SELECT ?1,
                    fv.record_key,
                    r.record_family,
                    CAST(fv.value AS TEXT),
                    CAST(fv.value AS REAL)
             FROM field_values fv
             JOIN records r ON r.record_key = fv.record_key
             WHERE r.is_default_visible = 1",
            value_sql = seed.value_sql
        );
        connection
            .execute(&sql, params![seed.field])
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }

    connection
        .execute_batch(
            "CREATE INDEX temp_discovery_values_field_scope_value_idx
               ON temp_discovery_values(field, record_family, value);
             CREATE INDEX temp_discovery_values_field_value_idx
               ON temp_discovery_values(field, value);
             CREATE INDEX temp_discovery_values_field_scope_numeric_idx
               ON temp_discovery_values(field, record_family, numeric_value);
             CREATE INDEX temp_discovery_values_field_record_idx
               ON temp_discovery_values(field, record_key);
             CREATE INDEX temp_discovery_values_field_scope_record_idx
               ON temp_discovery_values(field, record_family, record_key);",
        )
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    super::progress(
        "filter_discovery_values",
        total,
        total,
        "Staged filter discovery values".to_string(),
    );
    Ok(())
}

pub(super) fn drop_discovery_values(connection: &Connection) -> Result<(), IndexWriteError> {
    connection
        .execute_batch("DROP TABLE IF EXISTS temp_discovery_values;")
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}
