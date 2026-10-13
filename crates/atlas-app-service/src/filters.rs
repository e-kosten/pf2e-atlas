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
        // Editor metadata describes the global executable catalog. Invalid current
        // criteria must not prevent discovering the controls needed to repair them.
        self.with_retrieval(
            &crate::ReadExecutionContext::uncancellable(),
            move |retrieval| {
                let catalog = retrieval.discover_filter_fields()?;
                crate::discovery::filter_editor_view(catalog, &request.selected_field_ids)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture_worker;
    use atlas_app_model::{
        AppErrorCode, FilterOperandView, RetrievalModeView, SearchCriteria, SearchPageRequest,
        SearchResultsRequest,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    fn editor_request() -> DiscoverFilterEditorRequest {
        DiscoverFilterEditorRequest {
            selected_field_ids: vec!["actor.hp.maximum".into()],
        }
    }

    #[test]
    fn filter_editor_remains_available_to_repair_invalid_fields_and_choices() {
        let f = fixture_worker();
        let baseline = f.worker.discover_filter_editor(editor_request()).unwrap();
        for input in [
            json!({"kind":"compare","clause_id":"invalid","field":"invented.field","op":"eq","value":"example"}),
            json!({"kind":"in","clause_id":"invalid","field":"record.kind","values":["action"]}),
        ] {
            let predicate: QueryPredicate = serde_json::from_value(input).unwrap();
            assert!(
                !f.worker
                    .validate_filter(predicate.clone())
                    .unwrap()
                    .errors
                    .is_empty()
            );
            let search_error = f
                .worker
                .search_page(
                    SearchResultsRequest {
                        criteria: SearchCriteria::ListRecords {
                            filter: Some(predicate.clone()),
                        },
                        page: SearchPageRequest {
                            number: 1,
                            size: 10,
                        },
                        expected_artifact_generation: None,
                    },
                    &ReadExecutionContext::uncancellable(),
                )
                .unwrap_err()
                .into_app_error();
            assert_eq!(search_error.code, AppErrorCode::FilterInvalid);
            let count_error = f
                .worker
                .discover_filter_counts(
                    DiscoverFilterCountsRequest {
                        context: FilterDiscoveryContext::Filtered {
                            filter: Some(predicate),
                            text: None,
                            mode: RetrievalModeView::Lexical,
                        },
                        field_id: "record.kind".into(),
                        clause_id: Some("invalid".into()),
                    },
                    &ReadExecutionContext::uncancellable(),
                )
                .unwrap_err()
                .into_app_error();
            assert_eq!(count_error.code, AppErrorCode::FilterInvalid);
            assert_eq!(
                f.worker.discover_filter_editor(editor_request()).unwrap(),
                baseline,
            );
        }
    }

    #[test]
    fn filter_editor_operands_cover_the_real_executable_catalog() {
        use FilterOperandView::{List, Predicate, Range, Scalar, State};
        let f = fixture_worker();
        let editor = f.worker.discover_filter_editor(editor_request()).unwrap();
        let actual: BTreeMap<_, _> = editor
            .groups
            .iter()
            .flat_map(|group| &group.fields)
            .flat_map(|field| &field.operators)
            .map(|operator| (operator.operator.as_str(), operator.operand))
            .collect();
        assert_eq!(
            actual,
            BTreeMap::from([
                ("eq", Scalar),
                ("neq", Scalar),
                ("lt", Scalar),
                ("lte", Scalar),
                ("gt", Scalar),
                ("gte", Scalar),
                ("includes", Scalar),
                ("in", List),
                ("includes_any", List),
                ("includes_all", List),
                ("excludes_any", List),
                ("between", Range),
                ("state", State),
                ("exists", Predicate),
            ])
        );

        let includes = |values| {
            serde_json::from_value::<QueryPredicate>(json!({
                "kind":"set_match","field":"traits","op":"includes","values":values
            }))
            .unwrap()
        };
        assert!(
            f.worker
                .validate_filter(includes(vec!["fire"]))
                .unwrap()
                .errors
                .is_empty()
        );
        assert!(
            !f.worker
                .validate_filter(includes(vec!["fire", "magical"]))
                .unwrap()
                .errors
                .is_empty()
        );
    }
}
