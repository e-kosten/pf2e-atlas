//! Validated typed filters, catalog policy and parameterized compilation.
pub(crate) mod catalog;
mod cel;
pub(crate) mod compiler;
mod labels;
mod membership;
pub(crate) mod validation;
pub use catalog::{QUERY_CAPABILITY_VERSION, query_capabilities};
pub use cel::parse_where;
pub(crate) use membership::compile_root_membership;
pub use validation::{ValidatedQuery, validate_query};
#[cfg(test)]
mod cel_semantics;
#[cfg(test)]
mod corpus_tests;
#[cfg(test)]
mod membership_tests;
#[cfg(test)]
mod membership_work_probe;
#[cfg(test)]
mod tests;

mod cel_relationship;
