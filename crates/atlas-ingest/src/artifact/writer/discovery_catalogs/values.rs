use atlas_artifact::schema::{
    filter_numeric_catalog_insert_sql, filter_sample_catalog_insert_sql,
    filter_value_catalog_insert_sql,
};
use atlas_domain::FilterValuePolicy;
use rusqlite::{Connection, params};

use crate::error::IngestError;

use super::field_seeds::{ALL_FAMILIES, FIELD_SEEDS, FieldCatalogSeed};
use super::fields::known_family;

const SAMPLE_LIMIT: usize = 100;

pub(super) fn write_value_catalogs(connection: &Connection) -> Result<(), IngestError> {
    let value_insert = filter_value_catalog_insert_sql();
    let sample_insert = filter_sample_catalog_insert_sql();
    let numeric_insert = filter_numeric_catalog_insert_sql();
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
    for (index, seed) in catalog_seeds.iter().enumerate() {
        super::progress(
            "filter_value_catalogs",
            index as u64,
            total,
            format!("Writing filter value catalog: {}", seed.field),
        );
        match seed.value_policy {
            FilterValuePolicy::Enumerable => {
                write_discrete_values(connection, &value_insert, seed)?
            }
            FilterValuePolicy::Sample => write_sample_values(connection, &sample_insert, seed)?,
            FilterValuePolicy::NumericStats => {
                write_numeric_values(connection, &numeric_insert, seed)?
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
    write_metric_numeric_values(connection, &numeric_insert)?;
    Ok(())
}

fn write_discrete_values(
    connection: &Connection,
    insert_sql: &str,
    seed: &FieldCatalogSeed,
) -> Result<(), IngestError> {
    for row in collect_counts(connection, seed.value_sql)? {
        connection
            .execute(
                insert_sql,
                params![seed.field, row.record_family, row.value, row.count],
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_sample_values(
    connection: &Connection,
    insert_sql: &str,
    seed: &FieldCatalogSeed,
) -> Result<(), IngestError> {
    let mut rows = collect_counts(connection, seed.value_sql)?;
    rows.sort_by(|left, right| {
        left.record_family
            .cmp(&right.record_family)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.value.cmp(&right.value))
    });
    let mut current_scope = None::<Option<&'static str>>;
    let mut rank = 0_u64;
    for row in rows {
        if current_scope != Some(row.record_family) {
            current_scope = Some(row.record_family);
            rank = 0;
        }
        if rank >= SAMPLE_LIMIT as u64 {
            continue;
        }
        rank += 1;
        connection
            .execute(
                insert_sql,
                params![seed.field, row.record_family, row.value, row.count, rank],
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_numeric_values(
    connection: &Connection,
    insert_sql: &str,
    seed: &FieldCatalogSeed,
) -> Result<(), IngestError> {
    for row in collect_numeric_stats(connection, seed.value_sql)? {
        connection
            .execute(
                insert_sql,
                params![
                    seed.field,
                    row.record_family,
                    Option::<String>::None,
                    Option::<String>::None,
                    row.count,
                    row.null_count,
                    row.min,
                    row.p05,
                    row.p25,
                    row.p50,
                    row.mean,
                    row.p75,
                    row.p95,
                    row.max,
                ],
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_metric_numeric_values(
    connection: &Connection,
    insert_sql: &str,
) -> Result<(), IngestError> {
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT metric_domain, metric_key
             FROM record_metrics
             WHERE value_type = 'number'
             ORDER BY metric_domain, metric_key",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let metric_rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let total = metric_rows.len() as u64;
    for (index, (metric_domain, metric_key)) in metric_rows.iter().enumerate() {
        super::progress(
            "filter_metric_catalogs",
            index as u64,
            total,
            format!("Writing filter metric catalog: {metric_domain}.{metric_key}"),
        );
        for row in collect_metric_numeric_stats(connection, metric_domain, metric_key)? {
            connection
                .execute(
                    insert_sql,
                    params![
                        "metric",
                        row.record_family,
                        metric_domain,
                        metric_key,
                        row.count,
                        row.null_count,
                        row.min,
                        row.p05,
                        row.p25,
                        row.p50,
                        row.mean,
                        row.p75,
                        row.p95,
                        row.max,
                    ],
                )
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
    }
    super::progress(
        "filter_metric_catalogs",
        total,
        total,
        "Wrote filter metric catalogs".to_string(),
    );
    Ok(())
}

#[derive(Debug)]
struct CountRow {
    record_family: Option<&'static str>,
    value: String,
    count: u64,
}

fn collect_counts(connection: &Connection, value_sql: &str) -> Result<Vec<CountRow>, IngestError> {
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
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u64>(2)?,
            ))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut counts = Vec::new();
    for row in rows {
        let (scope, value, count) =
            row.map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        counts.push(CountRow {
            record_family: scope.and_then(|value| known_family(value.as_str())),
            value,
            count,
        });
    }
    Ok(counts)
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
    connection: &Connection,
    value_sql: &str,
) -> Result<Vec<NumericRow>, IngestError> {
    let mut rows = Vec::new();
    rows.push(numeric_scope(connection, value_sql, None)?);
    for family in ALL_FAMILIES {
        rows.push(numeric_scope(connection, value_sql, Some(*family))?);
    }
    Ok(rows.into_iter().flatten().collect())
}

fn collect_metric_numeric_stats(
    connection: &Connection,
    metric_domain: &str,
    metric_key: &str,
) -> Result<Vec<NumericRow>, IngestError> {
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
    connection: &Connection,
    metric_domain: &str,
    metric_key: &str,
    family: Option<&'static str>,
) -> Result<Option<NumericRow>, IngestError> {
    let family_predicate = if family.is_some() {
        "AND r.record_family = ?3"
    } else {
        ""
    };
    let sql = format!(
        "SELECT rm.number_value
         FROM record_metrics rm
         JOIN records r ON r.record_key = rm.record_key
         WHERE r.is_default_visible = 1
           AND rm.metric_domain = ?1
           AND rm.metric_key = ?2
           AND rm.value_type = 'number'
           AND rm.number_value IS NOT NULL
           {family_predicate}
         ORDER BY rm.number_value ASC"
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let values = if let Some(family) = family {
        statement
            .query_map(params![metric_domain, metric_key, family], |row| {
                row.get::<_, f64>(0)
            })
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
    } else {
        statement
            .query_map(params![metric_domain, metric_key], |row| {
                row.get::<_, f64>(0)
            })
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
    };
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

fn numeric_scope(
    connection: &Connection,
    value_sql: &str,
    family: Option<&'static str>,
) -> Result<Option<NumericRow>, IngestError> {
    let family_predicate = if family.is_some() {
        "AND r.record_family = ?1"
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
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let values = if let Some(family) = family {
        statement
            .query_map(params![family], |row| row.get::<_, f64>(0))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
    } else {
        statement
            .query_map([], |row| row.get::<_, f64>(0))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
    };
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

fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let rank = ((sorted_values.len() as f64) * percentile).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values.get(index).copied()
}
