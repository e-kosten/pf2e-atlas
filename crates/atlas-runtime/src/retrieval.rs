use crate::AtlasRuntime;
use atlas_embedding::embedding_model_for_model_id;
use atlas_search::{AtlasRetrievalService, SearchEmbeddingConfig, SearchError};
impl AtlasRuntime {
    pub fn open_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index()?;
        let model = index.context().semantic_model.as_ref().ok_or_else(|| {
            SearchError::vector_readiness_required(
                "artifact has no embeddings; rebuild with embeddings",
            )
        })?;
        let model = embedding_model_for_model_id(&model.model_id).ok_or_else(|| {
            SearchError::artifact_contract_violation("unsupported artifact embedding model")
        })?;
        AtlasRetrievalService::from_prepared_index(
            index,
            &SearchEmbeddingConfig::new(model, self.embedding_cache_root()),
        )
    }
    pub fn open_retrieval_service_no_embeddings(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        Ok(AtlasRetrievalService::from_prepared_index_without_embeddings(self.open_index()?))
    }
    pub fn open_retrieval_service_for_stored_vectors(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        Ok(
            AtlasRetrievalService::from_prepared_index_without_embeddings(
                self.open_search_index()?,
            ),
        )
    }
}
