//! Product labels/help decorate query-owned choices without changing their scope.
use crate::{AppServiceResult, definitions::HelpContext};
use atlas_app_model::*;
use atlas_domain::{
    DefinitionKey, DefinitionNamespace as N, QueryFieldDefinition, QueryLiteral, RecordKey,
};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest};
use std::collections::{BTreeMap, BTreeSet};

const ASSOCIATIONS: &[(N, &[(&str, &str)])] = &[
    (N::Trait, &[("traits", "With this trait")]),
    (N::Rarity, &[("rarity", "With this rarity")]),
    (N::Language, &[("languages", "With this language")]),
    (N::Sense, &[("senses", "With this sense")]),
    (
        N::ImmunityType,
        &[("defenses.immunities", "With this immunity")],
    ),
    (
        N::WeaknessType,
        &[("defenses.weaknesses", "With this weakness")],
    ),
    (
        N::ResistanceType,
        &[("defenses.resistances", "With this resistance")],
    ),
    (
        N::DamageType,
        &[
            ("weapon.damage.type", "With this weapon damage"),
            ("consumable.damage.type", "With this consumable damage"),
        ],
    ),
    (
        N::Domain,
        &[
            ("deity.domains.primary", "With this primary domain"),
            ("deity.domains.alternate", "With this alternate domain"),
        ],
    ),
    (N::WeaponGroup, &[("weapon.group", "In this weapon group")]),
    (N::ArmorGroup, &[("armor.group", "In this armor group")]),
    (
        N::Size,
        &[
            ("body.size", "With this body size"),
            ("physical.size", "With this item size"),
            ("ancestry.default_size", "With this ancestry size"),
        ],
    ),
    (N::Tradition, &[("spell.traditions", "With this tradition")]),
];

pub(crate) fn association_fields(namespace: N) -> &'static [(&'static str, &'static str)] {
    ASSOCIATIONS
        .iter()
        .find_map(|(kind, fields)| (*kind == namespace).then_some(*fields))
        .unwrap_or_default()
}

fn namespace(field: &QueryFieldDefinition) -> Option<N> {
    let path = match (field.scope.as_deref(), field.path.as_str()) {
        (_, "ancestry.languages" | "ancestry.additional_languages") => "languages",
        (Some("spell.damage"), "type") => return Some(N::DamageType),
        (
            Some(
                scope @ ("senses"
                | "defenses.immunities"
                | "defenses.weaknesses"
                | "defenses.resistances"),
            ),
            "type",
        ) => scope,
        (
            Some(scope @ ("defenses.immunities" | "defenses.weaknesses" | "defenses.resistances")),
            "exceptions",
        ) => scope,
        (Some("defenses.resistances"), "double_against") => "defenses.resistances",
        (
            None,
            "senses" | "defenses.immunities" | "defenses.weaknesses" | "defenses.resistances",
        ) => return None,
        (_, path) => path,
    };
    ASSOCIATIONS.iter().find_map(|(kind, fields)| {
        fields
            .iter()
            .any(|(candidate, _)| *candidate == path)
            .then_some(*kind)
    })
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
) -> FilterChoiceView {
    let mut label = label.unwrap_or_else(|| match &value {
        QueryLiteral::String(v) => v.clone(),
        QueryLiteral::Boolean(true) => "Yes".into(),
        QueryLiteral::Boolean(false) => "No".into(),
        QueryLiteral::Number(n) => n.to_string(),
    });
    let mut help = None;
    if let (Some(namespace), QueryLiteral::String(token)) = (namespace, &value) {
        context.keyed(namespace, token, &mut label, &mut help);
    }
    FilterChoiceView { value, label, help }
}

fn action_choices<'a>(
    r: &AtlasRetrievalService,
    field: &str,
    values: impl Iterator<Item = &'a QueryLiteral>,
) -> AppServiceResult<BTreeMap<String, FilterChoiceView>> {
    if field != "actions" {
        return Ok(BTreeMap::new());
    }
    let keys: Vec<_> = values
        .filter_map(|value| match value {
            QueryLiteral::String(key) => RecordKey::parse(key).ok(),
            _ => None,
        })
        .filter(|key| key.pack().as_str() == "bestiary-ability-glossary-srd")
        .collect();
    Ok(r.get_records(GetRecordsRequest { record_keys: &keys })?
        .records
        .into_iter()
        .filter(|record| {
            record.document_kind == "Item"
                && record
                    .source_type
                    .as_value()
                    .is_some_and(|value| value == "action")
        })
        .filter_map(|record| {
            let label = record.name.as_value()?.clone();
            let key = record.key.to_string();
            let navigation = crate::definitions::record_navigation(
                record.key,
                Vec::new(),
                r.source_fingerprint(),
            );
            Some((
                key.clone(),
                FilterChoiceView {
                    value: QueryLiteral::String(key),
                    label,
                    help: Some(ReferenceHelpView {
                        definition_key: None,
                        destination: ReferenceDestinationView::Record {
                            navigation: navigation.into(),
                        },
                    }),
                },
            ))
        })
        .collect())
}
fn fallback_label(definition: &QueryFieldDefinition, value: &QueryLiteral) -> Option<String> {
    let QueryLiteral::String(value) = value else {
        return None;
    };
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
    let field = catalog.fields.iter().find(|f| f.id == options.field);
    let namespace = field.and_then(namespace);
    let context = HelpContext::load(r, keys(namespace, options.options.iter().map(|o| &o.value)))?;
    let actions = action_choices(r, &options.field, options.options.iter().map(|o| &o.value))?;
    Ok(FilterValueOptionsView {
        field: options.field.clone(),
        total_values: options.total_values,
        exhaustive: options.exhaustive,
        count_basis: options.count_basis,
        options: options
            .options
            .into_iter()
            .map(|o| {
                let value = if let QueryLiteral::String(key) = &o.value
                    && let Some(choice) = actions.get(key)
                {
                    choice.clone()
                } else {
                    let label = field.and_then(|field| fallback_label(field, &o.value));
                    decorate(&context, namespace, o.value, label)
                };
                FilterValueOptionView {
                    value: value.value,
                    label: value.label,
                    help: value.help,
                    distinct_roots: o.distinct_roots,
                    selected: o.selected,
                }
            })
            .collect(),
    })
}
pub(crate) fn vocabulary(
    r: &AtlasRetrievalService,
    values: atlas_domain::QueryVocabulary,
) -> AppServiceResult<FilterVocabularyView> {
    let catalog = r.discover_filter_fields()?;
    let field = catalog.fields.iter().find(|f| f.id == values.field);
    let namespace = field.and_then(namespace);
    let context = HelpContext::load(r, keys(namespace, values.options.iter().map(|o| &o.value)))?;
    let mut actions = action_choices(
        r,
        &values.field,
        values
            .options
            .iter()
            .map(|o| &o.value)
            .chain(&values.selected_values),
    )?;
    let mut options = values
        .options
        .into_iter()
        .map(|o| {
            if let QueryLiteral::String(key) = &o.value
                && let Some(choice) = actions.remove(key)
            {
                return choice;
            }
            let label = field.and_then(|field| fallback_label(field, &o.value));
            decorate(&context, namespace, o.value, label)
        })
        .collect::<Vec<_>>();
    // Existing Actions selections need readable labels even outside this page or scope.
    options.extend(
        values
            .selected_values
            .iter()
            .filter_map(|value| match value {
                QueryLiteral::String(key) => actions.remove(key),
                _ => None,
            }),
    );
    Ok(FilterVocabularyView {
        field: values.field.clone(),
        selected_values: values.selected_values,
        total_values: values.total_values,
        basis: values.basis,
        options,
    })
}
pub(crate) fn editor(
    r: &AtlasRetrievalService,
    editor: &mut FilterEditorView,
) -> AppServiceResult<()> {
    let mut all_keys = BTreeSet::new();
    for field in editor.groups.iter().flat_map(|g| &g.fields) {
        all_keys.extend(keys(
            namespace(&field.definition),
            field.choices.iter().map(|o| &o.value),
        ));
    }
    let context = HelpContext::load(r, all_keys)?;
    for field in editor.groups.iter_mut().flat_map(|g| &mut g.fields) {
        let namespace = namespace(&field.definition);
        for option in &mut field.choices {
            *option = decorate(
                &context,
                namespace,
                option.value.clone(),
                Some(option.label.clone()),
            );
        }
    }
    Ok(())
}
