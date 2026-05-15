#![deny(unsafe_code)]

use std::path::Path;
use std::time::Instant;

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
    pub embedding_unit_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
    pub rank_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchTiming {
    pub query_embedding_duration_ms: u128,
    pub vector_search_duration_ms: u128,
    pub total_duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub hits: Vec<SemanticSearchHit>,
    pub timing: SemanticSearchTiming,
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
        Ok(self.semantic_with_timing(query, filter, limit)?.hits)
    }

    pub fn semantic_with_timing(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<SemanticSearchResult, SearchError> {
        let total_started_at = Instant::now();
        let embedding_started_at = Instant::now();
        let query_vector = self.embedder.embed_query(query)?;
        let query_embedding_duration_ms = embedding_started_at.elapsed().as_millis();
        let vector_started_at = Instant::now();
        let hits = self
            .index
            .query_vector_index(&query_vector, filter, limit)?;
        let vector_search_duration_ms = vector_started_at.elapsed().as_millis();
        Ok(SemanticSearchResult {
            hits: hits.into_iter().map(SemanticSearchHit::from).collect(),
            timing: SemanticSearchTiming {
                query_embedding_duration_ms,
                vector_search_duration_ms,
                total_duration_ms: total_started_at.elapsed().as_millis(),
            },
        })
    }
}

impl From<VectorSearchHit> for SemanticSearchHit {
    fn from(hit: VectorSearchHit) -> Self {
        Self {
            record_key: hit.record_key,
            embedding_unit_key: hit.embedding_unit_key,
            unit_kind: hit.unit_kind,
            label: hit.label,
            distance: hit.distance,
            rank_distance: hit.rank_distance,
        }
    }
}
