mod catalog;
mod dynamic;
mod error;
mod metrics;
mod request;
mod stats;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, SearchFilterNode};
use diesel::SqliteConnection;

use crate::FilterCompileError;
use crate::sqlite::SqliteIndexReader;

pub use error::DiscoveryError;
pub use request::{DiscoveryValueSort, FilterValueRequest};

pub(crate) fn list_filter_fields(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    filter_json: Option<serde_json::Value>,
) -> Result<FilterFieldDiscovery, DiscoveryError> {
    request::list_filter_fields(connection, filter, filter_json)
}

pub(crate) fn list_filter_values(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
    request: FilterValueRequest,
) -> Result<FilterValueDiscovery, DiscoveryError> {
    request::list_filter_values(connection, filter, request)
}

pub(crate) fn resolve_filter_metrics(
    connection: &mut SqliteConnection,
    filter: Option<&SearchFilterNode>,
) -> Result<Option<SearchFilterNode>, DiscoveryError> {
    metrics::resolve_filter_metrics(connection, filter)
}

impl SqliteIndexReader {
    pub fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        self.with_diesel_connection(|connection| {
            list_filter_fields(connection, filter, filter_json)
        })
    }

    pub fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        self.with_diesel_connection(|connection| list_filter_values(connection, filter, request))
    }

    pub fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, FilterCompileError> {
        self.with_diesel_connection(|connection| resolve_filter_metrics(connection, filter))
            .map_err(|error| FilterCompileError::InvalidValue(error.to_string()))
    }
}
