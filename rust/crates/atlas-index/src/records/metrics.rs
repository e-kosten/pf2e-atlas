use std::collections::BTreeMap;

use atlas_artifact::schema::record_metric_select_sql;
use atlas_domain::MetricValueType;
use atlas_record::{MetricRow, MetricValue};
use rusqlite::Connection;

use super::RecordLoadError;
use super::parse::{
    bool_column, parse_metric_domain, parse_metric_value_type, required_f64, required_string,
};

pub(super) fn read_metrics(
    connection: &Connection,
) -> Result<BTreeMap<String, Vec<MetricRow>>, RecordLoadError> {
    let mut statement = connection
        .prepare(&record_metric_select_sql())
        .map_err(|error| RecordLoadError::QueryFailed(error.to_string()))?;
    let mut metrics: BTreeMap<String, Vec<MetricRow>> = BTreeMap::new();
    let mut rows = statement
        .query([])
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
