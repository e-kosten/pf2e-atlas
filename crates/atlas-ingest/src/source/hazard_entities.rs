use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use atlas_record::{
    FactValue, HazardActionCapability, HazardActionCategory, HazardActionCount, HazardActionType,
    HazardActiveEffectLikeRule, HazardAuraRule, HazardCapability, HazardConditionCapability,
    HazardDamageCategory, HazardDamageDiceRule, HazardDiagnosticCode, HazardEffectCapability,
    HazardEmbeddedEntities, HazardEntity, HazardEntityFamily, HazardEntityId,
    HazardEntityOccurrence, HazardEntitySourceIdentity, HazardExpectedShape, HazardFact,
    HazardFlatModifierRule, HazardFrequency, HazardFrequencyInterval, HazardImmunityRule,
    HazardItemCommon, HazardNoteRule, HazardOccurrenceId, HazardOccurrenceIdentityStability,
    HazardPublication, HazardRelationship, HazardRelationshipId, HazardRelationshipKind,
    HazardRelationshipTarget, HazardRuleElement, HazardRuleMode, HazardRuleType, HazardSelfEffect,
    HazardSourceId, HazardSourceShape, HazardSourceValue, HazardStrikeCapability,
    HazardStrikeDamage, HazardTrait, HazardUnsupportedChildCapability, HazardUnsupportedFact,
    HazardUnsupportedField, HazardUnsupportedOwner, HazardUnsupportedRule, HazardUnsupportedValue,
    PublicationLicense,
};
use serde_json::{Map, Value};

use super::dto::{
    HazardDamageSource, HazardFrequencySource, HazardItemSource, HazardSelfEffectSource,
    HazardSourceField, HazardSourceValue as DtoValue, SourcePresence, ValueSummary,
    VersionedHazardSource,
};
use super::normalize::{LocalizationResolver, parse_foundry_content_with_localization};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardEmbeddedConversion {
    pub(crate) embedded: HazardFact<HazardEmbeddedEntities>,
    pub(crate) relationships: Vec<HazardRelationship>,
    pub(crate) diagnostics: Vec<HazardUnsupportedFact>,
    pub(crate) resolved_identities: Vec<HazardResolvedItemIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HazardResolvedItemIdentity {
    pub(crate) source_ordinal: u32,
    pub(crate) entity_id: HazardEntityId,
    pub(crate) occurrence_id: HazardOccurrenceId,
    pub(crate) source_identity: HazardEntitySourceIdentity,
    pub(crate) stability: HazardOccurrenceIdentityStability,
    pub(crate) stable_sort_id: Option<String>,
}

pub(crate) fn convert_hazard_embedded_entities(
    owner_record_key: &RecordKey,
    source: &VersionedHazardSource,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardEmbeddedConversion {
    let items = match &source.source.items {
        SourcePresence::Missing => {
            return empty_conversion(FactValue::Missing);
        }
        SourcePresence::Null => {
            return empty_conversion(FactValue::Null);
        }
        SourcePresence::Value(DtoValue::Unsupported(value)) => {
            return HazardEmbeddedConversion {
                embedded: HazardFact::source(
                    FactValue::Value(HazardSourceValue::Unsupported(summary_value(
                        value,
                        HazardExpectedShape::Array,
                        HazardUnsupportedOwner::Record(owner_record_key.clone()),
                        HazardDiagnosticCode::UnexpectedShape,
                    ))),
                    "/items",
                ),
                relationships: Vec::new(),
                diagnostics: Vec::new(),
                resolved_identities: Vec::new(),
            };
        }
        SourcePresence::Value(DtoValue::Typed(items)) => items,
    };

    let (resolved_identities, mut diagnostics) = resolve_identities(owner_record_key, items);
    let mut ordered = items
        .iter()
        .map(|item| {
            let identity = &resolved_identities[item.source_ordinal as usize];
            (item, identity)
        })
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        sort_value(&left.0.sort)
            .cmp(&sort_value(&right.0.sort))
            .then_with(|| {
                left.1
                    .stable_sort_id
                    .is_none()
                    .cmp(&right.1.stable_sort_id.is_none())
            })
            .then_with(|| left.1.stable_sort_id.cmp(&right.1.stable_sort_id))
            .then_with(|| left.0.source_ordinal.cmp(&right.0.source_ordinal))
    });

    let mut entities = Vec::with_capacity(items.len());
    let mut occurrences = Vec::with_capacity(items.len());
    let mut relationships = Vec::with_capacity(items.len());
    for (authored_order, (item, identity)) in ordered.into_iter().enumerate() {
        let family = family(item);
        let entity_id = identity.entity_id.clone();
        let occurrence_id = identity.occurrence_id.clone();
        let entity_owner = HazardUnsupportedOwner::Entity(entity_id.clone());
        let label =
            typed_string(&item.name).unwrap_or_else(|| format!("Unnamed {}", family.as_str()));
        let capability = capability(item, family, &entity_owner, localization, &mut diagnostics);
        entities.push(HazardEntity {
            id: entity_id.clone(),
            family,
            label,
            image: fact_clone(
                &item.image,
                &format!("/items/{}/img", item.source_ordinal),
                HazardExpectedShape::String,
                entity_owner.clone(),
            ),
            source_identity: identity.source_identity.clone(),
            capability,
        });
        occurrences.push(HazardEntityOccurrence {
            id: occurrence_id.clone(),
            owner_record_key: owner_record_key.clone(),
            entity_id: entity_id.clone(),
            family,
            authored_order: authored_order as u32,
            source_sort: fact_clone(
                &item.sort,
                &format!("/items/{}/sort", item.source_ordinal),
                HazardExpectedShape::Integer,
                HazardUnsupportedOwner::Occurrence(occurrence_id.clone()),
            ),
            source_folder: fact_clone(
                &item.folder,
                &format!("/items/{}/folder", item.source_ordinal),
                HazardExpectedShape::String,
                HazardUnsupportedOwner::Occurrence(occurrence_id.clone()),
            ),
            source_ordinal: item.source_ordinal,
            contextual_label: fact_clone(
                &item.name,
                &format!("/items/{}/name", item.source_ordinal),
                HazardExpectedShape::String,
                HazardUnsupportedOwner::Occurrence(occurrence_id.clone()),
            ),
            identity_stability: identity.stability,
        });
        relationships.push(HazardRelationship {
            id: HazardRelationshipId::new(format!("contains-{}", occurrence_id.as_str()))
                .expect("derived relationship id is valid"),
            authored_order: authored_order as u32,
            source_occurrence_id: Some(occurrence_id.clone()),
            kind: HazardRelationshipKind::Contains,
            target: HazardRelationshipTarget::Occurrence(occurrence_id),
        });
    }

    HazardEmbeddedConversion {
        embedded: HazardFact::source(
            FactValue::Value(HazardSourceValue::Typed(HazardEmbeddedEntities {
                entities,
                occurrences,
            })),
            "/items",
        ),
        relationships,
        diagnostics,
        resolved_identities,
    }
}

fn empty_conversion(
    value: FactValue<HazardSourceValue<HazardEmbeddedEntities>>,
) -> HazardEmbeddedConversion {
    HazardEmbeddedConversion {
        embedded: HazardFact::source(value, "/items"),
        relationships: Vec::new(),
        diagnostics: Vec::new(),
        resolved_identities: Vec::new(),
    }
}

pub(crate) fn family(item: &HazardItemSource) -> HazardEntityFamily {
    match typed_string(&item.item_type).as_deref() {
        Some("action") => HazardEntityFamily::Action,
        Some("melee") => HazardEntityFamily::Strike,
        Some("condition") => HazardEntityFamily::Condition,
        Some("effect") => HazardEntityFamily::Effect,
        _ => HazardEntityFamily::UnsupportedChild,
    }
}

fn sort_value(value: &HazardSourceField<i64>) -> (u8, i64) {
    match value {
        SourcePresence::Value(DtoValue::Typed(value)) => (0, *value),
        _ => (1, 0),
    }
}

fn resolve_identities(
    record_key: &RecordKey,
    items: &[HazardItemSource],
) -> (Vec<HazardResolvedItemIdentity>, Vec<HazardUnsupportedFact>) {
    let mut valid_counts = BTreeMap::<String, usize>::new();
    for item in items {
        if let Some(value) = valid_source_id(&item.id) {
            *valid_counts.entry(value).or_default() += 1;
        }
    }

    let mut diagnostics = Vec::new();
    let identities = items
        .iter()
        .map(|item| {
            let family = family(item);
            if let Some(value) = valid_source_id(&item.id)
                && valid_counts.get(&value) == Some(&1)
            {
                return HazardResolvedItemIdentity {
                    source_ordinal: item.source_ordinal,
                    entity_id: HazardEntityId::new(value.clone()).expect("validated entity id"),
                    occurrence_id: HazardOccurrenceId::new(value.clone())
                        .expect("validated occurrence id"),
                    source_identity: HazardEntitySourceIdentity::Stable {
                        source_id: HazardSourceId::new(value.clone()).expect("validated source id"),
                    },
                    stability: HazardOccurrenceIdentityStability::StableSourceIdentity,
                    stable_sort_id: Some(value),
                };
            }

            let locator = format!("{}:{}:{}", record_key, family.as_str(), item.source_ordinal);
            let fallback = format!("fallback-{}", item.source_ordinal);
            let entity_id = HazardEntityId::new(fallback.clone()).expect("fallback entity id");
            let occurrence_id = HazardOccurrenceId::new(fallback).expect("fallback occurrence id");
            let value = field_unsupported(
                &item.id,
                &format!("/items/{}/_id", item.source_ordinal),
                HazardExpectedShape::String,
                HazardUnsupportedOwner::Entity(entity_id.clone()),
                HazardDiagnosticCode::UnstableIdentity,
                |value| json_string(value),
            );
            diagnostics.push(HazardUnsupportedFact {
                field: HazardUnsupportedField::UnsupportedChildField("_id".to_string()),
                value: value.clone(),
            });
            HazardResolvedItemIdentity {
                source_ordinal: item.source_ordinal,
                entity_id,
                occurrence_id,
                source_identity: HazardEntitySourceIdentity::Fallback {
                    locator,
                    diagnostic: value,
                },
                stability: HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal,
                stable_sort_id: None,
            }
        })
        .collect();
    (identities, diagnostics)
}

fn valid_source_id(value: &HazardSourceField<String>) -> Option<String> {
    let value = typed_string(value)?;
    HazardSourceId::new(value.clone()).ok()?;
    HazardEntityId::new(value.clone()).ok()?;
    HazardOccurrenceId::new(value.clone()).ok()?;
    Some(value)
}

fn capability(
    item: &HazardItemSource,
    family: HazardEntityFamily,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
    diagnostics: &mut Vec<HazardUnsupportedFact>,
) -> HazardCapability {
    let common = convert_common(item, owner, localization);
    let mut unsupported = item
        .unclaimed
        .iter()
        .map(|value| unsupported_unclaimed(value, family, owner.clone()))
        .collect::<Vec<_>>();
    let capability = match family {
        HazardEntityFamily::Action => convert_action(item, common, owner, &unsupported),
        HazardEntityFamily::Strike => convert_strike(item, common, owner, &mut unsupported),
        HazardEntityFamily::Condition => {
            HazardCapability::Condition(Box::new(HazardConditionCapability {
                common,
                unsupported_fields: unsupported.clone(),
            }))
        }
        HazardEntityFamily::Effect => HazardCapability::Effect(Box::new(HazardEffectCapability {
            common,
            unsupported_fields: unsupported.clone(),
        })),
        HazardEntityFamily::UnsupportedChild => {
            let child_type = typed_string(&item.item_type).unwrap_or_else(|| "unknown".to_string());
            HazardCapability::UnsupportedChild(Box::new(HazardUnsupportedChildCapability {
                child_type,
                common,
                unsupported_fields: unsupported.clone(),
            }))
        }
    };
    diagnostics.extend(unsupported);
    capability
}

fn convert_common(
    item: &HazardItemSource,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardItemCommon {
    let ordinal = item.source_ordinal;
    let common = &item.common;
    HazardItemCommon {
        description: fact_map(
            &common.description,
            &format!("/items/{ordinal}/system/description/value"),
            HazardExpectedShape::RichDocument,
            owner.clone(),
            |value| Ok(parse_foundry_content_with_localization(value, localization).document),
            |value| json_string(value),
        ),
        publication: fact_map(
            &common.publication,
            &format!("/items/{ordinal}/system/publication"),
            HazardExpectedShape::Object,
            owner.clone(),
            |publication| {
                Ok(HazardPublication {
                    title: fact_clone(
                        &publication.title,
                        &format!("/items/{ordinal}/system/publication/title"),
                        HazardExpectedShape::String,
                        owner.clone(),
                    ),
                    remaster: fact_clone(
                        &publication.remaster,
                        &format!("/items/{ordinal}/system/publication/remaster"),
                        HazardExpectedShape::Boolean,
                        owner.clone(),
                    ),
                    license: fact_map(
                        &publication.license,
                        &format!("/items/{ordinal}/system/publication/license"),
                        HazardExpectedShape::String,
                        owner.clone(),
                        |value| {
                            PublicationLicense::new(value.clone())
                                .map_err(|_| HazardDiagnosticCode::InvalidCanonicalValue)
                        },
                        |value| json_string(value),
                    ),
                })
            },
            |_| "{}".to_string(),
        ),
        rules: fact_map(
            &common.rules,
            &format!("/items/{ordinal}/system/rules"),
            HazardExpectedShape::Array,
            owner.clone(),
            |values| {
                Ok(values
                    .iter()
                    .map(|value| convert_rule(value, owner, localization))
                    .collect())
            },
            |_| "[]".to_string(),
        ),
        slug: fact_clone(
            &common.slug,
            &format!("/items/{ordinal}/system/slug"),
            HazardExpectedShape::String,
            owner.clone(),
        ),
        traits: fact_map(
            &common.traits,
            &format!("/items/{ordinal}/system/traits/value"),
            HazardExpectedShape::Array,
            owner.clone(),
            |values| {
                values
                    .iter()
                    .map(|value| {
                        HazardTrait::new(value.clone())
                            .map_err(|_| HazardDiagnosticCode::InvalidCanonicalValue)
                    })
                    .collect()
            },
            |values| serde_json::to_string(values).expect("traits serialize"),
        ),
    }
}

fn convert_action(
    item: &HazardItemSource,
    common: HazardItemCommon,
    owner: &HazardUnsupportedOwner,
    unsupported: &[HazardUnsupportedFact],
) -> HazardCapability {
    let ordinal = item.source_ordinal;
    let source = &item.action;
    let action_type = closed_fact(
        &source.action_type,
        &format!("/items/{ordinal}/system/actionType/value"),
        owner.clone(),
        |value| match value {
            "action" => Some(HazardActionType::Action),
            "reaction" => Some(HazardActionType::Reaction),
            "free" => Some(HazardActionType::Free),
            "passive" => Some(HazardActionType::Passive),
            _ => None,
        },
    );
    let actions = fact_map(
        &source.actions,
        &format!("/items/{ordinal}/system/actions/value"),
        HazardExpectedShape::ClosedVocabulary,
        owner.clone(),
        |value| match value {
            1 => Ok(HazardActionCount::One),
            2 => Ok(HazardActionCount::Two),
            3 => Ok(HazardActionCount::Three),
            _ => Err(HazardDiagnosticCode::UnsupportedValue),
        },
        ToString::to_string,
    );
    let category = closed_fact(
        &source.category,
        &format!("/items/{ordinal}/system/category"),
        owner.clone(),
        |value| match value {
            "interaction" => Some(HazardActionCategory::Interaction),
            "defensive" => Some(HazardActionCategory::Defensive),
            "offensive" => Some(HazardActionCategory::Offensive),
            "familiar" => Some(HazardActionCategory::Familiar),
            _ => None,
        },
    );
    HazardCapability::Action(Box::new(HazardActionCapability {
        common,
        action_type,
        actions,
        category,
        death_note: fact_clone(
            &source.death_note,
            &format!("/items/{ordinal}/system/deathNote"),
            HazardExpectedShape::Boolean,
            owner.clone(),
        ),
        frequency: convert_frequency(&source.frequency, ordinal, owner),
        self_effect: convert_self_effect(&source.self_effect, ordinal, owner),
        unsupported_fields: unsupported.to_vec(),
    }))
}

fn convert_strike(
    item: &HazardItemSource,
    common: HazardItemCommon,
    owner: &HazardUnsupportedOwner,
    unsupported: &mut Vec<HazardUnsupportedFact>,
) -> HazardCapability {
    let ordinal = item.source_ordinal;
    let source = &item.strike;
    HazardCapability::Strike(Box::new(HazardStrikeCapability {
        common,
        bonus: fact_clone(
            &source.bonus,
            &format!("/items/{ordinal}/system/bonus/value"),
            HazardExpectedShape::Integer,
            owner.clone(),
        ),
        attack_effects: fact_clone(
            &source.attack_effects,
            &format!("/items/{ordinal}/system/attackEffects/value"),
            HazardExpectedShape::Array,
            owner.clone(),
        ),
        damage_rolls: fact_map(
            &source.damage_rolls,
            &format!("/items/{ordinal}/system/damageRolls"),
            HazardExpectedShape::Object,
            owner.clone(),
            |values| {
                Ok(values
                    .iter()
                    .map(|value| convert_damage(value, owner, unsupported))
                    .collect())
            },
            |_| "{}".to_string(),
        ),
        unsupported_fields: unsupported.clone(),
    }))
}

fn convert_damage(
    source: &HazardDamageSource,
    owner: &HazardUnsupportedOwner,
    unsupported: &mut Vec<HazardUnsupportedFact>,
) -> HazardStrikeDamage {
    for value in &source.unclaimed {
        unsupported.push(HazardUnsupportedFact {
            field: HazardUnsupportedField::StrikeUnexpected(value.source_path.clone()),
            value: summary_value(
                value,
                HazardExpectedShape::Object,
                owner.clone(),
                HazardDiagnosticCode::UnsupportedValue,
            ),
        });
    }
    HazardStrikeDamage {
        source_key: source.source_key.clone(),
        authored_order: source.authored_order,
        damage: fact_clone(
            &source.damage,
            &format!("{}/damage", damage_path(source)),
            HazardExpectedShape::String,
            owner.clone(),
        ),
        damage_type: fact_clone(
            &source.damage_type,
            &format!("{}/damageType", damage_path(source)),
            HazardExpectedShape::String,
            owner.clone(),
        ),
        category: closed_fact(
            &source.category,
            &format!("{}/category", damage_path(source)),
            owner.clone(),
            |value| match value {
                "persistent" => Some(HazardDamageCategory::Persistent),
                "precision" => Some(HazardDamageCategory::Precision),
                "splash" => Some(HazardDamageCategory::Splash),
                _ => None,
            },
        ),
    }
}

fn damage_path(source: &HazardDamageSource) -> String {
    source.source_path.clone()
}

fn convert_frequency(
    source: &HazardSourceField<HazardFrequencySource>,
    ordinal: u32,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<HazardFrequency> {
    fact_map(
        source,
        &format!("/items/{ordinal}/system/frequency"),
        HazardExpectedShape::Object,
        owner.clone(),
        |source| {
            Ok(HazardFrequency {
                value: fact_clone(
                    &source.value,
                    &format!("/items/{ordinal}/system/frequency/value"),
                    HazardExpectedShape::Integer,
                    owner.clone(),
                ),
                maximum: fact_clone(
                    &source.maximum,
                    &format!("/items/{ordinal}/system/frequency/max"),
                    HazardExpectedShape::Integer,
                    owner.clone(),
                ),
                per: closed_fact(
                    &source.per,
                    &format!("/items/{ordinal}/system/frequency/per"),
                    owner.clone(),
                    frequency_interval,
                ),
            })
        },
        |_| "{}".to_string(),
    )
}

fn frequency_interval(value: &str) -> Option<HazardFrequencyInterval> {
    match value {
        "turn" => Some(HazardFrequencyInterval::Turn),
        "round" => Some(HazardFrequencyInterval::Round),
        "PT1M" => Some(HazardFrequencyInterval::OneMinute),
        "PT10M" => Some(HazardFrequencyInterval::TenMinutes),
        "PT1H" => Some(HazardFrequencyInterval::OneHour),
        "PT24H" => Some(HazardFrequencyInterval::TwentyFourHours),
        "day" => Some(HazardFrequencyInterval::Day),
        "P1W" => Some(HazardFrequencyInterval::Week),
        "P1M" => Some(HazardFrequencyInterval::Month),
        "P1Y" => Some(HazardFrequencyInterval::Year),
        _ => None,
    }
}

fn convert_self_effect(
    source: &HazardSourceField<HazardSelfEffectSource>,
    ordinal: u32,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<HazardSelfEffect> {
    fact_map(
        source,
        &format!("/items/{ordinal}/system/selfEffect"),
        HazardExpectedShape::Object,
        owner.clone(),
        |source| {
            Ok(HazardSelfEffect {
                target_uuid: fact_clone(
                    &source.uuid,
                    &format!("/items/{ordinal}/system/selfEffect/uuid"),
                    HazardExpectedShape::String,
                    owner.clone(),
                ),
                label: fact_clone(
                    &source.name,
                    &format!("/items/{ordinal}/system/selfEffect/name"),
                    HazardExpectedShape::String,
                    owner.clone(),
                ),
            })
        },
        |_| "{}".to_string(),
    )
}

fn convert_rule(
    source: &ValueSummary,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardRuleElement {
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&source.value) else {
        return unsupported_rule(source, owner);
    };
    let Some(key) = map.get("key").and_then(Value::as_str) else {
        return unsupported_rule(source, owner);
    };
    let order = source
        .source_path
        .rsplit('/')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    match key {
        "Immunity" if only_keys(&map, &["key", "mode", "type"]) => {
            HazardRuleElement::Immunity(HazardImmunityRule {
                authored_order: order,
                mode: json_mode_fact(&map, "mode", &source.source_path, owner),
                immunity_types: json_rule_type_fact(&map, "type", &source.source_path, owner),
            })
        }
        "ActiveEffectLike" if only_keys(&map, &["key", "mode", "path", "value"]) => {
            HazardRuleElement::ActiveEffectLike(HazardActiveEffectLikeRule {
                authored_order: order,
                mode: json_mode_fact(&map, "mode", &source.source_path, owner),
                path: json_string_fact(&map, "path", &source.source_path, owner),
                value: json_bool_fact(&map, "value", &source.source_path, owner),
            })
        }
        "Aura" if only_keys(&map, &["key", "radius", "slug", "traits"]) => {
            HazardRuleElement::Aura(HazardAuraRule {
                authored_order: order,
                radius: json_integer_fact(&map, "radius", &source.source_path, owner),
                slug: json_string_fact(&map, "slug", &source.source_path, owner),
                traits: json_traits_fact(&map, "traits", &source.source_path, owner),
            })
        }
        "DamageDice"
            if only_keys(
                &map,
                &[
                    "key",
                    "critical",
                    "diceNumber",
                    "dieSize",
                    "damageType",
                    "selector",
                ],
            ) =>
        {
            HazardRuleElement::DamageDice(HazardDamageDiceRule {
                authored_order: order,
                critical: json_bool_fact(&map, "critical", &source.source_path, owner),
                dice_number: json_integer_fact(&map, "diceNumber", &source.source_path, owner),
                die_size: json_string_fact(&map, "dieSize", &source.source_path, owner),
                damage_type: json_string_fact(&map, "damageType", &source.source_path, owner),
                selector: json_string_fact(&map, "selector", &source.source_path, owner),
            })
        }
        "FlatModifier"
            if only_keys(
                &map,
                &["key", "critical", "damageType", "selector", "value"],
            ) =>
        {
            HazardRuleElement::FlatModifier(HazardFlatModifierRule {
                authored_order: order,
                critical: json_bool_fact(&map, "critical", &source.source_path, owner),
                damage_type: json_string_fact(&map, "damageType", &source.source_path, owner),
                selector: json_string_fact(&map, "selector", &source.source_path, owner),
                value: json_integer_fact(&map, "value", &source.source_path, owner),
            })
        }
        "Note"
            if only_keys(
                &map,
                &["key", "outcome", "selector", "text", "title", "visibility"],
            ) =>
        {
            HazardRuleElement::Note(HazardNoteRule {
                authored_order: order,
                outcomes: json_strings_fact(&map, "outcome", &source.source_path, owner),
                selector: json_string_fact(&map, "selector", &source.source_path, owner),
                text: json_rich_fact(&map, "text", &source.source_path, owner, localization),
                title: json_string_fact(&map, "title", &source.source_path, owner),
                visibility: json_string_fact(&map, "visibility", &source.source_path, owner),
            })
        }
        _ => unsupported_rule(source, owner),
    }
}

fn only_keys(map: &Map<String, Value>, allowed: &[&str]) -> bool {
    map.keys().all(|key| allowed.contains(&key.as_str()))
}

fn unsupported_rule(source: &ValueSummary, owner: &HazardUnsupportedOwner) -> HazardRuleElement {
    let order = source
        .source_path
        .rsplit('/')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    HazardRuleElement::Unsupported(HazardUnsupportedRule {
        authored_order: order,
        source: summary_value(
            source,
            HazardExpectedShape::Object,
            owner.clone(),
            HazardDiagnosticCode::UnsupportedRuleElement,
        ),
    })
}

fn json_mode_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<HazardRuleMode> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::ClosedVocabulary,
        owner,
        |value| {
            value.as_str().and_then(|value| match value {
                "add" => Some(HazardRuleMode::Add),
                "remove" => Some(HazardRuleMode::Remove),
                "override" => Some(HazardRuleMode::Override),
                _ => None,
            })
        },
    )
}
fn json_rule_type_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<HazardRuleType> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::StringOrArray,
        owner,
        |value| {
            value
                .as_str()
                .map(|v| HazardRuleType::Single(v.to_string()))
                .or_else(|| {
                    value.as_array().and_then(|values| {
                        values
                            .iter()
                            .map(Value::as_str)
                            .map(|v| v.map(str::to_string))
                            .collect::<Option<Vec<_>>>()
                            .map(HazardRuleType::Multiple)
                    })
                })
        },
    )
}
fn json_string_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<String> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::String,
        owner,
        |value| value.as_str().map(str::to_string),
    )
}
fn json_bool_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<bool> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::Boolean,
        owner,
        Value::as_bool,
    )
}
fn json_integer_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<i64> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::Integer,
        owner,
        Value::as_i64,
    )
}
fn json_strings_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<Vec<String>> {
    json_fact(map, key, base, HazardExpectedShape::Array, owner, |value| {
        value.as_array().and_then(|values| {
            values
                .iter()
                .map(Value::as_str)
                .map(|v| v.map(str::to_string))
                .collect()
        })
    })
}
fn json_traits_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
) -> HazardFact<Vec<HazardTrait>> {
    json_fact(map, key, base, HazardExpectedShape::Array, owner, |value| {
        value.as_array().and_then(|values| {
            values
                .iter()
                .map(Value::as_str)
                .map(|v| v.and_then(|v| HazardTrait::new(v.to_string()).ok()))
                .collect()
        })
    })
}
fn json_rich_fact(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    owner: &HazardUnsupportedOwner,
    localization: Option<&dyn LocalizationResolver>,
) -> HazardFact<atlas_record::RichDocument> {
    json_fact(
        map,
        key,
        base,
        HazardExpectedShape::RichDocument,
        owner,
        |value| {
            value
                .as_str()
                .map(|value| parse_foundry_content_with_localization(value, localization).document)
        },
    )
}

fn json_fact<T>(
    map: &Map<String, Value>,
    key: &str,
    base: &str,
    expected: HazardExpectedShape,
    owner: &HazardUnsupportedOwner,
    convert: impl FnOnce(&Value) -> Option<T>,
) -> HazardFact<T> {
    let path = format!("{base}/{key}");
    match map.get(key) {
        None => HazardFact::source(FactValue::Missing, path),
        Some(Value::Null) => HazardFact::source(FactValue::Null, path),
        Some(value) => {
            HazardFact::source(
                FactValue::Value(convert(value).map(HazardSourceValue::Typed).unwrap_or_else(
                    || {
                        HazardSourceValue::Unsupported(HazardUnsupportedValue {
                            exact_json: serde_json::to_string(value).expect("JSON serializes"),
                            expected_shape: expected,
                            actual_shape: value_shape(value),
                            relative_source_path: path.clone(),
                            owner: owner.clone(),
                            diagnostic_code: HazardDiagnosticCode::UnexpectedShape,
                        })
                    },
                )),
                path,
            )
        }
    }
}

fn unsupported_unclaimed(
    value: &ValueSummary,
    family: HazardEntityFamily,
    owner: HazardUnsupportedOwner,
) -> HazardUnsupportedFact {
    let field = match family {
        HazardEntityFamily::Action => {
            HazardUnsupportedField::ActionUnexpected(value.source_path.clone())
        }
        HazardEntityFamily::Strike if value.source_path.ends_with("/system/attack/value") => {
            HazardUnsupportedField::StrikeAttack
        }
        HazardEntityFamily::Strike if value.source_path.ends_with("/system/weaponType/value") => {
            HazardUnsupportedField::StrikeWeaponType
        }
        HazardEntityFamily::Strike
            if value.source_path.ends_with("/system/attackEffects/custom") =>
        {
            HazardUnsupportedField::StrikeAttackEffectsCustom
        }
        HazardEntityFamily::Strike if value.source_path.ends_with("/system/traits/rarity") => {
            HazardUnsupportedField::StrikeTraitRarity
        }
        HazardEntityFamily::Strike => {
            HazardUnsupportedField::StrikeUnexpected(value.source_path.clone())
        }
        HazardEntityFamily::Condition => {
            HazardUnsupportedField::ConditionUnexpected(value.source_path.clone())
        }
        HazardEntityFamily::Effect => {
            HazardUnsupportedField::EffectUnexpected(value.source_path.clone())
        }
        HazardEntityFamily::UnsupportedChild => {
            HazardUnsupportedField::UnsupportedChildField(value.source_path.clone())
        }
    };
    let expected = match &field {
        HazardUnsupportedField::StrikeAttack => HazardExpectedShape::Integer,
        HazardUnsupportedField::StrikeWeaponType
        | HazardUnsupportedField::StrikeAttackEffectsCustom => HazardExpectedShape::String,
        HazardUnsupportedField::StrikeTraitRarity => HazardExpectedShape::ClosedVocabulary,
        _ => HazardExpectedShape::Any,
    };
    HazardUnsupportedFact {
        field,
        value: summary_value(
            value,
            expected,
            owner,
            HazardDiagnosticCode::UnsupportedValue,
        ),
    }
}

fn typed_string(value: &HazardSourceField<String>) -> Option<String> {
    match value {
        SourcePresence::Value(DtoValue::Typed(value)) => Some(value.clone()),
        _ => None,
    }
}

fn closed_fact<T>(
    source: &HazardSourceField<String>,
    path: &str,
    owner: HazardUnsupportedOwner,
    convert: impl FnOnce(&str) -> Option<T>,
) -> HazardFact<T> {
    fact_map(
        source,
        path,
        HazardExpectedShape::ClosedVocabulary,
        owner,
        |value| convert(value).ok_or(HazardDiagnosticCode::UnsupportedValue),
        |value| json_string(value),
    )
}

fn fact_clone<T: Clone>(
    source: &HazardSourceField<T>,
    path: &str,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
) -> HazardFact<T> {
    fact_map(
        source,
        path,
        expected,
        owner,
        |value| Ok(value.clone()),
        |_| "null".to_string(),
    )
}

fn fact_map<T, U>(
    source: &HazardSourceField<T>,
    path: &str,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    convert: impl FnOnce(&T) -> Result<U, HazardDiagnosticCode>,
    render: impl FnOnce(&T) -> String,
) -> HazardFact<U> {
    let value = match source {
        SourcePresence::Missing => FactValue::Missing,
        SourcePresence::Null => FactValue::Null,
        SourcePresence::Value(DtoValue::Unsupported(value)) => {
            FactValue::Value(HazardSourceValue::Unsupported(summary_value(
                value,
                expected,
                owner,
                HazardDiagnosticCode::UnexpectedShape,
            )))
        }
        SourcePresence::Value(DtoValue::Typed(value)) => match convert(value) {
            Ok(value) => FactValue::Value(HazardSourceValue::Typed(value)),
            Err(code) => FactValue::Value(HazardSourceValue::Unsupported(HazardUnsupportedValue {
                exact_json: render(value),
                expected_shape: expected,
                actual_shape: shape_for_expected(expected),
                relative_source_path: path.to_string(),
                owner,
                diagnostic_code: code,
            })),
        },
    };
    HazardFact::source(value, path)
}

fn field_unsupported<T>(
    source: &HazardSourceField<T>,
    path: &str,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    code: HazardDiagnosticCode,
    render: impl FnOnce(&T) -> String,
) -> HazardUnsupportedValue {
    match source {
        SourcePresence::Missing => HazardUnsupportedValue {
            exact_json: "null".to_string(),
            expected_shape: expected,
            actual_shape: HazardSourceShape::Missing,
            relative_source_path: path.to_string(),
            owner,
            diagnostic_code: code,
        },
        SourcePresence::Null => HazardUnsupportedValue {
            exact_json: "null".to_string(),
            expected_shape: expected,
            actual_shape: HazardSourceShape::Null,
            relative_source_path: path.to_string(),
            owner,
            diagnostic_code: code,
        },
        SourcePresence::Value(DtoValue::Typed(value)) => HazardUnsupportedValue {
            exact_json: render(value),
            expected_shape: expected,
            actual_shape: shape_for_expected(expected),
            relative_source_path: path.to_string(),
            owner,
            diagnostic_code: code,
        },
        SourcePresence::Value(DtoValue::Unsupported(value)) => {
            summary_value(value, expected, owner, code)
        }
    }
}

fn summary_value(
    value: &ValueSummary,
    expected: HazardExpectedShape,
    owner: HazardUnsupportedOwner,
    code: HazardDiagnosticCode,
) -> HazardUnsupportedValue {
    HazardUnsupportedValue {
        exact_json: value.value.clone(),
        expected_shape: expected,
        actual_shape: summary_shape(value),
        relative_source_path: value.source_path.clone(),
        owner,
        diagnostic_code: code,
    }
}
fn summary_shape(value: &ValueSummary) -> HazardSourceShape {
    match value
        .shape
        .split_whitespace()
        .last()
        .unwrap_or(&value.shape)
    {
        "missing" => HazardSourceShape::Missing,
        "null" => HazardSourceShape::Null,
        "boolean" => HazardSourceShape::Boolean,
        "integer" | "number" => HazardSourceShape::Number,
        "string" => HazardSourceShape::String,
        "array" | "strings" => HazardSourceShape::Array,
        _ => HazardSourceShape::Object,
    }
}
fn value_shape(value: &Value) -> HazardSourceShape {
    match value {
        Value::Null => HazardSourceShape::Null,
        Value::Bool(_) => HazardSourceShape::Boolean,
        Value::Number(_) => HazardSourceShape::Number,
        Value::String(_) => HazardSourceShape::String,
        Value::Array(_) => HazardSourceShape::Array,
        Value::Object(_) => HazardSourceShape::Object,
    }
}
fn shape_for_expected(value: HazardExpectedShape) -> HazardSourceShape {
    match value {
        HazardExpectedShape::Any => HazardSourceShape::Object,
        HazardExpectedShape::Boolean => HazardSourceShape::Boolean,
        HazardExpectedShape::Integer => HazardSourceShape::Number,
        HazardExpectedShape::String
        | HazardExpectedShape::StringOrBoolean
        | HazardExpectedShape::StringOrArray
        | HazardExpectedShape::ClosedVocabulary
        | HazardExpectedShape::RichDocument => HazardSourceShape::String,
        HazardExpectedShape::Array => HazardSourceShape::Array,
        HazardExpectedShape::Object => HazardSourceShape::Object,
    }
}
fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serializes")
}
