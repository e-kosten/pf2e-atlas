#![deny(unsafe_code)]

mod discovery;
mod error;
mod executor;
mod filter;
mod filters;
mod projection;
mod records;
mod service;
mod windows;

#[cfg(test)]
mod test_support;

pub use error::{AppServiceError, AppServiceResult};
pub use service::{AtlasAppService, AtlasAppServiceOptions};
