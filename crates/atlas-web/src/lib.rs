#![deny(unsafe_code)]

#[cfg(feature = "embedded-ui")]
mod assets;
mod compatibility;
mod error;
mod handlers;
mod parties;
mod router;
mod service;

pub use router::{openapi, router};

#[cfg(test)]
mod tests;
