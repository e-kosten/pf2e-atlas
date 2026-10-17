use crate::SearchError;
use atlas_domain::SearchProfileId;
use atlas_embedding::SharedQueryInference;
use atlas_index::SqliteIndexReader;

pub struct AtlasRetrievalService {
    pub(crate) index: SqliteIndexReader,
    pub(crate) inference: Option<SharedQueryInference>,
    default_profile: SearchProfileId,
    #[cfg(test)]
    pub(crate) text_query_embedding_invocations: usize,
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
        default_profile: SearchProfileId,
        inference: SharedQueryInference,
    ) -> Result<Self, SearchError> {
        index.context().search_profile(&default_profile)?;
        index
            .capabilities()
            .vector_unavailable_reason
            .map_or(Ok(()), |reason| {
                Err(SearchError::vector_readiness_required(reason))
            })?;
        let service = Self {
            index,
            inference: Some(inference),
            default_profile,
            #[cfg(test)]
            text_query_embedding_invocations: 0,
        };
        service.require_current_query_profile(&service.default_profile)?;
        service
            .inference
            .as_ref()
            .ok_or_else(|| {
                SearchError::vector_readiness_required("query inference owner unavailable")
            })?
            .preload(&service.default_profile)
            .map_err(SearchError::query_embedding)?;
        Ok(service)
    }
    pub fn from_prepared_index_without_embeddings(
        index: SqliteIndexReader,
        default_profile: SearchProfileId,
    ) -> Self {
        Self {
            index,
            inference: None,
            default_profile,
            #[cfg(test)]
            text_query_embedding_invocations: 0,
        }
    }
    pub fn default_search_profile(&self) -> &SearchProfileId {
        &self.default_profile
    }
    pub fn resolve_search_profile(
        &self,
        requested: Option<&SearchProfileId>,
        required: bool,
    ) -> Result<SearchProfileId, SearchError> {
        let id = requested.unwrap_or(&self.default_profile);
        if requested.is_some() || required {
            self.index
                .context()
                .search_profile(id)
                .map_err(|error| SearchError::vector_readiness_required(error.to_string()))?;
        }
        Ok(id.clone())
    }
    pub(crate) fn embed_query(
        &self,
        id: &SearchProfileId,
        text: &str,
    ) -> Result<Vec<f32>, SearchError> {
        self.require_current_query_profile(id)?;
        self.inference
            .as_ref()
            .ok_or_else(|| {
                SearchError::vector_readiness_required(
                    "query embeddings are unavailable in this runtime mode",
                )
            })?
            .embed_query(id, text)
            .map_err(SearchError::query_embedding)
    }
    fn require_current_query_profile(&self, id: &SearchProfileId) -> Result<(), SearchError> {
        let stored = self.index.context().search_profile(id)?;
        let spec = atlas_embedding::search_profile(id).map_err(SearchError::query_embedding)?;
        if stored != &atlas_index::SourceSearchProfileIdentity::from_spec(spec) {
            return Err(SearchError::vector_readiness_required(format!(
                "stored profile `{id}` differs from the current query model/input contract; rebuild with --search-profile {id}"
            )));
        }
        Ok(())
    }
    /// Complete immutable artifact preparation identity, separate from source-local addresses.
    pub fn artifact_generation(&self) -> Result<String, crate::SearchError> {
        Ok(self.artifact_context().sha256()?)
    }

    /// Process-local identity used to compose independent readers of one file.
    pub fn artifact_instance(&self) -> &crate::ArtifactInstance {
        self.index.artifact_instance()
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
