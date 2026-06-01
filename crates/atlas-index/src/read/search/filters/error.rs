use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FilterCompileError {
    #[error("filter `{filter}` is not supported by the SQL keyset compiler")]
    Unsupported { filter: String },
    #[error("filter `{filter}` is missing required value `{value}`")]
    MissingValue { filter: String, value: String },
    #[error("filter query failed: {0}")]
    QueryFailed(String),
    #[error("filter query returned invalid value: {0}")]
    InvalidValue(String),
}
