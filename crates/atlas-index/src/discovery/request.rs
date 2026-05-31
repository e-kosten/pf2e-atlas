use atlas_domain::{
    FilterDiscoveryExecution, FilterFieldDiscovery, FilterValueDiscovery, FilterValuePolicy,
    FilterValueSort, RecordFamily, SearchFilterNode,
};
use diesel::SqliteConnection;

use super::catalog;
use super::definitions::{
    definition_for, metric_field_info, unknown_field_error, validate_options,
};
use super::dynamic;
use super::error::DiscoveryError;
use super::metrics;

const DEFAULT_SAMPLE_LIMIT: usize = 20;
const MAX_SAMPLE_LIMIT: usize = 100;

pub type DiscoveryValueSort = FilterValueSort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterValueRequest {
    pub field: String,
    pub filter_json: Option<serde_json::Value>,
    pub sort: Option<DiscoveryValueSort>,
    pub sample_limit: Option<usize>,
    pub metric: Option<String>,
    pub metric_prefix: Option<String>,
    pub metric_label: Option<String>,
    pub metric_query: Option<String>,
    pub metric_domain: Option<String>,
}

pub(super) fn list_filter_fields(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    filter_json: Option<serde_json::Value>,
) -> Result<FilterFieldDiscovery, DiscoveryError> {
    let scope = catalog_scope(filter);
    let execution = execution_for(filter, scope);
    let matching_record_count = dynamic::count_matching_records(connection, filter)?;
    let mut fields = if execution == FilterDiscoveryExecution::Catalog {
        catalog::fields(connection, scope)?
    } else {
        dynamic::fields(connection, filter)?
    };
    if metrics::metric_key_count(connection, filter, None, None, None, None)? > 0 {
        fields.push(metric_field_info(
            execution == FilterDiscoveryExecution::Catalog,
        ));
    }
    Ok(FilterFieldDiscovery {
        filter: filter_json,
        execution,
        matching_record_count,
        fields,
    })
}

pub(super) fn list_filter_values(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    request: FilterValueRequest,
) -> Result<FilterValueDiscovery, DiscoveryError> {
    if request.field == "metric" {
        return metrics::values(connection, filter, request);
    }
    let definition =
        definition_for(&request.field).ok_or_else(|| unknown_field_error(&request.field))?;
    let sample_limit = request.sample_limit.unwrap_or(DEFAULT_SAMPLE_LIMIT);
    if sample_limit == 0 || sample_limit > MAX_SAMPLE_LIMIT {
        return Err(DiscoveryError::InvalidOption(format!(
            "--sample-limit must be between 1 and {MAX_SAMPLE_LIMIT}"
        )));
    }
    validate_options(definition, request.sort, request.sample_limit)?;

    let scope = catalog_scope(filter);
    let mut execution = execution_for(filter, scope);
    if definition.value_policy == FilterValuePolicy::BooleanCounts {
        execution = FilterDiscoveryExecution::Dynamic;
    }
    let matching_record_count = dynamic::count_matching_records(connection, filter)?;
    if matching_record_count > 0 && !dynamic::field_applies(connection, definition, filter)? {
        return Err(DiscoveryError::FieldNotApplicable(format!(
            "field `{}` is not applicable in the current filter space",
            request.field
        )));
    }
    let payload = if execution == FilterDiscoveryExecution::Catalog {
        catalog::values(connection, definition, scope, &request)?
    } else {
        dynamic::values(connection, definition, filter, &request)?
    };
    Ok(FilterValueDiscovery {
        field: definition.field.to_string(),
        filter: request.filter_json,
        execution,
        matching_record_count,
        payload,
    })
}

fn execution_for(
    filter: Option<&SearchFilterNode>,
    scope: Option<RecordFamily>,
) -> FilterDiscoveryExecution {
    if scope.is_some() || filter.is_none() {
        FilterDiscoveryExecution::Catalog
    } else {
        FilterDiscoveryExecution::Dynamic
    }
}

fn catalog_scope(filter: Option<&SearchFilterNode>) -> Option<RecordFamily> {
    match filter {
        Some(SearchFilterNode::RecordFamily { value }) => Some(*value),
        _ => None,
    }
}
