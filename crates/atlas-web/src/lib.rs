#![deny(unsafe_code)]

mod assets;
mod error;
mod handlers;
mod parties;
mod router;
mod service;

pub use router::router;

#[cfg(test)]
mod tests;
