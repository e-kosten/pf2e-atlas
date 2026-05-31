use std::collections::BTreeMap;

use atlas_domain::{MetricValueType, RecordKey};
use atlas_record::{MetricRow, MetricValue};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::record_metrics;

use super::RecordLoadError;
use super::parse::{parse_metric_domain, parse_metric_value_type};

pub(super) fn read_metrics(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let rows = record_metrics::table
        .select(RecordMetricRow::as_select())
        .order((
            record_metrics::record_key.asc(),
            record_metrics::metric_domain.asc(),
            record_metrics::metric_key.asc(),
        ))
        .load::<RecordMetricRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    metrics_from_rows(rows)
}

pub(super) fn read_metrics_by_keys(
    connection: &mut SqliteConnection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let key_strings = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let rows = record_metrics::table
        .filter(record_metrics::record_key.eq_any(key_strings))
        .select(RecordMetricRow::as_select())
        .order((
            record_metrics::record_key.asc(),
            record_metrics::metric_domain.asc(),
            record_metrics::metric_key.asc(),
        ))
        .load::<RecordMetricRow>(connection)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    metrics_from_rows(rows)
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = record_metrics)]
#[diesel(check_for_backend(Sqlite))]
struct RecordMetricRow {
    record_key: String,
    metric_domain: String,
    metric_key: String,
    value_type: String,
    number_value: Option<f64>,
    text_value: Option<String>,
    bool_value: Option<bool>,
}

fn metrics_from_rows(
    rows: Vec<RecordMetricRow>,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let mut metrics: BTreeMap<String, Vec<MetricRow>> = BTreeMap::new();
    for row in rows {
        let domain = parse_metric_domain(&row.metric_domain)?;
        let value = match parse_metric_value_type(&row.value_type)? {
            MetricValueType::Number => MetricValue::Number(required_metric_value(
                "record_metrics.number_value",
                row.number_value,
            )?),
            MetricValueType::Text => MetricValue::Text(required_metric_value(
                "record_metrics.text_value",
                row.text_value,
            )?),
            MetricValueType::Boolean => MetricValue::Boolean(required_metric_value(
                "record_metrics.bool_value",
                row.bool_value,
            )?),
        };
        metrics.entry(row.record_key).or_default().push(MetricRow {
            domain,
            key: row.metric_key,
            value,
        });
    }
    Ok(metrics)
}

fn required_metric_value<T>(field: &'static str, value: Option<T>) -> Result<T, RecordLoadError> {
    value
        .ok_or_else(|| RecordLoadError::InvalidData(format!("required field `{field}` is missing")))
}
