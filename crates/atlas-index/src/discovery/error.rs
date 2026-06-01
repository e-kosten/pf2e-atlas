use crate::filters::FilterCompileError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("{0}")]
    InvalidField(String),
    #[error("{0}")]
    InvalidOption(String),
    #[error("{0}")]
    FieldNotApplicable(String),
    #[error("{0}")]
    AmbiguousMetric(String),
    #[error("filter failed: {0}")]
    Filter(#[from] FilterCompileError),
    #[error("index query failed: {0}")]
    QueryFailed(String),
}

pub(super) fn query_error(error: impl std::fmt::Display) -> DiscoveryError {
    DiscoveryError::QueryFailed(error.to_string())
}
