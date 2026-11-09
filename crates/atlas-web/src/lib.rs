#![deny(unsafe_code)]

#[cfg(feature = "embedded-ui")]
mod assets;
mod compatibility;
mod error;
mod handlers;
mod parties;
mod router;
mod schema_directions;
mod service;

/// The generated wire contract expected by a native Atlas client.
pub const API_CONTRACT: &str = compatibility::CONTRACT;

pub use router::{openapi, router};

#[cfg(test)]
mod tests;
