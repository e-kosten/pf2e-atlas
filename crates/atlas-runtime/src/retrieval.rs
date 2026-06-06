use atlas_embedding::{EmbeddingModelId, embedding_model_for_model_id};
use atlas_search::{AtlasRetrievalService, SearchEmbeddingConfig, SearchError};

use crate::runtime::AtlasRuntime;

impl AtlasRuntime {
    pub fn open_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index().map_err(search_error_from_index)?;
        let report = index
            .validate_embedding_readiness()
            .map_err(search_error_from_index)?;
        if report.status != atlas_index::ValidationStatus::Ok {
            return Err(SearchError::vector_readiness_required(report.message));
        }
        let config = SearchEmbeddingConfig::new(
            embedding_model_from_artifact_report(&report)?,
            self.paths().embedding_cache_root.clone(),
        );
        AtlasRetrievalService::from_prepared_index(index, &config)
    }

    pub fn open_retrieval_service_no_embeddings(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        Ok(
            AtlasRetrievalService::from_prepared_index_without_embeddings(
                self.open_index().map_err(search_error_from_index)?,
            ),
        )
    }

    pub fn open_retrieval_service_for_stored_vectors(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index().map_err(search_error_from_index)?;
        let report = index
            .validate_embedding_readiness()
            .map_err(search_error_from_index)?;
        if report.status != atlas_index::ValidationStatus::Ok {
            return Err(SearchError::artifact_contract_violation(
                atlas_index::IndexValidationError::InvalidArtifact(report.message).to_string(),
            ));
        }
        Ok(AtlasRetrievalService::from_prepared_index_without_embeddings(index))
    }
}

fn embedding_model_from_artifact_report(
    report: &atlas_index::ArtifactValidationReport,
) -> Result<EmbeddingModelId, SearchError> {
    let model_id = report.embedding_model_id.as_deref().ok_or_else(|| {
        SearchError::artifact_contract_violation(
            "artifact embedding metadata is missing `embedding_model_id`",
        )
    })?;
    embedding_model_for_model_id(model_id).ok_or_else(|| {
        SearchError::artifact_contract_violation(format!(
            "artifact embedding model `{model_id}` is not supported by this runtime"
        ))
    })
}

fn search_error_from_index(error: atlas_index::IndexValidationError) -> SearchError {
    match error {
        atlas_index::IndexValidationError::Unavailable(_) => {
            SearchError::index_unavailable(error.to_string())
        }
        atlas_index::IndexValidationError::InvalidArtifact(_) => {
            SearchError::artifact_contract_violation(error.to_string())
        }
        atlas_index::IndexValidationError::QueryFailed(_) => {
            SearchError::query_failed(error.to_string())
        }
    }
}
