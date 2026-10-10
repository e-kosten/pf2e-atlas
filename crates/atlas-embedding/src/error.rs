use thiserror::Error;
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("failed to load tokenizer `{path}`: {message}")]
    TokenizerLoadFailed { path: String, message: String },
    #[error("failed to tokenize embedding input: {0}")]
    TokenizationFailed(String),
    #[error("failed to load embedding model `{path}`: {message}")]
    ModelLoadFailed { path: String, message: String },
    #[error("failed to prepare embedding model cache path `{path}`: {message}")]
    ModelCachePrepareFailed { path: String, message: String },
    #[error("failed to download embedding model file `{url}` to `{path}`: {message}")]
    ModelCacheDownloadFailed {
        url: String,
        path: String,
        message: String,
    },
    #[error("embedding asset `{path}` checksum mismatch; expected {expected}, got {actual}")]
    AssetChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("unsupported embedding execution contract")]
    UnsupportedModelContract,
    #[error("failed to run embedding model: {0}")]
    ModelRunFailed(String),
    #[error("model returned {actual} dimensions, but expects {expected}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("embedding model returned {actual} outputs for {expected} inputs")]
    UnexpectedEmbeddingOutputCount { expected: usize, actual: usize },
    #[error("embedding input exceeded token budget: actual {actual}, max {max}")]
    TokenBudgetExceeded { actual: usize, max: usize },
    #[error("embedding input is empty or has no body tokens")]
    EmptyInput,
    #[error("embedding vector contains nonfinite values or is not normalized")]
    InvalidVector,
    #[error("text splitter failed: {0}")]
    SplitterFailed(String),
    #[error("invalid passage source address: {0}")]
    InvalidPassageAddress(String),
    #[error("prepared embedding input or reuse identity changed")]
    PreparedInputMismatch,
}
