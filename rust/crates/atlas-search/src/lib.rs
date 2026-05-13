#![deny(unsafe_code)]

use std::path::Path;

use atlas_domain::SearchFilterNode;
use atlas_embedding::{EmbeddingError, TextEmbedder};
use atlas_index::{AtlasIndex, IndexValidationError, VectorQueryError, VectorSearchHit};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use atlas_embedding::EmbeddingRuntimeConfig;

pub struct SemanticSearchService {
    index: AtlasIndex,
    embedder: TextEmbedder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchHit {
    pub record_key: String,
    pub distance: f64,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Index(#[from] IndexValidationError),
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
    #[error(transparent)]
    Vector(#[from] VectorQueryError),
}

impl SemanticSearchService {
    pub fn open(
        index_path: impl AsRef<Path>,
        embedding_config: &EmbeddingRuntimeConfig,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            index: AtlasIndex::open_read_only(index_path)?,
            embedder: TextEmbedder::load(embedding_config)?,
        })
    }

    pub fn semantic(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<SemanticSearchHit>, SearchError> {
        let query_vector = self.embedder.embed_query(query)?;
        let hits = self
            .index
            .query_vector_index(&query_vector, filter, limit)?;
        Ok(hits.into_iter().map(SemanticSearchHit::from).collect())
    }
}

impl From<VectorSearchHit> for SemanticSearchHit {
    fn from(hit: VectorSearchHit) -> Self {
        Self {
            record_key: hit.record_key,
            distance: hit.distance,
        }
    }
}
