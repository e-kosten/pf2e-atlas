use std::fmt;

use atlas_index::{FilterCompileError, RecordLoadError, VectorQueryError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchErrorKind {
    IndexUnavailable,
    ArtifactContractViolation,
    InvalidFilter,
    InvalidOptions,
    VectorReadinessRequired,
    EmbeddingUnavailable,
    QueryFailed,
}

#[derive(Debug)]
pub struct SearchError {
    kind: SearchErrorKind,
    message: String,
    vector_readiness_required: bool,
}

impl SearchError {
    pub fn kind(&self) -> SearchErrorKind {
        if self.vector_readiness_required {
            return SearchErrorKind::VectorReadinessRequired;
        }
        self.kind
    }

    pub fn is_vector_readiness_required(&self) -> bool {
        self.vector_readiness_required
    }

    pub fn index_unavailable(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::IndexUnavailable, message.into())
    }

    pub fn artifact_contract_violation(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::ArtifactContractViolation, message.into())
    }

    pub fn query_failed(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::QueryFailed, message.into())
    }

    pub fn vector_readiness_required(message: impl Into<String>) -> Self {
        Self::vector_ready(SearchErrorKind::QueryFailed, message.into())
    }

    pub(crate) fn from_record_load(error: RecordLoadError) -> Self {
        Self::new(SearchErrorKind::QueryFailed, error.to_string())
    }

    pub(crate) fn invalid_embedding_model(model: String, message: String) -> Self {
        Self::vector_ready(
            SearchErrorKind::EmbeddingUnavailable,
            format!("invalid embedding model `{model}`: {message}"),
        )
    }

    pub(crate) fn embedding(message: String) -> Self {
        Self::vector_ready(
            SearchErrorKind::EmbeddingUnavailable,
            format!("embedding operation failed: {message}"),
        )
    }

    pub(crate) fn invalid_search_options(message: String) -> Self {
        Self::new(
            SearchErrorKind::InvalidOptions,
            format!("invalid search options: {message}"),
        )
    }

    pub(crate) fn unsupported_retrieval_pattern(pattern: &'static str) -> Self {
        Self::vector_ready(
            SearchErrorKind::QueryFailed,
            format!("retrieval pattern is not implemented yet: {pattern}"),
        )
    }

    pub(crate) fn from_filter(error: FilterCompileError) -> Self {
        Self::new(SearchErrorKind::InvalidFilter, error.to_string())
    }

    pub(crate) fn from_vector(error: VectorQueryError) -> Self {
        let vector_readiness_required = matches!(error, VectorQueryError::QueryFailed(_));
        Self {
            kind: match error {
                VectorQueryError::Filter(_) => SearchErrorKind::InvalidFilter,
                VectorQueryError::QueryFailed(_) => SearchErrorKind::QueryFailed,
                _ => SearchErrorKind::QueryFailed,
            },
            message: error.to_string(),
            vector_readiness_required,
        }
    }

    fn new(kind: SearchErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            vector_readiness_required: false,
        }
    }

    fn vector_ready(kind: SearchErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            vector_readiness_required: true,
        }
    }
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SearchError {}
