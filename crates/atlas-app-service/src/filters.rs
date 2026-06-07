use atlas_app_model::{
    DiscoverFilterEditorRequest, DiscoverFilterValuesRequest, FilterDiscoveryContext,
    FilterEditorView, FilterValueListView,
};
use atlas_search::{
    DiscoverFilterFieldsRequest as SearchDiscoverFilterFieldsRequest,
    DiscoverFilterValuesRequest as SearchDiscoverFilterValuesRequest, FilterDiscoveryRetrieval,
    MetricDiscoverySelector,
};

use crate::discovery::{filter_editor_view, filter_value_list_view};
use crate::error::AppServiceResult;
use crate::filter::{
    app_filter_field_id, discovery_field_id, filter_context_excluding_field,
    lower_basic_filter_context,
};
use crate::service::AppServiceWorker;

impl AppServiceWorker {
    pub(super) fn discover_filter_editor(
        &self,
        request: DiscoverFilterEditorRequest,
    ) -> AppServiceResult<FilterEditorView> {
        let filter = lower_basic_filter_context(&request.context)?;
        let discovery =
            self.retrieval
                .discover_filter_fields(SearchDiscoverFilterFieldsRequest {
                    filter: filter.as_ref(),
                    filter_json: None,
                })?;
        let selected_candidates =
            self.retrieval
                .discover_filter_fields(SearchDiscoverFilterFieldsRequest {
                    filter: None,
                    filter_json: None,
                })?;
        let selected_field_ids = selected_filter_field_ids(&request);
        Ok(filter_editor_view(
            discovery,
            selected_candidates,
            &selected_field_ids,
        ))
    }

    pub(super) fn discover_filter_values(
        &self,
        request: DiscoverFilterValuesRequest,
    ) -> AppServiceResult<FilterValueListView> {
        let discovery_context = filter_context_excluding_field(&request.context, &request.field_id);
        let filter = lower_basic_filter_context(&discovery_context)?;
        let discovery =
            self.retrieval
                .discover_filter_values(SearchDiscoverFilterValuesRequest {
                    field: discovery_field_id(&request.field_id),
                    filter: filter.as_ref(),
                    filter_json: None,
                    sort: None,
                    sample_limit: None,
                    metric_selector: metric_selector(request.metric_query.as_deref()),
                    metric_domain: request.metric_domain.clone(),
                })?;
        filter_value_list_view(&request.field_id, &request.context, discovery)
    }
}

fn selected_filter_field_ids(request: &DiscoverFilterEditorRequest) -> Vec<String> {
    let mut fields = request
        .selected_field_ids
        .iter()
        .map(|field| app_filter_field_id(field))
        .collect::<Vec<_>>();
    match &request.context {
        FilterDiscoveryContext::Filtered { filter } => {
            fields.extend(
                filter
                    .clauses
                    .iter()
                    .map(|clause| app_filter_field_id(&clause.field)),
            );
        }
    }
    fields.sort();
    fields.dedup();
    fields
}

fn metric_selector(query: Option<&str>) -> Option<MetricDiscoverySelector> {
    let query = query?.trim();
    if query.is_empty() {
        None
    } else {
        Some(MetricDiscoverySelector::Query(query.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        BasicSearchFilter, FilterClause, FilterClauseOperator, FilterDiscoveryContext,
        FilterFieldApplicability,
    };

    use crate::test_support::fixture_worker;

    #[test]
    fn worker_discovers_app_facing_filter_fields_and_values() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;
        let context = FilterDiscoveryContext::Filtered {
            filter: BasicSearchFilter {
                clauses: vec![FilterClause {
                    id: "kind-include_any".to_string(),
                    field: "kind".to_string(),
                    operator: FilterClauseOperator::IncludeAny,
                    values: vec!["rule".to_string()],
                    range: None,
                    metric: None,
                }],
            },
        };

        let editor = worker
            .discover_filter_editor(atlas_app_model::DiscoverFilterEditorRequest {
                context: context.clone(),
                selected_field_ids: Vec::new(),
            })
            .expect("fixture editor discovery should succeed");

        assert_eq!(editor.matching_record_count, 3);
        let fields = editor
            .groups
            .iter()
            .flat_map(|group| group.fields.iter())
            .collect::<Vec<_>>();
        assert!(fields.iter().any(|field| field.id == "kind"));
        assert!(fields.iter().any(|field| field.id == "pack"));
        assert!(!fields.iter().any(|field| field.id == "pack_label"));

        let values = worker
            .discover_filter_values(atlas_app_model::DiscoverFilterValuesRequest {
                context,
                field_id: "pack".to_string(),
                metric_query: None,
                metric_domain: None,
            })
            .expect("fixture value discovery should succeed");

        assert_eq!(values.field_id, "pack");
        assert_eq!(values.matching_record_count, 3);
        assert_eq!(values.options[0].value, "Actions");
        assert_eq!(values.options[0].count, Some(3));
        assert!(!values.options[0].disabled);
    }

    #[test]
    fn worker_preserves_clause_selected_fields_when_editor_scope_has_no_matches() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;
        let editor = worker
            .discover_filter_editor(atlas_app_model::DiscoverFilterEditorRequest {
                context: FilterDiscoveryContext::Filtered {
                    filter: BasicSearchFilter {
                        clauses: vec![filter_clause("pack_label", "Missing")],
                    },
                },
                selected_field_ids: vec!["publication_category".to_string()],
            })
            .expect("fixture editor discovery should succeed");

        assert_eq!(editor.matching_record_count, 0);
        let fields = editor
            .groups
            .iter()
            .flat_map(|group| group.fields.iter())
            .collect::<Vec<_>>();
        let pack = fields
            .iter()
            .find(|field| field.id == "pack")
            .expect("active pack_label clause should preserve app-facing pack field");
        assert_eq!(
            pack.applicability,
            FilterFieldApplicability::SelectedUnavailable
        );
        let publication_family = fields
            .iter()
            .find(|field| field.id == "publication_family")
            .expect("explicit selected alias should preserve publication family field");
        assert_eq!(
            publication_family.applicability,
            FilterFieldApplicability::SelectedUnavailable
        );
        assert!(
            !fields.iter().any(|field| field.id == "kind"),
            "unselected unavailable candidates should not be returned"
        );
    }

    #[test]
    fn worker_discovers_values_with_same_app_field_excluded_from_scope() {
        let fixture = fixture_worker();
        let worker = &fixture.worker;
        let context = FilterDiscoveryContext::Filtered {
            filter: BasicSearchFilter {
                clauses: vec![
                    filter_clause("kind", "rule"),
                    filter_clause("pack_label", "Missing"),
                ],
            },
        };

        let values = worker
            .discover_filter_values(atlas_app_model::DiscoverFilterValuesRequest {
                context,
                field_id: "pack".to_string(),
                metric_query: None,
                metric_domain: None,
            })
            .expect("same-field exclusion should keep pack discovery valid");

        assert_eq!(values.field_id, "pack");
        assert_eq!(
            values.matching_record_count, 3,
            "pack_label clause should be excluded while kind=rule remains active"
        );
        assert_eq!(values.options.len(), 1);
        assert_eq!(values.options[0].value, "Actions");
        assert_eq!(values.options[0].count, Some(3));
    }

    fn filter_clause(field: &str, value: &str) -> FilterClause {
        FilterClause {
            id: format!("{field}-include_any"),
            field: field.to_string(),
            operator: FilterClauseOperator::IncludeAny,
            values: vec![value.to_string()],
            range: None,
            metric: None,
        }
    }
}
