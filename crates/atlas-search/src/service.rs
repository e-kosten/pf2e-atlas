use crate::SearchError;
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, TextEmbedder};
use atlas_index::SqliteIndexReader;
use std::path::{Path, PathBuf};
pub struct AtlasRetrievalService {
    pub(crate) index: SqliteIndexReader,
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
    pub fn source_fingerprint(&self) -> &str {
        &self.index.context().source_fingerprint
    }
    pub fn trait_label(&self, identifier: &str) -> Option<&str> {
        self.index
            .context()
            .used_trait_labels
            .get(identifier)
            .map(String::as_str)
    }
    pub fn from_prepared_index(
        index: SqliteIndexReader,
        config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        if index.context().semantic_model.is_none() {
            return Err(SearchError::vector_readiness_required(
                "artifact has no embeddings; rebuild with embeddings",
            ));
        }
        if index.context().semantic_model.as_ref()
            != Some(&atlas_index::SourceSemanticModelIdentity::current())
        {
            return Err(SearchError::artifact_contract_violation(
                "artifact query model identity differs from the pinned inference contract",
            ));
        }
        if let Some(reason) = index.capabilities().vector_unavailable_reason {
            return Err(SearchError::vector_readiness_required(reason));
        }
        let embedder = TextEmbedder::load(&EmbeddingRuntimeConfig::new(
            config.model,
            config.cache_root(),
        ))
        .map_err(|e| SearchError::embedding(e.to_string()))?;
        Ok(Self {
            index,
            embedder: Some(embedder),
        })
    }
    pub fn from_prepared_index_without_embeddings(index: SqliteIndexReader) -> Self {
        Self {
            index,
            embedder: None,
        }
    }
    pub fn artifact_context(&self) -> &atlas_index::SourceArtifactBuildContext {
        self.index.context()
    }
    pub fn validate_filter(
        &self,
        predicate: &atlas_domain::QueryPredicate,
    ) -> Result<atlas_index::ValidatedQuery, SearchError> {
        Ok(atlas_index::validate_query(predicate)?)
    }
    pub fn parse_where(&self, source: &str) -> Result<atlas_index::ValidatedQuery, SearchError> {
        Ok(atlas_index::parse_where(source)?)
    }
    pub(crate) fn predicate(
        &self,
        predicate: Option<&atlas_domain::QueryPredicate>,
    ) -> Result<atlas_index::ValidatedQuery, SearchError> {
        Ok(atlas_index::validate_query(
            predicate.unwrap_or(&atlas_domain::QueryPredicate::boolean(true)),
        )?)
    }
}
