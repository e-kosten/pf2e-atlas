use serde::Serialize;

use crate::{
    ContentIdentityStability, ContentOwner, ContentRole, DuplicateContentStatus, FactValue,
    HazardCapability, HazardEntitySourceIdentity, HazardFact, HazardItemCommon,
    HazardOccurrenceIdentityStability, HazardRecord, HazardRuleElement, HazardSourceValue,
    HazardUnsupportedFact, HazardUnsupportedField, HazardUnsupportedValue,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardAvailabilityStateJson {
    Missing,
    Null,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HazardAvailabilityJson {
    pub state: HazardAvailabilityStateJson,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HazardOccurrenceProvenanceJson {
    pub id: String,
    pub entity_id: String,
    pub family: &'static str,
    pub authored_order: u32,
    pub source_ordinal: u32,
    pub identity_stability: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HazardContentProvenanceJson {
    pub content_key: String,
    pub owner: String,
    pub role: &'static str,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
    pub visibility: &'static str,
    pub identity_stability: &'static str,
    pub source_path: String,
    pub field_or_pointer_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested_source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_from_record_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HazardProvenanceJson {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub convenience_rule_id: &'static str,
    pub convenience_rule_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_license: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub occurrences: Vec<HazardOccurrenceProvenanceJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<HazardContentProvenanceJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_rules: Vec<HazardUnsupportedValue>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identity_diagnostics: Vec<HazardUnsupportedValue>,
}

pub(super) fn availability(hazard: &HazardRecord) -> Vec<HazardAvailabilityJson> {
    let mut values = Vec::new();
    for (field, state) in [
        ("level", fact_state(&hazard.level)),
        ("rarity", fact_state(&hazard.rarity)),
        ("traits", fact_state(&hazard.traits)),
        ("size", fact_state(&hazard.size)),
        ("publication", fact_state(&hazard.publication)),
        ("complexity", fact_state(&hazard.complexity)),
        ("detection", fact_state(&hazard.detection)),
        ("defenses", fact_state(&hazard.defenses)),
        ("lifecycle", fact_state(&hazard.lifecycle)),
        ("emits_sound", fact_state(&hazard.emits_sound)),
        ("embedded_entities", fact_state(&hazard.embedded_entities)),
    ] {
        push_availability(&mut values, field, None, state);
    }

    if let Some(detection) = hazard.detection.typed() {
        push_availability(
            &mut values,
            "detection.stealth_modifier",
            None,
            fact_state(&detection.stealth_modifier),
        );
        push_availability(
            &mut values,
            "detection.details",
            None,
            fact_state(&detection.details),
        );
    }
    if let Some(defenses) = hazard.defenses.typed() {
        for (field, state) in [
            ("defenses.armor_class", fact_state(&defenses.armor_class)),
            ("defenses.hardness", fact_state(&defenses.hardness)),
            ("defenses.hit_points", fact_state(&defenses.hit_points)),
            ("defenses.saves", fact_state(&defenses.saves)),
            ("defenses.immunities", fact_state(&defenses.immunities)),
            ("defenses.weaknesses", fact_state(&defenses.weaknesses)),
            ("defenses.resistances", fact_state(&defenses.resistances)),
        ] {
            push_availability(&mut values, field, None, state);
        }
        if let Some(hit_points) = defenses.hit_points.typed() {
            for (field, state) in [
                (
                    "defenses.hit_points.current",
                    fact_state(&hit_points.current),
                ),
                (
                    "defenses.hit_points.maximum",
                    fact_state(&hit_points.maximum),
                ),
                (
                    "defenses.hit_points.temporary",
                    fact_state(&hit_points.temporary),
                ),
                (
                    "defenses.hit_points.details",
                    fact_state(&hit_points.details),
                ),
            ] {
                push_availability(&mut values, field, None, state);
            }
        }
        if let Some(saves) = defenses.saves.typed() {
            for (field, state) in [
                ("defenses.saves.fortitude", fact_state(&saves.fortitude)),
                ("defenses.saves.reflex", fact_state(&saves.reflex)),
                ("defenses.saves.will", fact_state(&saves.will)),
            ] {
                push_availability(&mut values, field, None, state);
            }
        }
    }
    if let Some(lifecycle) = hazard.lifecycle.typed() {
        for (field, state) in [
            ("lifecycle.description", fact_state(&lifecycle.description)),
            ("lifecycle.disable", fact_state(&lifecycle.disable)),
            ("lifecycle.routine", fact_state(&lifecycle.routine)),
            ("lifecycle.reset", fact_state(&lifecycle.reset)),
        ] {
            push_availability(&mut values, field, None, state);
        }
    }
    if let Some(embedded) = hazard.embedded_entities.typed() {
        for occurrence in &embedded.occurrences {
            let component_id = Some(occurrence.id.as_str().to_string());
            push_availability(
                &mut values,
                "occurrence.source_sort",
                component_id.clone(),
                fact_state(&occurrence.source_sort),
            );
            push_availability(
                &mut values,
                "occurrence.source_folder",
                component_id.clone(),
                fact_state(&occurrence.source_folder),
            );
            push_availability(
                &mut values,
                "occurrence.contextual_label",
                component_id,
                fact_state(&occurrence.contextual_label),
            );
        }
        for entity in &embedded.entities {
            let component_id = entity.id.as_str();
            if matches!(
                entity.source_identity,
                HazardEntitySourceIdentity::Fallback { .. }
            ) {
                push_availability(
                    &mut values,
                    "entity.source_identity",
                    Some(component_id.to_string()),
                    Some(HazardAvailabilityStateJson::Unsupported),
                );
            }
            let (common, unsupported) = match &entity.capability {
                HazardCapability::Action(value) => {
                    for (field, state) in [
                        ("action.action_type", fact_state(&value.action_type)),
                        ("action.actions", fact_state(&value.actions)),
                        ("action.category", fact_state(&value.category)),
                        ("action.death_note", fact_state(&value.death_note)),
                        ("action.frequency", fact_state(&value.frequency)),
                        ("action.self_effect", fact_state(&value.self_effect)),
                    ] {
                        push_availability(
                            &mut values,
                            field,
                            Some(component_id.to_string()),
                            state,
                        );
                    }
                    (&value.common, &value.unsupported_fields)
                }
                HazardCapability::Strike(value) => {
                    for (field, state) in [
                        ("strike.bonus", fact_state(&value.bonus)),
                        ("strike.attack_effects", fact_state(&value.attack_effects)),
                        ("strike.damage_rolls", fact_state(&value.damage_rolls)),
                    ] {
                        push_availability(
                            &mut values,
                            field,
                            Some(component_id.to_string()),
                            state,
                        );
                    }
                    (&value.common, &value.unsupported_fields)
                }
                HazardCapability::Condition(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::Effect(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::UnsupportedChild(value) => {
                    (&value.common, &value.unsupported_fields)
                }
            };
            collect_common_availability(&mut values, component_id, common);
            for unsupported in unsupported {
                push_availability(
                    &mut values,
                    &format!(
                        "entity.unsupported.{}",
                        unsupported_field_name(&unsupported.field)
                    ),
                    Some(component_id.to_string()),
                    Some(HazardAvailabilityStateJson::Unsupported),
                );
            }
            if common.rules.typed().is_some_and(|rules| {
                rules
                    .iter()
                    .any(|rule| matches!(rule, HazardRuleElement::Unsupported(_)))
            }) {
                push_availability(
                    &mut values,
                    "entity.rule",
                    Some(component_id.to_string()),
                    Some(HazardAvailabilityStateJson::Unsupported),
                );
            }
        }
    }
    for unsupported in &hazard.unsupported_fields {
        push_availability(
            &mut values,
            &format!(
                "hazard.unsupported.{}",
                unsupported_field_name(&unsupported.field)
            ),
            None,
            Some(HazardAvailabilityStateJson::Unsupported),
        );
    }
    values
}

pub(super) fn provenance(hazard: &HazardRecord) -> HazardProvenanceJson {
    let occurrences = hazard
        .embedded_entities
        .typed()
        .into_iter()
        .flat_map(|embedded| &embedded.occurrences)
        .map(|occurrence| HazardOccurrenceProvenanceJson {
            id: occurrence.id.as_str().to_string(),
            entity_id: occurrence.entity_id.as_str().to_string(),
            family: occurrence.family.as_str(),
            authored_order: occurrence.authored_order,
            source_ordinal: occurrence.source_ordinal,
            identity_stability: match occurrence.identity_stability {
                HazardOccurrenceIdentityStability::StableSourceIdentity => "stable_source_identity",
                HazardOccurrenceIdentityStability::UnstableAuthoredOrdinal => {
                    "unstable_authored_ordinal"
                }
            },
        })
        .collect();
    let content = hazard
        .content
        .documents
        .iter()
        .map(|document| HazardContentProvenanceJson {
            content_key: document.id.content_key.as_str().to_string(),
            owner: content_owner(&document.owner),
            role: content_role(document.role),
            authored_order: document.authored_order,
            label: document.label.clone(),
            content_hash: document.content_hash.as_str().to_string(),
            visibility: document.visibility.as_str(),
            identity_stability: match document.identity_stability {
                ContentIdentityStability::StableSourceIdentity => "stable_source_identity",
                ContentIdentityStability::UnstableAuthoredOrdinal => "unstable_authored_ordinal",
            },
            source_path: document.provenance.relative_source_path.clone(),
            field_or_pointer_family: document.provenance.field_or_pointer_family.clone(),
            nested_source_id: document.provenance.nested_source_id.clone(),
            copied_from_record_key: match &document.duplicate_status {
                DuplicateContentStatus::Unique => None,
                DuplicateContentStatus::CopiedFromCanonicalTarget { target_record_key } => {
                    Some(target_record_key.to_string())
                }
            },
        })
        .collect();
    let mut unsupported_fields = hazard.unsupported_fields.clone();
    let mut unsupported_rules = Vec::new();
    let mut identity_diagnostics = Vec::new();
    if let Some(defenses) = hazard.defenses.typed() {
        unsupported_fields.extend(defenses.unsupported_fields.iter().cloned());
        if let Some(hit_points) = defenses.hit_points.typed() {
            unsupported_fields.extend(hit_points.unsupported_fields.iter().cloned());
        }
        if let Some(saves) = defenses.saves.typed() {
            unsupported_fields.extend(saves.unsupported_fields.iter().cloned());
        }
    }
    if let Some(embedded) = hazard.embedded_entities.typed() {
        for entity in &embedded.entities {
            let (common, unsupported) = match &entity.capability {
                HazardCapability::Action(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::Strike(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::Condition(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::Effect(value) => (&value.common, &value.unsupported_fields),
                HazardCapability::UnsupportedChild(value) => {
                    (&value.common, &value.unsupported_fields)
                }
            };
            unsupported_fields.extend(unsupported.iter().cloned());
            unsupported_rules.extend(common.rules.typed().into_iter().flatten().filter_map(
                |rule| match rule {
                    HazardRuleElement::Unsupported(value) => Some(value.source.clone()),
                    _ => None,
                },
            ));
            if let HazardEntitySourceIdentity::Fallback { diagnostic, .. } = &entity.source_identity
            {
                identity_diagnostics.push(diagnostic.clone());
            }
        }
    }

    HazardProvenanceJson {
        source_path: hazard.provenance.source_path.clone(),
        source_contract_version: hazard.provenance.source_contract_version.clone(),
        source_system_version: hazard.provenance.source_system_version.clone(),
        source_upstream_commit: hazard.provenance.source_upstream_commit.clone(),
        convenience_rule_id: crate::HAZARD_CONVENIENCE_RULE_ID,
        convenience_rule_version: crate::HAZARD_CONVENIENCE_RULE_VERSION,
        publication_license: hazard
            .publication
            .typed()
            .and_then(|publication| publication.license.typed())
            .map(|license| license.as_str().to_string()),
        occurrences,
        content,
        unsupported_fields,
        unsupported_rules,
        identity_diagnostics,
    }
}

fn collect_common_availability(
    values: &mut Vec<HazardAvailabilityJson>,
    component_id: &str,
    common: &HazardItemCommon,
) {
    for (field, state) in [
        ("entity.description", fact_state(&common.description)),
        ("entity.publication", fact_state(&common.publication)),
        ("entity.rules", fact_state(&common.rules)),
        ("entity.slug", fact_state(&common.slug)),
        ("entity.traits", fact_state(&common.traits)),
    ] {
        push_availability(values, field, Some(component_id.to_string()), state);
    }
}

fn fact_state<T>(fact: &HazardFact<T>) -> Option<HazardAvailabilityStateJson> {
    match &fact.value {
        FactValue::Missing => Some(HazardAvailabilityStateJson::Missing),
        FactValue::Null => Some(HazardAvailabilityStateJson::Null),
        FactValue::Value(HazardSourceValue::Unsupported(_)) => {
            Some(HazardAvailabilityStateJson::Unsupported)
        }
        FactValue::Value(HazardSourceValue::Typed(_)) => None,
    }
}

fn push_availability(
    values: &mut Vec<HazardAvailabilityJson>,
    field: &str,
    component_id: Option<String>,
    state: Option<HazardAvailabilityStateJson>,
) {
    let Some(state) = state else {
        return;
    };
    let state_label = match state {
        HazardAvailabilityStateJson::Missing => "missing",
        HazardAvailabilityStateJson::Null => "null",
        HazardAvailabilityStateJson::Unsupported => "unsupported",
    };
    values.push(HazardAvailabilityJson {
        message: format!("{field} data is {state_label}."),
        state,
        field: field.to_string(),
        component_id,
    });
}

fn content_owner(owner: &ContentOwner) -> String {
    match owner {
        ContentOwner::Record(key) => format!("record:{key}"),
        ContentOwner::CreatureEntity(id) => format!("creature_entity:{}", id.as_str()),
        ContentOwner::CreatureOccurrence(id) => format!("creature_occurrence:{}", id.as_str()),
        ContentOwner::HazardEntity(id) => format!("hazard_entity:{}", id.as_str()),
        ContentOwner::HazardOccurrence(id) => format!("hazard_occurrence:{}", id.as_str()),
    }
}

const fn content_role(role: ContentRole) -> &'static str {
    match role {
        ContentRole::PrimaryDescription => "primary_description",
        ContentRole::Summary => "summary",
        ContentRole::SupplementalRules => "supplemental_rules",
        ContentRole::EmbeddedCapability => "embedded_capability",
        ContentRole::JournalPage => "journal_page",
        ContentRole::TableResult => "table_result",
        ContentRole::GeneratedNarrative => "generated_narrative",
        ContentRole::Provenance => "provenance",
    }
}

fn unsupported_field_name(field: &HazardUnsupportedField) -> String {
    match field {
        HazardUnsupportedField::HazardUnexpected(name) => format!("unexpected.{name}"),
        HazardUnsupportedField::DefensesHasHealth => "defenses.has_health".to_string(),
        HazardUnsupportedField::HitPointsTempMax => "hit_points.temp_max".to_string(),
        HazardUnsupportedField::SaveDetail(kind) => format!("save.{kind:?}.detail").to_lowercase(),
        HazardUnsupportedField::ActionUnexpected(name) => format!("action.unexpected.{name}"),
        HazardUnsupportedField::StrikeUnexpected(name) => format!("strike.unexpected.{name}"),
        HazardUnsupportedField::StrikeAttack => "strike.attack".to_string(),
        HazardUnsupportedField::StrikeWeaponType => "strike.weapon_type".to_string(),
        HazardUnsupportedField::StrikeAttackEffectsCustom => {
            "strike.attack_effects_custom".to_string()
        }
        HazardUnsupportedField::StrikeTraitRarity => "strike.trait_rarity".to_string(),
        HazardUnsupportedField::ConditionUnexpected(name) => {
            format!("condition.unexpected.{name}")
        }
        HazardUnsupportedField::EffectUnexpected(name) => format!("effect.unexpected.{name}"),
        HazardUnsupportedField::UnsupportedChildField(name) => {
            format!("unsupported_child.{name}")
        }
    }
}
