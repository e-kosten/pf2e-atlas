use atlas_domain::{MetricKeyDiscovery, RecordFamily, SearchFilterNode};
use diesel::SqliteConnection;

use super::query::{
    metric_label_matches, metric_matches_query, metric_query_tokens, normalize_metric_label,
};
use super::{MetricCatalogScope, catalog_metric_keys, metric_candidates};
use crate::discovery::error::DiscoveryError;

pub(in crate::discovery) fn resolve_filter_metrics(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
) -> Result<Option<SearchFilterNode>, DiscoveryError> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    let scope = metric_resolution_scope(filter).unwrap_or(MetricCatalogScope::Global);
    let candidates = catalog_metric_keys(connection, scope, None, None, None, None)?;
    resolve_filter_metric_node(filter, &candidates).map(Some)
}

fn resolve_filter_metric_node(
    node: &SearchFilterNode,
    candidates: &[MetricKeyDiscovery],
) -> Result<SearchFilterNode, DiscoveryError> {
    match node {
        SearchFilterNode::Metric { metric, r#match } => Ok(SearchFilterNode::metric(
            resolve_metric_reference(candidates, metric)?,
            r#match.clone(),
        )),
        SearchFilterNode::MetricCompare {
            left_metric,
            op,
            right_metric,
        } => Ok(SearchFilterNode::metric_compare(
            resolve_metric_reference(candidates, left_metric)?,
            *op,
            resolve_metric_reference(candidates, right_metric)?,
        )),
        SearchFilterNode::AnyOf { children } => Ok(SearchFilterNode::any_of(
            children
                .iter()
                .map(|child| resolve_filter_metric_node(child, candidates))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        SearchFilterNode::AllOf { children } => Ok(SearchFilterNode::all_of(
            children
                .iter()
                .map(|child| resolve_filter_metric_node(child, candidates))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        SearchFilterNode::Not { child } => Ok(SearchFilterNode::not_of(
            resolve_filter_metric_node(child, candidates)?,
        )),
        SearchFilterNode::RecordFamily { .. }
        | SearchFilterNode::LinksTo { .. }
        | SearchFilterNode::LinkedFrom { .. }
        | SearchFilterNode::MetadataPredicate { .. } => Ok(node.clone()),
    }
}

fn resolve_metric_reference(
    candidates: &[MetricKeyDiscovery],
    value: &str,
) -> Result<String, DiscoveryError> {
    let key_matches = candidates
        .iter()
        .filter(|metric| metric.metric_key == value)
        .collect::<Vec<_>>();
    match key_matches.as_slice() {
        [metric] => return Ok(metric.metric_key.clone()),
        [] => {}
        _ => return Err(ambiguous_metric_error("metric key", value, &key_matches)),
    }

    let normalized = normalize_metric_label(value);
    let label_matches = candidates
        .iter()
        .filter(|metric| {
            metric.known
                && (metric_label_matches(metric.label.as_deref(), &normalized)
                    || metric_label_matches(metric.short_label.as_deref(), &normalized))
        })
        .collect::<Vec<_>>();
    match label_matches.as_slice() {
        [metric] => Ok(metric.metric_key.clone()),
        [] => Err(unknown_metric_error(value, candidates)),
        _ => Err(ambiguous_metric_error(
            "metric label",
            value,
            &label_matches,
        )),
    }
}

fn unknown_metric_error(value: &str, candidates: &[MetricKeyDiscovery]) -> DiscoveryError {
    let tokens = metric_query_tokens(value);
    let suggestions = candidates
        .iter()
        .filter(|metric| metric_matches_query(metric, &tokens))
        .take(5)
        .collect::<Vec<_>>();
    let suggestion = if suggestions.is_empty() {
        String::new()
    } else {
        let suggestions = suggestions
            .into_iter()
            .cloned()
            .collect::<Vec<MetricKeyDiscovery>>();
        format!(" Did you mean {}?", metric_candidates(&suggestions))
    };
    DiscoveryError::InvalidOption(format!(
        "unknown metric `{value}`.{suggestion} Run `atlas filters values --field metric --metric-query {value}` to discover metrics."
    ))
}

fn ambiguous_metric_error(
    label: &str,
    value: &str,
    candidates: &[&MetricKeyDiscovery],
) -> DiscoveryError {
    DiscoveryError::AmbiguousMetric(format!(
        "{label} `{value}` is ambiguous; candidates: {}",
        metric_candidates(
            &candidates
                .iter()
                .copied()
                .cloned()
                .collect::<Vec<MetricKeyDiscovery>>()
        )
    ))
}

fn metric_resolution_scope(filter: &SearchFilterNode) -> Option<MetricCatalogScope> {
    let mut families = Vec::new();
    collect_record_families(filter, &mut families);
    families.sort();
    families.dedup();
    match families.as_slice() {
        [family] => Some(MetricCatalogScope::Family(*family)),
        _ => None,
    }
}

fn collect_record_families(filter: &SearchFilterNode, families: &mut Vec<RecordFamily>) {
    match filter {
        SearchFilterNode::RecordFamily { value } => families.push(*value),
        SearchFilterNode::AllOf { children } => {
            for child in children {
                collect_record_families(child, families);
            }
        }
        SearchFilterNode::AnyOf { .. }
        | SearchFilterNode::Not { .. }
        | SearchFilterNode::LinksTo { .. }
        | SearchFilterNode::LinkedFrom { .. }
        | SearchFilterNode::MetadataPredicate { .. }
        | SearchFilterNode::Metric { .. }
        | SearchFilterNode::MetricCompare { .. } => {}
    }
}
