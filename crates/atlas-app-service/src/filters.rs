use crate::{AppServiceResult, AtlasAppService};
use atlas_app_model::{
    DiscoverFilterCountsRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterCountsView, FilterDiscoveryContext, FilterEditorView, FilterValueListView,
};
use atlas_domain::{
    QueryCapability, QueryFieldCounts, QueryPredicate, QueryValueOptions, RecordKey,
};
use atlas_search::{
    DiscoverFilterCountsRequest as SearchCounts, DiscoverFilterValuesRequest as SearchValues,
    FilterDiscoveryContext as SearchContext, RecordScope, RetrievalMode,
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
        self.submit_retrieval(|retrieval| Ok(retrieval.discover_filter_fields()?))
    }
    pub fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        let (_, filter, _, _) = context_parts(self, request.context)?;
        self.submit_retrieval(move |retrieval| {
            if let Some(filter) = filter {
                retrieval.validate_filter(&filter)?;
            }
            let catalog = retrieval.discover_filter_fields()?;
            Ok(crate::discovery::filter_editor_view(
                catalog,
                &request.selected_field_ids,
            ))
        })
    }
    pub fn discover_raw_filter_values(
        &self,
        request: RawFilterValuesRequest,
    ) -> AppServiceResult<QueryValueOptions> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.discover_filter_values(SearchValues {
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
        })
    }
    pub fn discover_raw_filter_counts(
        &self,
        request: RawFilterCountsRequest,
    ) -> AppServiceResult<QueryFieldCounts> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.discover_filter_counts(SearchCounts {
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
        })
    }
    pub fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> AppServiceResult<FilterValueListView> {
        let (keys, filter, text, mode) = context_parts(self, request.context)?;
        self.submit_retrieval(move |retrieval| {
            Ok(FilterValueListView {
                values: retrieval.discover_filter_values(SearchValues {
                    field: &request.field_id,
                    clause_id: request.clause_id.as_deref(),
                    context: SearchContext {
                        text: text.as_deref(),
                        mode,
                        scope: scope(&keys),
                        prefer_remaster: true,
                        filter: filter.as_ref(),
                    },
                    text: request.text.as_deref(),
                    offset: request.offset,
                    limit: request.limit,
                })?,
            })
        })
    }
    pub fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
    ) -> AppServiceResult<FilterCountsView> {
        let (keys, filter, text, mode) = context_parts(self, request.context)?;
        self.submit_retrieval(move |retrieval| {
            Ok(FilterCountsView {
                counts: retrieval.discover_filter_counts(SearchCounts {
                    field: &request.field_id,
                    clause_id: request.clause_id.as_deref(),
                    context: SearchContext {
                        text: text.as_deref(),
                        mode,
                        scope: scope(&keys),
                        prefer_remaster: true,
                        filter: filter.as_ref(),
                    },
                })?,
            })
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
            Ok((None, filter, text, crate::windows::retrieval_mode(mode)))
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
            crate::windows::retrieval_mode(mode),
        )),
    }
}
fn scope(keys: &Option<Vec<RecordKey>>) -> RecordScope<'_> {
    keys.as_deref()
        .map(RecordScope::Keys)
        .unwrap_or(RecordScope::All)
}
