use crate::AtlasRuntime;
use atlas_search::{AtlasRetrievalService, SearchError};
pub struct SearchProfileReadiness {
    pub id: atlas_domain::SearchProfileId,
    pub label: &'static str,
    pub stored_vectors: bool,
    pub assets_ready: bool,
    pub usable: bool,
}
impl AtlasRuntime {
    pub fn resolve_default_search_profile(
        &self,
        context: &atlas_index::SourceArtifactBuildContext,
    ) -> Result<atlas_domain::SearchProfileId, SearchError> {
        if let Some(id) = &self.default_profile {
            context.search_profile(id)?;
            return Ok(id.clone());
        }
        Ok(match context.search_profiles.as_slice() {
            [profile] => profile.profile_id.clone(),
            _ => atlas_domain::SearchProfileId::new(atlas_embedding::DEFAULT_SEARCH_PROFILE),
        })
    }
    /// Check stored contracts and verified assets without constructing native sessions.
    pub fn search_profile_readiness(
        &self,
        stored: &[atlas_index::SourceSearchProfileIdentity],
    ) -> Vec<SearchProfileReadiness> {
        let loaded = self.inference.loaded_profiles();
        stored
            .iter()
            .filter_map(|stored_profile| {
                let profile = atlas_embedding::search_profile(&stored_profile.profile_id).ok()?;
                let expected = atlas_index::SourceSearchProfileIdentity::from_spec(profile);
                let assets_ready = self.profile_assets.ready(
                    &profile.profile_id(),
                    &atlas_embedding::EmbeddingRuntimeConfig::new(
                        profile.model,
                        self.embedding_cache_root(),
                    ),
                );
                Some(SearchProfileReadiness {
                    id: profile.profile_id(),
                    label: profile.label,
                    stored_vectors: true,
                    assets_ready,
                    usable: (assets_ready || loaded.contains(&profile.profile_id()))
                        && stored_profile == &expected,
                })
            })
            .collect()
    }
    pub fn open_retrieval_service(&self) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index()?;
        let default_profile = self.resolve_default_search_profile(index.context())?;
        AtlasRetrievalService::from_prepared_index(index, default_profile, self.inference.clone())
    }
    pub fn open_retrieval_service_no_embeddings(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_index()?;
        let default_profile = self.resolve_default_search_profile(index.context())?;
        Ok(AtlasRetrievalService::from_prepared_index_without_embeddings(index, default_profile))
    }
    pub fn open_retrieval_service_for_stored_vectors(
        &self,
    ) -> Result<AtlasRetrievalService, SearchError> {
        let index = self.open_search_index()?;
        let default_profile = self.resolve_default_search_profile(index.context())?;
        Ok(AtlasRetrievalService::from_prepared_index_without_embeddings(index, default_profile))
    }
}
