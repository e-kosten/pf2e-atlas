#![deny(unsafe_code)]

use std::path::PathBuf;
use std::str::FromStr;

use atlas_domain::RemasterLinkSource;
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use atlas_index::{
    FilterCompileError, GraphReadIndex, IndexValidationError, RecordLoadError, SearchIndex,
    SqliteIndexReader, VectorQueryError,
};
use atlas_record::PersistedRecord;
use thiserror::Error;

mod fusion;
mod graph_context;
mod graph_product;
mod query;
mod records;
mod references;
mod resolution;
mod semantic;
mod text;

pub use fusion::{
    DEFAULT_FTS_FUSION_POLICY_NAME, FtsMatchConfidence, FusionMethod, FusionOptions,
    TextSearchExplain,
};
pub use graph_context::{
    GraphContextEdge, GraphContextEdgeSource, GraphContextRequest, GraphContextResult,
    GraphContextSection,
};
pub use query::TextQueryAnalysis;
pub use records::FilterOnlyRecordPage;
pub use resolution::{RecordResolutionMatchKind, RecordResolutionResult};
pub use semantic::{
    SemanticSearchHit, SemanticSearchMode, SemanticSearchResult, SemanticSearchTiming,
};
pub use text::{
    AtlasSearchRequest, AtlasSearchResult, RetrievalMode, TextSearchMatch, TextSearchPage,
    TextSearchRecord, TextSearchRequest,
};

/// Initial top-level retrieval boundary for Rust runtime consumers.
///
/// The service owns product-facing retrieval entrypoints. Record get/resolve,
/// filter-only list, and ranked FTS/vector/hybrid search should be added here
/// rather than as peer public services.
pub struct AtlasRetrievalService {
    pub(crate) index: Box<dyn RetrievalIndex>,
    pub(crate) embedder: Option<TextEmbedder>,
}

pub trait RetrievalIndex: SearchIndex + GraphReadIndex {}

impl<T> RetrievalIndex for T where T: SearchIndex + GraphReadIndex {}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Index(#[from] IndexValidationError),
    #[error(transparent)]
    RecordLoad(#[from] RecordLoadError),
    #[error("invalid embedding model `{model}`: {message}")]
    InvalidEmbeddingModel { model: String, message: String },
    #[error("embedding operation failed: {0}")]
    Embedding(String),
    #[error("invalid search options: {0}")]
    InvalidSearchOptions(String),
    #[error("retrieval pattern is not implemented yet: {0}")]
    UnsupportedRetrievalPattern(&'static str),
    #[error(transparent)]
    Filter(#[from] FilterCompileError),
    #[error(transparent)]
    Vector(#[from] VectorQueryError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model_id: String,
    pub cache_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphVariantGroupResult {
    pub seed: Option<PersistedRecord>,
    pub variant_group_key: Option<String>,
    pub variants: Vec<PersistedRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphRemasterLinksResult {
    pub seed: PersistedRecord,
    pub links: Vec<GraphRemasterLinkResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphRemasterLinkResult {
    pub remaster_record: PersistedRecord,
    pub legacy_record: PersistedRecord,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

impl AtlasRetrievalService {
    pub fn new(
        index: SqliteIndexReader,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        Self::new_with_index(Box::new(index), embedding_config)
    }

    pub fn new_with_index(
        index: Box<dyn RetrievalIndex>,
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

    pub fn without_embeddings_with_index(index: Box<dyn RetrievalIndex>) -> Self {
        Self {
            index,
            embedder: None,
        }
    }

    pub fn search(
        &mut self,
        request: AtlasSearchRequest<'_>,
    ) -> Result<AtlasSearchResult, SearchError> {
        match request {
            AtlasSearchRequest::Semantic {
                query,
                filter,
                limit,
                mode,
            } => self
                .semantic_with_timing(query, filter, limit, mode)
                .map(AtlasSearchResult::Semantic),
            AtlasSearchRequest::FilterOnly { .. } => Err(SearchError::UnsupportedRetrievalPattern(
                "filter-only search",
            )),
            AtlasSearchRequest::Text(request) => {
                self.text_search(request).map(AtlasSearchResult::Text)
            }
        }
    }
}

fn load_embedder(embedding_config: &SearchEmbeddingConfig) -> Result<TextEmbedder, SearchError> {
    let model = EmbeddingModelId::from_str(&embedding_config.model_id).map_err(|error| {
        SearchError::InvalidEmbeddingModel {
            model: embedding_config.model_id.clone(),
            message: error.to_string(),
        }
    })?;
    let embedding_config = EmbeddingRuntimeConfig::new(model, &embedding_config.cache_root);
    TextEmbedder::load(&embedding_config).map_err(|error| SearchError::Embedding(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_retrieval_patterns_are_typed_errors() {
        let error = SearchError::UnsupportedRetrievalPattern("record get");

        assert!(matches!(
            error,
            SearchError::UnsupportedRetrievalPattern("record get")
        ));
        assert_eq!(
            error.to_string(),
            "retrieval pattern is not implemented yet: record get"
        );
    }
}
