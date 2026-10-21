#![deny(unsafe_code)]

mod completions;
mod contextual_references;
mod physical_labels;

mod discovery;
mod embedded_reading;
mod encounters;
mod error;
mod executor;
mod filter;
mod filters;
mod flat_check;
mod lists;
mod parties;
#[cfg(test)]
mod party_tests;
mod prepared_content;
mod presentation;
mod projection;
mod read_execution;
mod records;
mod related;
mod retrieval;
mod search_pages;
mod service;
mod table_roll;
mod traits;

#[cfg(test)]
mod embedded_reading_tests;
#[cfg(test)]
mod related_tests;
#[cfg(test)]
mod source_consumer_tests;
#[cfg(test)]
mod test_support;

pub use completions::{complete_record_keys, complete_saved_list_slugs};
pub use error::{AppServiceError, AppServiceResult};
pub use filters::{RawFilterCountsRequest, RawFilterValuesRequest};
pub use projection::record_summary_view;
pub use read_execution::ReadExecutionContext;
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
