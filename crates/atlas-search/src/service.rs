use std::path::PathBuf;

use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use atlas_index::{RetrievalReadIndex, SqliteIndexReader};

use crate::SearchError;

/// Product-facing retrieval boundary for Rust runtime consumers.
///
/// The service owns retrieval orchestration. Runtime code constructs it from
/// prepared index handles; callers consume its product methods or narrower
/// capability traits rather than assembling index/embedding pieces directly.
pub struct AtlasRetrievalService {
    pub(crate) index: Box<dyn RetrievalReadIndex>,
    pub(crate) embedder: Option<TextEmbedder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model: EmbeddingModelId,
    pub cache_root: PathBuf,
}

impl AtlasRetrievalService {
    pub fn new(
        index: SqliteIndexReader,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Self::new_with_index(Box::new(index), embedding_config)
    }

    pub(crate) fn new_with_index(
        index: Box<dyn RetrievalReadIndex>,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            embedder: Some(load_embedder(embedding_config)?),
            index,
        })
    }

    pub fn without_embeddings(index: SqliteIndexReader) -> Self {
        Self::without_embeddings_with_index(Box::new(index))
    }

    pub(crate) fn without_embeddings_with_index(index: Box<dyn RetrievalReadIndex>) -> Self {
        Self {
            index,
            embedder: None,
        }
    }
}

fn load_embedder(embedding_config: &SearchEmbeddingConfig) -> Result<TextEmbedder, SearchError> {
    let embedding_config =
        EmbeddingRuntimeConfig::new(embedding_config.model, &embedding_config.cache_root);
    TextEmbedder::load(&embedding_config).map_err(|error| SearchError::embedding(error.to_string()))
}
