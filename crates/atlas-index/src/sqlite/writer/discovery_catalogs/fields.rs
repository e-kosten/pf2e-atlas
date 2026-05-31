use std::collections::BTreeMap;

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::{QueryableByName, SqliteConnection, sql_query};

use crate::IndexWriteError;

use super::super::models::FilterFieldCatalogRow;
use super::field_seeds::{ALL_FAMILIES, FIELD_SEEDS, FieldCatalogSeed};
use super::stats::FieldStats;

pub(super) fn write_field_catalogs(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    let total = FIELD_SEEDS.len() as u64;
    let mut rows = Vec::new();
    for (index, seed) in FIELD_SEEDS.iter().enumerate() {
        super::progress(
            "filter_field_catalogs",
            index as u64,
            total,
            format!("Writing filter field catalog: {}", seed.field),
        );
        let stats_by_scope = collect_stats(connection, seed.value_sql)?;
        for family in seed.applicable_families {
            if let Some(stats) = stats_by_scope.get(&Some(*family)).copied()
                && stats.value_count > 0
            {
                rows.push(row_with_stats(seed, Some(*family), stats)?);
            }
        }
        if let Some(stats) = stats_by_scope.get(&None).copied()
            && stats.value_count > 0
        {
            rows.push(row_with_stats(seed, None, stats)?);
        }
    }
    if !rows.is_empty() {
        diesel::insert_into(crate::schema::filter_field_catalog::table)
            .values(&rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    super::progress(
        "filter_field_catalogs",
        total,
        total,
        "Wrote filter field catalogs".to_string(),
    );
    Ok(())
}

fn row_with_stats(
    seed: &FieldCatalogSeed,
    record_family: Option<&str>,
    stats: FieldStats,
) -> Result<FilterFieldCatalogRow, IndexWriteError> {
    Ok(FilterFieldCatalogRow {
        field: seed.field.to_string(),
        record_family: record_family.map(str::to_string),
        field_type: serde_json_string(seed.field_type)?,
        field_group: serde_json_string(seed.group)?,
        value_policy: serde_json_string(seed.value_policy)?,
        operators_json: serde_json::to_string(seed.operators)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
        cli_flags_json: serde_json::to_string(seed.cli_flags)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
        applicable_families_json: serde_json::to_string(seed.applicable_families)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
        value_count: count_to_i64(stats.value_count, "filter_field_catalog.value_count")?,
        matching_record_count: count_to_i64(
            stats.matching_record_count,
            "filter_field_catalog.matching_record_count",
        )?,
        null_count: count_to_i64(stats.null_count, "filter_field_catalog.null_count")?,
        distinct_count: count_to_i64(stats.distinct_count, "filter_field_catalog.distinct_count")?,
        singleton_count: count_to_i64(
            stats.singleton_count,
            "filter_field_catalog.singleton_count",
        )?,
        singleton_ratio: Some(stats.singleton_ratio()),
        observation_singleton_ratio: Some(stats.observation_singleton_ratio()),
        policy_reason: seed.policy_reason.to_string(),
    })
}

pub(super) fn collect_stats(
    connection: &mut SqliteConnection,
    value_sql: &str,
) -> Result<BTreeMap<Option<&'static str>, FieldStats>, IndexWriteError> {
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
                COALESCE(SUM(value_count), 0) AS value_count,
                COUNT(*) AS distinct_count,
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END) AS singleton_count,
                COALESCE((
                    SELECT observed_record_count
                    FROM observed_records observed
                    WHERE (observed.record_family IS NULL AND counts.record_family IS NULL)
                       OR observed.record_family = counts.record_family
                ), 0) AS observed_record_count
         FROM counts
         GROUP BY record_family"
    );
    let rows = sql_query(sql)
        .load::<FieldStatsRow>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut stats = BTreeMap::new();
    for row in rows {
        let scope = row.record_family.as_deref().and_then(known_family);
        let matching_record_count = matching_count(connection, scope)?;
        let observed_record_count = u64::try_from(row.observed_record_count).map_err(|_| {
            IndexWriteError::WriteFailed("observed record count was negative".to_string())
        })?;
        stats.insert(
            scope,
            FieldStats {
                value_count: non_negative_u64(row.value_count, "value_count")?,
                distinct_count: non_negative_u64(row.distinct_count, "distinct_count")?,
                singleton_count: non_negative_u64(row.singleton_count, "singleton_count")?,
                matching_record_count,
                null_count: matching_record_count.saturating_sub(observed_record_count),
            },
        );
    }
    Ok(stats)
}

#[derive(QueryableByName)]
struct FieldStatsRow {
    #[diesel(sql_type = Nullable<Text>)]
    record_family: Option<String>,
    #[diesel(sql_type = BigInt)]
    value_count: i64,
    #[diesel(sql_type = BigInt)]
    distinct_count: i64,
    #[diesel(sql_type = BigInt)]
    singleton_count: i64,
    #[diesel(sql_type = BigInt)]
    observed_record_count: i64,
}

fn matching_count(
    connection: &mut SqliteConnection,
    family: Option<&str>,
) -> Result<u64, IndexWriteError> {
    use crate::schema::records;

    let query = records::table
        .filter(records::is_default_visible.eq(true))
        .into_boxed();
    let count = if let Some(family) = family {
        query
            .filter(records::record_family.eq(family))
            .count()
            .get_result::<i64>(connection)
    } else {
        query.count().get_result::<i64>(connection)
    }
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    non_negative_u64(count, "matching_count")
}

pub(super) fn known_family(value: &str) -> Option<&'static str> {
    ALL_FAMILIES.iter().copied().find(|family| *family == value)
}

pub(super) fn count_to_i64(count: u64, field: &'static str) -> Result<i64, IndexWriteError> {
    i64::try_from(count)
        .map_err(|_| IndexWriteError::WriteFailed(format!("{field} does not fit in i64")))
}

pub(super) fn non_negative_u64(value: i64, field: &'static str) -> Result<u64, IndexWriteError> {
    u64::try_from(value).map_err(|_| IndexWriteError::WriteFailed(format!("{field} was negative")))
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
