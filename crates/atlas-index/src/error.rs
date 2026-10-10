use thiserror::Error;
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("artifact unavailable: {0}")]
    Unavailable(String),
    #[error("unsupported artifact: {0}; rebuild with atlas index build")]
    Unsupported(String),
    #[error("artifact invalid: {0}; rebuild with atlas index build")]
    Invalid(String),
    #[error("artifact operation failed: {0}")]
    Operation(String),
    #[error(transparent)]
    Query(#[from] atlas_domain::query::QueryError),
}
impl From<diesel::result::Error> for IndexError {
    fn from(e: diesel::result::Error) -> Self {
        Self::Operation(e.to_string())
    }
}
impl From<rusqlite::Error> for IndexError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Operation(e.to_string())
    }
}
impl From<serde_json::Error> for IndexError {
    fn from(e: serde_json::Error) -> Self {
        Self::Invalid(e.to_string())
    }
}
impl From<std::io::Error> for IndexError {
    fn from(e: std::io::Error) -> Self {
        Self::Operation(e.to_string())
    }
}
impl From<atlas_foundry_model::SnapshotError> for IndexError {
    fn from(e: atlas_foundry_model::SnapshotError) -> Self {
        Self::Invalid(e.to_string())
    }
}
