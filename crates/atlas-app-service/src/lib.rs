#![deny(unsafe_code)]

mod discovery;
mod encounters;
mod error;
mod executor;
mod filter;
mod filters;
mod lists;
mod owned_reading;
mod prepared_content;
mod presentation;
mod projection;
mod records;
mod related;
mod retrieval;
mod search_pages;
mod service;

#[cfg(test)]
mod owned_reading_tests;
#[cfg(test)]
mod related_tests;
#[cfg(test)]
mod source_consumer_tests;
#[cfg(test)]
mod test_support;

pub use error::{AppServiceError, AppServiceResult};
pub use filters::{RawFilterCountsRequest, RawFilterValuesRequest};
pub use projection::record_summary_view;
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
