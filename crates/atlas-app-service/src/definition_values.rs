//! Product labels/help decorate query-owned choices without changing their scope.
use crate::{AppServiceResult, definitions::HelpContext};
use atlas_app_model::*;
use atlas_domain::{DefinitionKey, DefinitionNamespace as N, QueryCapability, QueryLiteral};
use atlas_search::AtlasRetrievalService;
use std::collections::BTreeSet;

fn namespace(catalog: &QueryCapability, field: &str) -> Option<N> {
    let field = catalog.fields.iter().find(|f| f.id == field)?;
    match (field.scope.as_deref(), field.path.as_str()) {
        (_, "traits") => Some(N::Trait),
        (_, "rarity") => Some(N::Rarity),
        (_, "languages" | "ancestry.languages" | "ancestry.additional_languages") => {
            Some(N::Language)
        }
        (Some("senses"), "type") => Some(N::Sense),
        (Some("defenses.immunities"), "type" | "exceptions") => Some(N::ImmunityType),
        (Some("defenses.weaknesses"), "type" | "exceptions") => Some(N::WeaknessType),
        (Some("defenses.resistances"), "type" | "exceptions" | "double_against") => {
            Some(N::ResistanceType)
        }
        (Some("spell.damage"), "type") => Some(N::DamageType),
        (_, "weapon.damage.type" | "consumable.damage.type") => Some(N::DamageType),
        (_, "deity.domains.primary" | "deity.domains.alternate") => Some(N::Domain),
        (_, "weapon.group") => Some(N::WeaponGroup),
        (_, "armor.group") => Some(N::ArmorGroup),
        (_, "size" | "physical.size" | "ancestry.size") => Some(N::Size),
        (_, "spell.traditions") => Some(N::Tradition),
        _ => None,
    }
}
fn keys<'a>(
    namespace: Option<N>,
    values: impl IntoIterator<Item = &'a QueryLiteral>,
) -> BTreeSet<DefinitionKey> {
    values
        .into_iter()
        .filter_map(|value| match (namespace, value) {
            (Some(namespace), QueryLiteral::String(value)) => {
                DefinitionKey::new(namespace, value.clone()).ok()
            }
            _ => None,
        })
        .collect()
}
fn decorate(
    context: &HelpContext<'_>,
    namespace: Option<N>,
    value: QueryLiteral,
    label: Option<String>,
) -> AppServiceResult<FilterChoiceView> {
    let fallback = label.unwrap_or_else(|| match &value {
        QueryLiteral::String(v) => v.clone(),
        QueryLiteral::Boolean(true) => "Yes".into(),
        QueryLiteral::Boolean(false) => "No".into(),
        QueryLiteral::Number(n) => n.to_string(),
    });
    let mut reference = match &value {
        QueryLiteral::String(value) => ReferenceValueView {
            value: value.clone(),
            label: fallback,
            help: None,
        },
        _ => {
            return Ok(FilterChoiceView {
                value,
                label: fallback,
                help: None,
            });
        }
    };
    if let Some(namespace) = namespace {
        context.value(namespace, &mut reference)?;
    }
    Ok(FilterChoiceView {
        value,
        label: reference.label,
        help: reference.help,
    })
}
fn fallback_label(catalog: &QueryCapability, field: &str, value: &QueryLiteral) -> Option<String> {
    let QueryLiteral::String(value) = value else {
        return None;
    };
    let definition = catalog
        .fields
        .iter()
        .find(|definition| definition.id == field)?;
    if definition.path == "physical.base_type" {
        return Some(crate::physical_labels::base_type(value));
    }
    if definition.path == "physical.usage"
        && let Some((_, label)) = crate::physical_labels::USAGES
            .iter()
            .find(|(key, _)| key == value)
    {
        return Some((*label).into());
    }
    if definition.path == "kind" {
        return Some(crate::projection::kind_label(value));
    }
    if definition.choices.contains(value) {
        return Some(crate::discovery::words(value));
    }
    let value = value.trim();
    let mut chars = value.chars();
    Some(chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    }))
}
pub(crate) fn options(
    r: &AtlasRetrievalService,
    options: atlas_domain::QueryValueOptions,
) -> AppServiceResult<FilterValueOptionsView> {
    let catalog = r.discover_filter_fields()?;
    let namespace = namespace(&catalog, &options.field);
    let context = HelpContext::load(r, keys(namespace, options.options.iter().map(|o| &o.value)))?;
    Ok(FilterValueOptionsView {
        field: options.field.clone(),
        total_values: options.total_values,
        exhaustive: options.exhaustive,
        count_basis: options.count_basis,
        options: options
            .options
            .into_iter()
            .map(|o| {
                let label = fallback_label(&catalog, &options.field, &o.value);
                let value = decorate(&context, namespace, o.value, label)?;
                Ok(FilterValueOptionView {
                    value: value.value,
                    label: value.label,
                    help: value.help,
                    distinct_roots: o.distinct_roots,
                    selected: o.selected,
                })
            })
            .collect::<AppServiceResult<Vec<_>>>()?,
    })
}
pub(crate) fn vocabulary(
    r: &AtlasRetrievalService,
    values: atlas_domain::QueryVocabulary,
) -> AppServiceResult<FilterVocabularyView> {
    let catalog = r.discover_filter_fields()?;
    let namespace = namespace(&catalog, &values.field);
    let context = HelpContext::load(r, keys(namespace, values.options.iter().map(|o| &o.value)))?;
    Ok(FilterVocabularyView {
        field: values.field.clone(),
        selected_values: values.selected_values,
        total_values: values.total_values,
        basis: values.basis,
        options: values
            .options
            .into_iter()
            .map(|o| {
                let label = fallback_label(&catalog, &values.field, &o.value);
                decorate(&context, namespace, o.value, label)
            })
            .collect::<AppServiceResult<Vec<_>>>()?,
    })
}
pub(crate) fn editor(
    r: &AtlasRetrievalService,
    editor: &mut FilterEditorView,
) -> AppServiceResult<()> {
    let catalog = r.discover_filter_fields()?;
    let mut all_keys = BTreeSet::new();
    for field in editor.groups.iter().flat_map(|g| &g.fields) {
        all_keys.extend(keys(
            namespace(&catalog, &field.definition.id),
            field.choices.iter().map(|o| &o.value),
        ));
    }
    let context = HelpContext::load(r, all_keys)?;
    for field in editor.groups.iter_mut().flat_map(|g| &mut g.fields) {
        let namespace = namespace(&catalog, &field.definition.id);
        for option in &mut field.choices {
            *option = decorate(
                &context,
                namespace,
                option.value.clone(),
                Some(option.label.clone()),
            )?;
        }
    }
    Ok(())
}
