//! Offline Foundry source diagnostics; CLI presentation delegates discovery to ingest.
mod analyze;
pub(crate) mod args;
mod schema;
mod values;

pub(crate) use analyze::run_source_analyze;
pub(crate) use schema::run_source_schema;
pub(crate) use values::run_source_values;
