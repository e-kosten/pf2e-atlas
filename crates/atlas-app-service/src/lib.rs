#![deny(unsafe_code)]

mod completions;
mod content_roll;
mod content_roll_presentation;
mod contextual_references;
mod physical_labels;

mod connection_scope;
mod definition_values;
mod definitions;
mod discovery;
mod embedded_reading;
mod encounters;
mod error;
mod executor;
mod filter;
mod filters;
mod flat_check;
mod inline_content;
#[cfg(test)]
mod inline_content_tests;
mod lists;
mod parties;
#[cfg(test)]
mod party_tests;
mod prepared_content;
mod presentation;
mod projection;
mod read_execution;
mod record_grants;
mod records;
mod related;
mod relationship_search;
mod remaster;
mod retrieval;
mod saved_edit;
#[cfg(test)]
mod saved_edit_tests;
mod search_match_sections;
mod search_pages;
mod service;
mod similarity;
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
pub use encounters::portability::EncounterImportReport;
pub use error::{AppServiceError, AppServiceResult};
pub use filters::{CliFilterValuesRequest, RawFilterCountsRequest};
pub use projection::record_summary_view;
pub use read_execution::ReadExecutionContext;
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};

#[cfg(test)]
mod relationship_discovery_tests;

mod result_order;

#[cfg(test)]
mod result_order_tests;

#[cfg(test)]
mod contextual_content_tests;
