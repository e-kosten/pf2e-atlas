use atlas_domain::{MetricDomain, MetricValueType};
use atlas_record::{MetricRow, MetricValue};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) fn metric_order_digest(values: &[MetricRow]) -> Result<String, String> {
    let payloads = values.iter().map(MetricPayload::from).collect::<Vec<_>>();
    let encoded = serde_json::to_vec(&payloads).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

#[derive(Serialize)]
struct MetricPayload {
    domain: MetricDomain,
    key: String,
    value: MetricValuePayload,
}

impl From<&MetricRow> for MetricPayload {
    fn from(value: &MetricRow) -> Self {
        Self {
            domain: value.domain,
            key: value.key.clone(),
            value: MetricValuePayload::from(&value.value),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum MetricValuePayload {
    Number(f64),
    Text(String),
    Boolean(bool),
}

impl From<&MetricValue> for MetricValuePayload {
    fn from(value: &MetricValue) -> Self {
        match value {
            MetricValue::Number(value) => Self::Number(*value),
            MetricValue::Text(value) => Self::Text(value.clone()),
            MetricValue::Boolean(value) => Self::Boolean(*value),
        }
    }
}

pub(crate) fn metric_from_storage(
    domain: &str,
    key: String,
    value_type: &str,
    number_value: Option<f64>,
    text_value: Option<String>,
    bool_value: Option<bool>,
) -> Result<MetricRow, String> {
    let domain = MetricDomain::from_canonical(domain)
        .ok_or_else(|| format!("record_metrics.metric_domain: invalid value `{domain}`"))?;
    let value = match MetricValueType::from_canonical(value_type)
        .ok_or_else(|| format!("record_metrics.value_type: invalid value `{value_type}`"))?
    {
        MetricValueType::Number => {
            MetricValue::Number(required("record_metrics.number_value", number_value)?)
        }
        MetricValueType::Text => {
            MetricValue::Text(required("record_metrics.text_value", text_value)?)
        }
        MetricValueType::Boolean => {
            MetricValue::Boolean(required("record_metrics.bool_value", bool_value)?)
        }
    };
    Ok(MetricRow { domain, key, value })
}

fn required<T>(path: &str, value: Option<T>) -> Result<T, String> {
    value.ok_or_else(|| format!("{path}: required value is missing"))
}
