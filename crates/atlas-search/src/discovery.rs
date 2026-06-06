use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, FilterValueSort, SearchFilterNode};
use atlas_index::{
    DiscoveryError as IndexDiscoveryError, FilterValueRequest as IndexFilterValueRequest,
};
use serde_json::Value;
use thiserror::Error;

use crate::AtlasRetrievalService;

pub trait FilterDiscoveryRetrieval {
    fn discover_filter_fields(
        &self,
        request: DiscoverFilterFieldsRequest<'_>,
    ) -> Result<FilterFieldDiscovery, FilterDiscoveryError>;

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest<'_>,
    ) -> Result<FilterValueDiscovery, FilterDiscoveryError>;
}

#[derive(Debug, Clone)]
pub struct DiscoverFilterFieldsRequest<'a> {
    pub filter: Option<&'a SearchFilterNode>,
    pub filter_json: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct DiscoverFilterValuesRequest<'a> {
    pub field: String,
    pub filter: Option<&'a SearchFilterNode>,
    pub filter_json: Option<Value>,
    pub sort: Option<FilterValueSort>,
    pub sample_limit: Option<usize>,
    pub metric: Option<String>,
    pub metric_prefix: Option<String>,
    pub metric_label: Option<String>,
    pub metric_query: Option<String>,
    pub metric_domain: Option<String>,
}

#[derive(Debug, Error)]
pub enum FilterDiscoveryError {
    #[error("{0}")]
    InvalidField(String),
    #[error("{0}")]
    InvalidOption(String),
    #[error("{0}")]
    FieldNotApplicable(String),
    #[error("{0}")]
    AmbiguousMetric(String),
    #[error("filter failed: {0}")]
    InvalidFilter(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
}

impl From<IndexDiscoveryError> for FilterDiscoveryError {
    fn from(error: IndexDiscoveryError) -> Self {
        match error {
            IndexDiscoveryError::InvalidField(message) => Self::InvalidField(message),
            IndexDiscoveryError::InvalidOption(message) => Self::InvalidOption(message),
            IndexDiscoveryError::FieldNotApplicable(message) => Self::FieldNotApplicable(message),
            IndexDiscoveryError::AmbiguousMetric(message) => Self::AmbiguousMetric(message),
            IndexDiscoveryError::Filter(error) => Self::InvalidFilter(error.to_string()),
            IndexDiscoveryError::QueryFailed(message) => Self::QueryFailed(message),
        }
    }
}

impl FilterDiscoveryRetrieval for AtlasRetrievalService {
    fn discover_filter_fields(
        &self,
        request: DiscoverFilterFieldsRequest<'_>,
    ) -> Result<FilterFieldDiscovery, FilterDiscoveryError> {
        self.index
            .list_filter_fields(request.filter, request.filter_json)
            .map_err(FilterDiscoveryError::from)
    }

    fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest<'_>,
    ) -> Result<FilterValueDiscovery, FilterDiscoveryError> {
        self.index
            .list_filter_values(
                request.filter,
                IndexFilterValueRequest {
                    field: request.field,
                    filter_json: request.filter_json,
                    sort: request.sort,
                    sample_limit: request.sample_limit,
                    metric: request.metric,
                    metric_prefix: request.metric_prefix,
                    metric_label: request.metric_label,
                    metric_query: request.metric_query,
                    metric_domain: request.metric_domain,
                },
            )
            .map_err(FilterDiscoveryError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_index::FilterCompileError;

    #[test]
    fn discovery_errors_map_to_product_error_variants() {
        let cases = [
            (
                IndexDiscoveryError::InvalidField("field".to_string()),
                "InvalidField",
                "field",
            ),
            (
                IndexDiscoveryError::InvalidOption("option".to_string()),
                "InvalidOption",
                "option",
            ),
            (
                IndexDiscoveryError::FieldNotApplicable("scope".to_string()),
                "FieldNotApplicable",
                "scope",
            ),
            (
                IndexDiscoveryError::AmbiguousMetric("metric".to_string()),
                "AmbiguousMetric",
                "metric",
            ),
            (
                IndexDiscoveryError::Filter(FilterCompileError::InvalidValue(
                    "bad filter".to_string(),
                )),
                "InvalidFilter",
                "filter failed: filter query returned invalid value: bad filter",
            ),
            (
                IndexDiscoveryError::QueryFailed("database".to_string()),
                "QueryFailed",
                "index query failed: database",
            ),
        ];

        for (index_error, variant, message) in cases {
            let error = FilterDiscoveryError::from(index_error);
            match (variant, &error) {
                ("InvalidField", FilterDiscoveryError::InvalidField(value)) => {
                    assert_eq!(value, message);
                }
                ("InvalidOption", FilterDiscoveryError::InvalidOption(value)) => {
                    assert_eq!(value, message);
                }
                ("FieldNotApplicable", FilterDiscoveryError::FieldNotApplicable(value)) => {
                    assert_eq!(value, message);
                }
                ("AmbiguousMetric", FilterDiscoveryError::AmbiguousMetric(value)) => {
                    assert_eq!(value, message);
                }
                ("InvalidFilter", FilterDiscoveryError::InvalidFilter(value)) => {
                    assert_eq!(error.to_string(), message);
                    assert_eq!(value, "filter query returned invalid value: bad filter");
                }
                ("QueryFailed", FilterDiscoveryError::QueryFailed(value)) => {
                    assert_eq!(error.to_string(), message);
                    assert_eq!(value, "database");
                }
                _ => panic!("unexpected error mapping for {variant}: {error:?}"),
            }
        }
    }
}
