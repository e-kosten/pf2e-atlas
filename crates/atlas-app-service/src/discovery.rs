use atlas_app_model::{
    FilterClauseOperator, FilterControlView, FilterDiscoveryContext, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldApplicability, FilterFieldPlacement,
    FilterValueListView, FilterValueOption,
};
use atlas_domain::{
    FilterFieldDiscovery, FilterFieldInfo, FilterFieldType, FilterOperator, FilterValueCount,
    FilterValueDiscovery, FilterValuePayload, FilterValuePolicy,
};

use crate::AppServiceResult;
use crate::filter::app_filter_field_id;

pub(crate) fn filter_editor_view(
    discovery: FilterFieldDiscovery,
    selected_candidates: FilterFieldDiscovery,
    selected_field_ids: &[String],
) -> FilterEditorView {
    let mut groups = filter_editor_groups();
    let mut projected_field_ids = Vec::new();
    for field in discovery
        .fields
        .iter()
        .filter(|field| supported_basic_field(&field.field))
    {
        let app_id = app_field_id(&field.field);
        let group_id = field_group_id(&app_id);
        if let Some(group) = groups.iter_mut().find(|group| group.id == group_id) {
            group.fields.push(filter_editor_field_view(
                field,
                FilterFieldApplicability::Applicable,
            ));
            projected_field_ids.push(app_id);
        }
    }

    for field in selected_candidates
        .fields
        .iter()
        .filter(|field| supported_basic_field(&field.field))
    {
        let app_id = app_field_id(&field.field);
        if !selected_field_ids
            .iter()
            .any(|selected| app_filter_field_id(selected) == app_id)
            || projected_field_ids.contains(&app_id)
        {
            continue;
        }
        let group_id = field_group_id(&app_id);
        if let Some(group) = groups.iter_mut().find(|group| group.id == group_id) {
            group.fields.push(filter_editor_field_view(
                field,
                FilterFieldApplicability::SelectedUnavailable,
            ));
            projected_field_ids.push(app_id);
        }
    }
    sort_filter_editor_groups(&mut groups);
    groups.retain(|group| !group.fields.is_empty());
    FilterEditorView {
        matching_record_count: discovery.matching_record_count,
        groups,
    }
}

pub(crate) fn filter_value_list_view(
    field_id: &str,
    context: &FilterDiscoveryContext,
    discovery: FilterValueDiscovery,
) -> AppServiceResult<FilterValueListView> {
    let app_field_id = app_field_id(field_id);
    Ok(FilterValueListView {
        field_id: app_field_id.to_string(),
        matching_record_count: discovery.matching_record_count,
        options: filter_value_options(&app_field_id, context, discovery.payload)?,
    })
}

fn supported_basic_field(field: &str) -> bool {
    matches!(
        field,
        "record_kind"
            | "rarity"
            | "traits"
            | "level"
            | "pack_label"
            | "publication_title"
            | "publication_family"
            | "publication_remaster"
            | "traditions"
            | "spell_kinds"
            | "save_type"
            | "basic_save"
            | "sustained"
            | "damage_types"
            | "range_value"
            | "area_type"
            | "area_value"
            | "item_category"
            | "item_group"
            | "price_cp"
            | "bulk_value"
            | "hands"
            | "usage"
            | "base_item"
            | "size"
            | "speed_types"
            | "languages"
            | "senses"
            | "immunities"
            | "resistances"
            | "weaknesses"
            | "metric"
    )
}

fn app_field_id(field: &str) -> String {
    app_filter_field_id(field)
}

fn filter_field_label(field: &str) -> &'static str {
    match field {
        "kind" => "Kinds",
        "rarity" => "Rarity",
        "traits" => "Traits",
        "level" => "Level",
        "pack" => "Pack",
        "publication_title" => "Publication",
        "publication_family" => "Publication Family",
        "publication_remaster" => "Remaster",
        "traditions" => "Traditions",
        "spell_kinds" => "Spell Type",
        "save_type" => "Save Type",
        "basic_save" => "Basic Save",
        "sustained" => "Sustained",
        "damage_types" => "Damage Type",
        "range_value" => "Range",
        "area_type" => "Area Type",
        "area_value" => "Area Size",
        "item_category" => "Category",
        "item_group" => "Group",
        "price_cp" => "Price",
        "bulk_value" => "Bulk",
        "hands" => "Hands",
        "usage" => "Usage",
        "base_item" => "Base Item",
        "size" => "Size",
        "speed_types" => "Speeds",
        "languages" => "Languages",
        "senses" => "Senses",
        "immunities" => "Immunities",
        "resistances" => "Resistances",
        "weaknesses" => "Weaknesses",
        "metric" => "Metric",
        _ => "Filter",
    }
}

fn allowed_operators(field: &FilterFieldInfo) -> Vec<FilterClauseOperator> {
    if field.field_type == FilterFieldType::Metric {
        return vec![FilterClauseOperator::MetricCompare];
    }
    if field.field_type == FilterFieldType::Number {
        return vec![FilterClauseOperator::Range];
    }
    let mut operators = vec![
        FilterClauseOperator::IncludeAny,
        FilterClauseOperator::ExcludeAny,
    ];
    if field.field == "traits" {
        operators.insert(1, FilterClauseOperator::IncludeAll);
    }
    if field.operators.contains(&FilterOperator::Eq)
        || field.operators.contains(&FilterOperator::Includes)
    {
        operators
    } else {
        Vec::new()
    }
}

fn default_operator(field: &FilterFieldInfo) -> FilterClauseOperator {
    if field.field_type == FilterFieldType::Metric {
        FilterClauseOperator::MetricCompare
    } else if field.field == "traits" {
        FilterClauseOperator::IncludeAll
    } else if field.field_type == FilterFieldType::Number {
        FilterClauseOperator::Range
    } else {
        FilterClauseOperator::IncludeAny
    }
}

fn filter_value_options(
    field_id: &str,
    context: &FilterDiscoveryContext,
    payload: FilterValuePayload,
) -> AppServiceResult<Vec<FilterValueOption>> {
    let mut values = match payload {
        FilterValuePayload::Enumerable { values, .. } => values,
        FilterValuePayload::BooleanCounts { counts } => vec![
            FilterValueCount {
                value: "true".to_string(),
                count: counts.r#true,
            },
            FilterValueCount {
                value: "false".to_string(),
                count: counts.r#false,
            },
        ],
        FilterValuePayload::MetricKeys { metrics } => {
            let selected = selected_metric_keys(context);
            return Ok(metrics
                .into_iter()
                .map(|metric| {
                    let is_selected = selected.contains(&metric.metric_key);
                    FilterValueOption {
                        label: metric
                            .short_label
                            .or(metric.label)
                            .unwrap_or_else(|| filter_value_label(field_id, &metric.metric_key)),
                        value: metric.metric_key,
                        count: Some(metric.count),
                        selected: is_selected,
                        disabled: !is_selected && metric.count == 0,
                        status: if metric.count == 0 {
                            "empty"
                        } else {
                            "available"
                        }
                        .to_string(),
                    }
                })
                .collect());
        }
        _ => {
            return Ok(Vec::new());
        }
    };
    let selected = selected_values(field_id, context);
    sort_filter_values(field_id, &mut values);
    Ok(values
        .into_iter()
        .map(|value| {
            let is_selected = selected.contains(&value.value);
            FilterValueOption {
                label: filter_value_label(field_id, &value.value),
                value: value.value,
                count: Some(value.count),
                selected: is_selected,
                disabled: !is_selected && value.count == 0,
                status: if value.count == 0 {
                    "empty"
                } else {
                    "available"
                }
                .to_string(),
            }
        })
        .collect())
}

fn filter_editor_groups() -> Vec<FilterEditorGroupView> {
    [
        ("standard", "Standard"),
        ("source", "Source"),
        ("spells", "Spells"),
        ("spells_equipment", "Spells & Equipment"),
        ("equipment", "Equipment"),
        ("creatures", "Creatures"),
        ("metrics", "Metrics"),
    ]
    .into_iter()
    .map(|(id, label)| FilterEditorGroupView {
        id: id.to_string(),
        label: label.to_string(),
        fields: Vec::new(),
    })
    .collect()
}

fn sort_filter_editor_groups(groups: &mut [FilterEditorGroupView]) {
    for group in groups {
        group
            .fields
            .sort_by_key(|field| filter_field_order(&group.id, &field.id));
    }
}

fn filter_field_order(group_id: &str, field_id: &str) -> usize {
    match group_id {
        "standard" => match field_id {
            "kind" => 0,
            "level" => 1,
            "rarity" => 2,
            "traits" => 3,
            _ => 100,
        },
        "source" => match field_id {
            "pack" => 0,
            "publication_title" => 1,
            "publication_family" => 2,
            "publication_remaster" => 3,
            _ => 100,
        },
        _ => 100,
    }
}

fn filter_editor_field_view(
    field: &FilterFieldInfo,
    applicability: FilterFieldApplicability,
) -> FilterEditorFieldView {
    let app_id = app_field_id(&field.field);
    FilterEditorFieldView {
        id: app_id.clone(),
        label: filter_field_label(&app_id).to_string(),
        control: filter_control(field),
        placement: field_placement(&app_id),
        applicability,
        allowed_operators: allowed_operators(field),
        default_operator: default_operator(field),
        supports_counts: matches!(
            field.value_policy,
            FilterValuePolicy::Enumerable
                | FilterValuePolicy::BooleanCounts
                | FilterValuePolicy::MetricKeys
        ),
    }
}

fn filter_control(field: &FilterFieldInfo) -> FilterControlView {
    match field.field_type {
        FilterFieldType::Number => FilterControlView::Range {
            min_label: "Min".to_string(),
            max_label: "Max".to_string(),
            min: range_minimum(&field.field),
            max: range_maximum(&field.field),
            step: range_step(&field.field),
        },
        FilterFieldType::Boolean => FilterControlView::Boolean {
            true_label: "Yes".to_string(),
            false_label: "No".to_string(),
        },
        FilterFieldType::Set | FilterFieldType::EnumString | FilterFieldType::Text => {
            FilterControlView::MultiSelect
        }
        FilterFieldType::Metric => FilterControlView::MetricComparison {
            key_label: "Metric".to_string(),
            operator_label: "Operator".to_string(),
            value_label: "Value".to_string(),
        },
    }
}

fn field_placement(field: &str) -> FilterFieldPlacement {
    match field {
        "kind" | "rarity" | "traits" | "level" => FilterFieldPlacement::AlwaysVisible,
        "pack" => FilterFieldPlacement::InitiallyVisible,
        _ => FilterFieldPlacement::Addable,
    }
}

fn range_minimum(field: &str) -> Option<f64> {
    match field {
        "level" => Some(0.0),
        _ => None,
    }
}

fn range_maximum(field: &str) -> Option<f64> {
    match field {
        "level" => Some(30.0),
        _ => None,
    }
}

fn range_step(field: &str) -> Option<f64> {
    match field {
        "level" | "price_cp" | "hands" | "range_value" | "area_value" => Some(1.0),
        _ => None,
    }
}

fn field_group_id(field: &str) -> &'static str {
    match field {
        "kind" | "rarity" | "traits" | "level" => "standard",
        "pack" | "publication_title" | "publication_family" | "publication_remaster" => "source",
        "traditions" | "spell_kinds" | "save_type" | "basic_save" | "sustained" | "range_value"
        | "area_type" | "area_value" => "spells",
        "damage_types" => "spells_equipment",
        "item_category" | "item_group" | "price_cp" | "bulk_value" | "hands" | "usage"
        | "base_item" => "equipment",
        "size" | "speed_types" | "languages" | "senses" | "immunities" | "resistances"
        | "weaknesses" => "creatures",
        "metric" => "metrics",
        _ => "source",
    }
}

fn sort_filter_values(field_id: &str, values: &mut [FilterValueCount]) {
    if field_id == "rarity" {
        values.sort_by_key(|value| rarity_rank(&value.value));
    }
}

fn rarity_rank(value: &str) -> usize {
    match value {
        "common" => 0,
        "uncommon" => 1,
        "rare" => 2,
        "unique" => 3,
        _ => 4,
    }
}

fn selected_values(field_id: &str, context: &FilterDiscoveryContext) -> Vec<String> {
    let filter = match context {
        FilterDiscoveryContext::Filtered { filter } => filter,
    };
    filter
        .clauses
        .iter()
        .filter(|clause| app_field_id(&clause.field) == field_id)
        .flat_map(|clause| clause.values.iter().cloned())
        .collect()
}

fn selected_metric_keys(context: &FilterDiscoveryContext) -> Vec<String> {
    let filter = match context {
        FilterDiscoveryContext::Filtered { filter } => filter,
    };
    filter
        .clauses
        .iter()
        .filter_map(|clause| clause.metric.as_ref())
        .map(|metric| metric.key.clone())
        .collect()
}

fn filter_value_label(field_id: &str, value: &str) -> String {
    if matches!(
        field_id,
        "publication_remaster" | "basic_save" | "sustained"
    ) {
        return match value {
            "true" => "Yes".to_string(),
            "false" => "No".to_string(),
            _ => value.to_string(),
        };
    }
    if matches!(
        field_id,
        "kind"
            | "rarity"
            | "publication_family"
            | "spell_kinds"
            | "save_type"
            | "area_type"
            | "item_category"
            | "item_group"
            | "hands"
            | "usage"
            | "size"
            | "speed_types"
    ) {
        title_case(value)
    } else {
        value.to_string()
    }
}

fn title_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{BasicSearchFilter, FilterClause, MetricComparison};
    use atlas_domain::{
        BooleanFieldCounts, FilterDiscoveryExecution, FilterFieldGroup, FilterValueSort,
        MetricKeyDiscovery,
    };

    use super::*;

    #[test]
    fn editor_projection_groups_supported_fields_and_uses_app_ids() {
        let discovery = FilterFieldDiscovery {
            filter: None,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count: 3,
            fields: vec![
                field("level", FilterFieldType::Number),
                field("pack_label", FilterFieldType::EnumString),
                field("traits", FilterFieldType::Set),
                field("record_kind", FilterFieldType::EnumString),
                field("metric", FilterFieldType::Metric),
                field("foundry_document_type", FilterFieldType::EnumString),
            ],
        };
        let view = filter_editor_view(discovery.clone(), discovery, &[]);

        let standard = view
            .groups
            .iter()
            .find(|group| group.id == "standard")
            .expect("standard group should be present");
        let source = view
            .groups
            .iter()
            .find(|group| group.id == "source")
            .expect("source group should be present");
        let ids = standard
            .fields
            .iter()
            .chain(source.fields.iter())
            .map(|field| field.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["kind", "level", "traits", "pack"]);
        let metrics = view
            .groups
            .iter()
            .find(|group| group.id == "metrics")
            .expect("metrics group should be present");
        assert_eq!(standard.fields[0].label, "Kinds");
        assert_eq!(
            standard.fields[0].applicability,
            atlas_app_model::FilterFieldApplicability::Applicable
        );
        assert_eq!(
            standard.fields[0].placement,
            atlas_app_model::FilterFieldPlacement::AlwaysVisible
        );
        assert_eq!(
            standard.fields[1].default_operator,
            FilterClauseOperator::Range
        );
        assert_eq!(
            standard.fields[1].allowed_operators,
            vec![FilterClauseOperator::Range]
        );
        assert!(matches!(
            standard.fields[1].control,
            atlas_app_model::FilterControlView::Range {
                min: Some(0.0),
                max: Some(30.0),
                step: Some(1.0),
                ..
            }
        ));
        assert_eq!(
            standard.fields[2].default_operator,
            FilterClauseOperator::IncludeAll
        );
        assert_eq!(
            source.fields[0].placement,
            atlas_app_model::FilterFieldPlacement::InitiallyVisible
        );
        assert_eq!(metrics.fields[0].id, "metric");
        assert_eq!(
            metrics.fields[0].default_operator,
            FilterClauseOperator::MetricCompare
        );
        assert!(matches!(
            metrics.fields[0].control,
            atlas_app_model::FilterControlView::MetricComparison { .. }
        ));
        assert!(metrics.fields[0].supports_counts);
    }

    #[test]
    fn editor_projection_preserves_selected_unavailable_fields() {
        let applicable = FilterFieldDiscovery {
            filter: None,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count: 1,
            fields: vec![field("record_kind", FilterFieldType::EnumString)],
        };
        let candidates = FilterFieldDiscovery {
            filter: None,
            execution: FilterDiscoveryExecution::Catalog,
            matching_record_count: 3,
            fields: vec![
                field("record_kind", FilterFieldType::EnumString),
                field("publication_title", FilterFieldType::Text),
            ],
        };

        let view = filter_editor_view(applicable, candidates, &["publication".to_string()]);
        let fields = view
            .groups
            .iter()
            .flat_map(|group| group.fields.iter())
            .collect::<Vec<_>>();

        let publication = fields
            .iter()
            .find(|field| field.id == "publication_title")
            .expect("selected unavailable field should be preserved");
        assert_eq!(
            publication.applicability,
            atlas_app_model::FilterFieldApplicability::SelectedUnavailable
        );
    }

    #[test]
    fn rarity_values_use_domain_order() {
        let values = filter_value_list_view(
            "rarity",
            &FilterDiscoveryContext::Filtered {
                filter: BasicSearchFilter { clauses: vec![] },
            },
            FilterValueDiscovery {
                field: "rarity".to_string(),
                filter: None,
                execution: FilterDiscoveryExecution::Dynamic,
                matching_record_count: 4,
                payload: FilterValuePayload::Enumerable {
                    values: vec![
                        FilterValueCount {
                            value: "unique".to_string(),
                            count: 1,
                        },
                        FilterValueCount {
                            value: "common".to_string(),
                            count: 2,
                        },
                        FilterValueCount {
                            value: "rare".to_string(),
                            count: 3,
                        },
                        FilterValueCount {
                            value: "uncommon".to_string(),
                            count: 4,
                        },
                    ],
                    null_count: 0,
                    sort: FilterValueSort::Alpha,
                },
            },
        )
        .expect("rarity values should project");

        let order = values
            .options
            .iter()
            .map(|option| option.value.as_str())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["common", "uncommon", "rare", "unique"]);
    }

    #[test]
    fn value_projection_preserves_counts_selection_and_boolean_labels() {
        let context = FilterDiscoveryContext::Filtered {
            filter: BasicSearchFilter {
                clauses: vec![FilterClause {
                    id: "pack-include_any".to_string(),
                    field: "pack".to_string(),
                    operator: FilterClauseOperator::IncludeAny,
                    values: vec!["Actions".to_string()],
                    range: None,
                    metric: None,
                }],
            },
        };

        let packs = filter_value_list_view(
            "pack",
            &context,
            FilterValueDiscovery {
                field: "pack_label".to_string(),
                filter: None,
                execution: FilterDiscoveryExecution::Dynamic,
                matching_record_count: 3,
                payload: FilterValuePayload::Enumerable {
                    values: vec![
                        FilterValueCount {
                            value: "Actions".to_string(),
                            count: 0,
                        },
                        FilterValueCount {
                            value: "Spells".to_string(),
                            count: 0,
                        },
                    ],
                    null_count: 0,
                    sort: FilterValueSort::Alpha,
                },
            },
        )
        .expect("pack values should project");

        assert_eq!(packs.field_id, "pack");
        assert!(packs.options[0].selected);
        assert!(!packs.options[0].disabled);
        assert!(!packs.options[1].selected);
        assert!(packs.options[1].disabled);

        let remaster = filter_value_list_view(
            "publication_remaster",
            &FilterDiscoveryContext::Filtered {
                filter: BasicSearchFilter { clauses: vec![] },
            },
            FilterValueDiscovery {
                field: "publication_remaster".to_string(),
                filter: None,
                execution: FilterDiscoveryExecution::Dynamic,
                matching_record_count: 3,
                payload: FilterValuePayload::BooleanCounts {
                    counts: BooleanFieldCounts {
                        r#true: 2,
                        r#false: 1,
                        null: 0,
                    },
                },
            },
        )
        .expect("boolean values should project");

        assert_eq!(remaster.options[0].label, "Yes");
        assert_eq!(remaster.options[0].count, Some(2));
        assert_eq!(remaster.options[1].label, "No");
    }

    #[test]
    fn metric_key_values_project_labels_counts_and_selection() {
        let context = FilterDiscoveryContext::Filtered {
            filter: BasicSearchFilter {
                clauses: vec![FilterClause {
                    id: "metric-metric_compare".to_string(),
                    field: "metric".to_string(),
                    operator: FilterClauseOperator::MetricCompare,
                    values: vec![],
                    range: None,
                    metric: Some(MetricComparison {
                        key: "spell.area.value".to_string(),
                        op: "gte".to_string(),
                        value: 10.0,
                    }),
                }],
            },
        };

        let metrics = filter_value_list_view(
            "metric",
            &context,
            FilterValueDiscovery {
                field: "metric".to_string(),
                filter: None,
                execution: FilterDiscoveryExecution::Dynamic,
                matching_record_count: 3,
                payload: FilterValuePayload::MetricKeys {
                    metrics: vec![MetricKeyDiscovery {
                        metric_domain: "record".to_string(),
                        kind: "spell".to_string(),
                        namespace_prefix: "spell".to_string(),
                        metric_key: "spell.area.value".to_string(),
                        label: Some("Area Value".to_string()),
                        short_label: Some("Area".to_string()),
                        group: None,
                        known: true,
                        value_type: "number".to_string(),
                        count: 2,
                        numeric_stats: None,
                    }],
                },
            },
        )
        .expect("metric key values should project");

        assert_eq!(metrics.field_id, "metric");
        assert_eq!(metrics.options[0].value, "spell.area.value");
        assert_eq!(metrics.options[0].label, "Area");
        assert_eq!(metrics.options[0].count, Some(2));
        assert!(metrics.options[0].selected);
    }

    fn field(name: &str, field_type: FilterFieldType) -> FilterFieldInfo {
        FilterFieldInfo {
            field: name.to_string(),
            field_type,
            group: FilterFieldGroup::Record,
            value_policy: match field_type {
                FilterFieldType::Boolean => FilterValuePolicy::BooleanCounts,
                FilterFieldType::Number => FilterValuePolicy::NumericStats,
                FilterFieldType::Metric => FilterValuePolicy::MetricKeys,
                _ => FilterValuePolicy::Enumerable,
            },
            operators: vec![FilterOperator::Eq, FilterOperator::Includes],
            applicable_kinds: vec!["rule".to_string()],
            cli_flags: vec![],
            catalog_available: true,
        }
    }
}
