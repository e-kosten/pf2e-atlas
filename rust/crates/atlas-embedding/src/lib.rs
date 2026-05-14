#![deny(unsafe_code)]

mod catalog;
mod document_input;
mod document_renderer;
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
pub use document_input::hash_document_embedding_input;
pub use document_renderer::{
    EmbeddingInputChunk, EmbeddingInputSection, render_embedding_chunks_for_embedding,
    render_presentation_document_embedding_chunks, render_presentation_document_for_embedding,
};
pub use error::EmbeddingError;
pub use minilm::TextEmbedder;
pub use text::normalize_embedding_text;
pub use tokenization::{
    BudgetedEmbeddingInput, EmbeddingInputTokenization, EmbeddingSectionTruncation,
    TextEmbeddingTokenizer,
};

#[cfg(test)]
mod tests;
