//! Product editor metadata decorates, but never replaces, the executable catalog.
use atlas_app_model::{
    FilterChoiceView, FilterCollectionControlView, FilterControlView, FilterEditorFieldView,
    FilterEditorGroupView, FilterEditorView, FilterFieldPlacement, FilterOperandView,
    FilterOperatorView,
};
use atlas_domain::{QueryCapability, QueryFieldDefinition, QueryFieldType, QueryLiteral};
use std::collections::BTreeMap;

const COMMON: [&str; 4] = ["record.kind", "record.level_or_rank", "rarity", "traits"];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProductGroup {
    Common,
    Creatures,
    Defenses,
    Movement,
    Languages,
    Embedded,
    Hazards,
    Equipment,
    Spells,
    Feats,
    Ancestries,
    Heritages,
    Backgrounds,
    Classes,
    Deities,
    Effects,
    Afflictions,
    Campaign,
    Other,
    Source,
}
impl ProductGroup {
    fn metadata(self) -> (&'static str, &'static str) {
        match self {
            Self::Common => ("common", "Common"),
            Self::Creatures => ("creatures", "Creatures"),
            Self::Defenses => ("defenses", "Defenses"),
            Self::Movement => ("movement", "Movement and senses"),
            Self::Languages => ("languages", "Languages"),
            Self::Embedded => ("embedded", "Embedded records"),
            Self::Hazards => ("hazards", "Hazards"),
            Self::Equipment => ("equipment", "Equipment"),
            Self::Spells => ("spells", "Spells"),
            Self::Feats => ("feats", "Feats"),
            Self::Ancestries => ("ancestries", "Ancestries"),
            Self::Heritages => ("heritages", "Heritages"),
            Self::Backgrounds => ("backgrounds", "Backgrounds"),
            Self::Classes => ("classes", "Classes"),
            Self::Deities => ("deities", "Deities"),
            Self::Effects => ("effects", "Effects"),
            Self::Afflictions => ("afflictions", "Afflictions"),
            Self::Campaign => ("campaign", "Campaign features"),
            Self::Other => ("other", "Other reference fields"),
            Self::Source => ("source", "Source"),
        }
    }
}
pub(crate) fn filter_editor_view(
    catalog: QueryCapability,
    selected: &[String],
) -> crate::AppServiceResult<FilterEditorView> {
    let mut groups = BTreeMap::<ProductGroup, FilterEditorGroupView>::new();
    for definition in catalog.fields {
        // Source identifiers remain executable technical bindings, but are not
        // useful product filter controls. Ancestry has a separate slug filter.
        if definition.path == "heritage.ancestry.uuid" {
            continue;
        }
        let group = product_group(&definition);
        let control = match definition.field_type {
            QueryFieldType::String => FilterControlView::Text,
            QueryFieldType::Number => FilterControlView::Numeric,
            QueryFieldType::Boolean => FilterControlView::Boolean,
            QueryFieldType::Set => FilterControlView::Set,
            QueryFieldType::Collection => FilterControlView::Collection,
        };
        let placement =
            if selected.contains(&definition.id) || COMMON.contains(&definition.id.as_str()) {
                FilterFieldPlacement::InitiallyVisible
            } else {
                FilterFieldPlacement::Addable
            };
        let label = field_label(&definition);
        let choices: Vec<_> = definition
            .choices
            .iter()
            .map(|value| FilterChoiceView {
                value: QueryLiteral::String(value.clone()),
                label: if definition.path == "record.kind" {
                    crate::projection::kind_label(value)
                } else {
                    words(value)
                },
            })
            .chain(if definition.field_type == QueryFieldType::Boolean {
                vec![
                    FilterChoiceView {
                        value: QueryLiteral::Boolean(true),
                        label: "Yes".into(),
                    },
                    FilterChoiceView {
                        value: QueryLiteral::Boolean(false),
                        label: "No".into(),
                    },
                ]
            } else {
                vec![]
            })
            .collect();
        let operators = definition
            .operators
            .iter()
            .map(|operator| operator_view(operator))
            .collect::<crate::AppServiceResult<Vec<_>>>()?;
        let help = match definition.id.as_str() {
            "record.level_or_rank" => Some("Spell rank for spells; authored level for other applicable records. Unavailable values stay unknown.".into()),
            "item.level" => Some("Authored level for applicable physical items, feats, campaign features, effects and afflictions; use Level / rank to include spells and creatures.".into()),
            "source.document_kind" => Some("Source document classification; use Kind for reference categories.".into()),
            _ => None,
        };
        let collection_control = match definition.id.as_str() {
            "actor.speeds" | "actor.weaknesses" | "actor.resistances" => {
                Some(FilterCollectionControlView {
                    category_field_id: format!("{}[].type", definition.id),
                    amount_field_id: Some(format!("{}[].value", definition.id)),
                })
            }
            "actor.senses" | "actor.immunities" | "spell.damage" => {
                Some(FilterCollectionControlView {
                    category_field_id: format!("{}[].type", definition.id),
                    amount_field_id: None,
                })
            }
            _ => None,
        };
        let unit_fields = if ["rounds", "minutes", "hours", "days"]
            .iter()
            .any(|unit| definition.id == format!("effect.duration.{unit}"))
        {
            ["rounds", "minutes", "hours", "days"]
                .iter()
                .map(|unit| atlas_app_model::FilterUnitFieldView {
                    field_id: format!("effect.duration.{unit}"),
                    unit: (*unit).into(),
                })
                .collect()
        } else {
            Vec::new()
        };
        let label = if unit_fields.is_empty() {
            label
        } else {
            "Duration".into()
        };
        let field = FilterEditorFieldView {
            applicable: true,
            unit_fields,
            short_choices: matches!(definition.id.as_str(), "record.kind" | "rarity")
                || (!choices.is_empty() && choices.len() <= 12),
            definition,
            control,
            placement,
            label,
            help,
            choices,
            operators,
            collection_control,
        };
        groups
            .entry(group)
            .or_insert_with(|| {
                let (id, label) = group.metadata();
                FilterEditorGroupView {
                    id: id.into(),
                    label: label.into(),
                    fields: vec![],
                }
            })
            .fields
            .push(field);
    }
    for group in groups.values_mut() {
        group.fields.sort_by(|a, b| {
            let order = |id: &str| {
                COMMON
                    .iter()
                    .position(|common| common == &id)
                    .unwrap_or(COMMON.len())
            };
            order(&a.definition.id)
                .cmp(&order(&b.definition.id))
                .then(a.label.cmp(&b.label))
                .then(a.definition.id.cmp(&b.definition.id))
        });
    }

    Ok(FilterEditorView {
        catalog_version: catalog.version,
        limits: catalog.limits,
        groups: groups.into_values().collect(),
        common_field_ids: COMMON.into_iter().map(str::to_owned).collect(),
    })
}
fn product_group(d: &QueryFieldDefinition) -> ProductGroup {
    if COMMON.contains(&d.id.as_str()) {
        return ProductGroup::Common;
    }
    let path = d.scope.as_deref().unwrap_or(&d.path);
    if path == "actor.items" {
        return ProductGroup::Embedded;
    }
    if path == "actor.languages" {
        return ProductGroup::Languages;
    }
    if matches!(
        path,
        "actor.speeds" | "actor.senses" | "actor.speed.land" | "actor.perception"
    ) {
        return ProductGroup::Movement;
    }
    if matches!(
        path,
        "actor.armor_class"
            | "actor.hp.maximum"
            | "actor.immunities"
            | "actor.weaknesses"
            | "actor.resistances"
    ) || path.starts_with("actor.saves.")
    {
        return ProductGroup::Defenses;
    }
    match path.split('.').next().unwrap_or("") {
        "actor" => ProductGroup::Creatures,
        "hazard" => ProductGroup::Hazards,
        "item" | "physical" | "weapon" | "armor" | "shield" | "consumable" | "equipment"
        | "backpack" | "treasure" => ProductGroup::Equipment,
        "spell" => ProductGroup::Spells,
        "feat" | "action" => ProductGroup::Feats,
        "ancestry" => ProductGroup::Ancestries,
        "heritage" => ProductGroup::Heritages,
        "background" => ProductGroup::Backgrounds,
        "class" => ProductGroup::Classes,
        "deity" => ProductGroup::Deities,
        "effect" | "condition" => ProductGroup::Effects,
        "affliction" => ProductGroup::Afflictions,
        "campaignFeature" => ProductGroup::Campaign,
        "source" | "publication" => ProductGroup::Source,
        _ => ProductGroup::Other,
    }
}
fn field_label(d: &QueryFieldDefinition) -> String {
    if d.path == "actor.items" {
        return "Embedded records".into();
    }
    if d.scope.as_deref() == Some("actor.items") && d.path == "source.type" {
        return "Item type".into();
    }
    d.label.clone()
}
fn words(value: &str) -> String {
    let spaced = value.replace(['_', '-'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
fn operator_view(operator: &str) -> crate::AppServiceResult<FilterOperatorView> {
    use FilterOperandView::{List, Predicate, Range, Scalar, State};
    let (label, operand) = match operator {
        "eq" => ("is", Scalar),
        "neq" => ("is not", Scalar),
        "in" => ("is any of", List),
        "lt" => ("below", Scalar),
        "lte" => ("at most", Scalar),
        "gt" => ("above", Scalar),
        "gte" => ("at least", Scalar),
        "between" => ("between", Range),
        "includes" => ("includes", Scalar),
        "includes_any" => ("includes any", List),
        "includes_all" => ("includes all", List),
        "excludes_any" => ("excludes any", List),
        "exists" => ("has a matching entry", Predicate),
        "state" => ("availability is", State),
        _ => {
            return Err(crate::AppServiceError::new(
                atlas_app_model::AppErrorCode::InternalError,
                format!("Executable operator {operator} has no editor operand contract"),
            ));
        }
    };
    Ok(FilterOperatorView {
        operator: operator.into(),
        label: label.into(),
        operand,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn unsupported_catalog_operator_has_no_fallback_operand() {
        let error = super::operator_view("new_operator")
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, atlas_app_model::AppErrorCode::InternalError);
        assert!(error.message.contains("new_operator"));
    }
}
