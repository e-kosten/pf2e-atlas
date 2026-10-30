use crate::RecordScope;
use atlas_domain::QueryPredicate;
#[derive(Clone, Copy)]
pub struct FilterDiscoveryContext<'a> {
    pub content_scope: atlas_domain::SearchContentScope,
    pub search_profile: Option<&'a atlas_domain::SearchProfileId>,
    pub text: Option<&'a str>,
    pub scope: RecordScope<'a>,
    pub prefer_remaster: bool,
    pub filter: Option<&'a QueryPredicate>,
}
impl FilterDiscoveryContext<'_> {
    fn validate_content_scope(self) -> Result<(), crate::SearchError> {
        if self.content_scope == atlas_domain::SearchContentScope::Core
            && self.text.is_none_or(|text| text.trim().is_empty())
        {
            return Err(crate::SearchError::invalid_search_options(
                "Core search content requires a nonempty text search context",
            ));
        }
        Ok(())
    }
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
    pub fn discover_filter_value_counts(
        &mut self,
        request: DiscoverFilterValueCountsRequest<'_>,
    ) -> Result<QueryFacetOutcome<QueryValueOptions>, SearchError> {
        request.context.validate_content_scope()?;
        atlas_index::validate_value_counts(request.field, request.values)?;
        let query = self.predicate(request.context.filter)?;
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
        context.validate_content_scope()?;
        let query = self.predicate(context.filter)?;
        self.resolve_search_profile(context.search_profile, false)?;
        // Index owns self-exclusion analysis, including errors in the addressed clause.
        if let QueryFacetOutcome::Unavailable { reason } =
            atlas_index::facet_base_query(field, clause, &query)?
        {
            return Ok(QueryFacetOutcome::Unavailable { reason });
        }
        if context.text.is_some_and(|text| !text.trim().is_empty()) {
            return Ok(QueryFacetOutcome::Unavailable {
                reason: atlas_domain::FacetUnavailableReason::TextContext,
            });
        }
        Ok(QueryFacetOutcome::Available {
            value: (
                query,
                QueryFacetContext {
                    eligible_keys: context.scope.keys().map(<[_]>::to_vec),
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
