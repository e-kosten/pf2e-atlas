use std::collections::BTreeMap;

use atlas_artifact::schema::filter_field_catalog_insert_sql;
use rusqlite::{Connection, params};

use crate::IndexWriteError;

use super::stats::FieldStats;

use super::field_seeds::{ALL_FAMILIES, FIELD_SEEDS, FieldCatalogSeed};
pub(super) fn write_field_catalogs(connection: &Connection) -> Result<(), IndexWriteError> {
    let insert_sql = filter_field_catalog_insert_sql();
    let all_stats = collect_all_stats(connection)?;
    let total = FIELD_SEEDS.len() as u64;
    for (index, seed) in FIELD_SEEDS.iter().enumerate() {
        super::progress(
            "filter_field_catalogs",
            index as u64,
            total,
            format!("Writing filter field catalog: {}", seed.field),
        );
        let stats_by_scope = all_stats.get(seed.field);
        for family in seed.applicable_families {
            if let Some(stats) = stats_by_scope
                .and_then(|stats| stats.get(&Some(*family)))
                .copied()
                && stats.value_count > 0
            {
                write_scope_with_stats(connection, &insert_sql, seed, Some(*family), stats)?;
            }
        }
        if let Some(stats) = stats_by_scope.and_then(|stats| stats.get(&None)).copied()
            && stats.value_count > 0
        {
            write_scope_with_stats(connection, &insert_sql, seed, None, stats)?;
        }
    }
    super::progress(
        "filter_field_catalogs",
        total,
        total,
        "Wrote filter field catalogs".to_string(),
    );
    Ok(())
}

fn write_scope_with_stats(
    connection: &Connection,
    insert_sql: &str,
    seed: &FieldCatalogSeed,
    record_family: Option<&str>,
    stats: FieldStats,
) -> Result<(), IndexWriteError> {
    connection
        .execute(
            insert_sql,
            params![
                seed.field,
                record_family,
                serde_json_string(seed.field_type)?,
                serde_json_string(seed.group)?,
                serde_json_string(seed.value_policy)?,
                serde_json::to_string(seed.operators)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                serde_json::to_string(seed.cli_flags)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
                serde_json::to_string(seed.applicable_families)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
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
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    Ok(())
}

fn collect_all_stats(
    connection: &Connection,
) -> Result<BTreeMap<String, BTreeMap<Option<&'static str>, FieldStats>>, IndexWriteError> {
    super::progress(
        "filter_field_catalogs",
        0,
        1,
        "Collecting filter field catalog stats".to_string(),
    );
    let matching_counts = matching_counts(connection)?;
    let observed_counts = observed_counts(connection)?;
    let sql = "SELECT field,
                NULL AS record_family,
                COALESCE(SUM(value_count), 0),
                COUNT(*),
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END)
         FROM (
            SELECT field, value, COUNT(*) AS value_count
            FROM temp_discovery_values
            WHERE value IS NOT NULL AND value <> ''
            GROUP BY field, value
         )
         GROUP BY field
         UNION ALL
         SELECT field,
                record_family,
                COALESCE(SUM(value_count), 0),
                COUNT(*),
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END)
         FROM (
            SELECT field, record_family, value, COUNT(*) AS value_count
            FROM temp_discovery_values
            WHERE value IS NOT NULL AND value <> ''
            GROUP BY field, record_family, value
         )
         GROUP BY field, record_family";
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                FieldStats {
                    value_count: row.get(2)?,
                    distinct_count: row.get(3)?,
                    singleton_count: row.get(4)?,
                    matching_record_count: 0,
                    null_count: 0,
                },
            ))
        })
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut stats = BTreeMap::<String, BTreeMap<Option<&'static str>, FieldStats>>::new();
    for row in rows {
        let (field, scope, mut field_stats) =
            row.map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let scope = scope.and_then(|value| known_family(value.as_str()));
        let matching_record_count = matching_counts.get(&scope).copied().unwrap_or_default();
        let observed_record_count = observed_counts
            .get(&(field.clone(), scope))
            .copied()
            .unwrap_or_default();
        field_stats.matching_record_count = matching_record_count;
        field_stats.null_count = matching_record_count.saturating_sub(observed_record_count);
        stats.entry(field).or_default().insert(scope, field_stats);
    }
    Ok(stats)
}

fn observed_counts(
    connection: &Connection,
) -> Result<BTreeMap<(String, Option<&'static str>), u64>, IndexWriteError> {
    let mut statement = connection
        .prepare(
            "SELECT field, NULL AS record_family, COUNT(DISTINCT record_key)
             FROM temp_discovery_values
             WHERE value IS NOT NULL AND value <> ''
             GROUP BY field
             UNION ALL
             SELECT field, record_family, COUNT(DISTINCT record_key)
             FROM temp_discovery_values
             WHERE value IS NOT NULL AND value <> ''
             GROUP BY field, record_family",
        )
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, u64>(2)?,
            ))
        })
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (field, scope, count) =
            row.map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        counts.insert(
            (field, scope.and_then(|value| known_family(value.as_str()))),
            count,
        );
    }
    Ok(counts)
}

fn matching_counts(
    connection: &Connection,
) -> Result<BTreeMap<Option<&'static str>, u64>, IndexWriteError> {
    let mut statement = connection
        .prepare(
            "SELECT NULL AS record_family, COUNT(*)
             FROM records
             WHERE is_default_visible = 1
             UNION ALL
             SELECT record_family, COUNT(*)
             FROM records
             WHERE is_default_visible = 1
             GROUP BY record_family",
        )
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, Option<String>>(0)?, row.get::<_, u64>(1)?))
        })
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (scope, count) =
            row.map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        counts.insert(scope.and_then(|value| known_family(value.as_str())), count);
    }
    Ok(counts)
}

pub(super) fn known_family(value: &str) -> Option<&'static str> {
    ALL_FAMILIES.iter().copied().find(|family| *family == value)
}

fn serde_json_string<T: serde::Serialize>(value: T) -> Result<String, IndexWriteError> {
    let value = serde_json::to_value(value)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    value.as_str().map(str::to_string).ok_or_else(|| {
        IndexWriteError::WriteFailed(
            "discovery field metadata did not serialize as a string".to_string(),
        )
    })
}
