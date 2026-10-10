use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("source root is unavailable: {0}")]
    SourceUnavailable(String),
    #[error("source manifest failed to parse: {0}")]
    ManifestParseFailed(String),
    #[error("source record failed to parse: {0}")]
    RecordParseFailed(String),
    #[error("source content selection failed: {0}")]
    SourceSelectionFailed(String),
    #[error("localization catalog is unavailable or invalid: {0}")]
    LocalizationFailed(String),
    #[error("source contains no loadable Foundry records")]
    NoRecordsLoaded,
    #[error("invalid embedding model `{model}`: {message}")]
    InvalidEmbeddingModel { model: String, message: String },
    #[error("artifact write failed: {0}")]
    ArtifactWriteFailed(String),
    #[error("document embedding generation failed: {0}")]
    DocumentEmbeddingFailed(String),
}
