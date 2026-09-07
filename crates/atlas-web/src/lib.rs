#![deny(unsafe_code)]

mod assets;
mod error;
mod handlers;
mod router;
mod service;

pub use router::router;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod spell_http_tests;
