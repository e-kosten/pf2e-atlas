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
    pub metric_selector: Option<MetricDiscoverySelector>,
    pub metric_domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricDiscoverySelector {
    ExactKey(String),
    Prefix(String),
    Label(String),
    Query(String),
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
        let filter = request.filter;
        let index_request = index_filter_value_request(request)?;
        self.index
            .list_filter_values(filter, index_request)
            .map_err(FilterDiscoveryError::from)
    }
}

fn index_filter_value_request(
    request: DiscoverFilterValuesRequest<'_>,
) -> Result<IndexFilterValueRequest, FilterDiscoveryError> {
    if request.metric_selector.is_some() && request.field != "metric" {
        return Err(FilterDiscoveryError::InvalidOption(
            "metric discovery options apply only to field `metric`".to_string(),
        ));
    }
    if request.metric_domain.is_some() && request.field != "metric" {
        return Err(FilterDiscoveryError::InvalidOption(
            "metric domain applies only to field `metric`".to_string(),
        ));
    }
    let mut index_request = IndexFilterValueRequest {
        field: request.field,
        filter_json: request.filter_json,
        sort: request.sort,
        sample_limit: request.sample_limit,
        metric: None,
        metric_prefix: None,
        metric_label: None,
        metric_query: None,
        metric_domain: request.metric_domain,
    };
    match request.metric_selector {
        Some(MetricDiscoverySelector::ExactKey(metric)) => index_request.metric = Some(metric),
        Some(MetricDiscoverySelector::Prefix(prefix)) => index_request.metric_prefix = Some(prefix),
        Some(MetricDiscoverySelector::Label(label)) => index_request.metric_label = Some(label),
        Some(MetricDiscoverySelector::Query(query)) => index_request.metric_query = Some(query),
        None => {}
    }
    Ok(index_request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_index::FilterCompileError;

    #[test]
    fn metric_selector_applies_only_to_metric_field() {
        let error = index_filter_value_request(DiscoverFilterValuesRequest {
            field: "traits".to_string(),
            filter: None,
            filter_json: None,
            sort: None,
            sample_limit: None,
            metric_selector: Some(MetricDiscoverySelector::Query("armor".to_string())),
            metric_domain: None,
        })
        .expect_err("metric selector should be rejected for non-metric fields");

        assert!(matches!(error, FilterDiscoveryError::InvalidOption(_)));
        assert_eq!(
            error.to_string(),
            "metric discovery options apply only to field `metric`"
        );
    }

    #[test]
    fn metric_domain_applies_only_to_metric_field() {
        let error = index_filter_value_request(DiscoverFilterValuesRequest {
            field: "traits".to_string(),
            filter: None,
            filter_json: None,
            sort: None,
            sample_limit: None,
            metric_selector: None,
            metric_domain: Some("actor".to_string()),
        })
        .expect_err("metric domain should be rejected for non-metric fields");

        assert!(matches!(error, FilterDiscoveryError::InvalidOption(_)));
        assert_eq!(
            error.to_string(),
            "metric domain applies only to field `metric`"
        );
    }

    #[test]
    fn metric_selector_lowers_to_index_request() {
        let request = index_filter_value_request(DiscoverFilterValuesRequest {
            field: "metric".to_string(),
            filter: None,
            filter_json: None,
            sort: None,
            sample_limit: None,
            metric_selector: Some(MetricDiscoverySelector::Prefix("save.".to_string())),
            metric_domain: Some("actor".to_string()),
        })
        .expect("metric selector should lower");

        assert_eq!(request.metric, None);
        assert_eq!(request.metric_prefix.as_deref(), Some("save."));
        assert_eq!(request.metric_label, None);
        assert_eq!(request.metric_query, None);
        assert_eq!(request.metric_domain.as_deref(), Some("actor"));
    }

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
