use crate::{RecordScope, RetrievalMode};
use atlas_domain::QueryPredicate;
#[derive(Clone, Copy)]
pub struct FilterDiscoveryContext<'a> {
    pub search_profile: Option<&'a atlas_domain::SearchProfileId>,
    pub text: Option<&'a str>,
    pub mode: RetrievalMode,
    pub scope: RecordScope<'a>,
    pub prefer_remaster: bool,
    pub filter: Option<&'a QueryPredicate>,
}
pub struct DiscoverFilterValuesRequest<'a> {
    pub only_matching: bool,
    pub field: &'a str,
    pub clause_id: Option<&'a str>,
    pub context: FilterDiscoveryContext<'a>,
    pub text: Option<&'a str>,
    pub offset: usize,
    pub limit: usize,
}
pub struct DiscoverFilterCountsRequest<'a> {
    pub field: &'a str,
    pub clause_id: Option<&'a str>,
    pub context: FilterDiscoveryContext<'a>,
}
pub struct DiscoverFilterValueCountsRequest<'a> {
    pub field: &'a str,
    pub clause_id: Option<&'a str>,
    pub context: FilterDiscoveryContext<'a>,
    pub values: &'a [atlas_domain::QueryLiteral],
}
use crate::{AtlasRetrievalService, SearchError};
use atlas_domain::{
    QueryCapability, QueryFacetOutcome, QueryFieldCounts, QueryLiteral, QueryValueOptions,
    QueryVocabulary,
};
use atlas_index::{
    QueryCountsRequest, QueryFacetContext, QueryValuesRequest, QueryVocabularyRequest,
};
pub struct DiscoverFilterVocabularyRequest<'a> {
    pub field: &'a str,
    pub scope: RecordScope<'a>,
    pub text: Option<&'a str>,
    pub offset: usize,
    pub limit: usize,
    pub selected_values: &'a [QueryLiteral],
}
impl AtlasRetrievalService {
    /// Preflight contextual eligibility before the caller performs query inference.
    pub fn discovery_supports_ranking(
        &self,
        field: &str,
        clause: Option<&str>,
        filter: Option<&QueryPredicate>,
    ) -> Result<bool, SearchError> {
        let query = self.predicate(filter)?;
        Ok(matches!(
            atlas_index::facet_base_query(field, clause, &query)?,
            QueryFacetOutcome::Available { .. }
        ))
    }

    pub fn discover_filter_value_counts(
        &mut self,
        request: DiscoverFilterValueCountsRequest<'_>,
    ) -> Result<QueryFacetOutcome<QueryValueOptions>, SearchError> {
        atlas_index::validate_value_counts(request.field, request.values)?;
        let query = self
            .predicate(request.context.filter)?
            .with_relationships(request.context.scope.relationships());
        self.resolve_search_profile(request.context.search_profile, false)?;
        if let QueryFacetOutcome::Unavailable { reason } =
            atlas_index::facet_base_query(request.field, request.clause_id, &query)?
        {
            return Ok(QueryFacetOutcome::Unavailable { reason });
        }
        if request
            .context
            .text
            .is_some_and(|text| !text.trim().is_empty())
        {
            return Ok(QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::TextContext,
            });
        }
        Ok(self
            .index
            .field_value_counts(&atlas_index::QueryValueCountsRequest {
                field: request.field.into(),
                clause_id: request.clause_id.map(str::to_owned),
                query,
                context: QueryFacetContext {
                    eligible_keys: request.context.scope.keys().map(<[_]>::to_vec),
                    bounded_candidates: false,
                    prefer_remaster: request.context.prefer_remaster,
                },
                values: request.values.to_vec(),
            })?)
    }
    pub fn discover_filter_fields(&self) -> Result<QueryCapability, SearchError> {
        Ok(atlas_index::query_capabilities()?)
    }
    fn discovery_context(
        &mut self,
        field: &str,
        clause: Option<&str>,
        context: FilterDiscoveryContext<'_>,
    ) -> Result<QueryFacetOutcome<(atlas_index::ValidatedQuery, QueryFacetContext)>, SearchError>
    {
        let query = self
            .predicate(context.filter)?
            .with_relationships(context.scope.relationships());
        self.resolve_search_profile(context.search_profile, false)?;
        // Index owns the only self-exclusion analysis. Perform it even without
        // text, and before ranking or loading a query embedder.
        let base = match atlas_index::facet_base_query(field, clause, &query)? {
            QueryFacetOutcome::Available { value } => value,
            QueryFacetOutcome::Unavailable { reason } => {
                return Ok(QueryFacetOutcome::Unavailable { reason });
            }
        };
        let search_profile = self.resolve_search_profile(
            context.search_profile,
            context.mode.uses_vector() && context.text.is_some_and(|t| !t.trim().is_empty()),
        )?;
        let Some(text) = context.text.filter(|s| !s.trim().is_empty()) else {
            return Ok(QueryFacetOutcome::Available {
                value: (
                    query,
                    QueryFacetContext {
                        eligible_keys: context.scope.keys().map(<[_]>::to_vec),
                        bounded_candidates: false,
                        prefer_remaster: context.prefer_remaster,
                    },
                ),
            });
        };
        let vector = if context.mode.uses_vector() {
            Some(self.embed_query(&search_profile, text)?)
        } else {
            None
        };
        // Freeze the separate clause-removed universe before per-option pair policy.
        let (roots, _) = self.rank_candidates(crate::text::CandidateRequest {
            ranking: Default::default(),
            search_profile: &search_profile,
            text,
            mode: context.mode,
            query: &base,
            scope: context.scope,
            prefer: context.prefer_remaster,
            suppress: false,
            vector: vector.as_deref(),
            exclude: None,
        })?;
        Ok(QueryFacetOutcome::Available {
            value: (
                query,
                QueryFacetContext {
                    eligible_keys: Some(roots.into_iter().map(|r| r.key).collect()),
                    bounded_candidates: context.mode.uses_vector(),
                    prefer_remaster: context.prefer_remaster,
                },
            ),
        })
    }
    pub fn discover_filter_values(
        &mut self,
        request: DiscoverFilterValuesRequest<'_>,
    ) -> Result<QueryFacetOutcome<QueryValueOptions>, SearchError> {
        atlas_index::validate_value_discovery(
            request.field,
            request.offset,
            request.limit,
            request.text,
        )?;
        let (query, context) =
            match self.discovery_context(request.field, request.clause_id, request.context)? {
                QueryFacetOutcome::Available { value } => value,
                QueryFacetOutcome::Unavailable { reason } => {
                    return Ok(QueryFacetOutcome::Unavailable { reason });
                }
            };
        Ok(self.index.field_values(&QueryValuesRequest {
            only_matching: request.only_matching,
            field: request.field.to_owned(),
            clause_id: request.clause_id.map(str::to_owned),
            query,
            context,
            text: request.text.map(str::to_owned),
            offset: request.offset,
            limit: request.limit,
        })?)
    }
    pub fn discover_filter_counts(
        &mut self,
        request: DiscoverFilterCountsRequest<'_>,
    ) -> Result<QueryFacetOutcome<QueryFieldCounts>, SearchError> {
        let (query, context) =
            match self.discovery_context(request.field, request.clause_id, request.context)? {
                QueryFacetOutcome::Available { value } => value,
                QueryFacetOutcome::Unavailable { reason } => {
                    return Ok(QueryFacetOutcome::Unavailable { reason });
                }
            };
        Ok(self.index.field_counts(&QueryCountsRequest {
            field: request.field.to_owned(),
            clause_id: request.clause_id.map(str::to_owned),
            query,
            context,
        })?)
    }
    pub fn discover_filter_vocabulary(
        &self,
        request: DiscoverFilterVocabularyRequest<'_>,
    ) -> Result<QueryVocabulary, SearchError> {
        Ok(self.index.field_vocabulary(&QueryVocabularyRequest {
            field: request.field.to_owned(),
            eligible_keys: request.scope.keys().map(<[_]>::to_vec),
            text: request.text.map(str::to_owned),
            offset: request.offset,
            limit: request.limit,
            selected_values: request.selected_values.to_vec(),
        })?)
    }
}
