//! Product editor metadata decorates, but never replaces, the executable catalog.
use atlas_app_model::{
    FilterChoiceView, FilterControlView, FilterEditorFieldView, FilterEditorGroupView,
    FilterEditorView, FilterFieldPlacement, FilterOperatorView,
};
use atlas_domain::{QueryCapability, QueryFieldDefinition, QueryFieldType, QueryLiteral};
use std::collections::BTreeMap;

const COMMON: [&str; 4] = ["record.kind", "record.level_or_rank", "rarity", "traits"];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProductGroup {
    Common,
    Creatures,
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
) -> FilterEditorView {
    let mut groups = BTreeMap::<ProductGroup, FilterEditorGroupView>::new();
    for definition in catalog.fields {
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
        let choices = definition
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
            .map(|operator| FilterOperatorView {
                operator: operator.clone(),
                label: operator_label(operator).into(),
            })
            .collect();
        let help = match definition.id.as_str() {
            "record.level_or_rank" => Some("Spell rank for spells; authored level for other applicable records. Unavailable values stay unknown.".into()),
            "item.level" => Some("Authored level for applicable physical items, feats, campaign features, effects and afflictions; use Level / rank to include spells and creatures.".into()),
            "source.document_kind" => Some("Source document classification; use Kind for reference categories.".into()),
            _ => None,
        };
        let field = FilterEditorFieldView {
            definition,
            control,
            placement,
            label,
            help,
            choices,
            operators,
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

    FilterEditorView {
        catalog_version: catalog.version,
        limits: catalog.limits,
        groups: groups.into_values().collect(),
        common_field_ids: COMMON.into_iter().map(str::to_owned).collect(),
    }
}
fn product_group(d: &QueryFieldDefinition) -> ProductGroup {
    if COMMON.contains(&d.id.as_str()) {
        return ProductGroup::Common;
    }
    let path = d.scope.as_deref().unwrap_or(&d.path);
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
    let label = &d.label;
    if let Some(scope) = &d.scope {
        let context = match scope.as_str() {
            "actor.items" => "Embedded item",
            "actor.immunities" => "Immunity",
            "actor.weaknesses" => "Weakness",
            "actor.resistances" => "Resistance",
            "actor.speeds" => "Movement",
            "actor.senses" => "Sense",
            "spell.damage" => "Spell damage",
            _ => "Entry",
        };
        format!("{context}: {label}")
    } else {
        label.clone()
    }
}
fn words(value: &str) -> String {
    let spaced = value.replace(['_', '-'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
fn operator_label(operator: &str) -> &str {
    match operator {
        "eq" => "is",
        "neq" => "is not",
        "in" => "is any of",
        "lt" => "below",
        "lte" => "at most",
        "gt" => "above",
        "gte" => "at least",
        "between" => "between",
        "includes" => "includes",
        "includes_any" => "includes any",
        "includes_all" => "includes all",
        "excludes_any" => "excludes any",
        "exists" => "has a matching entry",
        "state" => "availability is",
        _ => operator,
    }
}
