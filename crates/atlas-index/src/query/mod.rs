//! Validated typed filters, catalog policy and parameterized compilation.
pub(crate) mod catalog;
mod cel;
pub(crate) mod compiler;
pub(crate) mod validation;
pub use catalog::{QUERY_CAPABILITY_VERSION, query_capabilities};
pub use cel::parse_where;
pub(crate) use compiler::compile_predicate;
pub use validation::{ValidatedQuery, validate_query};
#[cfg(test)]
mod cel_semantics;
#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod tests;
