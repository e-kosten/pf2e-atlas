use atlas_domain::FilterValuePolicy;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Double, Nullable, Text};
use diesel::{QueryableByName, SqliteConnection, sql_query};

use crate::IndexWriteError;

use super::super::models::{
    FilterNumericCatalogRow, FilterSampleCatalogRow, FilterValueCatalogRow,
};
use super::field_seeds::{ALL_FAMILIES, FIELD_SEEDS, FieldCatalogSeed};
use super::fields::{count_to_i64, known_family, non_negative_u64};

const SAMPLE_LIMIT: usize = 100;

pub(super) fn write_value_catalogs(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    let catalog_seeds = FIELD_SEEDS
        .iter()
        .filter(|seed| {
            matches!(
                seed.value_policy,
                FilterValuePolicy::Enumerable
                    | FilterValuePolicy::Sample
                    | FilterValuePolicy::NumericStats
            )
        })
        .collect::<Vec<_>>();
    let total = catalog_seeds.len() as u64;
    let mut value_rows = Vec::new();
    let mut sample_rows = Vec::new();
    let mut numeric_rows = Vec::new();
    for (index, seed) in catalog_seeds.iter().enumerate() {
        super::progress(
            "filter_value_catalogs",
            index as u64,
            total,
            format!("Writing filter value catalog: {}", seed.field),
        );
        match seed.value_policy {
            FilterValuePolicy::Enumerable => {
                value_rows.extend(discrete_value_rows(connection, seed)?)
            }
            FilterValuePolicy::Sample => sample_rows.extend(sample_value_rows(connection, seed)?),
            FilterValuePolicy::NumericStats => {
                numeric_rows.extend(numeric_value_rows(connection, seed)?)
            }
            _ => {}
        }
    }
    super::progress(
        "filter_value_catalogs",
        total,
        total,
        "Wrote filter value catalogs".to_string(),
    );
    numeric_rows.extend(metric_numeric_value_rows(connection)?);

    for rows in value_rows.chunks(super::super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::filter_value_catalog::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in sample_rows.chunks(super::super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::filter_sample_catalog::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in numeric_rows.chunks(super::super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::filter_numeric_catalog::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn discrete_value_rows(
    connection: &mut SqliteConnection,
    seed: &FieldCatalogSeed,
) -> Result<Vec<FilterValueCatalogRow>, IndexWriteError> {
    collect_counts(connection, seed.value_sql)?
        .into_iter()
        .map(|row| {
            Ok(FilterValueCatalogRow {
                field: seed.field.to_string(),
                record_family: row.record_family.map(str::to_string),
                value: row.value,
                catalog_count: count_to_i64(row.count, "filter_value_catalog.catalog_count")?,
            })
        })
        .collect()
}

fn sample_value_rows(
    connection: &mut SqliteConnection,
    seed: &FieldCatalogSeed,
) -> Result<Vec<FilterSampleCatalogRow>, IndexWriteError> {
    let mut counts = collect_counts(connection, seed.value_sql)?;
    counts.sort_by(|left, right| {
        left.record_family
            .cmp(&right.record_family)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.value.cmp(&right.value))
    });
    let mut current_scope = None::<Option<&'static str>>;
    let mut rank = 0_u64;
    let mut rows = Vec::new();
    for row in counts {
        if current_scope != Some(row.record_family) {
            current_scope = Some(row.record_family);
            rank = 0;
        }
        if rank >= SAMPLE_LIMIT as u64 {
            continue;
        }
        rank += 1;
        rows.push(FilterSampleCatalogRow {
            field: seed.field.to_string(),
            record_family: row.record_family.map(str::to_string),
            value: row.value,
            catalog_count: count_to_i64(row.count, "filter_sample_catalog.catalog_count")?,
            sample_rank: count_to_i64(rank, "filter_sample_catalog.sample_rank")?,
        });
    }
    Ok(rows)
}

fn numeric_value_rows(
    connection: &mut SqliteConnection,
    seed: &FieldCatalogSeed,
) -> Result<Vec<FilterNumericCatalogRow>, IndexWriteError> {
    collect_numeric_stats(connection, seed.value_sql)?
        .into_iter()
        .map(|row| {
            Ok(FilterNumericCatalogRow {
                field: seed.field.to_string(),
                record_family: row.record_family.map(str::to_string),
                metric_domain: None,
                metric_key: None,
                catalog_count: count_to_i64(row.count, "filter_numeric_catalog.catalog_count")?,
                null_count: count_to_i64(row.null_count, "filter_numeric_catalog.null_count")?,
                min: row.min,
                p05: row.p05,
                p25: row.p25,
                p50: row.p50,
                mean: row.mean,
                p75: row.p75,
                p95: row.p95,
                max: row.max,
            })
        })
        .collect()
}

fn metric_numeric_value_rows(
    connection: &mut SqliteConnection,
) -> Result<Vec<FilterNumericCatalogRow>, IndexWriteError> {
    use crate::schema::record_metrics as rm;

    let metric_rows = rm::table
        .filter(rm::value_type.eq("number"))
        .select((rm::metric_domain, rm::metric_key))
        .distinct()
        .order((rm::metric_domain, rm::metric_key))
        .load::<(String, String)>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let total = metric_rows.len() as u64;
    let mut rows = Vec::new();
    for (index, (metric_domain, metric_key)) in metric_rows.iter().enumerate() {
        super::progress(
            "filter_metric_catalogs",
            index as u64,
            total,
            format!("Writing filter metric catalog: {metric_domain}.{metric_key}"),
        );
        for row in collect_metric_numeric_stats(connection, metric_domain, metric_key)? {
            rows.push(FilterNumericCatalogRow {
                field: "metric".to_string(),
                record_family: row.record_family.map(str::to_string),
                metric_domain: Some(metric_domain.clone()),
                metric_key: Some(metric_key.clone()),
                catalog_count: count_to_i64(row.count, "filter_numeric_catalog.catalog_count")?,
                null_count: count_to_i64(row.null_count, "filter_numeric_catalog.null_count")?,
                min: row.min,
                p05: row.p05,
                p25: row.p25,
                p50: row.p50,
                mean: row.mean,
                p75: row.p75,
                p95: row.p95,
                max: row.max,
            });
        }
    }
    super::progress(
        "filter_metric_catalogs",
        total,
        total,
        "Wrote filter metric catalogs".to_string(),
    );
    Ok(rows)
}

#[derive(Debug)]
struct CountRow {
    record_family: Option<&'static str>,
    value: String,
    count: u64,
}

fn collect_counts(
    connection: &mut SqliteConnection,
    value_sql: &str,
) -> Result<Vec<CountRow>, IndexWriteError> {
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql})
         SELECT NULL AS record_family, value, COUNT(*) AS catalog_count
         FROM field_values fv
         JOIN records r ON r.record_key = fv.record_key
         WHERE r.is_default_visible = 1 AND value IS NOT NULL AND CAST(value AS TEXT) <> ''
         GROUP BY value
         UNION ALL
         SELECT r.record_family, value, COUNT(*) AS catalog_count
         FROM field_values fv
         JOIN records r ON r.record_key = fv.record_key
         WHERE r.is_default_visible = 1 AND value IS NOT NULL AND CAST(value AS TEXT) <> ''
         GROUP BY r.record_family, value
         ORDER BY record_family, catalog_count DESC, value ASC"
    );
    sql_query(sql)
        .load::<CountQueryRow>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
        .into_iter()
        .map(|row| {
            Ok(CountRow {
                record_family: row.record_family.as_deref().and_then(known_family),
                value: row.value,
                count: non_negative_u64(row.catalog_count, "catalog_count")?,
            })
        })
        .collect()
}

#[derive(QueryableByName)]
struct CountQueryRow {
    #[diesel(sql_type = Nullable<Text>)]
    record_family: Option<String>,
    #[diesel(sql_type = Text)]
    value: String,
    #[diesel(sql_type = BigInt)]
    catalog_count: i64,
}

#[derive(Debug)]
struct NumericRow {
    record_family: Option<&'static str>,
    count: u64,
    null_count: u64,
    min: Option<f64>,
    p05: Option<f64>,
    p25: Option<f64>,
    p50: Option<f64>,
    mean: Option<f64>,
    p75: Option<f64>,
    p95: Option<f64>,
    max: Option<f64>,
}

fn collect_numeric_stats(
    connection: &mut SqliteConnection,
    value_sql: &str,
) -> Result<Vec<NumericRow>, IndexWriteError> {
    let mut rows = Vec::new();
    rows.push(numeric_scope(connection, value_sql, None)?);
    for family in ALL_FAMILIES {
        rows.push(numeric_scope(connection, value_sql, Some(*family))?);
    }
    Ok(rows.into_iter().flatten().collect())
}

fn collect_metric_numeric_stats(
    connection: &mut SqliteConnection,
    metric_domain: &str,
    metric_key: &str,
) -> Result<Vec<NumericRow>, IndexWriteError> {
    let mut rows = Vec::new();
    rows.push(metric_numeric_scope(
        connection,
        metric_domain,
        metric_key,
        None,
    )?);
    for family in ALL_FAMILIES {
        rows.push(metric_numeric_scope(
            connection,
            metric_domain,
            metric_key,
            Some(*family),
        )?);
    }
    Ok(rows.into_iter().flatten().collect())
}

fn metric_numeric_scope(
    connection: &mut SqliteConnection,
    metric_domain: &str,
    metric_key: &str,
    family: Option<&'static str>,
) -> Result<Option<NumericRow>, IndexWriteError> {
    let mut query =
        crate::schema::record_metrics::table
            .inner_join(crate::schema::records::table.on(
                crate::schema::record_metrics::record_key.eq(crate::schema::records::record_key),
            ))
            .filter(crate::schema::records::is_default_visible.eq(true))
            .filter(crate::schema::record_metrics::metric_domain.eq(metric_domain))
            .filter(crate::schema::record_metrics::metric_key.eq(metric_key))
            .filter(crate::schema::record_metrics::value_type.eq("number"))
            .filter(crate::schema::record_metrics::number_value.is_not_null())
            .select(crate::schema::record_metrics::number_value)
            .order(crate::schema::record_metrics::number_value.asc())
            .into_boxed();
    if let Some(family) = family {
        query = query.filter(crate::schema::records::record_family.eq(family));
    }
    let values = query
        .load::<Option<f64>>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    numeric_row_from_values(connection, family, values)
}

fn numeric_scope(
    connection: &mut SqliteConnection,
    value_sql: &str,
    family: Option<&'static str>,
) -> Result<Option<NumericRow>, IndexWriteError> {
    let family_predicate = if family.is_some() {
        "AND r.record_family = ?"
    } else {
        ""
    };
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql})
         SELECT value
         FROM field_values fv
         JOIN records r ON r.record_key = fv.record_key
         WHERE r.is_default_visible = 1
           {family_predicate}
           AND value IS NOT NULL
         ORDER BY value ASC"
    );
    let mut query = sql_query(sql).into_boxed();
    if let Some(family) = family {
        query = query.bind::<Text, _>(family);
    }
    let values = query
        .load::<NumericValueRow>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
        .into_iter()
        .map(|row| row.value)
        .collect::<Vec<_>>();
    numeric_row_from_values(connection, family, values)
}

#[derive(QueryableByName)]
struct NumericValueRow {
    #[diesel(sql_type = Double)]
    value: f64,
}

fn numeric_row_from_values(
    connection: &mut SqliteConnection,
    family: Option<&'static str>,
    values: Vec<f64>,
) -> Result<Option<NumericRow>, IndexWriteError> {
    if values.is_empty() {
        return Ok(None);
    }
    let matching_count = matching_count(connection, family)?;
    Ok(Some(NumericRow {
        record_family: family,
        count: values.len() as u64,
        null_count: matching_count.saturating_sub(values.len() as u64),
        min: values.first().copied(),
        p05: percentile(&values, 0.05),
        p25: percentile(&values, 0.25),
        p50: percentile(&values, 0.50),
        mean: Some(values.iter().sum::<f64>() / values.len() as f64),
        p75: percentile(&values, 0.75),
        p95: percentile(&values, 0.95),
        max: values.last().copied(),
    }))
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

fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let rank = ((sorted_values.len() as f64) * percentile).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values.get(index).copied()
}
