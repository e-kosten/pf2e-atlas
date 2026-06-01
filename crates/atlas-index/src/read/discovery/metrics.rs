use atlas_domain::{
    FilterDiscoveryExecution, FilterValueDiscovery, FilterValuePayload, SearchFilterNode,
};
use diesel::SqliteConnection;

use super::error::DiscoveryError;
use super::request::FilterValueRequest;

mod catalog;
mod dynamic;
mod keys;
mod query;
mod resolution;
mod scope;
mod values;

use catalog::catalog_metric_keys;
use dynamic::metric_keys;
use keys::resolve_metric_from_candidates;
pub(super) use resolution::resolve_filter_metrics;
use scope::metric_catalog_scope;
use values::metric_values;

pub(super) fn values(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    request: FilterValueRequest,
) -> Result<FilterValueDiscovery, DiscoveryError> {
    if request.sort.is_some() {
        return Err(DiscoveryError::InvalidOption(
            "--sort applies only to enumerable metadata value fields".to_string(),
        ));
    }
    if request.sample_limit.is_some() {
        return Err(DiscoveryError::InvalidOption(
            "--sample-limit applies only to sampled text fields".to_string(),
        ));
    }
    let matching_record_count = super::dynamic::count_matching_records(connection, filter)?;
    let catalog_scope = metric_catalog_scope(filter);
    let execution = if catalog_scope.is_some() {
        FilterDiscoveryExecution::Catalog
    } else {
        FilterDiscoveryExecution::Dynamic
    };
    let payload = if let Some(metric) = request.metric.as_deref() {
        if request.metric_prefix.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-prefix cannot be combined with --metric".to_string(),
            ));
        }
        if request.metric_label.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-label cannot be combined with --metric".to_string(),
            ));
        }
        if request.metric_query.is_some() {
            return Err(DiscoveryError::InvalidOption(
                "--metric-query cannot be combined with --metric".to_string(),
            ));
        }
        let metric = if let Some(scope) = catalog_scope {
            let metrics = catalog_metric_keys(
                connection,
                scope,
                None,
                None,
                None,
                request.metric_domain.as_deref(),
            )?;
            resolve_metric_from_candidates(metrics, metric)?
        } else {
            let metrics = metric_keys(
                connection,
                filter,
                None,
                None,
                None,
                request.metric_domain.as_deref(),
            )?;
            resolve_metric_from_candidates(metrics, metric)?
        };
        let values = metric_values(connection, filter, catalog_scope, &metric)?;
        FilterValuePayload::MetricValues {
            metric: Box::new(metric),
            values,
        }
    } else {
        let metrics = if let Some(scope) = catalog_scope {
            catalog_metric_keys(
                connection,
                scope,
                request.metric_prefix.as_deref(),
                request.metric_label.as_deref(),
                request.metric_query.as_deref(),
                request.metric_domain.as_deref(),
            )?
        } else {
            metric_keys(
                connection,
                filter,
                request.metric_prefix.as_deref(),
                request.metric_label.as_deref(),
                request.metric_query.as_deref(),
                request.metric_domain.as_deref(),
            )?
        };
        FilterValuePayload::MetricKeys { metrics }
    };
    Ok(FilterValueDiscovery {
        field: "metric".to_string(),
        filter: request.filter_json,
        execution,
        matching_record_count,
        payload,
    })
}

pub(super) fn metric_key_count(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    prefix: Option<&str>,
    label_query: Option<&str>,
    metric_query: Option<&str>,
    domain: Option<&str>,
) -> Result<u64, DiscoveryError> {
    Ok(metric_keys(
        connection,
        filter,
        prefix,
        label_query,
        metric_query,
        domain,
    )?
    .len() as u64)
}
