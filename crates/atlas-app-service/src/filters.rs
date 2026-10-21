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
    DiscoverFilterCountsRequest as SearchCounts,
    DiscoverFilterValueCountsRequest as SearchValueCounts,
    DiscoverFilterValuesRequest as SearchValues,
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
                let mut editor = crate::discovery::filter_editor_view(catalog, &request.selected_field_ids)?;
                // One family inventory answers applicability; never fan out across fields.
                let filter = request.filter.as_ref().filter(|filter| retrieval.validate_filter(filter).is_ok());
                let families = editor_values(retrieval, "source.type", filter)?;
                let kinds = editor_values(retrieval, "kind", None)?;
                for field in editor.groups.iter_mut().flat_map(|group| &mut group.fields) {
                    field.applicable = field.definition.scope.is_some()
                        || families.is_empty()
                        || field.definition.family_types.is_empty()
                        || field.definition.family_types.iter().any(|family| families.contains(family));
                    if field.definition.id == "kind" {
                        field.choices.retain(|choice| matches!(&choice.value, atlas_domain::QueryLiteral::String(value) if kinds.contains(value)));
                    }
                }
                Ok(editor)
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
                    only_matching: false,
                    field: &request.field,
                    clause_id: request.clause_id.as_deref(),
                    context: SearchContext {
                        search_profile: None,
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
                        search_profile: None,
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
                only_matching,
                context,
                field_id,
                clause_id,
                text: value_text,
                offset,
                limit,
            } => {
                let (keys, filter, text, mode, search_profile, expected_generation) =
                    context_parts(context);
                let preflight_field = field_id.clone();
                let preflight_clause = clause_id.clone();
                let preflight_filter = filter.clone();
                let expected = expected_generation.clone();
                let supports_ranking = self.with_retrieval(execution, move |r| {
                    check_generation(r, expected.as_deref())?;
                    Ok(r.discovery_supports_ranking(
                        &preflight_field,
                        preflight_clause.as_deref(),
                        preflight_filter.as_ref(),
                    )?)
                })?;
                let prepared = if supports_ranking {
                    self.prepare_query(
                        search_profile.as_ref(),
                        text.as_deref().unwrap_or(""),
                        mode,
                    )?
                } else {
                    None
                };
                self.with_prepared_workspace(execution, prepared, move |workspace| {
                    let keys = scoped_list_keys(workspace, keys.as_deref())?;
                    let retrieval = workspace.retrieval_mut();
                    check_generation(retrieval, expected_generation.as_deref())?;
                    let resolved =
                        retrieval.resolve_search_profile(search_profile.as_ref(), false)?;
                    let generation = retrieval.artifact_generation()?;
                    let outcome = retrieval.discover_filter_values(SearchValues {
                        only_matching,
                        field: &field_id,
                        clause_id: clause_id.as_deref(),
                        context: SearchContext {
                            search_profile: search_profile.as_ref(),
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
                    Ok(FilterValueListView::Contextual {
                        outcome,
                        search_profile: resolved,
                        artifact_generation: generation,
                    })
                })
            }
            DiscoverFilterValuesRequest::Vocabulary {
                scope: vocabulary_scope,
                field_id,
                text,
                offset,
                limit,
                selected_values,
            } => self.with_workspace(execution, move |workspace| {
                let keys = match &vocabulary_scope {
                    FilterVocabularyScope::Artifact => None,
                    FilterVocabularyScope::SavedList { list_ref } => {
                        scoped_list_keys(workspace, Some(list_ref))?
                    }
                };
                let retrieval = workspace.retrieval_mut();
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
            }),
        }
    }
    pub fn discover_filter_counts(
        &self,
        request: DiscoverFilterCountsRequest,
        execution: &ReadExecutionContext,
    ) -> AppServiceResult<FilterCountsView> {
        execution.check()?;
        let (keys, filter, text, mode, search_profile, expected_generation) =
            context_parts(request.context);
        let projection_execution = execution.clone();
        self.with_workspace(execution, move |workspace| {
            let keys = scoped_list_keys(workspace, keys.as_deref())?;
            let retrieval = workspace.retrieval_mut();
            check_generation(retrieval, expected_generation.as_deref())?;
            let resolved = retrieval.resolve_search_profile(search_profile.as_ref(), false)?;
            let generation = retrieval.artifact_generation()?;
            let counts = retrieval.discover_filter_value_counts(SearchValueCounts {
                field: &request.field_id,
                clause_id: request.clause_id.as_deref(),
                values: &request.values,
                context: SearchContext {
                    search_profile: search_profile.as_ref(),
                    text: text.as_deref(),
                    mode,
                    scope: scope(&keys),
                    prefer_remaster: true,
                    filter: filter.as_ref(),
                },
            })?;
            projection_execution.check()?;
            Ok(FilterCountsView {
                counts,
                search_profile: resolved,
                artifact_generation: generation,
            })
        })
    }
}
fn editor_values(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    field: &str,
    filter: Option<&QueryPredicate>,
) -> AppServiceResult<Vec<String>> {
    let mut values = Vec::new();
    let mut offset = 0;
    loop {
        let result = retrieval.discover_filter_values(SearchValues {
            only_matching: true,
            field,
            clause_id: None,
            context: SearchContext {
                search_profile: None,
                text: None,
                mode: RetrievalMode::Fts,
                scope: RecordScope::All,
                prefer_remaster: false,
                filter,
            },
            text: None,
            offset,
            limit: 40,
        })?;
        let QueryFacetOutcome::Available { value } = result else {
            return Ok(values);
        };
        let length = value.options.len();
        values.extend(
            value
                .options
                .into_iter()
                .filter_map(|option| match option.value {
                    atlas_domain::QueryLiteral::String(value) => Some(value),
                    _ => None,
                }),
        );
        offset += length;
        if length == 0 || offset as u64 >= value.total_values {
            break;
        }
    }
    Ok(values)
}
type ContextParts = (
    Option<String>,
    Option<QueryPredicate>,
    Option<String>,
    RetrievalMode,
    Option<atlas_domain::SearchProfileId>,
    Option<String>,
);
fn context_parts(context: FilterDiscoveryContext) -> ContextParts {
    match context {
        FilterDiscoveryContext::Filtered {
            filter,
            text,
            mode,
            search_profile,
            expected_artifact_generation,
        } => (
            None,
            filter,
            text,
            crate::retrieval::retrieval_mode(mode),
            search_profile,
            expected_artifact_generation,
        ),
        FilterDiscoveryContext::SavedList {
            list_ref,
            filter,
            text,
            mode,
            search_profile,
            expected_artifact_generation,
        } => (
            Some(list_ref),
            filter,
            text,
            crate::retrieval::retrieval_mode(mode),
            search_profile,
            expected_artifact_generation,
        ),
    }
}
fn scoped_list_keys(
    workspace: &atlas_runtime::WorkspaceRead,
    list_ref: Option<&str>,
) -> AppServiceResult<Option<Vec<RecordKey>>> {
    let Some(list_ref) = list_ref else {
        return Ok(None);
    };
    let list = workspace
        .local_state()
        .saved_lists()
        .get_with_items(list_ref)?
        .ok_or_else(|| {
            crate::AppServiceError::new(
                atlas_app_model::AppErrorCode::SavedListNotFound,
                format!("saved list `{list_ref}` was not found"),
            )
        })?;
    Ok(Some(
        list.items
            .into_iter()
            .map(|item| {
                RecordKey::parse(&item.record_key)
                    .map_err(|e| crate::AppServiceError::invalid_request(e.to_string()))
            })
            .collect::<AppServiceResult<Vec<_>>>()?,
    ))
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

fn check_generation(
    retrieval: &atlas_search::AtlasRetrievalService,
    expected: Option<&str>,
) -> AppServiceResult<()> {
    let actual = retrieval.artifact_generation()?;
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(crate::AppServiceError::artifact_generation_changed(
            expected,
            &actual,
            atlas_app_model::AppRecoverableAction::RefreshResults,
        ));
    }
    Ok(())
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
            filter: None,
            selected_field_ids: vec!["defenses.hp.maximum".into()],
        }
    }

    #[test]
    fn filter_editor_remains_available_to_repair_invalid_fields_and_choices() {
        let f = fixture_worker();
        let baseline = f.worker.discover_filter_editor(editor_request()).unwrap();
        for input in [
            json!({"kind":"compare","clause_id":"invalid","field":"invented.field","op":"eq","value":"example"}),
            json!({"kind":"in","clause_id":"invalid","field":"kind","values":["action"]}),
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
                        relationships: None,
                        ranking: None,
                        search_profile: None,
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
                            search_profile: None,
                            expected_artifact_generation: None,
                            filter: Some(predicate),
                            text: None,
                            mode: RetrievalModeView::Lexical,
                        },
                        field_id: "kind".into(),
                        clause_id: Some("invalid".into()),
                        values: vec![],
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
    fn editor_narrows_product_controls_without_removing_repair_metadata() {
        let f = fixture_worker();
        let editor = f
            .worker
            .discover_filter_editor(DiscoverFilterEditorRequest {
                filter: Some(
                    serde_json::from_value(
                        json!({"kind":"in","field":"kind","values":["creature"]}),
                    )
                    .unwrap(),
                ),
                selected_field_ids: vec![],
            })
            .unwrap();
        let fields: Vec<_> = editor
            .groups
            .iter()
            .flat_map(|group| &group.fields)
            .collect();
        assert!(
            !fields
                .iter()
                .any(|field| field.definition.path == "heritage.ancestry.uuid")
        );
        let hardness = fields
            .iter()
            .find(|field| field.definition.id == "hazard.hardness")
            .unwrap();
        assert!(!hardness.applicable);
        let kind = fields
            .iter()
            .find(|field| field.definition.id == "kind")
            .unwrap();
        assert!(
            !kind
                .choices
                .iter()
                .any(|choice| choice.value
                    == atlas_domain::QueryLiteral::String("affliction".into()))
        );
        let duration = fields
            .iter()
            .find(|field| field.definition.id == "effect.duration.rounds")
            .unwrap();
        assert_eq!(duration.unit_fields.len(), 4);
        assert_eq!(duration.label, "Duration");
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
