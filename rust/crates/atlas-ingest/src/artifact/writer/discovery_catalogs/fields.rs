use std::collections::BTreeMap;

use atlas_artifact::schema::filter_field_catalog_insert_sql;
use rusqlite::{Connection, params};

use crate::error::IngestError;

use super::stats::FieldStats;

use super::field_seeds::{ALL_FAMILIES, FIELD_SEEDS, FieldCatalogSeed};
pub(super) fn write_field_catalogs(connection: &Connection) -> Result<(), IngestError> {
    let insert_sql = filter_field_catalog_insert_sql();
    for seed in FIELD_SEEDS {
        let stats_by_scope = collect_stats(connection, seed.value_sql)?;
        for family in seed.applicable_families {
            if let Some(stats) = stats_by_scope.get(&Some(*family)).copied()
                && stats.value_count > 0
            {
                write_scope_with_stats(connection, &insert_sql, seed, Some(*family), stats)?;
            }
        }
        if let Some(stats) = stats_by_scope.get(&None).copied()
            && stats.value_count > 0
        {
            write_scope_with_stats(connection, &insert_sql, seed, None, stats)?;
        }
    }
    Ok(())
}

fn write_scope_with_stats(
    connection: &Connection,
    insert_sql: &str,
    seed: &FieldCatalogSeed,
    record_family: Option<&str>,
    stats: FieldStats,
) -> Result<(), IngestError> {
    connection
        .execute(
            insert_sql,
            params![
                seed.field,
                record_family,
                serde_json_string(seed.field_type),
                serde_json_string(seed.group),
                serde_json_string(seed.value_policy),
                serde_json::to_string(seed.operators).expect("operators serialize"),
                serde_json::to_string(seed.cli_flags).expect("cli flags serialize"),
                serde_json::to_string(seed.applicable_families).expect("families serialize"),
                stats.value_count,
                stats.matching_record_count,
                stats.null_count,
                stats.distinct_count,
                stats.singleton_count,
                stats.singleton_ratio(),
                stats.observation_singleton_ratio(),
                seed.policy_reason,
            ],
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    Ok(())
}

pub(super) fn collect_stats(
    connection: &Connection,
    value_sql: &str,
) -> Result<BTreeMap<Option<&'static str>, FieldStats>, IngestError> {
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql}),
              scoped_values AS (
                SELECT NULL AS record_family, r.record_key, value
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
                UNION ALL
                SELECT r.record_family, r.record_key, value
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
              ),
              counts AS (
                SELECT NULL AS record_family, value, COUNT(*) AS value_count
                FROM scoped_values
                WHERE record_family IS NULL
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
                GROUP BY value
                UNION ALL
                SELECT record_family, value, COUNT(*) AS value_count
                FROM scoped_values
                WHERE record_family IS NOT NULL
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
                GROUP BY record_family, value
              ),
              observed_records AS (
                SELECT record_family, COUNT(DISTINCT record_key) AS observed_record_count
                FROM scoped_values
                WHERE value IS NOT NULL AND CAST(value AS TEXT) <> ''
                GROUP BY record_family
              )
         SELECT record_family,
                COALESCE(SUM(value_count), 0),
                COUNT(*),
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END),
                COALESCE((
                    SELECT observed_record_count
                    FROM observed_records observed
                    WHERE (observed.record_family IS NULL AND counts.record_family IS NULL)
                       OR observed.record_family = counts.record_family
                ), 0)
         FROM counts
         GROUP BY record_family"
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                FieldStats {
                    value_count: row.get(1)?,
                    distinct_count: row.get(2)?,
                    singleton_count: row.get(3)?,
                    matching_record_count: 0,
                    null_count: 0,
                },
                row.get::<_, u64>(4)?,
            ))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut stats = BTreeMap::new();
    for row in rows {
        let (scope, mut field_stats, observed_record_count) =
            row.map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        let scope = scope.and_then(|value| known_family(value.as_str()));
        let matching_record_count = matching_count(connection, scope)?;
        field_stats.matching_record_count = matching_record_count;
        field_stats.null_count = matching_record_count.saturating_sub(observed_record_count);
        stats.insert(scope, field_stats);
    }
    Ok(stats)
}

fn matching_count(connection: &Connection, family: Option<&str>) -> Result<u64, IngestError> {
    match family {
        Some(family) => connection
            .query_row(
                "SELECT COUNT(*) FROM records WHERE is_default_visible = 1 AND record_family = ?1",
                params![family],
                |row| row.get(0),
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string())),
        None => connection
            .query_row(
                "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string())),
    }
}

pub(super) fn known_family(value: &str) -> Option<&'static str> {
    ALL_FAMILIES.iter().copied().find(|family| *family == value)
}

fn serde_json_string<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .expect("discovery field metadata should serialize as a string")
}
