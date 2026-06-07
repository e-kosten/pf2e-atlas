#![deny(unsafe_code)]

mod discovery;
mod error;
mod filter;
mod projection;
mod service;

pub use error::{AppServiceError, AppServiceResult};
pub use service::{AtlasAppService, AtlasAppServiceOptions};
