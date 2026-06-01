use atlas_domain::{
    BooleanFieldCounts, FilterValueCount, MetricKeyDiscovery, MetricValuePayload,
    NumericFieldStats, SearchFilterNode,
};
use diesel::SqliteConnection;

use crate::read::discovery::error::DiscoveryError;

use super::catalog::{
    catalog_metric_boolean_counts, catalog_metric_numeric_stats, catalog_metric_text_values,
};
use super::dynamic::{metric_boolean_counts, metric_numeric_stats, metric_text_values};
use super::scope::MetricCatalogScope;

pub(super) fn metric_values(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<MetricValuePayload, DiscoveryError> {
    match metric.value_type.as_str() {
        "number" => metric_numeric_stats_for_scope(connection, filter, catalog_scope, metric)
            .map(|stats| MetricValuePayload::NumericStats { stats }),
        "boolean" => metric_boolean_counts_for_scope(connection, filter, catalog_scope, metric)
            .map(|counts| MetricValuePayload::BooleanCounts { counts }),
        _ => metric_text_values_for_scope(connection, filter, catalog_scope, metric)
            .map(|values| MetricValuePayload::TextValues { values }),
    }
}

fn metric_text_values_for_scope(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<Vec<FilterValueCount>, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_text_values(connection, scope, metric);
    }
    metric_text_values(connection, filter, metric)
}

fn metric_boolean_counts_for_scope(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<BooleanFieldCounts, DiscoveryError> {
    if let Some(scope) = catalog_scope {
        return catalog_metric_boolean_counts(connection, scope, metric);
    }
    metric_boolean_counts(connection, filter, metric)
}

fn metric_numeric_stats_for_scope(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    catalog_scope: Option<MetricCatalogScope>,
    metric: &MetricKeyDiscovery,
) -> Result<NumericFieldStats, DiscoveryError> {
    if let Some(scope) = catalog_scope
        && let Some(stats) = catalog_metric_numeric_stats(connection, scope, metric)?
    {
        return Ok(stats);
    }
    metric_numeric_stats(connection, filter, metric)
}
