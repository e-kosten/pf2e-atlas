use crate::SearchError;
use atlas_domain::SearchProfileId;
use atlas_embedding::SharedQueryInference;
use atlas_index::SqliteIndexReader;

pub struct AtlasRetrievalService {
    pub(crate) index: SqliteIndexReader,
    query_embedding: Option<PreparedQueryEmbedding>,
    default_profile: SearchProfileId,
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
    /// A bounded read operation. Query inference must already be complete.
    pub fn from_index(
        index: SqliteIndexReader,
        default_profile: SearchProfileId,
        query_embedding: Option<PreparedQueryEmbedding>,
    ) -> Result<Self, SearchError> {
        if let Some(prepared) = &query_embedding
            && index
                .context()
                .search_profile(&prepared.profile.profile_id)?
                != &prepared.profile
        {
            return Err(SearchError::vector_readiness_required(
                "query model changed during preparation; retry",
            ));
        }
        Ok(Self {
            index,
            default_profile,
            query_embedding,
        })
    }
    /// Connection plumbing for the runtime composition owner. Product consumers
    /// use runtime's typed local-state capability instead of this session.
    #[doc(hidden)]
    pub fn workspace_session(&self) -> &atlas_storage::ReadSession {
        self.index.workspace_session()
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
        let prepared = self.query_embedding.as_ref().ok_or_else(|| {
            SearchError::vector_readiness_required(
                "semantic query must be prepared before opening the read operation",
            )
        })?;
        if &prepared.profile.profile_id != id || prepared.text != text {
            return Err(SearchError::invalid_search_options(
                "prepared query does not match requested text/profile",
            ));
        }
        if self.index.context().search_profile(id)? != &prepared.profile {
            return Err(SearchError::vector_readiness_required(
                "query model changed during preparation; retry",
            ));
        }
        Ok(prepared.vector.clone())
    }
    /// Durable workspace identity and the revision of this admitted snapshot.
    pub fn artifact_generation(&self) -> Result<String, crate::SearchError> {
        Ok(self.index.artifact_instance().generation())
    }

    /// Durable identity of the admitted workspace reference revision.
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
        let query = atlas_index::validate_query(predicate)?;
        self.check_relationship_leaves(predicate)?;
        Ok(query)
    }
    pub fn parse_where(&self, source: &str) -> Result<atlas_index::ValidatedQuery, SearchError> {
        let query = atlas_index::parse_where(source)?;
        self.validate_filter(query.predicate())
    }
    pub(crate) fn predicate(
        &self,
        predicate: Option<&atlas_domain::QueryPredicate>,
    ) -> Result<atlas_index::ValidatedQuery, SearchError> {
        self.validate_filter(predicate.unwrap_or(&atlas_domain::QueryPredicate::boolean(true)))
    }
}

/// Exact query/model binding prepared without an admitted database session.
pub struct PreparedQueryEmbedding {
    profile: atlas_index::SourceSearchProfileIdentity,
    text: String,
    vector: Vec<f32>,
}
impl PreparedQueryEmbedding {
    pub fn prepare(
        inference: &SharedQueryInference,
        profile: &SearchProfileId,
        text: &str,
    ) -> Result<Self, SearchError> {
        if text.trim().is_empty() {
            return Err(SearchError::invalid_search_options(
                "query must not be empty",
            ));
        }
        let spec =
            atlas_embedding::search_profile(profile).map_err(SearchError::query_embedding)?;
        let vector = inference
            .embed_query(profile, text)
            .map_err(SearchError::query_embedding)?;
        Ok(Self {
            profile: atlas_index::SourceSearchProfileIdentity::from_spec(spec),
            text: text.into(),
            vector,
        })
    }
}
