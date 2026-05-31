use std::collections::BTreeMap;

use diesel::prelude::*;
use diesel::{ExpressionMethods, NullableExpressionMethods, SqliteConnection};

use super::models::{MetricKeyCatalogRow, MetricValueCatalogRow};
use crate::IndexWriteError;

#[derive(Debug, Clone, Queryable)]
struct MetricSourceRow {
    metric_domain: String,
    record_family: String,
    metric_key: String,
    value_type: String,
    number_value: Option<f64>,
    text_value: Option<String>,
    bool_value: Option<bool>,
}

pub(super) fn write_metric_catalogs(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    let source_rows = load_metric_source_rows(connection)?;
    let key_rows = metric_key_catalog_rows(&source_rows)?;
    let value_rows = metric_value_catalog_rows(&source_rows)?;

    if !key_rows.is_empty() {
        diesel::insert_into(crate::schema::metric_key_catalog::table)
            .values(&key_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    if !value_rows.is_empty() {
        diesel::insert_into(crate::schema::metric_value_catalog::table)
            .values(&value_rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn load_metric_source_rows(
    connection: &mut SqliteConnection,
) -> Result<Vec<MetricSourceRow>, IndexWriteError> {
    use crate::schema::record_metrics as rm;
    use crate::schema::records as r;

    rm::table
        .inner_join(r::table.on(rm::record_key.eq(r::record_key)))
        .filter(r::is_default_visible.eq(true))
        .select((
            rm::metric_domain,
            r::record_family,
            rm::metric_key,
            rm::value_type,
            rm::number_value,
            rm::text_value,
            rm::bool_value.nullable(),
        ))
        .load::<MetricSourceRow>(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn metric_key_catalog_rows(
    source_rows: &[MetricSourceRow],
) -> Result<Vec<MetricKeyCatalogRow>, IndexWriteError> {
    #[derive(Default)]
    struct Aggregate {
        count: usize,
        numeric_min: Option<f64>,
        numeric_max: Option<f64>,
    }

    let mut aggregates =
        BTreeMap::<(String, Option<String>, String, String, String), Aggregate>::new();
    for row in source_rows {
        for scope in [None, Some(row.record_family.clone())] {
            let key = (
                row.metric_domain.clone(),
                scope,
                namespace_prefix(&row.metric_key),
                row.metric_key.clone(),
                row.value_type.clone(),
            );
            let aggregate = aggregates.entry(key).or_default();
            aggregate.count += 1;
            if row.value_type == "number"
                && let Some(value) = row.number_value
            {
                aggregate.numeric_min = Some(
                    aggregate
                        .numeric_min
                        .map_or(value, |current| current.min(value)),
                );
                aggregate.numeric_max = Some(
                    aggregate
                        .numeric_max
                        .map_or(value, |current| current.max(value)),
                );
            }
        }
    }

    aggregates
        .into_iter()
        .map(
            |(
                (metric_domain, record_family, namespace_prefix, metric_key, value_type),
                aggregate,
            )| {
                Ok(MetricKeyCatalogRow {
                    metric_domain,
                    record_family,
                    namespace_prefix,
                    metric_key,
                    value_type,
                    catalog_count: count_to_i64(
                        aggregate.count,
                        "metric_key_catalog.catalog_count",
                    )?,
                    numeric_min: aggregate.numeric_min,
                    numeric_max: aggregate.numeric_max,
                })
            },
        )
        .collect()
}

fn metric_value_catalog_rows(
    source_rows: &[MetricSourceRow],
) -> Result<Vec<MetricValueCatalogRow>, IndexWriteError> {
    let mut aggregates = BTreeMap::<(String, Option<String>, String, String), usize>::new();
    for row in source_rows {
        let Some(value) = metric_catalog_value(row) else {
            continue;
        };
        for scope in [None, Some(row.record_family.clone())] {
            let key = (
                row.metric_domain.clone(),
                scope,
                row.metric_key.clone(),
                value.clone(),
            );
            *aggregates.entry(key).or_default() += 1;
        }
    }

    aggregates
        .into_iter()
        .map(
            |((metric_domain, record_family, metric_key, value), count)| {
                Ok(MetricValueCatalogRow {
                    metric_domain,
                    record_family,
                    metric_key,
                    value,
                    catalog_count: count_to_i64(count, "metric_value_catalog.catalog_count")?,
                })
            },
        )
        .collect()
}

fn metric_catalog_value(row: &MetricSourceRow) -> Option<String> {
    match row.value_type.as_str() {
        "text" => row.text_value.clone(),
        "boolean" => row
            .bool_value
            .map(|value| if value { "1" } else { "0" }.to_string()),
        _ => None,
    }
}

fn namespace_prefix(metric_key: &str) -> String {
    metric_key
        .split_once('.')
        .map(|(prefix, _)| format!("{prefix}."))
        .unwrap_or_default()
}

fn count_to_i64(count: usize, field: &'static str) -> Result<i64, IndexWriteError> {
    i64::try_from(count)
        .map_err(|_| IndexWriteError::WriteFailed(format!("{field} does not fit in i64")))
}
