use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalStateError {
    #[error("local-state path is not valid UTF-8: {0}")]
    NonUtf8Path(String),
    #[error("invalid saved-list slug `{slug}`: {reason}")]
    InvalidSlug { slug: String, reason: &'static str },
    #[error("invalid saved-list record key `{record_key}`: {reason}")]
    InvalidRecordKey { record_key: String, reason: String },
    #[error("saved list already exists: {0}")]
    ListAlreadyExists(String),
    #[error("saved list not found: {0}")]
    ListNotFound(String),
    #[error("unsupported local-state metadata `{key}` value `{value}`")]
    UnsupportedMetadata { key: &'static str, value: String },
    #[error("local-state database is incompatible: {0}")]
    IncompatibleSchema(String),
    #[error("local-state database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("local-state filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("local-state timestamp error: {0}")]
    Timestamp(#[from] time::error::Format),
}

pub type LocalStateResult<T> = Result<T, LocalStateError>;
