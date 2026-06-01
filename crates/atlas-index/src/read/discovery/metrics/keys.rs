use atlas_domain::{MetricDomain, MetricKeyDiscovery, NumericFieldStats};
use atlas_record::{MetricRow, MetricValue, definition_for, label_for_row};

use crate::read::discovery::error::DiscoveryError;

pub(super) fn resolve_metric_from_candidates(
    metrics: Vec<MetricKeyDiscovery>,
    value: &str,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let key_matches = metrics
        .iter()
        .filter(|metric| metric.metric_key == value)
        .cloned()
        .collect::<Vec<_>>();
    match key_matches.as_slice() {
        [metric] => return Ok(metric.clone()),
        [] => {}
        _ => {
            return Err(DiscoveryError::AmbiguousMetric(format!(
                "metric key `{value}` is ambiguous; candidates: {}",
                metric_candidates(&key_matches)
            )));
        }
    }
    let normalized = super::query::normalize_metric_label(value);
    let matches = metrics
        .into_iter()
        .filter(|metric| {
            metric.known
                && (super::query::metric_label_matches(metric.label.as_deref(), &normalized)
                    || super::query::metric_label_matches(
                        metric.short_label.as_deref(),
                        &normalized,
                    ))
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [metric] => Ok(metric.clone()),
        [] => Err(DiscoveryError::InvalidOption(format!(
            "metric `{value}` did not match a metric key, exact known label, or exact known short label"
        ))),
        _ => Err(DiscoveryError::AmbiguousMetric(format!(
            "metric label `{value}` is ambiguous; candidates: {}",
            metric_candidates(&matches)
        ))),
    }
}

pub(super) fn metric_candidates(metrics: &[MetricKeyDiscovery]) -> String {
    metrics
        .iter()
        .map(|metric| {
            format!(
                "{} ({}, {}, {})",
                metric.metric_key, metric.metric_domain, metric.record_family, metric.value_type
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn metric_key_from_parts(
    domain: String,
    record_family: String,
    metric_key: String,
    value_type: String,
    count: u64,
    numeric_stats: Option<NumericFieldStats>,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let metric_value = match value_type.as_str() {
        "number" => MetricValue::Number(0.0),
        "boolean" => MetricValue::Boolean(false),
        _ => MetricValue::Text(String::new()),
    };
    let domain = parse_metric_domain(&domain)?;
    let row = MetricRow {
        domain,
        key: metric_key.clone(),
        value: metric_value,
    };
    let label = label_for_row(&row);
    let group = definition_for(domain, &metric_key)
        .map(|matched| matched.definition.group().as_str().to_string());
    Ok(MetricKeyDiscovery {
        metric_domain: metric_domain_string(domain),
        record_family,
        namespace_prefix: metric_key
            .split_once('.')
            .map(|(prefix, _)| format!("{prefix}."))
            .unwrap_or_default(),
        metric_key,
        label: Some(label.label),
        short_label: label.short_label,
        group,
        known: label.known,
        value_type,
        count,
        numeric_stats,
    })
}

fn parse_metric_domain(value: &str) -> Result<MetricDomain, DiscoveryError> {
    MetricDomain::from_canonical(value)
        .ok_or_else(|| DiscoveryError::QueryFailed(format!("unknown metric domain `{value}`")))
}

fn metric_domain_string(value: MetricDomain) -> String {
    value.as_str().to_string()
}
