use atlas_app_model::{
    FilterClauseOperator, FilterControlView, FilterDiscoveryContext, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldPlacement, FilterValueListView,
    FilterValueOption,
};
use atlas_domain::{
    FilterFieldDiscovery, FilterFieldInfo, FilterFieldType, FilterOperator, FilterValueCount,
    FilterValueDiscovery, FilterValuePayload, FilterValuePolicy,
};

use crate::AppServiceResult;
use crate::filter::discovery_field_id;

pub(crate) fn filter_editor_view(discovery: FilterFieldDiscovery) -> FilterEditorView {
    let mut groups = filter_editor_groups();
    for field in discovery
        .fields
        .iter()
        .filter(|field| supported_basic_field(&field.field))
    {
        let app_id = app_field_id(&field.field);
        let group_id = field_group_id(&app_id);
        if let Some(group) = groups.iter_mut().find(|group| group.id == group_id) {
            group.fields.push(filter_editor_field_view(field));
        }
    }
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
    )
}

fn app_field_id(field: &str) -> String {
    match discovery_field_id(field).as_str() {
        "record_kind" => "kind".to_string(),
        "pack_label" => "pack".to_string(),
        other => other.to_string(),
    }
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
        _ => "Filter",
    }
}

fn allowed_operators(field: &FilterFieldInfo) -> Vec<FilterClauseOperator> {
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
    if field.field == "traits" {
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
    ]
    .into_iter()
    .map(|(id, label)| FilterEditorGroupView {
        id: id.to_string(),
        label: label.to_string(),
        fields: Vec::new(),
    })
    .collect()
}

fn filter_editor_field_view(field: &FilterFieldInfo) -> FilterEditorFieldView {
    let app_id = app_field_id(&field.field);
    FilterEditorFieldView {
        id: app_id.clone(),
        label: filter_field_label(&app_id).to_string(),
        control: filter_control(field),
        placement: field_placement(&app_id),
        allowed_operators: allowed_operators(field),
        default_operator: default_operator(field),
        supports_counts: matches!(
            field.value_policy,
            FilterValuePolicy::Enumerable | FilterValuePolicy::BooleanCounts
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
        FilterFieldType::Metric => FilterControlView::MultiSelect,
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
    use atlas_app_model::{BasicSearchFilter, FilterClause};
    use atlas_domain::{
        BooleanFieldCounts, FilterDiscoveryExecution, FilterFieldGroup, FilterValueSort,
    };

    use super::*;

    #[test]
    fn editor_projection_groups_supported_fields_and_uses_app_ids() {
        let view = filter_editor_view(FilterFieldDiscovery {
            filter: None,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count: 3,
            fields: vec![
                field("record_kind", FilterFieldType::EnumString),
                field("pack_label", FilterFieldType::EnumString),
                field("traits", FilterFieldType::Set),
                field("level", FilterFieldType::Number),
                field("foundry_document_type", FilterFieldType::EnumString),
            ],
        });

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
        assert_eq!(ids, vec!["kind", "traits", "level", "pack"]);
        assert_eq!(standard.fields[0].label, "Kinds");
        assert_eq!(
            standard.fields[0].placement,
            atlas_app_model::FilterFieldPlacement::AlwaysVisible
        );
        assert_eq!(
            standard.fields[1].default_operator,
            FilterClauseOperator::IncludeAll
        );
        assert_eq!(
            standard.fields[2].allowed_operators,
            vec![FilterClauseOperator::Range]
        );
        assert!(matches!(
            standard.fields[2].control,
            atlas_app_model::FilterControlView::Range {
                min: Some(0.0),
                max: Some(30.0),
                step: Some(1.0),
                ..
            }
        ));
        assert_eq!(
            source.fields[0].placement,
            atlas_app_model::FilterFieldPlacement::InitiallyVisible
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

    fn field(name: &str, field_type: FilterFieldType) -> FilterFieldInfo {
        FilterFieldInfo {
            field: name.to_string(),
            field_type,
            group: FilterFieldGroup::Record,
            value_policy: match field_type {
                FilterFieldType::Boolean => FilterValuePolicy::BooleanCounts,
                FilterFieldType::Number => FilterValuePolicy::NumericStats,
                _ => FilterValuePolicy::Enumerable,
            },
            operators: vec![FilterOperator::Eq, FilterOperator::Includes],
            applicable_kinds: vec!["rule".to_string()],
            cli_flags: vec![],
            catalog_available: true,
        }
    }
}
