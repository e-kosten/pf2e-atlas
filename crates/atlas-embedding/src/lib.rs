#![deny(unsafe_code)]
mod catalog;
mod document_input;
mod error;
mod inference;
mod model_cache;
mod passages;
mod tokenization;

pub const EMBEDDING_UNIT_POLICY_VERSION: &str =
    "source-passages/v1/text-splitter0.29.3/body256-context64-overlap32/identity480/special512";
pub const BODY_TOKEN_BUDGET: usize = 256;
pub const CONTEXT_TOKEN_BUDGET: usize = 64;
pub const OVERLAP_TOKEN_BUDGET: usize = 32;
pub const IDENTITY_TOKEN_BUDGET: usize = 480;
pub use catalog::{
    ALL_EMBEDDING_MODELS, DEFAULT_EMBEDDING_MODEL, DistanceMetric, EmbeddingModelId,
    EmbeddingModelSpec, EmbeddingRuntimeConfig, Normalization, PoolingStrategy, VectorDType,
    default_embedding_model_spec, embedding_model_for_model_id, embedding_model_spec,
    supported_embedding_model_ids,
};
pub use document_input::{embedding_reuse_key, hash_document_embedding_input};
pub use error::EmbeddingError;
pub use inference::{
    GeneratedEmbeddings, TextEmbedder, generate_prepared_embeddings, validate_embedding_vector,
};
pub use model_cache::{
    EmbeddingModelCacheFile, prepare_embedding_model_cache, required_embedding_model_cache_files,
    validate_embedding_model_cache,
};
pub use passages::{PassageSection, PreparedEmbeddingInput, prepare_embedding_section};
pub use tokenization::TextEmbeddingTokenizer;
#[cfg(test)]
mod tests;
