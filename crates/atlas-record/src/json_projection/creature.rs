use atlas_domain::DetailLevel;
use serde::Serialize;

use crate::{
    ActivityRollAbility, CreatureActionCost, CreatureCapability, CreatureDamage,
    CreatureDamageKind, CreatureEntity, CreatureEntityOccurrence, CreatureEntityTarget,
    CreatureFrequency, CreatureIwr, CreatureMovementMode, CreatureNumber,
    CreatureOccurrenceContext, CreatureOccurrenceParent, CreaturePreparedSpellSlot, CreatureRecord,
    CreatureResourceAmount, CreatureRoll, CreatureRollKind, CreatureSave, CreatureSkill,
    CreatureSkillVariant, CreatureSourceScalar, CreatureSpellPreparation, CreatureSpellSlot,
    CreatureUnmodeledSkillReason, CreatureUseLimit, FactValue, ResourceCurrentPolicy,
};

use super::{CreaturePerceptionJson, RecordPresentationJson};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureDefensesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ac: Option<CreatureArmorClassJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<CreatureHitPointsJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saves: Option<CreatureSavesJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_saves_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immunities: Option<Vec<CreatureIwrJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resistances: Option<Vec<CreatureIwrJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weaknesses: Option<Vec<CreatureIwrJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureArmorClassJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureHitPointsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary_maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSavesJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fortitude: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflex: Option<CreatureSaveJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will: Option<CreatureSaveJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSaveJson {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureIwrJson {
    pub id: String,
    pub order: u32,
    pub iwr_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub double_vs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_once: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSenseJson {
    pub id: String,
    pub order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acuity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_feet: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillJson {
    pub id: String,
    pub order: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_entries: Vec<CreatureSkillSourceEntryJson>,
    pub slug: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<CreatureSkillVariantJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmodeled: Option<CreatureUnmodeledSkillJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillSourceEntryJson {
    pub authored_key: String,
    pub modifier: CreatureIntegerPresenceJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureUnmodeledSkillJson {
    pub authored_key: String,
    pub base: CreatureIntegerPresenceJson,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum CreatureIntegerPresenceJson {
    Missing,
    Null,
    Value(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSkillVariantJson {
    pub id: String,
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureMovementJson {
    pub modes: Vec<CreatureMovementModeJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureMovementModeJson {
    pub id: String,
    pub order: u32,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_feet: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureResourceJson {
    pub id: String,
    pub order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
    pub current_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActionCostJson {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureFrequencyJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureStrikeJson {
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attack_effects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolls: Option<Vec<CreatureRollJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureActionJson {
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<CreatureFrequencyJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolls: Option<Vec<CreatureRollJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureSpellcastingJson {
    pub entries: Vec<CreatureSpellcastingEntryJson>,
    pub spells: Vec<CreatureSpellJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellcastingEntryJson {
    pub id: String,
    pub order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots: Option<Vec<CreatureSpellSlotJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellSlotJson {
    pub rank: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<Vec<CreaturePreparedSpellJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreaturePreparedSpellJson {
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureUseLimitJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serialized_value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct CreatureOccurrenceContextJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<CreatureUseLimitJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contextual_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureSpellJson {
    pub id: String,
    pub order: u32,
    pub label: String,
    pub traits: Vec<String>,
    pub action_cost: CreatureActionCostJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_entry_id: Option<String>,
    pub context: CreatureOccurrenceContextJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_rank: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<Vec<CreatureDamageJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureRollJson {
    pub id: String,
    pub label: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureDamageJson {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_modifier: Option<bool>,
}

pub(super) fn creature_presentation(
    creature: &CreatureRecord,
    detail: DetailLevel,
) -> RecordPresentationJson {
    let include_scan = matches!(
        detail,
        DetailLevel::Preview | DetailLevel::Standard | DetailLevel::Full
    );
    let include_details = matches!(detail, DetailLevel::Standard | DetailLevel::Full);
    if !include_scan {
        return RecordPresentationJson::Creature {
            defenses: None,
            perception: None,
            languages: None,
            skills: None,
            movement: None,
            resources: None,
            strikes: None,
            actions: None,
            spellcasting: None,
        };
    }

    let mut strikes = Vec::new();
    let mut actions = Vec::new();
    let mut entries = Vec::new();
    let mut spells = Vec::new();
    if let Some(embedded) = creature.embedded_entities.value.as_value() {
        for occurrence in &embedded.occurrences {
            let label = occurrence_label(occurrence, &embedded.entities);
            match &occurrence.capability {
                CreatureCapability::Strike(capability) => strikes.push(CreatureStrikeJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    attack_effects: strings(&capability.attack_effects),
                    rolls: include_details.then(|| rolls(&capability.rolls)),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                }),
                CreatureCapability::Action(capability) => actions.push(CreatureActionJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    category: text(&capability.category),
                    frequency: capability.frequency.as_value().map(frequency),
                    requirements: include_details
                        .then(|| text(&capability.requirements))
                        .flatten(),
                    cost: include_details.then(|| text(&capability.cost)).flatten(),
                    rolls: include_details.then(|| rolls(&capability.rolls)),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                }),
                CreatureCapability::SpellcastingEntry(capability) => {
                    entries.push(CreatureSpellcastingEntryJson {
                        id: occurrence.id.as_str().to_string(),
                        order: occurrence.authored_order,
                        label,
                        preparation: capability.preparation.as_value().map(preparation),
                        tradition: text(&capability.tradition),
                        attack: integer(&capability.attack),
                        dc: integer(&capability.dc),
                        slots: include_details
                            .then(|| spell_slots(&capability.slots))
                            .flatten(),
                    });
                }
                CreatureCapability::Spell(capability) => spells.push(CreatureSpellJson {
                    id: occurrence.id.as_str().to_string(),
                    order: occurrence.authored_order,
                    label,
                    traits: strings(&capability.traits),
                    action_cost: action_cost(&capability.action_cost),
                    target_record_key: match &occurrence.target {
                        CreatureEntityTarget::CanonicalRecord(key) => Some(key.to_string()),
                        CreatureEntityTarget::ActorOwned(_) => None,
                    },
                    parent_entry_id: match &occurrence.parent {
                        CreatureOccurrenceParent::SpellcastingEntry(parent) => {
                            Some(parent.as_str().to_string())
                        }
                        CreatureOccurrenceParent::Creature => None,
                    },
                    context: occurrence_context(&occurrence.context),
                    base_rank: integer(&capability.base_rank),
                    signature: boolean(&capability.signature),
                    traditions: strings(&capability.traditions),
                    requirements: include_details
                        .then(|| text(&capability.requirements))
                        .flatten(),
                    cost: include_details.then(|| text(&capability.cost)).flatten(),
                    target: include_details.then(|| text(&capability.target)).flatten(),
                    range: include_details.then(|| text(&capability.range)).flatten(),
                    time: include_details.then(|| text(&capability.time)).flatten(),
                    damage: include_details
                        .then(|| damage(&capability.damage))
                        .flatten(),
                }),
                CreatureCapability::Equipment(_)
                | CreatureCapability::Lore(_)
                | CreatureCapability::Unsupported(_) => {}
            }
        }
    }
    strikes.sort_by_key(|item| item.order);
    actions.sort_by_key(|item| item.order);
    entries.sort_by_key(|item| item.order);
    spells.sort_by_key(|item| item.order);

    RecordPresentationJson::Creature {
        defenses: creature.defenses.value.as_value().map(defenses),
        perception: creature.perception.value.as_value().map(perception),
        languages: creature
            .languages
            .value
            .as_value()
            .and_then(|languages| languages.values.as_value())
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            }),
        skills: creature
            .skills
            .value
            .as_value()
            .map(|values| values.iter().map(skill).collect()),
        movement: creature
            .movement
            .value
            .as_value()
            .map(|values| CreatureMovementJson {
                modes: values.iter().map(movement).collect(),
            }),
        resources: creature
            .resources
            .value
            .as_value()
            .map(|values| values.iter().map(resource).collect()),
        strikes: creature.embedded_entities.value.as_value().map(|_| strikes),
        actions: creature.embedded_entities.value.as_value().map(|_| actions),
        spellcasting: creature
            .embedded_entities
            .value
            .as_value()
            .map(|_| CreatureSpellcastingJson { entries, spells }),
    }
}

fn defenses(value: &crate::CreatureDefenses) -> CreatureDefensesJson {
    CreatureDefensesJson {
        ac: value
            .armor_class
            .as_value()
            .map(|ac| CreatureArmorClassJson {
                value: integer(&ac.value),
                details: note(&ac.details),
            }),
        hp: value.hit_points.as_value().map(|hp| CreatureHitPointsJson {
            value: hp.value.as_value().and_then(|value| match value {
                CreatureNumber::Integer(value) => Some(*value),
                CreatureNumber::Unsupported(_) => None,
            }),
            maximum: integer(&hp.maximum),
            temporary: integer(&hp.temporary),
            temporary_maximum: integer(&hp.temporary_maximum),
            details: note(&hp.details),
        }),
        hardness: integer(&value.hardness),
        saves: value.saves.as_value().map(|saves| CreatureSavesJson {
            fortitude: saves.fortitude.as_value().map(save),
            reflex: saves.reflex.as_value().map(save),
            will: saves.will.as_value().map(save),
        }),
        all_saves_note: note(&value.all_saves_note),
        immunities: iwr_values(&value.immunities),
        resistances: iwr_values(&value.resistances),
        weaknesses: iwr_values(&value.weaknesses),
    }
}

fn save(value: &CreatureSave) -> CreatureSaveJson {
    CreatureSaveJson {
        id: value.id.as_str().to_string(),
        value: integer(&value.value),
        details: note(&value.details),
    }
}

fn iwr_values(values: &FactValue<Vec<CreatureIwr>>) -> Option<Vec<CreatureIwrJson>> {
    values.as_value().map(|values| {
        values
            .iter()
            .map(|value| CreatureIwrJson {
                id: value.id.as_str().to_string(),
                order: value.authored_order,
                iwr_type: value.iwr_type.as_str().to_string(),
                value: integer(&value.value),
                exceptions: value
                    .exceptions
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                double_vs: value
                    .double_vs
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| value.as_str().to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                apply_once: boolean(&value.apply_once),
            })
            .collect()
    })
}

fn perception(value: &crate::CreaturePerception) -> CreaturePerceptionJson {
    CreaturePerceptionJson {
        modifier: integer(&value.modifier),
        details: note(&value.details),
        has_vision: boolean(&value.has_vision),
        senses: value.senses.as_value().map(|values| {
            values
                .iter()
                .map(|sense| CreatureSenseJson {
                    id: sense.id.as_str().to_string(),
                    order: sense.authored_order,
                    kind: sense.sense_type.as_str().to_string(),
                    acuity: sense.acuity.as_value().map(|acuity| match acuity {
                        crate::SenseAcuity::Precise => "precise".to_string(),
                        crate::SenseAcuity::Imprecise => "imprecise".to_string(),
                        crate::SenseAcuity::Vague => "vague".to_string(),
                        crate::SenseAcuity::Unsupported(value) => value.value.clone(),
                    }),
                    range_feet: integer(&sense.range),
                })
                .collect()
        }),
    }
}

fn skill(value: &CreatureSkill) -> CreatureSkillJson {
    CreatureSkillJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        source_entries: value
            .source_entries
            .iter()
            .map(|entry| CreatureSkillSourceEntryJson {
                authored_key: entry.authored_key.clone(),
                modifier: integer_presence(&entry.modifier),
            })
            .collect(),
        slug: value.kind.source_slug().to_string(),
        label: value.label.clone(),
        modifier: value
            .unmodeled
            .as_value()
            .is_none()
            .then(|| integer(&value.modifier))
            .flatten(),
        note: note(&value.note),
        variants: value
            .variants
            .as_value()
            .map(|values| values.iter().map(skill_variant).collect())
            .unwrap_or_default(),
        source_item_id: value
            .source_item_id
            .as_value()
            .map(|value| value.as_str().to_string()),
        unmodeled: value
            .unmodeled
            .as_value()
            .map(|unmodeled| CreatureUnmodeledSkillJson {
                authored_key: unmodeled.authored_key.clone(),
                base: integer_presence(&unmodeled.base),
                reason: match unmodeled.reason {
                    CreatureUnmodeledSkillReason::UnknownAuthoredKey => "unknown_authored_key",
                },
            }),
    }
}

fn integer_presence(value: &FactValue<i64>) -> CreatureIntegerPresenceJson {
    match value {
        FactValue::Missing => CreatureIntegerPresenceJson::Missing,
        FactValue::Null => CreatureIntegerPresenceJson::Null,
        FactValue::Value(value) => CreatureIntegerPresenceJson::Value(*value),
    }
}

fn skill_variant(value: &CreatureSkillVariant) -> CreatureSkillVariantJson {
    CreatureSkillVariantJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        modifier: integer(&value.modifier),
        label: text(&value.label),
        predicates: value
            .predicate
            .as_value()
            .map(|values| values.iter().map(predicate).collect())
            .unwrap_or_default(),
    }
}

fn predicate(value: &crate::CreaturePredicate) -> String {
    match value {
        crate::CreaturePredicate::Term(term) => term.as_str().to_string(),
        crate::CreaturePredicate::Not(term) => format!("not:{}", term.as_str()),
        crate::CreaturePredicate::Any(terms) => format!(
            "any:{}",
            terms
                .iter()
                .map(|term| term.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ),
        crate::CreaturePredicate::AtLeast { term, minimum } => {
            format!("at_least:{minimum}:{}", term.as_str())
        }
        crate::CreaturePredicate::Unsupported(value) => value.value.clone(),
    }
}

fn movement(value: &crate::CreatureSpeed) -> CreatureMovementModeJson {
    CreatureMovementModeJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        mode: match &value.mode {
            CreatureMovementMode::Land => "land".to_string(),
            CreatureMovementMode::Burrow => "burrow".to_string(),
            CreatureMovementMode::Climb => "climb".to_string(),
            CreatureMovementMode::Fly => "fly".to_string(),
            CreatureMovementMode::Swim => "swim".to_string(),
            CreatureMovementMode::Unsupported(value) => value.value.clone(),
        },
        label: text(&value.label),
        value_feet: integer(&value.value),
        details: note(&value.details),
    }
}

fn resource(value: &crate::CreatureResource) -> CreatureResourceJson {
    CreatureResourceJson {
        id: value.id.as_str().to_string(),
        order: value.authored_order,
        kind: value.kind.as_str().to_string(),
        label: value.label.clone(),
        maximum: resource_amount(&value.maximum),
        serialized_value: resource_amount(&value.serialized_value),
        current_policy: match value.current_policy {
            ResourceCurrentPolicy::SerializedValueIsProvenanceOnly => {
                "serialized_value_is_provenance_only"
            }
        },
    }
}

fn resource_amount(value: &FactValue<CreatureResourceAmount>) -> Option<i64> {
    value.as_value().and_then(|value| match value {
        CreatureResourceAmount::Integer(value) => Some(*value),
        CreatureResourceAmount::Unsupported(_) => None,
    })
}

fn occurrence_label(occurrence: &CreatureEntityOccurrence, entities: &[CreatureEntity]) -> String {
    if let Some(label) = occurrence.context.contextual_label.as_value() {
        return label.clone();
    }
    if let CreatureEntityTarget::ActorOwned(id) = &occurrence.target
        && let Some(entity) = entities.iter().find(|entity| &entity.id == id)
    {
        return entity.label.clone();
    }
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => key.to_string(),
        CreatureEntityTarget::ActorOwned(id) => id.as_str().to_string(),
    }
}

fn occurrence_context(value: &CreatureOccurrenceContext) -> CreatureOccurrenceContextJson {
    CreatureOccurrenceContextJson {
        group: text(&value.group),
        rank: integer(&value.rank),
        location: text(&value.location),
        slot: text(&value.slot),
        uses: value.uses.as_value().map(use_limit),
        contextual_label: text(&value.contextual_label),
    }
}

fn use_limit(value: &CreatureUseLimit) -> CreatureUseLimitJson {
    CreatureUseLimitJson {
        maximum: integer(&value.maximum),
        serialized_value: integer(&value.serialized_value),
    }
}

fn action_cost(value: &CreatureActionCost) -> CreatureActionCostJson {
    let (kind, actions, time, unsupported) = match value {
        CreatureActionCost::Passive => ("passive", None, None, None),
        CreatureActionCost::Reaction => ("reaction", None, None, None),
        CreatureActionCost::FreeAction => ("free_action", None, None, None),
        CreatureActionCost::Actions(actions) => ("actions", Some(*actions), None, None),
        CreatureActionCost::Time(time) => ("time", None, Some(time.clone()), None),
        CreatureActionCost::Unsupported(value) => {
            ("unsupported", None, None, Some(value.value.clone()))
        }
    };
    CreatureActionCostJson {
        kind,
        actions,
        time,
        unsupported,
    }
}

fn frequency(value: &CreatureFrequency) -> CreatureFrequencyJson {
    CreatureFrequencyJson {
        maximum: integer(&value.maximum),
        period: text(&value.period),
        serialized_value: integer(&value.serialized_value),
    }
}

fn preparation(value: &CreatureSpellPreparation) -> String {
    match value {
        CreatureSpellPreparation::Prepared => "prepared".to_string(),
        CreatureSpellPreparation::Spontaneous => "spontaneous".to_string(),
        CreatureSpellPreparation::Focus => "focus".to_string(),
        CreatureSpellPreparation::Innate => "innate".to_string(),
        CreatureSpellPreparation::Ritual => "ritual".to_string(),
        CreatureSpellPreparation::Unsupported(value) => value.value.clone(),
    }
}

fn spell_slots(values: &FactValue<Vec<CreatureSpellSlot>>) -> Option<Vec<CreatureSpellSlotJson>> {
    values
        .as_value()
        .map(|values| values.iter().map(spell_slot).collect())
}

fn spell_slot(value: &CreatureSpellSlot) -> CreatureSpellSlotJson {
    CreatureSpellSlotJson {
        rank: value.rank,
        maximum: source_integer(&value.maximum),
        serialized_value: source_integer(&value.serialized_value),
        prepared: value.prepared.as_value().map(|values| {
            values
                .iter()
                .filter_map(|value| match value {
                    CreaturePreparedSpellSlot::Spell {
                        id,
                        name,
                        expended,
                        prepared,
                        authored_order,
                    } => Some(CreaturePreparedSpellJson {
                        order: *authored_order,
                        id: id.as_value().map(|value| value.as_str().to_string()),
                        name: text(name),
                        expended: boolean(expended),
                        prepared: boolean(prepared),
                    }),
                    CreaturePreparedSpellSlot::Unsupported(_) => None,
                })
                .collect()
        }),
    }
}

fn source_integer(value: &FactValue<CreatureSourceScalar<i64>>) -> Option<i64> {
    value.as_value().and_then(|value| match value {
        CreatureSourceScalar::Value(value) => Some(*value),
        CreatureSourceScalar::Unsupported(_) => None,
    })
}

fn rolls(values: &[CreatureRoll]) -> Vec<CreatureRollJson> {
    values
        .iter()
        .map(|value| CreatureRollJson {
            id: value.id.clone(),
            label: value.label.clone(),
            kind: match value.kind {
                CreatureRollKind::Attack => "attack",
                CreatureRollKind::DifficultyClass => "difficulty_class",
                CreatureRollKind::Check => "check",
            },
            value: integer(&value.value),
            ability: value.ability.as_value().copied().map(ability),
        })
        .collect()
}

fn damage(values: &FactValue<Vec<CreatureDamage>>) -> Option<Vec<CreatureDamageJson>> {
    values.as_value().map(|values| {
        values
            .iter()
            .map(|value| CreatureDamageJson {
                id: value.id.clone(),
                formula: text(&value.formula),
                damage_type: text(&value.damage_type),
                category: text(&value.category),
                kinds: value
                    .kinds
                    .as_value()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|kind| match kind {
                                CreatureDamageKind::Damage => Some("damage"),
                                CreatureDamageKind::Healing => Some("healing"),
                                CreatureDamageKind::Unsupported(_) => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                apply_modifier: value
                    .apply_modifier
                    .as_value()
                    .and_then(|value| match value {
                        CreatureSourceScalar::Value(value) => Some(*value),
                        CreatureSourceScalar::Unsupported(_) => None,
                    }),
            })
            .collect()
    })
}

fn integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn boolean(value: &FactValue<bool>) -> Option<bool> {
    value.as_value().copied()
}

fn text(value: &FactValue<String>) -> Option<String> {
    value.as_value().cloned()
}

fn note(value: &FactValue<crate::CreatureNote>) -> Option<String> {
    value.as_value().map(|value| value.as_str().to_string())
}

fn strings(value: &FactValue<Vec<String>>) -> Vec<String> {
    value.as_value().cloned().unwrap_or_default()
}

fn ability(value: ActivityRollAbility) -> &'static str {
    match value {
        ActivityRollAbility::Strength => "strength",
        ActivityRollAbility::Dexterity => "dexterity",
        ActivityRollAbility::Constitution => "constitution",
        ActivityRollAbility::Intelligence => "intelligence",
        ActivityRollAbility::Wisdom => "wisdom",
        ActivityRollAbility::Charisma => "charisma",
    }
}
