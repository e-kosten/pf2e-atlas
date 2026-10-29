#![deny(unsafe_code)]
mod catalog;
mod document_input;
mod error;
mod inference;
mod model_cache;
mod passage_reuse;
mod passages;
mod prepared_batch;
mod profiles;
mod query_sessions;
pub use profiles::{
    BASELINE_INPUT_RECIPE, BASELINE_SEGMENTATION_TOKENIZER_SHA256, DEFAULT_SEARCH_PROFILE,
    SEARCH_PROFILES, SearchProfileSpec, profile_for_model, search_profile,
};
pub use query_sessions::SharedQueryInference;
mod tokenization;

pub const EMBEDDING_UNIT_POLICY_VERSION: &str = "source-passages/v2/text-splitter0.29.3/body256-context64-overlap32/identity480/baseline-bge512/native-final-by-profile";
/// Pinned inference libraries and dense execution behavior used for vector reuse.
pub const EMBEDDING_INFERENCE_POLICY_VERSION: &str =
    "fastembed5.15.0/tokenizers0.22.2/f32/native-specials/no-truncation/right-padding/threads4";
pub const BODY_TOKEN_BUDGET: usize = 256;
pub const CONTEXT_TOKEN_BUDGET: usize = 64;
pub const OVERLAP_TOKEN_BUDGET: usize = 32;
pub const IDENTITY_TOKEN_BUDGET: usize = 480;
pub use catalog::{
    ALL_EMBEDDING_MODELS, DistanceMetric, EmbeddingModelId, EmbeddingModelSpec,
    EmbeddingRuntimeConfig, Normalization, PoolingStrategy, VectorDType,
    embedding_model_for_model_id, embedding_model_spec, supported_embedding_model_ids,
};
pub use document_input::hash_document_embedding_input;
pub use error::EmbeddingError;
pub use inference::{
    GeneratedEmbeddings, TextEmbedder, generate_prepared_embeddings, validate_embedding_vector,
};
pub use model_cache::{
    EmbeddingModelCacheFile, baseline_tokenizer_cache_file, prepare_baseline_tokenizer_cache,
    prepare_embedding_model_cache, required_embedding_model_cache_files,
    validate_embedding_model_cache,
};
pub use passage_reuse::{PassageLayout, reuse_embedding_section};
pub use passages::{PassageSection, PreparedEmbeddingInput, prepare_embedding_section};
pub use prepared_batch::PreparedEmbeddingBatch;
pub use tokenization::TextEmbeddingTokenizer;
#[cfg(test)]
mod tests;

mod centroid;
pub use centroid::normalized_centroid;
