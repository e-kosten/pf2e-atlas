use atlas_domain::QueryError;
use atlas_index::IndexError;
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
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct SearchError {
    kind: SearchErrorKind,
    message: String,
    pub query_error: Option<Box<QueryError>>,
}
impl SearchError {
    pub fn kind(&self) -> SearchErrorKind {
        self.kind
    }
    pub fn is_vector_readiness_required(&self) -> bool {
        self.kind == SearchErrorKind::VectorReadinessRequired
    }
    pub fn index_unavailable(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::IndexUnavailable, message)
    }
    pub fn artifact_contract_violation(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::ArtifactContractViolation, message)
    }
    pub fn query_failed(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::QueryFailed, message)
    }
    pub fn vector_readiness_required(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::VectorReadinessRequired, message)
    }
    pub(crate) fn invalid_search_options(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::InvalidOptions, message)
    }
    pub(crate) fn embedding(message: impl Into<String>) -> Self {
        Self::new(SearchErrorKind::EmbeddingUnavailable, message)
    }
    fn new(kind: SearchErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            query_error: None,
        }
    }
}
impl From<QueryError> for SearchError {
    fn from(e: QueryError) -> Self {
        Self {
            kind: if e.code == "catalog_initialization" {
                SearchErrorKind::ArtifactContractViolation
            } else {
                SearchErrorKind::InvalidFilter
            },
            message: e.to_string(),
            query_error: Some(Box::new(e)),
        }
    }
}
impl From<IndexError> for SearchError {
    fn from(e: IndexError) -> Self {
        match e {
            IndexError::Query(e) => e.into(),
            IndexError::Unavailable(s) => Self::index_unavailable(s),
            IndexError::Unsupported(s) | IndexError::Invalid(s) => {
                Self::artifact_contract_violation(s)
            }
            e => Self::query_failed(e.to_string()),
        }
    }
}
