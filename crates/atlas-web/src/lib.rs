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
mod consumable_http_tests;
#[cfg(test)]
mod reference_filter_http_tests;
#[cfg(test)]
mod spell_http_tests;
