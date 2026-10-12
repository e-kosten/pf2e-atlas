use crate::{AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    DiscoverFilterCountsRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterCountsView, FilterDiscoveryContext, FilterEditorView, FilterValueListView,
    FilterVocabularyScope,
};
use atlas_domain::{
    QueryCapability, QueryFacetOutcome, QueryFieldCounts, QueryPredicate, QueryValueOptions,
    RecordKey,
};
use atlas_search::{
    DiscoverFilterCountsRequest as SearchCounts, DiscoverFilterValuesRequest as SearchValues,
    DiscoverFilterVocabularyRequest as SearchVocabulary, FilterDiscoveryContext as SearchContext,
    RecordScope, RetrievalMode,
};

#[derive(Debug, Clone)]
pub struct RawFilterValuesRequest {
    pub field: String,
    pub filter: Option<QueryPredicate>,
    pub clause_id: Option<String>,
    pub text: Option<String>,
    pub offset: usize,
    pub limit: usize,
}
#[derive(Debug, Clone)]
pub struct RawFilterCountsRequest {
    pub field: String,
    pub filter: Option<QueryPredicate>,
    pub clause_id: Option<String>,
}
impl AtlasAppService {
    pub fn discover_filter_fields(&self) -> AppServiceResult<QueryCapability> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), |retrieval| {
            Ok(retrieval.discover_filter_fields()?)
        })
    }
    pub fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        let (_, filter, _, _) = context_parts(self, request.context)?;
        self.with_retrieval(
            &crate::ReadExecutionContext::uncancellable(),
            move |retrieval| {
                if let Some(filter) = filter {
                    retrieval.validate_filter(&filter)?;
                }
                let catalog = retrieval.discover_filter_fields()?;
                Ok(crate::discovery::filter_editor_view(
                    catalog,
                    &request.selected_field_ids,
                ))
            },
        )
    }
    pub fn discover_raw_filter_values(
        &self,
        request: RawFilterValuesRequest,
    ) -> AppServiceResult<QueryValueOptions> {
        self.with_retrieval(
            &crate::ReadExecutionContext::uncancellable(),
            move |retrieval| {
                required_facet(retrieval.discover_filter_values(SearchValues {
                    field: &request.field,
                    clause_id: request.clause_id.as_deref(),
                    context: SearchContext {
                        text: None,
                        mode: RetrievalMode::Fts,
                        scope: RecordScope::All,
                        prefer_remaster: true,
                        filter: request.filter.as_ref(),
                    },
                    text: request.text.as_deref(),
                    offset: request.offset,
                    limit: request.limit,
                })?)
            },
        )
    }
    pub fn discover_raw_filter_counts(
        &self,
        request: RawFilterCountsRequest,
    ) -> AppServiceResult<QueryFieldCounts> {
        self.with_retrieval(
            &crate::ReadExecutionContext::uncancellable(),
            move |retrieval| {
                required_facet(retrieval.discover_filter_counts(SearchCounts {
                    field: &request.field,
                    clause_id: request.clause_id.as_deref(),
                    context: SearchContext {
                        text: None,
                        mode: RetrievalMode::Fts,
                        scope: RecordScope::All,
                        prefer_remaster: true,
                        filter: request.filter.as_ref(),
                    },
                })?)
            },
        )
    }
    pub fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<FilterValueListView> {
        execution.check()?;
        let projection_execution = execution.clone();
        match request {
            DiscoverFilterValuesRequest::Contextual {
                context,
                field_id,
                clause_id,
                text: value_text,
                offset,
                limit,
            } => {
                let (keys, filter, text, mode) = context_parts(self, context)?;
                self.with_retrieval(execution, move |retrieval| {
                    let outcome = retrieval.discover_filter_values(SearchValues {
                        field: &field_id,
                        clause_id: clause_id.as_deref(),
                        context: SearchContext {
                            text: text.as_deref(),
                            mode,
                            scope: scope(&keys),
                            prefer_remaster: true,
                            filter: filter.as_ref(),
                        },
                        text: value_text.as_deref(),
                        offset,
                        limit,
                    })?;
                    projection_execution.check()?;
                    Ok(FilterValueListView::Contextual { outcome })
                })
            }
            DiscoverFilterValuesRequest::Vocabulary {
                scope: vocabulary_scope,
                field_id,
                text,
                offset,
                limit,
                selected_values,
            } => {
                let keys = match &vocabulary_scope {
                    FilterVocabularyScope::Artifact => None,
                    FilterVocabularyScope::SavedList { list_ref } => {
                        Some(self.saved_list_record_keys(list_ref)?)
                    }
                };
                self.with_retrieval(execution, move |retrieval| {
                    let values = retrieval.discover_filter_vocabulary(SearchVocabulary {
                        field: &field_id,
                        scope: scope(&keys),
                        text: text.as_deref(),
                        offset,
                        limit,
                        selected_values: &selected_values,
                    })?;
                    projection_execution.check()?;
                    Ok(FilterValueListView::Vocabulary {
                        scope: vocabulary_scope,
                        values,
                    })
                })
            }
        }
    }
    pub fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<FilterCountsView> {
        execution.check()?;
        let (keys, filter, text, mode) = context_parts(self, request.context)?;
        let projection_execution = execution.clone();
        self.with_retrieval(execution, move |retrieval| {
            let counts = retrieval.discover_filter_counts(SearchCounts {
                field: &request.field_id,
                clause_id: request.clause_id.as_deref(),
                context: SearchContext {
                    text: text.as_deref(),
                    mode,
                    scope: scope(&keys),
                    prefer_remaster: true,
                    filter: filter.as_ref(),
                },
            })?;
            projection_execution.check()?;
            Ok(FilterCountsView { counts })
        })
    }
}
type ContextParts = (
    Option<Vec<RecordKey>>,
    Option<QueryPredicate>,
    Option<String>,
    RetrievalMode,
);
fn context_parts(
    service: &AtlasAppService,
    context: FilterDiscoveryContext,
) -> AppServiceResult<ContextParts> {
    match context {
        FilterDiscoveryContext::Filtered { filter, text, mode } => {
            Ok((None, filter, text, crate::retrieval::retrieval_mode(mode)))
        }
        FilterDiscoveryContext::SavedList {
            list_ref,
            filter,
            text,
            mode,
        } => Ok((
            Some(service.saved_list_record_keys(&list_ref)?),
            filter,
            text,
            crate::retrieval::retrieval_mode(mode),
        )),
    }
}
fn scope(keys: &Option<Vec<RecordKey>>) -> RecordScope<'_> {
    keys.as_deref()
        .map(RecordScope::Keys)
        .unwrap_or(RecordScope::All)
}

fn required_facet<T>(outcome: QueryFacetOutcome<T>) -> AppServiceResult<T> {
    match outcome {
        QueryFacetOutcome::Available { value } => Ok(value),
        QueryFacetOutcome::Unavailable { reason } => {
            Err(crate::AppServiceError::invalid_request(format!(
                "Contextual discovery is unavailable for this condition ({reason:?}); query execution remains supported. Discover unscoped values without a target clause."
            )))
        }
    }
}
