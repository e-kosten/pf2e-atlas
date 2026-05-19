use std::collections::BTreeSet;

use atlas_domain::{MetricDomain, MetricValueType};
use atlas_record::metrics as metric_definitions;
use serde_json::Value;

use crate::records::{MetricRow, MetricValue};

mod actor;
mod disable;
mod emit;
mod item;
mod specs;
#[cfg(test)]
mod tests;
mod value;

pub(crate) use value::{
    first_number_like_at_paths, number_like_at_pointer, slugify_metric_segment,
};

pub(crate) fn extract_metrics(
    raw: &Value,
    document_type: &str,
    record_type: &str,
) -> Result<Vec<MetricRow>, String> {
    let metrics = match document_type {
        "Actor" => actor::extract_actor_metrics(raw),
        "Item" => item::extract_item_metrics(raw, record_type),
        _ => Vec::new(),
    };
    let metrics = dedupe_metrics(metrics);
    validate_metric_rows(&metrics)?;
    Ok(metrics)
}

fn dedupe_metrics(metrics: Vec<MetricRow>) -> Vec<MetricRow> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for metric in metrics.into_iter().rev() {
        if seen.insert((metric.domain, metric.key.clone())) {
            deduped.push(metric);
        }
    }
    deduped.reverse();
    deduped
}

fn add_defined_metric_number(
    metrics: &mut Vec<MetricRow>,
    definition: metric_definitions::MetricDefinition,
    value: Option<f64>,
) {
    add_metric_number(
        metrics,
        definition.domain(),
        exact_metric_key(definition),
        value,
    );
}

fn exact_metric_key(definition: metric_definitions::MetricDefinition) -> &'static str {
    definition
        .exact_key()
        .expect("static metric definition should have an exact key")
}

fn add_metric_number(
    metrics: &mut Vec<MetricRow>,
    domain: MetricDomain,
    key: &str,
    value: Option<f64>,
) {
    let Some(value) = value.filter(|value| value.is_finite()) else {
        return;
    };
    metrics.push(MetricRow {
        domain,
        key: key.to_string(),
        value: MetricValue::Number(value),
    });
}

fn validate_metric_rows(metrics: &[MetricRow]) -> Result<(), String> {
    for metric in metrics {
        let Some(definition) = metric_definitions::definition_for(metric.domain, &metric.key)
        else {
            return Err(format!(
                "emitted metric {}.{} has no typed definition",
                metric.domain.as_str(),
                metric.key
            ));
        };
        let actual = metric_value_type(&metric.value);
        let expected = definition.definition.value_type();
        if actual != expected {
            return Err(format!(
                "emitted metric {}.{} has type {}, expected {}",
                metric.domain.as_str(),
                metric.key,
                actual.as_str(),
                expected.as_str()
            ));
        }
    }
    Ok(())
}

fn metric_value_type(value: &MetricValue) -> MetricValueType {
    match value {
        MetricValue::Number(_) => MetricValueType::Number,
        MetricValue::Text(_) => MetricValueType::Text,
        MetricValue::Boolean(_) => MetricValueType::Boolean,
    }
}
