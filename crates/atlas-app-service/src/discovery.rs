use atlas_app_model::{
    FilterClauseOperator, FilterDiscoveryContext, FilterFieldListView, FilterFieldView,
    FilterValueListView, FilterValueOption,
};
use atlas_domain::{
    FilterFieldDiscovery, FilterFieldInfo, FilterFieldType, FilterOperator, FilterValueCount,
    FilterValueDiscovery, FilterValuePayload, FilterValuePolicy,
};

use crate::AppServiceResult;
use crate::filter::discovery_field_id;

pub(crate) fn filter_field_list_view(discovery: FilterFieldDiscovery) -> FilterFieldListView {
    FilterFieldListView {
        matching_record_count: discovery.matching_record_count,
        fields: discovery
            .fields
            .iter()
            .filter(|field| supported_basic_field(&field.field))
            .map(filter_field_view)
            .collect(),
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

fn filter_field_view(field: &FilterFieldInfo) -> FilterFieldView {
    let app_id = app_field_id(&field.field);
    FilterFieldView {
        id: app_id.clone(),
        label: filter_field_label(&app_id).to_string(),
        cardinality: cardinality(field).to_string(),
        value_kind: value_kind(field.field_type).to_string(),
        allowed_operators: allowed_operators(field),
        default_operator: default_operator(field),
        ui_hint: ui_hint(field).to_string(),
        supports_counts: matches!(
            field.value_policy,
            FilterValuePolicy::Enumerable | FilterValuePolicy::BooleanCounts
        ),
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

fn cardinality(field: &FilterFieldInfo) -> &'static str {
    match field.field_type {
        FilterFieldType::Set => "many",
        FilterFieldType::Number => "range",
        _ => "one",
    }
}

fn value_kind(field_type: FilterFieldType) -> &'static str {
    match field_type {
        FilterFieldType::Set | FilterFieldType::EnumString | FilterFieldType::Text => "string",
        FilterFieldType::Number => "number",
        FilterFieldType::Boolean => "boolean",
        FilterFieldType::Metric => "metric",
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

fn ui_hint(field: &FilterFieldInfo) -> &'static str {
    match field.field_type {
        FilterFieldType::Number => "range",
        FilterFieldType::Set | FilterFieldType::EnumString | FilterFieldType::Text => {
            "multi_select"
        }
        FilterFieldType::Boolean => "checkbox",
        FilterFieldType::Metric => "metric",
    }
}

fn filter_value_options(
    field_id: &str,
    context: &FilterDiscoveryContext,
    payload: FilterValuePayload,
) -> AppServiceResult<Vec<FilterValueOption>> {
    let values = match payload {
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
    fn field_projection_filters_unsupported_fields_and_uses_app_ids() {
        let view = filter_field_list_view(FilterFieldDiscovery {
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

        let ids = view
            .fields
            .iter()
            .map(|field| field.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["kind", "pack", "traits", "level"]);
        assert_eq!(view.fields[0].label, "Kinds");
        assert_eq!(
            view.fields[2].default_operator,
            FilterClauseOperator::IncludeAll
        );
        assert_eq!(
            view.fields[3].allowed_operators,
            vec![FilterClauseOperator::Range]
        );
        assert_eq!(view.fields[3].ui_hint, "range");
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
