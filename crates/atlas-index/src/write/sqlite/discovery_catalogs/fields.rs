use std::collections::BTreeMap;

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::{QueryableByName, SqliteConnection, sql_query};

use crate::IndexWriteError;
use crate::discovery::definitions::{
    DISCOVERY_ALL_KINDS as ALL_KINDS, DISCOVERY_FIELD_DEFINITIONS as FIELD_SEEDS,
    DiscoveryFieldDefinition as FieldCatalogSeed,
};

use super::super::models::FilterFieldCatalogRow;
use super::stats::FieldStats;

pub(super) fn write_field_catalogs(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    let all_stats = collect_all_stats(connection)?;
    let total = FIELD_SEEDS.len() as u64;
    let mut rows = Vec::new();
    for (index, seed) in FIELD_SEEDS.iter().enumerate() {
        super::progress(
            "filter_field_catalogs",
            index as u64,
            total,
            format!("Writing filter field catalog: {}", seed.field),
        );
        let stats_by_scope = all_stats.get(seed.field);
        for kind in seed.applicable_kinds {
            if let Some(stats) = stats_by_scope
                .and_then(|stats| stats.get(&Some(*kind)))
                .copied()
                && stats.value_count > 0
            {
                rows.push(row_with_stats(seed, Some(*kind), stats)?);
            }
        }
        if let Some(stats) = stats_by_scope.and_then(|stats| stats.get(&None)).copied()
            && stats.value_count > 0
        {
            rows.push(row_with_stats(seed, None, stats)?);
        }
    }
    for rows in rows.chunks(super::super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::filter_field_catalog::table)
            .values(rows)
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
    record_kind: Option<&str>,
    stats: FieldStats,
) -> Result<FilterFieldCatalogRow, IndexWriteError> {
    Ok(FilterFieldCatalogRow {
        field: seed.field.to_string(),
        record_kind: record_kind.map(str::to_string),
        field_type: serde_json_string(seed.field_type)?,
        field_group: serde_json_string(seed.group)?,
        value_policy: serde_json_string(seed.value_policy)?,
        operators_json: serde_json::to_string(seed.operators)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
        cli_flags_json: serde_json::to_string(seed.cli_flags)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
        applicable_kinds_json: serde_json::to_string(seed.applicable_kinds)
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

fn collect_all_stats(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, BTreeMap<Option<&'static str>, FieldStats>>, IndexWriteError> {
    super::progress(
        "filter_field_catalogs",
        0,
        1,
        "Collecting filter field catalog stats".to_string(),
    );
    let matching_counts = matching_counts(connection)?;
    let observed_counts = observed_counts(connection)?;
    let rows = sql_query(
        "SELECT field,
                NULL AS record_kind,
                COALESCE(SUM(value_count), 0) AS value_count,
                COUNT(*) AS distinct_count,
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END) AS singleton_count
         FROM (
            SELECT field, value, COUNT(*) AS value_count
            FROM temp_discovery_values
            WHERE value IS NOT NULL AND value <> ''
            GROUP BY field, value
         )
         GROUP BY field
         UNION ALL
         SELECT field,
                record_kind,
                COALESCE(SUM(value_count), 0) AS value_count,
                COUNT(*) AS distinct_count,
                SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END) AS singleton_count
         FROM (
            SELECT field, record_kind, value, COUNT(*) AS value_count
            FROM temp_discovery_values
            WHERE value IS NOT NULL AND value <> ''
            GROUP BY field, record_kind, value
         )
         GROUP BY field, record_kind",
    )
    .load::<FieldAggregateRow>(connection)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut stats = BTreeMap::<String, BTreeMap<Option<&'static str>, FieldStats>>::new();
    for row in rows {
        let scope = row.record_kind.as_deref().and_then(known_kind);
        let matching_record_count = matching_counts.get(&scope).copied().unwrap_or_default();
        let observed_record_count = observed_counts
            .get(&(row.field.clone(), scope))
            .copied()
            .unwrap_or_default();
        stats.entry(row.field).or_default().insert(
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
struct FieldAggregateRow {
    #[diesel(sql_type = Text)]
    field: String,
    #[diesel(sql_type = Nullable<Text>)]
    record_kind: Option<String>,
    #[diesel(sql_type = BigInt)]
    value_count: i64,
    #[diesel(sql_type = BigInt)]
    distinct_count: i64,
    #[diesel(sql_type = BigInt)]
    singleton_count: i64,
}

fn observed_counts(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<(String, Option<&'static str>), u64>, IndexWriteError> {
    let rows = sql_query(
        "SELECT field, NULL AS record_kind, COUNT(DISTINCT record_key) AS observed_count
         FROM temp_discovery_values
         WHERE value IS NOT NULL AND value <> ''
         GROUP BY field
         UNION ALL
         SELECT field, record_kind, COUNT(DISTINCT record_key) AS observed_count
         FROM temp_discovery_values
         WHERE value IS NOT NULL AND value <> ''
         GROUP BY field, record_kind",
    )
    .load::<ObservedCountRow>(connection)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        counts.insert(
            (row.field, row.record_kind.as_deref().and_then(known_kind)),
            non_negative_u64(row.observed_count, "observed_count")?,
        );
    }
    Ok(counts)
}

#[derive(QueryableByName)]
struct ObservedCountRow {
    #[diesel(sql_type = Text)]
    field: String,
    #[diesel(sql_type = Nullable<Text>)]
    record_kind: Option<String>,
    #[diesel(sql_type = BigInt)]
    observed_count: i64,
}

fn matching_counts(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<Option<&'static str>, u64>, IndexWriteError> {
    let rows = sql_query(
        "SELECT NULL AS record_kind, COUNT(*) AS matching_count
         FROM records
         WHERE is_default_visible = 1
         UNION ALL
         SELECT record_kind, COUNT(*) AS matching_count
         FROM records
         WHERE is_default_visible = 1
         GROUP BY record_kind",
    )
    .load::<MatchingCountRow>(connection)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        counts.insert(
            row.record_kind.as_deref().and_then(known_kind),
            non_negative_u64(row.matching_count, "matching_count")?,
        );
    }
    Ok(counts)
}

#[derive(QueryableByName)]
struct MatchingCountRow {
    #[diesel(sql_type = Nullable<Text>)]
    record_kind: Option<String>,
    #[diesel(sql_type = BigInt)]
    matching_count: i64,
}

pub(super) fn known_kind(value: &str) -> Option<&'static str> {
    ALL_KINDS.iter().copied().find(|kind| *kind == value)
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
