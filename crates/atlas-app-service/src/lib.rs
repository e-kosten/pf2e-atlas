#![deny(unsafe_code)]

mod discovery;
mod encounters;
mod error;
mod executor;
mod filter;
mod filters;
mod lists;
mod projection;
mod records;
mod retrieval;
mod service;
mod surfaces;
mod windows;

#[cfg(test)]
mod source_consumer_tests;
#[cfg(test)]
mod test_support;

pub use error::{AppServiceError, AppServiceResult};
pub use filters::{RawFilterCountsRequest, RawFilterValuesRequest};
pub use projection::record_summary_view;
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
