#![deny(unsafe_code)]

mod catalog;
mod document_input;
mod document_renderer;
mod document_units;
mod error;
mod minilm;
mod model_cache;
mod text;
mod tokenization;
mod unit_kind;
mod vector_math;

pub const EMBEDDING_UNIT_POLICY_VERSION: &str = "explicit-heading-sections/v1";

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
pub use document_units::{
    DocumentEmbeddingChunkBudgetDiagnostic, DocumentEmbeddingChunkBudgetDiagnosticChunk,
    DocumentEmbeddingContentSource, DocumentEmbeddingRecordTruncationCoverage,
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingSource,
    DocumentEmbeddingTokenizationTelemetry, DocumentEmbeddingTruncationExample,
    DocumentEmbeddingUnitKindTruncation, GeneratedDocumentEmbedding, GeneratedDocumentEmbeddings,
    PendingDocumentEmbedding, ReusableDocumentEmbedding, apply_document_embedding_token_budget,
    apply_document_embedding_token_budget_with_diagnostics, build_document_embedding_units,
    generate_document_embeddings, generate_document_embeddings_with_reuse,
    generate_document_embeddings_with_reuse_using,
    generate_document_embeddings_with_reuse_using_batch,
};
pub use error::EmbeddingError;
pub use minilm::TextEmbedder;
pub use model_cache::{
    EmbeddingModelCacheFile, prepare_embedding_model_cache, required_embedding_model_cache_files,
};
pub use text::normalize_embedding_text;
pub use tokenization::{
    BudgetedEmbeddingInput, EmbeddingChunkBudgetDiagnostic, EmbeddingChunkBudgetOutcome,
    EmbeddingInputTokenization, EmbeddingSectionTruncation, TextEmbeddingTokenizer,
};
pub use unit_kind::{EmbeddingUnitKind, ParseEmbeddingUnitKindError};

#[cfg(test)]
mod tests;
