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
        "record_kind" | "rarity" | "traits" | "level" | "pack_label" | "publication_title"
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
        default_operator: default_operator(&app_id),
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

fn default_operator(field: &str) -> FilterClauseOperator {
    if field == "traits" {
        FilterClauseOperator::IncludeAll
    } else if field == "level" {
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
        FilterDiscoveryContext::Browse { filter }
        | FilterDiscoveryContext::TextSearch { filter, .. } => filter,
    };
    filter
        .clauses
        .iter()
        .filter(|clause| app_field_id(&clause.field) == field_id)
        .flat_map(|clause| clause.values.iter().cloned())
        .collect()
}

fn filter_value_label(field_id: &str, value: &str) -> String {
    if field_id == "kind" || field_id == "rarity" {
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
