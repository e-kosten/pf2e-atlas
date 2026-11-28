#![deny(unsafe_code)]
mod character_context;
mod character_management;
mod character_sheet;
mod characters;
mod encounter_characters;

mod completions;
mod content_roll;
mod content_roll_presentation;
mod contextual_references;
mod physical_labels;

mod connection_scope;
mod creature_family;
mod definition_values;
mod definitions;
mod discovery;
mod embedded_reading;
mod encounters;
mod equipment_family;
mod error;
mod executor;
mod filter;
mod filters;
mod flat_check;
mod inline_content;
#[cfg(test)]
mod inline_content_tests;
mod kind_choices;
mod lists;
mod parties;
#[cfg(test)]
mod party_tests;
mod pathbuilder;
pub mod pathbuilder_sheet;
mod pc_reference;
mod prepared_content;
mod presentation;
mod progression;
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
mod search_toolbar;
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

pub use completions::{
    complete_character_keys, complete_encounter_keys, complete_party_keys, complete_record_keys,
    complete_saved_list_keys, complete_saved_list_slugs,
};
pub use discovery::{filter_group_fields, filter_group_names};
pub use encounters::EncounterReadDetail;
pub use encounters::portability::EncounterImportReport;
pub use error::{AppServiceError, AppServiceResult};
pub use filters::{
    CliFilterValues, CliFilterValuesRequest, discover_filter_fields, filter_field_definition,
};
pub use read_execution::ReadExecutionContext;
pub use records::{RecordReadDetail, RecordReadSection};
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
pub use table_roll::RolledTable;

#[cfg(test)]
mod relationship_discovery_tests;

mod result_order;

#[cfg(test)]
mod result_order_tests;

#[cfg(test)]
mod contextual_content_tests;
