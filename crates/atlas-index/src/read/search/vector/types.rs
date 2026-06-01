use thiserror::Error;

use crate::read::search::filters::FilterCompileError;
use crate::read::sql::SqlBindValue;

#[derive(Debug, Clone, PartialEq)]
pub struct VectorKnnQuery {
    pub sql: String,
    pub parameters: Vec<SqlBindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchHit {
    pub record_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordEmbeddingVector {
    pub unit_kind: String,
    pub label: Option<String>,
    pub ordinal: i64,
    pub vector: Vec<f32>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VectorQueryError {
    #[error("vector query limit must be greater than zero")]
    InvalidLimit,
    #[error("query vector must not be empty")]
    EmptyQueryVector,
    #[error("vector query failed: {0}")]
    QueryFailed(String),
    #[error("vector query returned invalid unit kind: {0}")]
    InvalidUnitKind(String),
    #[error(
        "stored embedding vector for `{record_key}` ({unit_kind} #{ordinal}) had invalid vector data: {message}"
    )]
    InvalidStoredVector {
        record_key: String,
        unit_kind: String,
        ordinal: i64,
        message: String,
    },
    #[error(
        "stored embedding vector for `{record_key}` ({unit_kind} #{ordinal}) declared {declared_dimensions} dimensions but decoded to {decoded_dimensions}"
    )]
    InvalidStoredDimensions {
        record_key: String,
        unit_kind: String,
        ordinal: i64,
        declared_dimensions: usize,
        decoded_dimensions: usize,
    },
    #[error(transparent)]
    Filter(#[from] FilterCompileError),
}
