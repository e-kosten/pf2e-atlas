use std::path::{Path, PathBuf};

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
    model: EmbeddingModelId,
    cache_root: PathBuf,
}

impl SearchEmbeddingConfig {
    pub fn new(model: EmbeddingModelId, cache_root: impl Into<PathBuf>) -> Self {
        Self {
            model,
            cache_root: cache_root.into(),
        }
    }

    pub fn model(&self) -> EmbeddingModelId {
        self.model
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }
}

impl AtlasRetrievalService {
    /// Low-level constructor for callers that have already applied runtime path,
    /// readiness, and embedding-model policy.
    ///
    /// Product callers should normally use `AtlasRuntime::open_retrieval_service`.
    pub fn from_prepared_index(
        index: SqliteIndexReader,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Self::from_prepared_read_index(Box::new(index), embedding_config)
    }

    pub(crate) fn from_prepared_read_index(
        index: Box<dyn RetrievalReadIndex>,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            embedder: Some(load_embedder(embedding_config)?),
            index,
        })
    }

    /// Low-level constructor for callers that have already applied runtime path
    /// and artifact-readiness policy and intentionally do not need embeddings.
    ///
    /// Product callers should normally use
    /// `AtlasRuntime::open_retrieval_service_no_embeddings`.
    pub fn from_prepared_index_without_embeddings(index: SqliteIndexReader) -> Self {
        Self::from_prepared_read_index_without_embeddings(Box::new(index))
    }

    pub(crate) fn from_prepared_read_index_without_embeddings(
        index: Box<dyn RetrievalReadIndex>,
    ) -> Self {
        Self {
            index,
            embedder: None,
        }
    }
}

fn load_embedder(embedding_config: &SearchEmbeddingConfig) -> Result<TextEmbedder, SearchError> {
    let embedding_config =
        EmbeddingRuntimeConfig::new(embedding_config.model(), embedding_config.cache_root());
    TextEmbedder::load(&embedding_config).map_err(|error| SearchError::embedding(error.to_string()))
}
