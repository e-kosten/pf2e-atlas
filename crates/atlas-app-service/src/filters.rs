use crate::{AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    DiscoverFilterCountsRequest, DiscoverFilterEditorRequest, DiscoverFilterValuesRequest,
    FilterCountsView, FilterDiscoveryContext, FilterEditorView, FilterValueListView,
    FilterValueOptionsView, FilterVocabularyScope,
};
use atlas_domain::{
    QueryCapability, QueryFacetOutcome, QueryFieldCounts, QueryPredicate, RecordKey,
};
use atlas_search::{
    DiscoverFilterCountsRequest as SearchCounts,
    DiscoverFilterValueCountsRequest as SearchValueCounts,
    DiscoverFilterValuesRequest as SearchValues,
    DiscoverFilterVocabularyRequest as SearchVocabulary, FilterDiscoveryContext as SearchContext,
    RecordScope,
};

#[derive(Debug, Clone)]
pub struct CliFilterValuesRequest {
    pub field: &'static atlas_domain::QueryFieldDefinition,
    pub context: FilterDiscoveryContext,
    pub clause_id: Option<String>,
    pub text: Option<String>,
    pub offset: usize,
    pub limit: usize,
}
pub struct CliFilterValues {
    pub counts: QueryFieldCounts,
    pub values: Option<FilterValueOptionsView>,
}
pub fn discover_filter_fields() -> AppServiceResult<QueryCapability> {
    Ok(atlas_search::discover_filter_fields()?)
}
pub fn filter_field_definition(
    id: &str,
) -> AppServiceResult<&'static atlas_domain::QueryFieldDefinition> {
    Ok(atlas_search::filter_field_definition(id)?)
}
impl AtlasAppService {
    pub fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        // Editor metadata describes the global executable catalog. Invalid current
        // criteria must not prevent discovering the controls needed to repair them.
        self.with_retrieval(
            &crate::ReadExecutionContext::uncancellable(),
            move |retrieval| {
                check_generation(retrieval, request.expected_artifact_generation.as_deref())?;
                let generation = retrieval.artifact_generation()?;
                let presentations =
                    crate::relationship_search::presentations(retrieval, request.filter.as_ref())?;
                let catalog = atlas_search::discover_filter_fields()?;
                let editor = crate::discovery::filter_editor_view(
                    catalog,
                    &request.selected_field_ids,
                    generation,
                    presentations,
                )?;
                let filter = request
                    .filter
                    .as_ref()
                    .filter(|filter| retrieval.validate_filter(filter).is_ok());
                enrich_editor(retrieval, editor, filter)
            },
        )
    }
    pub fn discover_cli_filter_group(
        &self,
        group: String,
    ) -> AppServiceResult<Option<FilterEditorView>> {
        self.with_retrieval(&ReadExecutionContext::uncancellable(), move |retrieval| {
            let mut catalog = atlas_search::discover_filter_fields()?;
            if !crate::discovery::select_filter_group(&mut catalog, &group) {
                return Ok(None);
            }
            let editor = crate::discovery::filter_editor_view(
                catalog,
                &[],
                retrieval.artifact_generation()?,
                vec![],
            )?;
            enrich_editor(retrieval, editor, None).map(Some)
        })
    }

    pub fn discover_cli_filter_values(
        &self,
        request: CliFilterValuesRequest,
    ) -> AppServiceResult<CliFilterValues> {
        let execution = ReadExecutionContext::uncancellable();
        let (list, filter, text, profile, expected, content_scope, connection_scope) =
            context_parts(request.context)?;
        self.with_workspace(&execution, move |workspace| {
            let keys = discovery_keys(workspace, list.as_deref(), connection_scope.as_ref())?;
            let r = workspace.retrieval_mut();
            check_generation(r, expected.as_deref())?;
            let context = SearchContext {
                content_scope,
                search_profile: profile.as_ref(),
                text: text.as_deref(),
                scope: scope(&keys),
                filter: filter.as_ref(),
            };
            let field = request.field.id.as_str();
            let clause_id = request.clause_id.as_deref();
            let counts = required_facet(r.discover_filter_counts(SearchCounts {
                field,
                clause_id,
                context,
            })?)?;
            let values = if matches!(
                request.field.field_type,
                atlas_domain::QueryFieldType::Number | atlas_domain::QueryFieldType::Collection
            ) {
                None
            } else {
                let values = required_facet(r.discover_filter_values(SearchValues {
                    only_matching: false,
                    field,
                    clause_id,
                    context,
                    text: request.text.as_deref(),
                    offset: request.offset,
                    limit: request.limit,
                })?)?;
                Some(crate::definition_values::options(r, values)?)
            };
            Ok(CliFilterValues { counts, values })
        })
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
                let (
                    keys,
                    filter,
                    text,
                    search_profile,
                    expected_generation,
                    content_scope,
                    connection_scope,
                ) = context_parts(*context)?;
                self.with_workspace(execution, move |workspace| {
                    let keys =
                        discovery_keys(workspace, keys.as_deref(), connection_scope.as_ref())?;
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
                            content_scope,
                            search_profile: search_profile.as_ref(),
                            text: text.as_deref(),
                            scope: keys
                                .as_deref()
                                .map(RecordScope::Keys)
                                .unwrap_or(RecordScope::All),

                            filter: filter.as_ref(),
                        },
                        text: value_text.as_deref(),
                        offset,
                        limit,
                    })?;
                    projection_execution.check()?;
                    let outcome = match outcome {
                        QueryFacetOutcome::Available { value } => QueryFacetOutcome::Available {
                            value: crate::definition_values::options(retrieval, value)?,
                        },
                        QueryFacetOutcome::Unavailable { reason } => {
                            QueryFacetOutcome::Unavailable { reason }
                        }
                    };
                    Ok(FilterValueListView::Contextual {
                        outcome,
                        search_profile: resolved,
                        artifact_generation: generation,
                    })
                })
            }
            DiscoverFilterValuesRequest::Vocabulary {
                expected_artifact_generation,
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
                let artifact_generation = retrieval.artifact_generation()?;
                crate::connection_scope::check_generation(
                    expected_artifact_generation.as_deref(),
                    &artifact_generation,
                )?;
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
                    artifact_generation,
                    scope: vocabulary_scope,
                    values: crate::definition_values::vocabulary(retrieval, values)?,
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
        let (
            keys,
            filter,
            text,
            search_profile,
            expected_generation,
            content_scope,
            connection_scope,
        ) = context_parts(request.context)?;
        let projection_execution = execution.clone();
        self.with_workspace(execution, move |workspace| {
            let keys = discovery_keys(workspace, keys.as_deref(), connection_scope.as_ref())?;
            let retrieval = workspace.retrieval_mut();
            check_generation(retrieval, expected_generation.as_deref())?;
            let resolved = retrieval.resolve_search_profile(search_profile.as_ref(), false)?;
            let generation = retrieval.artifact_generation()?;
            let counts = retrieval.discover_filter_value_counts(SearchValueCounts {
                field: &request.field_id,
                clause_id: request.clause_id.as_deref(),
                values: &request.values,
                context: SearchContext {
                    content_scope,
                    search_profile: search_profile.as_ref(),
                    text: text.as_deref(),
                    scope: keys
                        .as_deref()
                        .map(RecordScope::Keys)
                        .unwrap_or(RecordScope::All),

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
fn enrich_editor(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    mut editor: FilterEditorView,
    filter: Option<&QueryPredicate>,
) -> AppServiceResult<FilterEditorView> {
    let families = editor_values(retrieval, "source.type", filter)?;
    for field in editor.groups.iter_mut().flat_map(|group| &mut group.fields) {
        field.applicable = field.definition.scope.is_some()
            || families.is_empty()
            || field.definition.family_types.is_empty()
            || field
                .definition
                .family_types
                .iter()
                .any(|family| families.contains(family));
        if field.definition.id == "kind" {
            let packs = retrieval.journal_kind_packs()?;
            field.choices.extend(packs.iter().map(|(pack, _)| {
                let selector = format!("{}{pack}", atlas_domain::root_kind::JOURNAL_PACK_PREFIX);
                crate::kind_choices::choice(&selector, &packs)
            }));
        }
    }
    crate::definition_values::editor(retrieval, &mut editor)?;
    editor.sense_qualifiers = Some(
        retrieval
            .sense_qualifiers()?
            .into_iter()
            .map(|(sense, mut acuity, mut has_range)| {
                if let Some((_, fixed, unlimited)) =
                    atlas_foundry_model::generated::reference_definitions::SENSE_RULES
                        .iter()
                        .find(|(kind, _, _)| *kind == sense)
                {
                    if fixed.is_some() {
                        acuity = None;
                    }
                    has_range &= !unlimited;
                }
                atlas_app_model::SenseQualifierView {
                    sense,
                    acuity,
                    has_range,
                }
            })
            .collect(),
    );
    Ok(editor)
}

fn editor_values(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    field: &str,
    filter: Option<&QueryPredicate>,
) -> AppServiceResult<Vec<String>> {
    Ok(retrieval
        .matching_field_values(field, filter)?
        .into_iter()
        .filter_map(|value| match value {
            atlas_domain::QueryLiteral::String(value) => Some(value),
            _ => None,
        })
        .collect())
}
type ContextParts = (
    Option<String>,
    Option<QueryPredicate>,
    Option<String>,
    Option<atlas_domain::SearchProfileId>,
    Option<String>,
    atlas_domain::SearchContentScope,
    Option<atlas_app_model::SearchConnectionScope>,
);
fn context_parts(context: FilterDiscoveryContext) -> AppServiceResult<ContextParts> {
    let (list, filter, text, profile, expected, search_content, connection_scope) = match context {
        FilterDiscoveryContext::Filtered {
            filter,
            text,
            search_profile,
            expected_artifact_generation,
            search_content,
            connection_scope,
        } => (
            None,
            filter,
            text,
            search_profile,
            expected_artifact_generation,
            search_content,
            connection_scope,
        ),
        FilterDiscoveryContext::SavedList {
            list_ref,
            filter,
            text,
            search_profile,
            expected_artifact_generation,
            search_content,
            connection_scope,
        } => (
            Some(list_ref),
            filter,
            text,
            search_profile,
            expected_artifact_generation,
            search_content,
            connection_scope,
        ),
    };
    if search_content.is_some() && text.as_deref().is_none_or(|text| text.trim().is_empty()) {
        return Err(crate::AppServiceError::invalid_request(
            "Search content requires a text query; omit it for browse or list discovery",
        ));
    }
    Ok((
        list,
        filter,
        text,
        profile,
        expected,
        search_content.unwrap_or_default(),
        connection_scope,
    ))
}

fn discovery_keys(
    workspace: &mut atlas_runtime::WorkspaceRead,
    list_ref: Option<&str>,
    connection: Option<&atlas_app_model::SearchConnectionScope>,
) -> AppServiceResult<Option<Vec<RecordKey>>> {
    let list = scoped_list_keys(workspace, list_ref)?;
    let connected = connection
        .map(|context| crate::connection_scope::resolve_connection_scope(workspace, context))
        .transpose()?
        .and_then(|scope| scope.keys);
    Ok(match (list, connected) {
        (Some(list), Some(connected)) => {
            let allowed: std::collections::BTreeSet<_> = connected.into_iter().collect();
            Some(
                list.into_iter()
                    .filter(|key| allowed.contains(key))
                    .collect(),
            )
        }
        (list, connected) => list.or(connected),
    })
}
pub(crate) fn scoped_list_keys(
    workspace: &atlas_runtime::WorkspaceRead,
    list_ref: Option<&str>,
) -> AppServiceResult<Option<Vec<RecordKey>>> {
    let Some(list_ref) = list_ref else {
        return Ok(None);
    };
    let list = workspace
        .local_state()?
        .saved_lists()
        .record_keys(list_ref)?
        .ok_or_else(|| {
            crate::AppServiceError::new(
                atlas_app_model::AppErrorCode::SavedListNotFound,
                format!("saved list `{list_ref}` was not found"),
            )
        })?;
    Ok(Some(
        list.into_iter()
            .map(|item| {
                RecordKey::parse(&item)
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
        QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::TextContext,
        } => Err(crate::AppServiceError::invalid_request(
            "Filter discovery uses structured filters and saved-list scope. Omit the record-content query; --text searches filter-value names.",
        )),
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

    #[test]
    fn raw_text_context_recovery_identifies_record_query_not_target_clause() {
        let error = required_facet::<()>(QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::TextContext,
        })
        .unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("Omit the record-content query"),
            "{message}"
        );
        assert!(
            message.contains("--text searches filter-value names"),
            "{message}"
        );
    }

    #[test]
    fn explicit_discovery_scope_requires_text_and_legacy_browse_keeps_all() {
        let context = |text, search_content| FilterDiscoveryContext::Filtered {
            connection_scope: None,
            search_content,
            search_profile: None,
            expected_artifact_generation: None,
            filter: None,
            text,
        };
        for text in [None, Some(" ".into())] {
            for scope in [
                atlas_domain::SearchContentScope::All,
                atlas_domain::SearchContentScope::Core,
            ] {
                assert!(context_parts(context(text.clone(), Some(scope))).is_err());
            }
        }
        assert_eq!(
            context_parts(context(None, None)).unwrap().5,
            atlas_domain::SearchContentScope::All
        );
        assert_eq!(
            context_parts(context(
                Some("fire".into()),
                Some(atlas_domain::SearchContentScope::Core)
            ))
            .unwrap()
            .5,
            atlas_domain::SearchContentScope::Core
        );
    }
    use atlas_app_model::{
        AppErrorCode, FilterOperandView, SearchCriteria, SearchPageRequest, SearchResultsRequest,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    fn editor_request() -> DiscoverFilterEditorRequest {
        DiscoverFilterEditorRequest {
            expected_artifact_generation: None,
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
            json!({"kind":"in","clause_id":"invalid","field":"kind","values":["rule"]}),
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
                        include_summary_definitions: true,
                        result_detail: atlas_app_model::SearchResultDetail::Evidence,
                        connection_scope: None,
                        sort: None,

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
                            connection_scope: None,
                            search_content: None,
                            search_profile: None,
                            expected_artifact_generation: None,
                            filter: Some(predicate),
                            text: None,
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
                expected_artifact_generation: None,
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
            kind.choices
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
