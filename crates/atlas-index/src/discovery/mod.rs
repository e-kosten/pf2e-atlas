mod catalog;
pub(crate) mod definitions;
mod dynamic;
mod error;
mod metrics;
mod request;
mod stats;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, SearchFilterNode};
use diesel::SqliteConnection;

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
