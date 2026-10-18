use crate::AtlasRuntime;
use atlas_search::SearchError;
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
    /// Resolve profile metadata under a short snapshot, then infer with no
    /// admitted database session. The consuming operation checks the profile again.
    pub fn prepare_query(
        &self,
        requested: Option<&atlas_domain::SearchProfileId>,
        text: &str,
    ) -> Result<atlas_search::PreparedQueryEmbedding, SearchError> {
        let profile = {
            let index = self.open_index()?;
            index
                .require_vectors()
                .map_err(|error| SearchError::vector_readiness_required(error.to_string()))?;
            let profile = requested
                .cloned()
                .unwrap_or(self.resolve_default_search_profile(index.context())?);
            let spec =
                atlas_embedding::search_profile(&profile).map_err(SearchError::query_embedding)?;
            if index.context().search_profile(&profile)?
                != &atlas_index::SourceSearchProfileIdentity::from_spec(spec)
            {
                return Err(SearchError::vector_readiness_required(
                    "stored query model differs from the executable contract",
                ));
            }
            profile
        };
        atlas_search::PreparedQueryEmbedding::prepare(&self.inference, &profile, text)
    }
}
