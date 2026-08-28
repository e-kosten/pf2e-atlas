use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::MetricRow;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper, SqliteConnection};

use crate::schema::record_metrics;

use super::RecordLoadError;

pub(super) fn read_metrics(
    connection: &mut SqliteConnection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let rows = record_metrics::table
        .select(RecordMetricRow::as_select())
        .order((
            record_metrics::record_key.asc(),
            record_metrics::ordinal.asc(),
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
            record_metrics::ordinal.asc(),
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
    ordinal: i64,
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
        let expected = metrics.get(&row.record_key).map_or(0, Vec::len);
        let expected = i64::try_from(expected).map_err(|_| {
            RecordLoadError::InvalidData(format!(
                "record_metrics[{}] has too many rows",
                row.record_key
            ))
        })?;
        if row.ordinal != expected {
            return Err(RecordLoadError::InvalidData(format!(
                "record_metrics[{}] expected ordinal {expected}, found {}",
                row.record_key, row.ordinal
            )));
        }
        let metric = super::children::metric_from_storage(
            &row.metric_domain,
            row.metric_key,
            &row.value_type,
            row.number_value,
            row.text_value,
            row.bool_value,
        )
        .map_err(RecordLoadError::InvalidData)?;
        metrics.entry(row.record_key).or_default().push(metric);
    }
    Ok(metrics)
}
