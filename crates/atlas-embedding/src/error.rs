use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("failed to load tokenizer `{path}`: {message}")]
    TokenizerLoadFailed { path: String, message: String },
    #[error("failed to tokenize query: {0}")]
    TokenizationFailed(String),
    #[error("failed to load ONNX model `{path}`: {message}")]
    ModelLoadFailed { path: String, message: String },
    #[error("failed to prepare embedding model cache path `{path}`: {message}")]
    ModelCachePrepareFailed { path: String, message: String },
    #[error("failed to download embedding model file `{url}` to `{path}`: {message}")]
    ModelCacheDownloadFailed {
        url: String,
        path: String,
        message: String,
    },
    #[error("failed to prepare ONNX tensor: {0}")]
    TensorPrepareFailed(String),
    #[error("failed to run ONNX model: {0}")]
    ModelRunFailed(String),
    #[error("model did not return a hidden-state tensor")]
    MissingHiddenState,
    #[error("expected hidden-state shape [batch, tokens, dims], got {0:?}")]
    UnexpectedHiddenStateShape(Vec<usize>),
    #[error("model returned {actual} dimensions, but embedding catalog expects {expected}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("embedding model returned {actual} outputs for {expected} inputs")]
    UnexpectedEmbeddingOutputCount { expected: usize, actual: usize },
    #[error(
        "composed embedding input exceeded token budget: estimated {estimated}, actual {actual}, max {max}"
    )]
    TokenBudgetExceeded {
        estimated: usize,
        actual: usize,
        max: usize,
    },
}
