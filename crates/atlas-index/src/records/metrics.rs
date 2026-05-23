use std::collections::BTreeMap;

use atlas_artifact::schema::{RECORD_METRIC_COLUMNS, record_metric_select_sql, record_metrics};
use atlas_domain::{MetricValueType, RecordKey};
use atlas_record::{MetricRow, MetricValue};
use rusqlite::{Connection, params_from_iter, types::Value};

use super::RecordLoadError;
use super::parse::{
    bool_column, parse_metric_domain, parse_metric_value_type, required_f64, required_string,
};

pub(super) fn read_metrics(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    read_metrics_from_sql(connection, &record_metric_select_sql(), Vec::new())
}

pub(super) fn read_metrics_by_keys(
    connection: &Connection,
    keys: &[RecordKey],
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let parameters = key_parameters(keys);
    let sql = scoped_select_sql(
        record_metrics::TABLE.name(),
        RECORD_METRIC_COLUMNS,
        record_metrics::columns::RECORD_KEY.name(),
        &[
            record_metrics::columns::RECORD_KEY.name(),
            record_metrics::columns::METRIC_DOMAIN.name(),
            record_metrics::columns::METRIC_KEY.name(),
        ],
        parameters.len(),
    );
    read_metrics_from_sql(connection, &sql, parameters)
}

fn read_metrics_from_sql(
    connection: &Connection,
    sql: &str,
    parameters: Vec<Value>,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut metrics: BTreeMap<String, Vec<MetricRow>> = BTreeMap::new();
    let mut rows = statement
        .query(params_from_iter(parameters.iter()))
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?
    {
        let record_key = required_string(row, "record_key")?;
        let domain = parse_metric_domain(&required_string(row, "metric_domain")?)?;
        let key = required_string(row, "metric_key")?;
        let value_type = required_string(row, "value_type")?;
        let value = match parse_metric_value_type(&value_type)? {
            MetricValueType::Number => MetricValue::Number(required_f64(row, "number_value")?),
            MetricValueType::Text => MetricValue::Text(required_string(row, "text_value")?),
            MetricValueType::Boolean => {
                MetricValue::Boolean(bool_column("record_metrics.bool_value", row, "bool_value")?)
            }
        };
        let metric = MetricRow { domain, key, value };
        metrics.entry(record_key).or_default().push(metric);
    }
    Ok(metrics)
}

fn scoped_select_sql(
    table: &str,
    columns: &[atlas_artifact::schema::Column],
    key_column: &str,
    order_by: &[&str],
    key_count: usize,
) -> String {
    let placeholders = (1..=key_count)
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let columns = columns
        .iter()
        .map(|column| column.name())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "SELECT {columns} FROM {table} WHERE {key_column} IN ({placeholders}) ORDER BY {order_by}",
        order_by = order_by.join(", ")
    )
}

fn key_parameters(keys: &[RecordKey]) -> Vec<Value> {
    keys.iter()
        .map(|key| Value::Text(key.to_string()))
        .collect()
}
