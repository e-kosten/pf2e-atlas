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
mod windows;

#[cfg(test)]
mod test_support;

pub use error::{AppServiceError, AppServiceResult};
pub use filters::RawFilterValuesRequest;
pub use service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
