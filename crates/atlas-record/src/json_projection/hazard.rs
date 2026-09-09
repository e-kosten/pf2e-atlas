use serde::Serialize;

use crate::{
    ContentIdentityStability, ContentOwner, ContentRole, DuplicateContentStatus, FactValue,
    HazardCapability, HazardEntitySourceIdentity, HazardFact, HazardItemCommon,
    HazardOccurrenceIdentityStability, HazardRecord, HazardRuleElement, HazardSourceMetadataFact,
    HazardSourceMetadataField, HazardSourceValue, HazardUnsupportedFact, HazardUnsupportedField,
    HazardUnsupportedValue, project_hazard_source_metadata,
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
    #[serde(skip)]
    pub human_label: &'static str,
    #[serde(skip)]
    pub human_message: String,
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
    pub source_metadata: Vec<HazardSourceMetadataFact>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_rules: Vec<HazardUnsupportedValue>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identity_diagnostics: Vec<HazardUnsupportedValue>,
}

pub(super) fn availability(hazard: &HazardRecord) -> Vec<HazardAvailabilityJson> {
    let mut values = Vec::new();
    for (field, human_label, state) in [
        ("level", "Level", fact_state(&hazard.level)),
        ("rarity", "Rarity", fact_state(&hazard.rarity)),
        ("traits", "Traits", fact_state(&hazard.traits)),
        ("size", "Size", fact_state(&hazard.size)),
        (
            "publication",
            "Publication",
            fact_state(&hazard.publication),
        ),
        ("complexity", "Complexity", fact_state(&hazard.complexity)),
        ("detection", "Detection", fact_state(&hazard.detection)),
        ("defenses", "Defenses", fact_state(&hazard.defenses)),
        ("lifecycle", "Lifecycle", fact_state(&hazard.lifecycle)),
        ("emits_sound", "Sound", fact_state(&hazard.emits_sound)),
        (
            "embedded_entities",
            "Activities",
            fact_state(&hazard.embedded_entities),
        ),
    ] {
        push_availability(&mut values, field, human_label, None, state);
    }

    if let Some(detection) = hazard.detection.typed() {
        push_availability(
            &mut values,
            "detection.stealth_modifier",
            "Stealth",
            None,
            fact_state(&detection.stealth_modifier),
        );
        push_availability(
            &mut values,
            "detection.details",
            "Detection details",
            None,
            fact_state(&detection.details),
        );
    }
    if let Some(defenses) = hazard.defenses.typed() {
        for (field, human_label, state) in [
            (
                "defenses.armor_class",
                "Armor Class",
                fact_state(&defenses.armor_class),
            ),
            (
                "defenses.hardness",
                "Hardness",
                fact_state(&defenses.hardness),
            ),
            (
                "defenses.hit_points",
                "Hit Points",
                fact_state(&defenses.hit_points),
            ),
            ("defenses.saves", "Saves", fact_state(&defenses.saves)),
            (
                "defenses.immunities",
                "Immunities",
                fact_state(&defenses.immunities),
            ),
            (
                "defenses.weaknesses",
                "Weaknesses",
                fact_state(&defenses.weaknesses),
            ),
            (
                "defenses.resistances",
                "Resistances",
                fact_state(&defenses.resistances),
            ),
        ] {
            push_availability(&mut values, field, human_label, None, state);
        }
        if let Some(hit_points) = defenses.hit_points.typed() {
            for (field, human_label, state) in [
                (
                    "defenses.hit_points.current",
                    "Current Hit Points",
                    fact_state(&hit_points.current),
                ),
                (
                    "defenses.hit_points.maximum",
                    "Maximum Hit Points",
                    fact_state(&hit_points.maximum),
                ),
                (
                    "defenses.hit_points.temporary",
                    "Temporary Hit Points",
                    fact_state(&hit_points.temporary),
                ),
                (
                    "defenses.hit_points.details",
                    "Hit Point details",
                    fact_state(&hit_points.details),
                ),
            ] {
                push_availability(&mut values, field, human_label, None, state);
            }
        }
        if let Some(saves) = defenses.saves.typed() {
            for (field, human_label, state) in [
                (
                    "defenses.saves.fortitude",
                    "Fortitude",
                    fact_state(&saves.fortitude),
                ),
                ("defenses.saves.reflex", "Reflex", fact_state(&saves.reflex)),
                ("defenses.saves.will", "Will", fact_state(&saves.will)),
            ] {
                push_availability(&mut values, field, human_label, None, state);
            }
        }
    }
    if let Some(lifecycle) = hazard.lifecycle.typed() {
        for (field, human_label, state) in [
            (
                "lifecycle.description",
                "Description",
                fact_state(&lifecycle.description),
            ),
            (
                "lifecycle.disable",
                "Disable",
                fact_state(&lifecycle.disable),
            ),
            (
                "lifecycle.routine",
                "Routine",
                fact_state(&lifecycle.routine),
            ),
            ("lifecycle.reset", "Reset", fact_state(&lifecycle.reset)),
        ] {
            push_availability(&mut values, field, human_label, None, state);
        }
    }
    if let Some(embedded) = hazard.embedded_entities.typed() {
        for occurrence in &embedded.occurrences {
            let component_id = Some(occurrence.id.as_str().to_string());
            push_availability(
                &mut values,
                "occurrence.source_sort",
                "Activity order",
                component_id.clone(),
                fact_state(&occurrence.source_sort),
            );
            push_availability(
                &mut values,
                "occurrence.source_folder",
                "Activity folder",
                component_id.clone(),
                fact_state(&occurrence.source_folder),
            );
            push_availability(
                &mut values,
                "occurrence.contextual_label",
                "Activity label",
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
                    "Activity identity",
                    Some(component_id.to_string()),
                    Some(HazardAvailabilityStateJson::Unsupported),
                );
            }
            let (common, unsupported) = match &entity.capability {
                HazardCapability::Action(value) => {
                    for (field, human_label, state) in [
                        (
                            "action.action_type",
                            "Action type",
                            fact_state(&value.action_type),
                        ),
                        ("action.actions", "Action cost", fact_state(&value.actions)),
                        (
                            "action.category",
                            "Action category",
                            fact_state(&value.category),
                        ),
                        (
                            "action.death_note",
                            "Death note",
                            fact_state(&value.death_note),
                        ),
                        (
                            "action.frequency",
                            "Frequency",
                            fact_state(&value.frequency),
                        ),
                        (
                            "action.self_effect",
                            "Self effect",
                            fact_state(&value.self_effect),
                        ),
                    ] {
                        push_availability(
                            &mut values,
                            field,
                            human_label,
                            Some(component_id.to_string()),
                            state,
                        );
                    }
                    (&value.common, &value.unsupported_fields)
                }
                HazardCapability::Strike(value) => {
                    for (field, human_label, state) in [
                        ("strike.bonus", "Strike bonus", fact_state(&value.bonus)),
                        (
                            "strike.attack_effects",
                            "Attack effects",
                            fact_state(&value.attack_effects),
                        ),
                        (
                            "strike.damage_rolls",
                            "Strike damage",
                            fact_state(&value.damage_rolls),
                        ),
                    ] {
                        push_availability(
                            &mut values,
                            field,
                            human_label,
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
                    unsupported_field_label(&unsupported.field),
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
                    "Activity rule",
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
            unsupported_field_label(&unsupported.field),
            None,
            Some(HazardAvailabilityStateJson::Unsupported),
        );
    }
    for issue in project_hazard_source_metadata(hazard).issues {
        push_availability_message(
            &mut values,
            issue.field_key(),
            source_metadata_label(issue.field),
            issue.component_id().map(str::to_string),
            HazardAvailabilityStateJson::Unsupported,
            issue.message(),
            issue.message(),
        );
    }
    values
}

pub(super) fn provenance(hazard: &HazardRecord) -> HazardProvenanceJson {
    let source_metadata = project_hazard_source_metadata(hazard).facts;
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
    let unsupported_fields = hazard
        .unsupported_facts()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut unsupported_rules = Vec::new();
    let mut identity_diagnostics = Vec::new();
    if let Some(embedded) = hazard.embedded_entities.typed() {
        for entity in &embedded.entities {
            let common = match &entity.capability {
                HazardCapability::Action(value) => &value.common,
                HazardCapability::Strike(value) => &value.common,
                HazardCapability::Condition(value) => &value.common,
                HazardCapability::Effect(value) => &value.common,
                HazardCapability::UnsupportedChild(value) => &value.common,
            };
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
        source_metadata,
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
    for (field, human_label, state) in [
        (
            "entity.description",
            "Activity description",
            fact_state(&common.description),
        ),
        (
            "entity.publication",
            "Activity publication",
            fact_state(&common.publication),
        ),
        ("entity.rules", "Activity rules", fact_state(&common.rules)),
        ("entity.slug", "Activity slug", fact_state(&common.slug)),
        (
            "entity.traits",
            "Activity traits",
            fact_state(&common.traits),
        ),
    ] {
        push_availability(
            values,
            field,
            human_label,
            Some(component_id.to_string()),
            state,
        );
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
    human_label: &'static str,
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
    push_availability_message(
        values,
        field,
        human_label,
        component_id,
        state,
        &format!("{field} data is {state_label}."),
        &format!("{human_label} is {state_label}."),
    );
}

fn push_availability_message(
    values: &mut Vec<HazardAvailabilityJson>,
    field: &str,
    human_label: &'static str,
    component_id: Option<String>,
    state: HazardAvailabilityStateJson,
    message: &str,
    human_message: &str,
) {
    let value = HazardAvailabilityJson {
        message: message.to_string(),
        state,
        field: field.to_string(),
        component_id,
        human_label,
        human_message: human_message.to_string(),
    };
    if !values.contains(&value) {
        values.push(value);
    }
}

fn content_owner(owner: &ContentOwner) -> String {
    match owner {
        ContentOwner::Record(key) => format!("record:{key}"),
        ContentOwner::CreatureEntity(id) => format!("creature_entity:{}", id.as_str()),
        ContentOwner::CreatureOccurrence(id) => format!("creature_occurrence:{}", id.as_str()),
        ContentOwner::HazardEntity(id) => format!("hazard_entity:{}", id.as_str()),
        ContentOwner::HazardOccurrence(id) => format!("hazard_occurrence:{}", id.as_str()),
        ContentOwner::Child(locator) => {
            format!("child:{}", crate::encode_content_child_locator(locator))
        }
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
        HazardUnsupportedField::ActionUnexpected(name) => format!("action.unexpected.{name}"),
        HazardUnsupportedField::StrikeUnexpected(name) => format!("strike.unexpected.{name}"),
        HazardUnsupportedField::ConditionUnexpected(name) => {
            format!("condition.unexpected.{name}")
        }
        HazardUnsupportedField::EffectUnexpected(name) => format!("effect.unexpected.{name}"),
        HazardUnsupportedField::UnsupportedChildField(name) => {
            format!("unsupported_child.{name}")
        }
    }
}

const fn unsupported_field_label(field: &HazardUnsupportedField) -> &'static str {
    match field {
        HazardUnsupportedField::HazardUnexpected(_) => "Hazard source fact",
        HazardUnsupportedField::ActionUnexpected(_) => "Action source fact",
        HazardUnsupportedField::StrikeUnexpected(_) => "Strike source fact",
        HazardUnsupportedField::ConditionUnexpected(_) => "Condition source fact",
        HazardUnsupportedField::EffectUnexpected(_) => "Effect source fact",
        HazardUnsupportedField::UnsupportedChildField(_) => "Embedded source fact",
    }
}

const fn source_metadata_label(field: HazardSourceMetadataField) -> &'static str {
    match field {
        HazardSourceMetadataField::TokenName => "Token metadata",
        HazardSourceMetadataField::HasHealth => "Hit Point metadata",
        HazardSourceMetadataField::TemporaryMaximum => "Temporary Hit Point metadata",
        HazardSourceMetadataField::SaveDetail(_) => "Save metadata",
        HazardSourceMetadataField::ItemRarity => "Activity rarity metadata",
        HazardSourceMetadataField::ItemLineage => "Activity lineage metadata",
        HazardSourceMetadataField::StrikeAttack => "Strike attack metadata",
        HazardSourceMetadataField::StrikeWeaponType => "Strike mode metadata",
        HazardSourceMetadataField::StrikeAttackEffectsCustom => "Strike effect metadata",
    }
}
