#![deny(unsafe_code)]

mod catalog;
mod document_input;
mod error;
mod minilm;
mod text;
mod tokenization;
mod vector_math;

pub use catalog::{
    ALL_EMBEDDING_MODELS, DEFAULT_EMBEDDING_MODEL, DistanceMetric, EmbeddingModelId,
    EmbeddingModelSpec, EmbeddingRuntimeConfig, Normalization, PoolingStrategy, VectorDType,
    default_embedding_model_spec, embedding_model_for_model_id, embedding_model_spec,
    supported_embedding_model_ids,
};
pub use document_input::{
    DocumentEmbeddingInputParts, build_document_embedding_input, hash_document_embedding_input,
};
pub use error::EmbeddingError;
pub use minilm::TextEmbedder;
pub use text::normalize_embedding_text;
pub use tokenization::{EmbeddingInputTokenization, TextEmbeddingTokenizer};

#[cfg(test)]
mod tests;
