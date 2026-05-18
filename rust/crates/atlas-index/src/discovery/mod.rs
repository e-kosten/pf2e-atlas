mod catalog;
mod definitions;
mod dynamic;
mod error;
mod metrics;
mod request;
mod stats;

use atlas_domain::{FilterFieldDiscovery, FilterValueDiscovery, SearchFilterNode};
use rusqlite::Connection;

pub use error::DiscoveryError;
pub use request::{DiscoveryValueSort, FilterValueRequest};

pub(crate) fn list_filter_fields(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    filter_json: Option<serde_json::Value>,
) -> Result<FilterFieldDiscovery, DiscoveryError> {
    request::list_filter_fields(connection, filter, filter_json)
}

pub(crate) fn list_filter_values(
    connection: &Connection,
    filter: Option<&SearchFilterNode>,
    request: FilterValueRequest,
) -> Result<FilterValueDiscovery, DiscoveryError> {
    request::list_filter_values(connection, filter, request)
}
