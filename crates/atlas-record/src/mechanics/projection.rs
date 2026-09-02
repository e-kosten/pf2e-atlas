use atlas_domain::RecordKey;

use crate::{
    CreatureActionCost, CreatureCapability, CreatureDamage, CreatureDamageKind,
    CreatureEmbeddedEntities, CreatureEntityOccurrence, CreatureEntityTarget, CreatureFrequency,
    CreatureMovementMode, CreatureNumber, CreatureOccurrenceId, CreaturePreparedSpellSlot,
    CreatureRecord, CreatureResourceAmount, CreatureRoll, CreatureSourceScalar,
    CreatureSpellPreparation, CreatureSpellSave, CreatureSpellSlot, CreatureUnsupportedSourceFact,
    CreatureUseLimit, FactValue, UnsupportedMechanicNote, UnsupportedSourceValue,
};

use super::{MechanicFacets, MechanicSourceFamily, MechanicTarget, SaveKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalMechanicsProjection {
    pub record_key: RecordKey,
    pub level: FactValue<i64>,
    pub facts: Vec<MechanicFact>,
    pub activities: Vec<CanonicalMechanicActivity>,
    pub unsupported: Vec<UnsupportedMechanic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicFact {
    pub target: MechanicTarget,
    pub label: String,
    pub value: MechanicBaseValue,
    pub facets: MechanicFacets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MechanicBaseValue {
    Integer(FactValue<i64>),
    Number(FactValue<CreatureNumber>),
    ResourceAmount(FactValue<CreatureResourceAmount>),
    ActionCost(CreatureActionCost),
    Frequency(FactValue<CreatureFrequency>),
    Uses(FactValue<CreatureUseLimit>),
    Roll(CreatureRoll),
    Damage(CreatureDamage),
    SourceInteger(FactValue<CreatureSourceScalar<i64>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalMechanicActivity {
    pub occurrence_id: CreatureOccurrenceId,
    pub family: MechanicActivityFamily,
    pub label: String,
    pub authored_order: u32,
    pub facts: Vec<MechanicFact>,
    pub unsupported: Vec<UnsupportedMechanic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicActivityFamily {
    Strike,
    SpellcastingEntry,
    Spell,
    Action,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedMechanic {
    pub target: Option<MechanicTarget>,
    pub activity_occurrence_id: Option<CreatureOccurrenceId>,
    pub source_path: String,
    pub value: UnsupportedMechanicValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedMechanicValue {
    Source(UnsupportedSourceValue),
    Note(UnsupportedMechanicNote),
    Capability {
        source_item_type: String,
        source_slug: FactValue<String>,
    },
    PreparedSlot(UnsupportedSourceValue),
    ResourceDrift(CreatureUnsupportedSourceFact),
}

pub fn project_creature_mechanics(creature: &CreatureRecord) -> CanonicalMechanicsProjection {
    let mut projection = CanonicalMechanicsProjection {
        record_key: creature.identity.record_key.clone(),
        level: creature.level.value.clone(),
        facts: Vec::new(),
        activities: Vec::new(),
        unsupported: Vec::new(),
    };

    project_defenses(creature, &mut projection);
    project_awareness_and_skills(creature, &mut projection);
    project_movement(creature, &mut projection);
    project_resources(creature, &mut projection);
    if let FactValue::Value(embedded) = &creature.embedded_entities.value {
        project_embedded(embedded, &mut projection);
    }

    projection
}

fn project_defenses(creature: &CreatureRecord, projection: &mut CanonicalMechanicsProjection) {
    let Some(defenses) = creature.defenses.value.as_value() else {
        return;
    };
    if let Some(armor_class) = defenses.armor_class.as_value() {
        projection.facts.push(MechanicFact {
            target: MechanicTarget::ArmorClass,
            label: "AC".to_string(),
            value: MechanicBaseValue::Integer(armor_class.value.clone()),
            facets: MechanicFacets::armor_class(),
        });
    }
    if let Some(hit_points) = defenses.hit_points.as_value() {
        let target = MechanicTarget::MaxHp;
        projection.facts.push(MechanicFact {
            target: target.clone(),
            label: "Max HP".to_string(),
            value: MechanicBaseValue::Number(hit_points.value.clone()),
            facets: MechanicFacets::hit_points(),
        });
        if let FactValue::Value(CreatureNumber::Unsupported(value)) = &hit_points.value {
            projection.unsupported.push(unsupported_source(
                Some(target),
                None,
                "defenses.hit_points.value",
                value,
            ));
        }
    }
    if let Some(saves) = defenses.saves.as_value() {
        for save in [&saves.fortitude, &saves.reflex, &saves.will]
            .into_iter()
            .filter_map(FactValue::as_value)
        {
            let save_kind = SaveKind::from(save.kind);
            projection.facts.push(MechanicFact {
                target: MechanicTarget::Save { save: save_kind },
                label: save_kind.label().to_string(),
                value: MechanicBaseValue::Integer(save.value.clone()),
                facets: MechanicFacets::saving_throw(save_kind),
            });
        }
    }
}

fn project_awareness_and_skills(
    creature: &CreatureRecord,
    projection: &mut CanonicalMechanicsProjection,
) {
    if let Some(perception) = creature.perception.value.as_value() {
        projection.facts.push(MechanicFact {
            target: MechanicTarget::Perception,
            label: "Perception".to_string(),
            value: MechanicBaseValue::Integer(perception.modifier.clone()),
            facets: MechanicFacets::perception(),
        });
    }
    let Some(skills) = creature.skills.value.as_value() else {
        return;
    };
    for skill in skills {
        if skill.kind == crate::CreatureSkillKind::Unmodeled {
            continue;
        }
        projection.facts.push(MechanicFact {
            target: MechanicTarget::CreatureSkill {
                skill_id: skill.id.clone(),
                kind: skill.kind,
            },
            label: skill.label.clone(),
            value: MechanicBaseValue::Integer(skill.modifier.clone()),
            facets: MechanicFacets::creature_skill(skill.kind),
        });
    }
}

fn project_movement(creature: &CreatureRecord, projection: &mut CanonicalMechanicsProjection) {
    let Some(speeds) = creature.movement.value.as_value() else {
        return;
    };
    for speed in speeds {
        let target = MechanicTarget::Movement {
            speed_id: speed.id.clone(),
        };
        projection.facts.push(MechanicFact {
            target: target.clone(),
            label: speed
                .label
                .as_value()
                .cloned()
                .unwrap_or_else(|| movement_label(&speed.mode)),
            value: MechanicBaseValue::Integer(speed.value.clone()),
            facets: MechanicFacets::movement(),
        });
        if let CreatureMovementMode::Unsupported(value) = &speed.mode {
            projection.unsupported.push(unsupported_source(
                Some(target),
                None,
                format!("movement.{}.mode", speed.id.as_str()),
                value,
            ));
        }
    }
}

fn project_resources(creature: &CreatureRecord, projection: &mut CanonicalMechanicsProjection) {
    let Some(resources) = creature.resources.value.as_value() else {
        return;
    };
    for resource in resources {
        let target = MechanicTarget::ResourceMaximum {
            resource_id: resource.id.clone(),
        };
        projection.facts.push(MechanicFact {
            target: target.clone(),
            label: resource.label.clone(),
            value: MechanicBaseValue::ResourceAmount(resource.maximum.clone()),
            facets: MechanicFacets::resource(),
        });
        if let FactValue::Value(CreatureResourceAmount::Unsupported(value)) = &resource.maximum {
            projection.unsupported.push(unsupported_source(
                Some(target.clone()),
                None,
                format!("resources.{}.maximum", resource.id.as_str()),
                value,
            ));
        }
        if let FactValue::Value(drift) = &resource.source_drift {
            projection
                .unsupported
                .extend(drift.iter().cloned().map(|value| UnsupportedMechanic {
                    target: Some(target.clone()),
                    activity_occurrence_id: None,
                    source_path: format!("resources.{}.source_drift", resource.id.as_str()),
                    value: UnsupportedMechanicValue::ResourceDrift(value),
                }));
        }
    }
}

fn project_embedded(
    embedded: &CreatureEmbeddedEntities,
    projection: &mut CanonicalMechanicsProjection,
) {
    if let FactValue::Value(actor_spellcasting) = &embedded.actor_spellcasting {
        let target = MechanicTarget::ActorRitualDc;
        projection.facts.push(MechanicFact {
            target: target.clone(),
            label: "Ritual DC".to_string(),
            value: MechanicBaseValue::SourceInteger(actor_spellcasting.rituals_dc.clone()),
            facets: MechanicFacets::spellcasting_dc(),
        });
        collect_source_scalar_unsupported(
            &actor_spellcasting.rituals_dc,
            Some(target),
            None,
            "embedded_entities.actor_spellcasting.rituals_dc",
            &mut projection.unsupported,
        );
        projection.unsupported.extend(
            actor_spellcasting
                .unsupported_notes
                .iter()
                .cloned()
                .map(|note| unsupported_note(None, None, note)),
        );
    }

    for occurrence in &embedded.occurrences {
        let unsupported_only_notes = match &occurrence.capability {
            CreatureCapability::Equipment(equipment) => Some(&equipment.unsupported_notes),
            CreatureCapability::Lore(lore) => Some(&lore.unsupported_notes),
            CreatureCapability::Strike(_)
            | CreatureCapability::Action(_)
            | CreatureCapability::SpellcastingEntry(_)
            | CreatureCapability::Spell(_)
            | CreatureCapability::Unsupported(_) => None,
        };
        if let Some(notes) = unsupported_only_notes {
            projection.unsupported.extend(
                notes
                    .iter()
                    .cloned()
                    .map(|note| unsupported_note(None, Some(occurrence.id.clone()), note)),
            );
            continue;
        }
        let Some(mut activity) = project_activity(occurrence, embedded) else {
            continue;
        };
        activity
            .facts
            .sort_by(|left, right| left.target.cmp(&right.target));
        projection.activities.push(activity);
    }
}

fn project_activity(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
) -> Option<CanonicalMechanicActivity> {
    let occurrence_id = occurrence.id.clone();
    let (family, source_family) = match &occurrence.capability {
        CreatureCapability::Strike(_) => {
            (MechanicActivityFamily::Strike, MechanicSourceFamily::Strike)
        }
        CreatureCapability::SpellcastingEntry(_) => (
            MechanicActivityFamily::SpellcastingEntry,
            MechanicSourceFamily::Spellcasting,
        ),
        CreatureCapability::Spell(_) => {
            (MechanicActivityFamily::Spell, MechanicSourceFamily::Spell)
        }
        CreatureCapability::Action(_) => {
            (MechanicActivityFamily::Action, MechanicSourceFamily::Action)
        }
        CreatureCapability::Unsupported(_) => (
            MechanicActivityFamily::Unsupported,
            MechanicSourceFamily::Unsupported,
        ),
        CreatureCapability::Equipment(_) | CreatureCapability::Lore(_) => return None,
    };
    let mut activity = CanonicalMechanicActivity {
        occurrence_id: occurrence_id.clone(),
        family,
        label: activity_label(occurrence, embedded),
        authored_order: occurrence.authored_order,
        facts: Vec::new(),
        unsupported: Vec::new(),
    };

    activity.facts.push(MechanicFact {
        target: MechanicTarget::ActivityUses {
            occurrence_id: occurrence_id.clone(),
        },
        label: "Uses".to_string(),
        value: MechanicBaseValue::Uses(occurrence.context.uses.clone()),
        facets: MechanicFacets::uses(source_family),
    });

    match &occurrence.capability {
        CreatureCapability::Strike(strike) => {
            project_action_cost(
                &mut activity,
                source_family,
                &strike.action_cost,
                &occurrence_id,
            );
            project_rolls_and_damage(
                &mut activity,
                source_family,
                &strike.rolls,
                strike.damage.as_value(),
            );
            append_notes(&mut activity, &strike.unsupported_notes);
        }
        CreatureCapability::Action(action) => {
            project_action_cost(
                &mut activity,
                source_family,
                &action.action_cost,
                &occurrence_id,
            );
            let frequency_target = MechanicTarget::ActivityFrequency {
                occurrence_id: occurrence_id.clone(),
            };
            activity.facts.push(MechanicFact {
                target: frequency_target,
                label: "Frequency".to_string(),
                value: MechanicBaseValue::Frequency(action.frequency.clone()),
                facets: MechanicFacets::frequency(source_family),
            });
            project_rolls_and_damage(
                &mut activity,
                source_family,
                &action.rolls,
                action.damage.as_value(),
            );
            append_notes(&mut activity, &action.unsupported_notes);
        }
        CreatureCapability::Spell(spell) => {
            project_action_cost(
                &mut activity,
                source_family,
                &spell.action_cost,
                &occurrence_id,
            );
            project_rolls_and_damage(&mut activity, source_family, &[], spell.damage.as_value());
            if let Some(defense) = spell.defense.as_value()
                && let FactValue::Value(CreatureSpellSave::Unsupported(value)) = &defense.save
            {
                activity.unsupported.push(unsupported_source(
                    None,
                    Some(occurrence_id.clone()),
                    format!("activities.{}.spell.defense.save", occurrence_id.as_str()),
                    value,
                ));
            }
            append_notes(&mut activity, &spell.unsupported_notes);
        }
        CreatureCapability::SpellcastingEntry(entry) => {
            let attack_target = MechanicTarget::SpellcastingAttack {
                entry_occurrence_id: occurrence_id.clone(),
            };
            activity.facts.push(MechanicFact {
                target: attack_target,
                label: "Spell Attack".to_string(),
                value: MechanicBaseValue::Integer(entry.attack.clone()),
                facets: MechanicFacets::spellcasting_attack(),
            });
            let dc_target = MechanicTarget::SpellcastingDc {
                entry_occurrence_id: occurrence_id.clone(),
            };
            activity.facts.push(MechanicFact {
                target: dc_target,
                label: "Spell DC".to_string(),
                value: MechanicBaseValue::Integer(entry.dc.clone()),
                facets: MechanicFacets::spellcasting_dc(),
            });
            if let FactValue::Value(slots) = &entry.slots {
                for slot in slots {
                    let target = MechanicTarget::SpellSlotMaximum {
                        entry_occurrence_id: occurrence_id.clone(),
                        rank: slot.rank,
                    };
                    activity.facts.push(MechanicFact {
                        target: target.clone(),
                        label: format!("Rank {} slots", slot.rank),
                        value: MechanicBaseValue::SourceInteger(slot.maximum.clone()),
                        facets: MechanicFacets::spell_slot(),
                    });
                    collect_spell_slot_unsupported(
                        slot,
                        target,
                        &occurrence_id,
                        &mut activity.unsupported,
                    );
                }
            }
            if let FactValue::Value(CreatureSpellPreparation::Unsupported(value)) =
                &entry.preparation
            {
                activity.unsupported.push(unsupported_source(
                    None,
                    Some(occurrence_id.clone()),
                    format!(
                        "activities.{}.spellcasting.preparation",
                        occurrence_id.as_str()
                    ),
                    value,
                ));
            }
            append_notes(&mut activity, &entry.unsupported_notes);
        }
        CreatureCapability::Unsupported(capability) => {
            activity.unsupported.push(UnsupportedMechanic {
                target: None,
                activity_occurrence_id: Some(occurrence_id),
                source_path: "embedded_entities.occurrences.capability".to_string(),
                value: UnsupportedMechanicValue::Capability {
                    source_item_type: capability.source_item_type.clone(),
                    source_slug: capability.source_slug.clone(),
                },
            });
            append_notes(&mut activity, &capability.unsupported_notes);
        }
        CreatureCapability::Equipment(_) | CreatureCapability::Lore(_) => return None,
    }

    Some(activity)
}

fn project_action_cost(
    activity: &mut CanonicalMechanicActivity,
    family: MechanicSourceFamily,
    action_cost: &CreatureActionCost,
    occurrence_id: &CreatureOccurrenceId,
) {
    let target = MechanicTarget::ActivityActionCost {
        occurrence_id: occurrence_id.clone(),
    };
    activity.facts.push(MechanicFact {
        target: target.clone(),
        label: "Action cost".to_string(),
        value: MechanicBaseValue::ActionCost(action_cost.clone()),
        facets: MechanicFacets::action_economy(family),
    });
    if let CreatureActionCost::Unsupported(value) = action_cost {
        activity.unsupported.push(unsupported_source(
            Some(target),
            Some(occurrence_id.clone()),
            format!("activities.{}.action_cost", occurrence_id.as_str()),
            value,
        ));
    }
}

fn project_rolls_and_damage(
    activity: &mut CanonicalMechanicActivity,
    family: MechanicSourceFamily,
    rolls: &[CreatureRoll],
    damage: Option<&Vec<CreatureDamage>>,
) {
    for roll in rolls {
        activity.facts.push(MechanicFact {
            target: MechanicTarget::ActivityRoll {
                occurrence_id: activity.occurrence_id.clone(),
                roll_id: roll.id.clone(),
            },
            label: roll.label.clone(),
            value: MechanicBaseValue::Roll(roll.clone()),
            facets: MechanicFacets::roll(family, roll.kind),
        });
    }
    let Some(damage) = damage else {
        return;
    };
    for part in damage {
        let target = MechanicTarget::ActivityDamage {
            occurrence_id: activity.occurrence_id.clone(),
            damage_id: part.id.clone(),
        };
        activity.facts.push(MechanicFact {
            target: target.clone(),
            label: part.id.clone(),
            value: MechanicBaseValue::Damage(part.clone()),
            facets: MechanicFacets::damage(family),
        });
        collect_damage_unsupported(
            part,
            target,
            &activity.occurrence_id,
            &mut activity.unsupported,
        );
    }
}

fn collect_damage_unsupported(
    damage: &CreatureDamage,
    target: MechanicTarget,
    occurrence_id: &CreatureOccurrenceId,
    unsupported: &mut Vec<UnsupportedMechanic>,
) {
    if let FactValue::Value(kinds) = &damage.kinds {
        for kind in kinds {
            if let CreatureDamageKind::Unsupported(value) = kind {
                unsupported.push(unsupported_source(
                    Some(target.clone()),
                    Some(occurrence_id.clone()),
                    format!(
                        "activities.{}.damage.{}.kinds",
                        occurrence_id.as_str(),
                        damage.id
                    ),
                    value,
                ));
            }
        }
    }
    if let FactValue::Value(CreatureSourceScalar::Unsupported(value)) = &damage.apply_modifier {
        unsupported.push(unsupported_source(
            Some(target),
            Some(occurrence_id.clone()),
            format!(
                "activities.{}.damage.{}.apply_modifier",
                occurrence_id.as_str(),
                damage.id
            ),
            value,
        ));
    }
}

fn collect_spell_slot_unsupported(
    slot: &CreatureSpellSlot,
    target: MechanicTarget,
    occurrence_id: &CreatureOccurrenceId,
    unsupported: &mut Vec<UnsupportedMechanic>,
) {
    collect_source_scalar_unsupported(
        &slot.maximum,
        Some(target.clone()),
        Some(occurrence_id.clone()),
        format!(
            "activities.{}.spellcasting.slots.{}.maximum",
            occurrence_id.as_str(),
            slot.rank
        ),
        unsupported,
    );
    collect_source_scalar_unsupported(
        &slot.serialized_value,
        Some(target.clone()),
        Some(occurrence_id.clone()),
        format!(
            "activities.{}.spellcasting.slots.{}.serialized_value",
            occurrence_id.as_str(),
            slot.rank
        ),
        unsupported,
    );
    if let FactValue::Value(prepared) = &slot.prepared {
        for prepared_slot in prepared {
            if let CreaturePreparedSpellSlot::Unsupported(value) = prepared_slot {
                unsupported.push(UnsupportedMechanic {
                    target: Some(target.clone()),
                    activity_occurrence_id: Some(occurrence_id.clone()),
                    source_path: format!(
                        "activities.{}.spellcasting.slots.{}.prepared",
                        occurrence_id.as_str(),
                        slot.rank
                    ),
                    value: UnsupportedMechanicValue::PreparedSlot(value.clone()),
                });
            }
        }
    }
}

fn collect_source_scalar_unsupported<T>(
    value: &FactValue<CreatureSourceScalar<T>>,
    target: Option<MechanicTarget>,
    activity_occurrence_id: Option<CreatureOccurrenceId>,
    source_path: impl Into<String>,
    unsupported: &mut Vec<UnsupportedMechanic>,
) {
    if let FactValue::Value(CreatureSourceScalar::Unsupported(value)) = value {
        unsupported.push(UnsupportedMechanic {
            target,
            activity_occurrence_id,
            source_path: source_path.into(),
            value: UnsupportedMechanicValue::Source(value.clone()),
        });
    }
}

fn append_notes(activity: &mut CanonicalMechanicActivity, notes: &[UnsupportedMechanicNote]) {
    activity.unsupported.extend(
        notes
            .iter()
            .cloned()
            .map(|note| unsupported_note(None, Some(activity.occurrence_id.clone()), note)),
    );
}

fn unsupported_source(
    target: Option<MechanicTarget>,
    activity_occurrence_id: Option<CreatureOccurrenceId>,
    source_path: impl Into<String>,
    value: &UnsupportedSourceValue,
) -> UnsupportedMechanic {
    UnsupportedMechanic {
        target,
        activity_occurrence_id,
        source_path: source_path.into(),
        value: UnsupportedMechanicValue::Source(value.clone()),
    }
}

fn unsupported_note(
    target: Option<MechanicTarget>,
    activity_occurrence_id: Option<CreatureOccurrenceId>,
    note: UnsupportedMechanicNote,
) -> UnsupportedMechanic {
    UnsupportedMechanic {
        target,
        activity_occurrence_id,
        source_path: note.source_path.clone(),
        value: UnsupportedMechanicValue::Note(note),
    }
}

fn activity_label(
    occurrence: &CreatureEntityOccurrence,
    embedded: &CreatureEmbeddedEntities,
) -> String {
    if let Some(label) = occurrence.context.contextual_label.as_value() {
        return label.clone();
    }
    match &occurrence.target {
        CreatureEntityTarget::CanonicalRecord(key) => key.to_string(),
        CreatureEntityTarget::ActorOwned(entity_id) => embedded
            .entities
            .iter()
            .find(|entity| &entity.id == entity_id)
            .map(|entity| entity.label.clone())
            .unwrap_or_else(|| entity_id.as_str().to_string()),
    }
}

fn movement_label(mode: &CreatureMovementMode) -> String {
    match mode {
        CreatureMovementMode::Land => "Land Speed".to_string(),
        CreatureMovementMode::Burrow => "Burrow Speed".to_string(),
        CreatureMovementMode::Climb => "Climb Speed".to_string(),
        CreatureMovementMode::Fly => "Fly Speed".to_string(),
        CreatureMovementMode::Swim => "Swim Speed".to_string(),
        CreatureMovementMode::Unsupported(value) => format!("Unsupported Speed ({})", value.value),
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RecordKey};

    use super::{
        CanonicalMechanicActivity, MechanicBaseValue, MechanicFact, MechanicTarget,
        UnsupportedMechanic, UnsupportedMechanicValue, project_creature_mechanics,
    };
    use crate::{
        ActivityRollAbility, CreatureActionCapability, CreatureActionCost,
        CreatureActorSpellcastingContext, CreatureArmorClass, CreatureCapability,
        CreatureComponentId, CreatureDamage, CreatureDamageKind, CreatureDefenses,
        CreatureEmbeddedEntities, CreatureEntity, CreatureEntityFamily, CreatureEntityId,
        CreatureEntityOccurrence, CreatureEntitySourceIdentity, CreatureEntityTarget,
        CreatureEquipmentCapability, CreatureFact, CreatureFamily, CreatureFrequency,
        CreatureHitPoints, CreatureIdentity, CreatureLoreCapability, CreatureMovementMode,
        CreatureNumber, CreatureOccurrenceContext, CreatureOccurrenceId, CreatureOccurrenceParent,
        CreaturePerception, CreaturePreparedSpellSlot, CreatureProvenance, CreatureRecord,
        CreatureResource, CreatureResourceAmount, CreatureResourceKind, CreatureRoll,
        CreatureRollKind, CreatureSave, CreatureSaveKind, CreatureSaves, CreatureSkill,
        CreatureSkillKind, CreatureSourceField, CreatureSourceId, CreatureSourceScalar,
        CreatureSpeed, CreatureSpellCapability, CreatureSpellDefense, CreatureSpellPreparation,
        CreatureSpellSave, CreatureSpellSlot, CreatureSpellcastingEntryCapability,
        CreatureStrikeCapability, CreatureUnsupportedCapability, CreatureUnsupportedSourceFact,
        CreatureUnsupportedSourceField, FactValue, MechanicActivityFamily, MechanicSourceFamily,
        MechanicSurface, OccurrenceIdentityStability, ResourceCurrentPolicy,
        UnsupportedMechanicNote, UnsupportedSourceReason, UnsupportedSourceShape,
        UnsupportedSourceValue,
    };

    #[test]
    fn projects_typed_base_facts_for_every_supported_mechanics_family() {
        let projection = project_creature_mechanics(&mechanics_creature());

        for target in [
            MechanicTarget::ArmorClass,
            MechanicTarget::MaxHp,
            MechanicTarget::Perception,
            MechanicTarget::Save {
                save: super::SaveKind::Fortitude,
            },
            MechanicTarget::CreatureSkill {
                skill_id: component("component:arcana"),
                kind: CreatureSkillKind::Arcana,
            },
            MechanicTarget::Movement {
                speed_id: component("component:land-speed"),
            },
            MechanicTarget::ResourceMaximum {
                resource_id: component("component:focus-points"),
            },
            MechanicTarget::ActorRitualDc,
        ] {
            assert!(
                fact(&projection.facts, &target).is_some(),
                "missing {target:?}"
            );
        }

        let strike = activity(&projection.activities, "occurrence:strike");
        assert_eq!(strike.family, MechanicActivityFamily::Strike);
        assert_activity_target(
            strike,
            MechanicTarget::ActivityActionCost {
                occurrence_id: occurrence("occurrence:strike"),
            },
            MechanicSurface::ActionEconomy,
        );
        assert_activity_target(
            strike,
            MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:strike"),
                roll_id: "prefix.skill.deception".to_string(),
            },
            MechanicSurface::AttackRoll,
        );
        assert_activity_target(
            strike,
            MechanicTarget::ActivityDamage {
                occurrence_id: occurrence("occurrence:strike"),
                damage_id: "prefix.hp.max".to_string(),
            },
            MechanicSurface::Damage,
        );

        let action = activity(&projection.activities, "occurrence:action");
        assert_eq!(action.family, MechanicActivityFamily::Action);
        assert_activity_target(
            action,
            MechanicTarget::ActivityFrequency {
                occurrence_id: occurrence("occurrence:action"),
            },
            MechanicSurface::Frequency,
        );
        assert_activity_target(
            action,
            MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:action"),
                roll_id: "frightful-dc".to_string(),
            },
            MechanicSurface::Dc,
        );

        let spellcasting = activity(&projection.activities, "occurrence:spellcasting");
        assert_eq!(
            spellcasting.family,
            MechanicActivityFamily::SpellcastingEntry
        );
        assert_activity_target(
            spellcasting,
            MechanicTarget::SpellcastingAttack {
                entry_occurrence_id: occurrence("occurrence:spellcasting"),
            },
            MechanicSurface::AttackRoll,
        );
        assert_activity_target(
            spellcasting,
            MechanicTarget::SpellcastingDc {
                entry_occurrence_id: occurrence("occurrence:spellcasting"),
            },
            MechanicSurface::Dc,
        );
        assert_activity_target(
            spellcasting,
            MechanicTarget::SpellSlotMaximum {
                entry_occurrence_id: occurrence("occurrence:spellcasting"),
                rank: 5,
            },
            MechanicSurface::SpellSlot,
        );

        let spell = activity(&projection.activities, "occurrence:spell");
        assert_eq!(spell.family, MechanicActivityFamily::Spell);
        assert_activity_target(
            spell,
            MechanicTarget::ActivityActionCost {
                occurrence_id: occurrence("occurrence:spell"),
            },
            MechanicSurface::ActionEconomy,
        );
        assert_activity_target(
            spell,
            MechanicTarget::ActivityDamage {
                occurrence_id: occurrence("occurrence:spell"),
                damage_id: "spell-damage".to_string(),
            },
            MechanicSurface::Damage,
        );

        assert!(projection.unsupported.iter().any(|unsupported| {
            unsupported.target
                == Some(MechanicTarget::Movement {
                    speed_id: component("component:unsupported-speed"),
                })
                && matches!(unsupported.value, UnsupportedMechanicValue::Source(_))
        }));
        assert!(
            activity(&projection.activities, "occurrence:unsupported")
                .unsupported
                .iter()
                .any(|unsupported| {
                    unsupported.activity_occurrence_id == Some(occurrence("occurrence:unsupported"))
                        && matches!(
                            unsupported.value,
                            UnsupportedMechanicValue::Capability { .. }
                        )
                })
        );
    }

    #[test]
    fn canonical_targets_use_typed_identity_without_prefix_inference() {
        let projection = project_creature_mechanics(&mechanics_creature());

        assert!(projection.facts.iter().any(|fact| {
            fact.label == "save.fortitude"
                && fact.target
                    == MechanicTarget::CreatureSkill {
                        skill_id: component("component:arcana"),
                        kind: CreatureSkillKind::Arcana,
                    }
        }));
        assert!(!projection.facts.iter().any(|fact| {
            matches!(
                fact.target,
                MechanicTarget::Skill { .. } | MechanicTarget::AbilityModifier { .. }
            )
        }));

        let strike = activity(&projection.activities, "occurrence:strike");
        let deceptive_roll = fact(
            &strike.facts,
            &MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:strike"),
                roll_id: "prefix.skill.deception".to_string(),
            },
        )
        .expect("typed strike roll target");
        assert_eq!(deceptive_roll.facets.family, MechanicSourceFamily::Strike);
        assert_eq!(deceptive_roll.facets.surface, MechanicSurface::AttackRoll);
    }

    #[test]
    fn lore_and_equipment_notes_survive_without_duplicate_canonical_facts() {
        let projection = project_creature_mechanics(&mechanics_creature());

        for (occurrence_id, source_path, value) in [
            (
                "occurrence:equipment",
                "items.equipment.unsupported",
                "equipment-note",
            ),
            ("occurrence:lore", "items.lore.unsupported", "lore-note"),
        ] {
            let preserved = projection
                .unsupported
                .iter()
                .find(|unsupported| unsupported.source_path == source_path)
                .expect("unsupported-only occurrence note");
            assert_eq!(
                preserved.activity_occurrence_id,
                Some(occurrence(occurrence_id))
            );
            assert_eq!(preserved.target, None);
            assert!(matches!(
                &preserved.value,
                UnsupportedMechanicValue::Note(UnsupportedMechanicNote {
                    source_path: note_path,
                    value: UnsupportedSourceValue { value: note_value, .. },
                }) if note_path == source_path && note_value == value
            ));
            assert!(
                !projection
                    .activities
                    .iter()
                    .any(|activity| activity.occurrence_id == occurrence(occurrence_id))
            );
        }

        assert_eq!(
            projection
                .facts
                .iter()
                .filter(|fact| matches!(fact.target, MechanicTarget::CreatureSkill { .. }))
                .count(),
            1,
            "Lore occurrence notes must not duplicate the canonical Lore/skill owner"
        );
    }

    #[test]
    fn source_mutations_change_values_without_changing_target_identity() {
        let mut creature = mechanics_creature();
        let before = project_creature_mechanics(&creature);

        let FactValue::Value(defenses) = &mut creature.defenses.value else {
            panic!("fixture defenses");
        };
        let FactValue::Value(armor_class) = &mut defenses.armor_class else {
            panic!("fixture armor class");
        };
        armor_class.value = FactValue::Value(31);

        let FactValue::Value(skills) = &mut creature.skills.value else {
            panic!("fixture skills");
        };
        skills[0].label = "prefix.activity.roll.changed".to_string();
        let FactValue::Value(movement) = &mut creature.movement.value else {
            panic!("fixture movement");
        };
        movement[0].label = FactValue::Value("prefix.resource.changed".to_string());
        let FactValue::Value(resources) = &mut creature.resources.value else {
            panic!("fixture resources");
        };
        resources[0].label = "prefix.spellcasting.changed".to_string();

        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("fixture embedded entities");
        };
        for entity in &mut embedded.entities {
            entity.label = format!("prefix.changed.{}", entity.id.as_str());
        }
        let strike = embedded
            .occurrences
            .iter_mut()
            .find(|entry| entry.id == occurrence("occurrence:strike"))
            .expect("fixture strike");
        let CreatureCapability::Strike(strike) = &mut strike.capability else {
            panic!("fixture strike capability");
        };
        strike.action_cost = CreatureActionCost::Actions(2);
        strike.rolls[0].value = FactValue::Value(24);
        strike.rolls[0].label = "prefix.damage.changed".to_string();
        let FactValue::Value(damage) = &mut strike.damage else {
            panic!("fixture strike damage");
        };
        damage[0].formula = FactValue::Value("3d8+8".to_string());

        let after = project_creature_mechanics(&creature);
        assert_eq!(projection_targets(&before), projection_targets(&after));
        assert_eq!(
            fact(&after.facts, &MechanicTarget::ArmorClass).map(|fact| &fact.value),
            Some(&MechanicBaseValue::Integer(FactValue::Value(31)))
        );

        let strike = activity(&after.activities, "occurrence:strike");
        assert_eq!(
            fact(
                &strike.facts,
                &MechanicTarget::ActivityActionCost {
                    occurrence_id: occurrence("occurrence:strike"),
                },
            )
            .map(|fact| &fact.value),
            Some(&MechanicBaseValue::ActionCost(CreatureActionCost::Actions(
                2
            )))
        );
        let roll = fact(
            &strike.facts,
            &MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:strike"),
                roll_id: "prefix.skill.deception".to_string(),
            },
        )
        .expect("mutated roll");
        assert!(matches!(
            &roll.value,
            MechanicBaseValue::Roll(CreatureRoll {
                value: FactValue::Value(24),
                ..
            })
        ));
        let damage = fact(
            &strike.facts,
            &MechanicTarget::ActivityDamage {
                occurrence_id: occurrence("occurrence:strike"),
                damage_id: "prefix.hp.max".to_string(),
            },
        )
        .expect("mutated damage");
        assert!(matches!(
            &damage.value,
            MechanicBaseValue::Damage(CreatureDamage {
                formula: FactValue::Value(formula),
                ..
            }) if formula == "3d8+8"
        ));
    }

    #[test]
    fn typed_identity_mutations_change_projected_targets() {
        let mut creature = mechanics_creature();
        let before = project_creature_mechanics(&creature);

        let FactValue::Value(skills) = &mut creature.skills.value else {
            panic!("fixture skills");
        };
        skills[0].id = component("component:arcana/mutated");
        let FactValue::Value(movement) = &mut creature.movement.value else {
            panic!("fixture movement");
        };
        movement[0].id = component("component:land-speed/mutated");
        let FactValue::Value(resources) = &mut creature.resources.value else {
            panic!("fixture resources");
        };
        resources[0].id = component("component:focus-points/mutated");

        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("fixture embedded entities");
        };
        let strike = embedded
            .occurrences
            .iter_mut()
            .find(|entry| entry.id == occurrence("occurrence:strike"))
            .expect("fixture strike");
        strike.id = occurrence("occurrence:strike/mutated");
        let CreatureCapability::Strike(capability) = &mut strike.capability else {
            panic!("fixture strike capability");
        };
        capability.rolls[0].id = "roll/mutated".to_string();
        let FactValue::Value(damage) = &mut capability.damage else {
            panic!("fixture strike damage");
        };
        damage[0].id = "damage/mutated".to_string();

        let after = project_creature_mechanics(&creature);
        assert_ne!(projection_targets(&before), projection_targets(&after));
        for target in [
            MechanicTarget::CreatureSkill {
                skill_id: component("component:arcana/mutated"),
                kind: CreatureSkillKind::Arcana,
            },
            MechanicTarget::Movement {
                speed_id: component("component:land-speed/mutated"),
            },
            MechanicTarget::ResourceMaximum {
                resource_id: component("component:focus-points/mutated"),
            },
        ] {
            assert!(fact(&after.facts, &target).is_some(), "missing {target:?}");
        }
        let strike = activity(&after.activities, "occurrence:strike/mutated");
        for target in [
            MechanicTarget::ActivityActionCost {
                occurrence_id: occurrence("occurrence:strike/mutated"),
            },
            MechanicTarget::ActivityRoll {
                occurrence_id: occurrence("occurrence:strike/mutated"),
                roll_id: "roll/mutated".to_string(),
            },
            MechanicTarget::ActivityDamage {
                occurrence_id: occurrence("occurrence:strike/mutated"),
                damage_id: "damage/mutated".to_string(),
            },
        ] {
            assert!(fact(&strike.facts, &target).is_some(), "missing {target:?}");
        }
    }

    #[test]
    fn presence_states_and_numeric_zero_remain_distinct_in_base_facts() {
        for value in [FactValue::Missing, FactValue::Null, FactValue::Value(0)] {
            let mut creature = mechanics_creature();
            let FactValue::Value(perception) = &mut creature.perception.value else {
                panic!("fixture perception");
            };
            perception.modifier = value.clone();

            let projection = project_creature_mechanics(&creature);
            assert_eq!(
                fact(&projection.facts, &MechanicTarget::Perception).map(|fact| &fact.value),
                Some(&MechanicBaseValue::Integer(value))
            );
        }

        for absent in [FactValue::Missing, FactValue::Null] {
            let mut creature = mechanics_creature();
            creature.perception.value = absent;
            let projection = project_creature_mechanics(&creature);
            assert!(fact(&projection.facts, &MechanicTarget::Perception).is_none());
        }

        for maximum in [
            FactValue::Missing,
            FactValue::Null,
            FactValue::Value(CreatureResourceAmount::Integer(0)),
        ] {
            let mut creature = mechanics_creature();
            let FactValue::Value(resources) = &mut creature.resources.value else {
                panic!("fixture resources");
            };
            resources[0].maximum = maximum.clone();
            let target = MechanicTarget::ResourceMaximum {
                resource_id: resources[0].id.clone(),
            };
            let projection = project_creature_mechanics(&creature);
            assert_eq!(
                fact(&projection.facts, &target).map(|fact| &fact.value),
                Some(&MechanicBaseValue::ResourceAmount(maximum))
            );
        }
    }

    #[test]
    fn activity_and_inner_fact_order_are_repeatable_and_preserve_authored_input() {
        let creature = mechanics_creature();
        let first = project_creature_mechanics(&creature);
        let second = project_creature_mechanics(&creature);
        assert_eq!(first, second);

        let expected = [
            ("occurrence:strike", 0),
            ("occurrence:action", 1),
            ("occurrence:spellcasting", 2),
            ("occurrence:spell", 3),
            ("occurrence:unsupported", 4),
        ];
        assert_eq!(
            first
                .activities
                .iter()
                .map(|activity| (activity.occurrence_id.as_str(), activity.authored_order))
                .collect::<Vec<_>>(),
            expected
        );
        assert!(first.activities.iter().all(|activity| {
            activity
                .facts
                .windows(2)
                .all(|pair| pair[0].target <= pair[1].target)
        }));

        let mut reversed = creature;
        let FactValue::Value(embedded) = &mut reversed.embedded_entities.value else {
            panic!("fixture embedded entities");
        };
        embedded.occurrences.reverse();
        let reversed = project_creature_mechanics(&reversed);
        assert_eq!(
            reversed
                .activities
                .iter()
                .map(|activity| (activity.occurrence_id.as_str(), activity.authored_order))
                .collect::<Vec<_>>(),
            expected.into_iter().rev().collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_unsupported_branch_retains_exact_target_occurrence_and_source_path() {
        let projection = project_creature_mechanics(&unsupported_mechanics_creature());

        for (source_path, target, occurrence_id, value) in [
            (
                "defenses.hit_points.value",
                Some(MechanicTarget::MaxHp),
                None,
                "unsupported-hp",
            ),
            (
                "movement.component:unsupported-speed.mode",
                Some(MechanicTarget::Movement {
                    speed_id: component("component:unsupported-speed"),
                }),
                None,
                "teleport",
            ),
            (
                "resources.component:focus-points.maximum",
                Some(MechanicTarget::ResourceMaximum {
                    resource_id: component("component:focus-points"),
                }),
                None,
                "unsupported-resource",
            ),
            (
                "embedded_entities.actor_spellcasting.rituals_dc",
                Some(MechanicTarget::ActorRitualDc),
                None,
                "unsupported-ritual-dc",
            ),
            (
                "activities.occurrence:strike.action_cost",
                Some(MechanicTarget::ActivityActionCost {
                    occurrence_id: occurrence("occurrence:strike"),
                }),
                Some("occurrence:strike"),
                "unsupported-action-cost",
            ),
            (
                "activities.occurrence:strike.damage.prefix.hp.max.kinds",
                Some(MechanicTarget::ActivityDamage {
                    occurrence_id: occurrence("occurrence:strike"),
                    damage_id: "prefix.hp.max".to_string(),
                }),
                Some("occurrence:strike"),
                "unsupported-damage-kind",
            ),
            (
                "activities.occurrence:strike.damage.prefix.hp.max.apply_modifier",
                Some(MechanicTarget::ActivityDamage {
                    occurrence_id: occurrence("occurrence:strike"),
                    damage_id: "prefix.hp.max".to_string(),
                }),
                Some("occurrence:strike"),
                "unsupported-apply-modifier",
            ),
            (
                "activities.occurrence:spell.spell.defense.save",
                None,
                Some("occurrence:spell"),
                "unsupported-spell-save",
            ),
            (
                "activities.occurrence:spellcasting.spellcasting.preparation",
                None,
                Some("occurrence:spellcasting"),
                "unsupported-preparation",
            ),
            (
                "activities.occurrence:spellcasting.spellcasting.slots.5.maximum",
                Some(MechanicTarget::SpellSlotMaximum {
                    entry_occurrence_id: occurrence("occurrence:spellcasting"),
                    rank: 5,
                }),
                Some("occurrence:spellcasting"),
                "unsupported-slot-maximum",
            ),
            (
                "activities.occurrence:spellcasting.spellcasting.slots.5.serialized_value",
                Some(MechanicTarget::SpellSlotMaximum {
                    entry_occurrence_id: occurrence("occurrence:spellcasting"),
                    rank: 5,
                }),
                Some("occurrence:spellcasting"),
                "unsupported-slot-value",
            ),
        ] {
            let unsupported = unsupported_at(&projection, source_path);
            assert_eq!(unsupported.target, target, "target for {source_path}");
            assert_eq!(
                unsupported.activity_occurrence_id,
                occurrence_id.map(occurrence),
                "occurrence for {source_path}"
            );
            assert_eq!(
                unsupported.value,
                UnsupportedMechanicValue::Source(unsupported_value(value)),
                "value for {source_path}"
            );
        }

        let drift = unsupported_at(&projection, "resources.component:focus-points.source_drift");
        assert_eq!(
            drift.target,
            Some(MechanicTarget::ResourceMaximum {
                resource_id: component("component:focus-points"),
            })
        );
        assert_eq!(drift.activity_occurrence_id, None);
        assert_eq!(
            drift.value,
            UnsupportedMechanicValue::ResourceDrift(CreatureUnsupportedSourceFact {
                field: CreatureUnsupportedSourceField::ResourceMaximumDrift,
                value: unsupported_value("unsupported-resource-drift"),
            })
        );

        let prepared = unsupported_at(
            &projection,
            "activities.occurrence:spellcasting.spellcasting.slots.5.prepared",
        );
        assert_eq!(
            prepared.target,
            Some(MechanicTarget::SpellSlotMaximum {
                entry_occurrence_id: occurrence("occurrence:spellcasting"),
                rank: 5,
            })
        );
        assert_eq!(
            prepared.activity_occurrence_id,
            Some(occurrence("occurrence:spellcasting"))
        );
        assert_eq!(
            prepared.value,
            UnsupportedMechanicValue::PreparedSlot(unsupported_value("unsupported-prepared-slot"))
        );

        for (source_path, occurrence_id, value) in [
            ("actor.note", None, "actor-note"),
            ("strike.note", Some("occurrence:strike"), "strike-note"),
            ("action.note", Some("occurrence:action"), "action-note"),
            (
                "spellcasting.note",
                Some("occurrence:spellcasting"),
                "spellcasting-note",
            ),
            ("spell.note", Some("occurrence:spell"), "spell-note"),
            (
                "items.mystery.system",
                Some("occurrence:unsupported"),
                "unknown-system",
            ),
            (
                "items.equipment.unsupported",
                Some("occurrence:equipment"),
                "equipment-note",
            ),
            (
                "items.lore.unsupported",
                Some("occurrence:lore"),
                "lore-note",
            ),
        ] {
            let unsupported = unsupported_at(&projection, source_path);
            assert_eq!(unsupported.target, None, "note target for {source_path}");
            assert_eq!(
                unsupported.activity_occurrence_id,
                occurrence_id.map(occurrence),
                "note occurrence for {source_path}"
            );
            assert_eq!(
                unsupported.value,
                UnsupportedMechanicValue::Note(unsupported_note_fixture(source_path, value)),
                "note value for {source_path}"
            );
        }

        let capability = unsupported_at(&projection, "embedded_entities.occurrences.capability");
        assert_eq!(capability.target, None);
        assert_eq!(
            capability.activity_occurrence_id,
            Some(occurrence("occurrence:unsupported"))
        );
        assert_eq!(
            capability.value,
            UnsupportedMechanicValue::Capability {
                source_item_type: "mystery".to_string(),
                source_slug: FactValue::Value("mystery".to_string()),
            }
        );
    }

    fn unsupported_mechanics_creature() -> CreatureRecord {
        let mut creature = mechanics_creature();
        let FactValue::Value(defenses) = &mut creature.defenses.value else {
            panic!("fixture defenses");
        };
        let FactValue::Value(hit_points) = &mut defenses.hit_points else {
            panic!("fixture hit points");
        };
        hit_points.value = FactValue::Value(CreatureNumber::Unsupported(unsupported_value(
            "unsupported-hp",
        )));

        let FactValue::Value(resources) = &mut creature.resources.value else {
            panic!("fixture resources");
        };
        resources[0].maximum = FactValue::Value(CreatureResourceAmount::Unsupported(
            unsupported_value("unsupported-resource"),
        ));
        resources[0].source_drift = FactValue::Value(vec![CreatureUnsupportedSourceFact {
            field: CreatureUnsupportedSourceField::ResourceMaximumDrift,
            value: unsupported_value("unsupported-resource-drift"),
        }]);

        let FactValue::Value(embedded) = &mut creature.embedded_entities.value else {
            panic!("fixture embedded entities");
        };
        let FactValue::Value(actor_spellcasting) = &mut embedded.actor_spellcasting else {
            panic!("fixture actor spellcasting");
        };
        actor_spellcasting.rituals_dc = FactValue::Value(CreatureSourceScalar::Unsupported(
            unsupported_value("unsupported-ritual-dc"),
        ));
        actor_spellcasting.unsupported_notes =
            vec![unsupported_note_fixture("actor.note", "actor-note")];

        let CreatureCapability::Strike(strike) = capability_mut(embedded, "occurrence:strike")
        else {
            panic!("fixture strike");
        };
        strike.action_cost =
            CreatureActionCost::Unsupported(unsupported_value("unsupported-action-cost"));
        strike.unsupported_notes = vec![unsupported_note_fixture("strike.note", "strike-note")];
        let FactValue::Value(damage) = &mut strike.damage else {
            panic!("fixture strike damage");
        };
        damage[0].kinds = FactValue::Value(vec![CreatureDamageKind::Unsupported(
            unsupported_value("unsupported-damage-kind"),
        )]);
        damage[0].apply_modifier = FactValue::Value(CreatureSourceScalar::Unsupported(
            unsupported_value("unsupported-apply-modifier"),
        ));

        let CreatureCapability::Action(action) = capability_mut(embedded, "occurrence:action")
        else {
            panic!("fixture action");
        };
        action.unsupported_notes = vec![unsupported_note_fixture("action.note", "action-note")];

        let CreatureCapability::SpellcastingEntry(spellcasting) =
            capability_mut(embedded, "occurrence:spellcasting")
        else {
            panic!("fixture spellcasting");
        };
        spellcasting.preparation = FactValue::Value(CreatureSpellPreparation::Unsupported(
            unsupported_value("unsupported-preparation"),
        ));
        spellcasting.unsupported_notes = vec![unsupported_note_fixture(
            "spellcasting.note",
            "spellcasting-note",
        )];
        let FactValue::Value(slots) = &mut spellcasting.slots else {
            panic!("fixture spell slots");
        };
        slots[0].maximum = FactValue::Value(CreatureSourceScalar::Unsupported(unsupported_value(
            "unsupported-slot-maximum",
        )));
        slots[0].serialized_value = FactValue::Value(CreatureSourceScalar::Unsupported(
            unsupported_value("unsupported-slot-value"),
        ));
        slots[0].prepared = FactValue::Value(vec![CreaturePreparedSpellSlot::Unsupported(
            unsupported_value("unsupported-prepared-slot"),
        )]);

        let CreatureCapability::Spell(spell) = capability_mut(embedded, "occurrence:spell") else {
            panic!("fixture spell");
        };
        let FactValue::Value(defense) = &mut spell.defense else {
            panic!("fixture spell defense");
        };
        defense.save = FactValue::Value(CreatureSpellSave::Unsupported(unsupported_value(
            "unsupported-spell-save",
        )));
        spell.unsupported_notes = vec![unsupported_note_fixture("spell.note", "spell-note")];

        creature
    }

    fn mechanics_creature() -> CreatureRecord {
        let owner = record_key();
        CreatureRecord {
            identity: CreatureIdentity {
                record_key: owner.clone(),
                source_id: CreatureSourceId::new("test-creature").expect("source id"),
                name: "Test Creature".to_string(),
                family: CreatureFamily::Npc,
            },
            level: source(FactValue::Value(9), CreatureSourceField::Level),
            rarity: source(FactValue::Missing, CreatureSourceField::Rarity),
            traits: source(FactValue::Missing, CreatureSourceField::Traits),
            size: source(FactValue::Missing, CreatureSourceField::Size),
            publication: source(FactValue::Missing, CreatureSourceField::Publication),
            adjustment: source(FactValue::Missing, CreatureSourceField::Adjustment),
            source_alliance: source(FactValue::Missing, CreatureSourceField::SourceAlliance),
            perception: source(
                FactValue::Value(CreaturePerception {
                    modifier: FactValue::Value(18),
                    details: FactValue::Missing,
                    has_vision: FactValue::Missing,
                    senses: FactValue::Missing,
                }),
                CreatureSourceField::Perception,
            ),
            initiative: source(FactValue::Missing, CreatureSourceField::Initiative),
            languages: source(FactValue::Missing, CreatureSourceField::Languages),
            skills: source(
                FactValue::Value(vec![CreatureSkill {
                    id: component("component:arcana"),
                    authored_order: 0,
                    source_entries: vec![crate::CreatureSkillSourceEntry {
                        authored_key: "arcana".to_string(),
                        modifier: FactValue::Value(18),
                    }],
                    kind: CreatureSkillKind::Arcana,
                    label: "save.fortitude".to_string(),
                    modifier: FactValue::Value(18),
                    note: FactValue::Missing,
                    variants: FactValue::Missing,
                    source_item_id: FactValue::Missing,
                    unmodeled: FactValue::Missing,
                }]),
                CreatureSourceField::Skills,
            ),
            legacy_abilities: source(FactValue::Missing, CreatureSourceField::LegacyAbilities),
            defenses: source(
                FactValue::Value(CreatureDefenses {
                    armor_class: FactValue::Value(CreatureArmorClass {
                        value: FactValue::Value(28),
                        details: FactValue::Missing,
                    }),
                    hit_points: FactValue::Value(CreatureHitPoints {
                        value: FactValue::Value(CreatureNumber::Integer(170)),
                        maximum: FactValue::Value(170),
                        temporary: FactValue::Value(0),
                        temporary_maximum: FactValue::Missing,
                        details: FactValue::Missing,
                    }),
                    hardness: FactValue::Missing,
                    shield: FactValue::Missing,
                    saves: FactValue::Value(CreatureSaves {
                        fortitude: FactValue::Value(save(CreatureSaveKind::Fortitude, 19)),
                        reflex: FactValue::Value(save(CreatureSaveKind::Reflex, 17)),
                        will: FactValue::Value(save(CreatureSaveKind::Will, 18)),
                    }),
                    all_saves_note: FactValue::Missing,
                    immunities: FactValue::Missing,
                    resistances: FactValue::Missing,
                    weaknesses: FactValue::Missing,
                }),
                CreatureSourceField::Defenses,
            ),
            movement: source(
                FactValue::Value(vec![
                    speed(
                        "component:land-speed",
                        CreatureMovementMode::Land,
                        FactValue::Value(25),
                    ),
                    speed(
                        "component:unsupported-speed",
                        CreatureMovementMode::Unsupported(unsupported("teleport")),
                        FactValue::Null,
                    ),
                ]),
                CreatureSourceField::Movement,
            ),
            resources: source(
                FactValue::Value(vec![CreatureResource {
                    id: component("component:focus-points"),
                    authored_order: 0,
                    kind: CreatureResourceKind::new("focus").expect("resource kind"),
                    label: "Focus Points".to_string(),
                    maximum: FactValue::Value(CreatureResourceAmount::Integer(3)),
                    serialized_value: FactValue::Value(CreatureResourceAmount::Integer(2)),
                    source_drift: FactValue::Missing,
                    current_policy: ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
                }]),
                CreatureSourceField::Resources,
            ),
            embedded_entities: source(
                FactValue::Value(embedded_entities(&owner)),
                CreatureSourceField::EmbeddedEntities,
            ),
            content: crate::OwnedRichContent::default(),
            provenance: CreatureProvenance {
                source_path: "packs/test/test-creature.json".to_string(),
                source_contract_version: "test/v1".to_string(),
                source_system_version: "test".to_string(),
                source_upstream_commit: "test".to_string(),
            },
        }
    }

    fn embedded_entities(owner: &RecordKey) -> CreatureEmbeddedEntities {
        let strike = CreatureCapability::Strike(CreatureStrikeCapability {
            traits: FactValue::Missing,
            attack_effects: FactValue::Missing,
            rolls: vec![CreatureRoll {
                id: "prefix.skill.deception".to_string(),
                label: "hp.max".to_string(),
                kind: CreatureRollKind::Attack,
                value: FactValue::Value(20),
                ability: FactValue::<ActivityRollAbility>::Missing,
            }],
            damage: FactValue::Value(vec![damage("prefix.hp.max", "2d8+8")]),
            action_cost: CreatureActionCost::Actions(1),
            unsupported_notes: Vec::new(),
        });
        let action = CreatureCapability::Action(CreatureActionCapability {
            category: FactValue::Missing,
            traits: FactValue::Missing,
            action_cost: CreatureActionCost::Reaction,
            frequency: FactValue::Value(CreatureFrequency {
                maximum: FactValue::Value(1),
                period: FactValue::Value("day".to_string()),
                serialized_value: FactValue::Value(1),
            }),
            self_effect: FactValue::Missing,
            self_effect_label: FactValue::Missing,
            requirements: FactValue::Missing,
            cost: FactValue::Missing,
            rolls: vec![CreatureRoll {
                id: "frightful-dc".to_string(),
                label: "Frightful Presence DC".to_string(),
                kind: CreatureRollKind::DifficultyClass,
                value: FactValue::Value(28),
                ability: FactValue::Missing,
            }],
            damage: FactValue::Value(Vec::new()),
            unsupported_notes: Vec::new(),
        });
        let spellcasting =
            CreatureCapability::SpellcastingEntry(CreatureSpellcastingEntryCapability {
                preparation: FactValue::Value(CreatureSpellPreparation::Prepared),
                tradition: FactValue::Value("arcane".to_string()),
                attack: FactValue::Value(20),
                dc: FactValue::Value(28),
                slots: FactValue::Value(vec![CreatureSpellSlot {
                    rank: 5,
                    maximum: FactValue::Value(CreatureSourceScalar::Value(3)),
                    serialized_value: FactValue::Value(CreatureSourceScalar::Value(2)),
                    prepared: FactValue::Missing,
                }]),
                unsupported_notes: Vec::new(),
            });
        let spell = CreatureCapability::Spell(CreatureSpellCapability {
            traits: FactValue::Missing,
            base_rank: FactValue::Value(5),
            signature: FactValue::Missing,
            traditions: FactValue::Missing,
            requirements: FactValue::Missing,
            cost: FactValue::Missing,
            counteraction: FactValue::Missing,
            ritual: FactValue::Missing,
            target: FactValue::Missing,
            area: FactValue::Missing,
            range: FactValue::Missing,
            time: FactValue::Missing,
            duration: FactValue::Missing,
            defense: FactValue::Value(CreatureSpellDefense {
                save: FactValue::Value(CreatureSpellSave::Reflex),
                basic: FactValue::Value(true),
            }),
            damage: FactValue::Value(vec![damage("spell-damage", "6d6")]),
            action_cost: CreatureActionCost::Actions(2),
            unsupported_notes: Vec::new(),
        });
        let unsupported_capability =
            CreatureCapability::Unsupported(CreatureUnsupportedCapability {
                source_item_type: "mystery".to_string(),
                source_slug: FactValue::Value("mystery".to_string()),
                traits: FactValue::Missing,
                unsupported_notes: vec![UnsupportedMechanicNote {
                    source_path: "items.mystery.system".to_string(),
                    value: unsupported("unknown-system"),
                }],
            });
        let equipment = CreatureCapability::Equipment(CreatureEquipmentCapability {
            traits: FactValue::Missing,
            level: FactValue::Missing,
            usage: FactValue::Missing,
            quantity: FactValue::Missing,
            uses: FactValue::Missing,
            unsupported_notes: vec![unsupported_note_fixture(
                "items.equipment.unsupported",
                "equipment-note",
            )],
        });
        let lore = CreatureCapability::Lore(CreatureLoreCapability {
            modifier: FactValue::Value(18),
            unsupported_notes: vec![unsupported_note_fixture(
                "items.lore.unsupported",
                "lore-note",
            )],
        });
        let capabilities = [
            ("occurrence:strike", CreatureEntityFamily::Strike, strike),
            ("occurrence:action", CreatureEntityFamily::Action, action),
            (
                "occurrence:spellcasting",
                CreatureEntityFamily::SpellcastingEntry,
                spellcasting,
            ),
            ("occurrence:spell", CreatureEntityFamily::Spell, spell),
            (
                "occurrence:unsupported",
                CreatureEntityFamily::Unsupported,
                unsupported_capability,
            ),
            (
                "occurrence:equipment",
                CreatureEntityFamily::Equipment,
                equipment,
            ),
            ("occurrence:lore", CreatureEntityFamily::Lore, lore),
        ];
        let mut entities = Vec::new();
        let mut occurrences = Vec::new();
        for (authored_order, (id, family, capability)) in capabilities.into_iter().enumerate() {
            let occurrence_id = occurrence(id);
            let entity_id = CreatureEntityId::new(format!("entity:{id}")).expect("entity id");
            let source_identity = source_identity();
            entities.push(CreatureEntity {
                id: entity_id.clone(),
                family,
                label: id.to_string(),
                source_identity: source_identity.clone(),
            });
            occurrences.push(CreatureEntityOccurrence {
                id: occurrence_id,
                identity_stability: OccurrenceIdentityStability::StableNestedSourceId,
                owner: owner.clone(),
                target: CreatureEntityTarget::ActorOwned(entity_id),
                family,
                authored_order: authored_order as u32,
                source_sort: FactValue::Missing,
                source_folder: FactValue::Missing,
                source_identity,
                parent: CreatureOccurrenceParent::Creature,
                context: CreatureOccurrenceContext::default(),
                capability,
                deltas: Vec::new(),
            });
        }
        CreatureEmbeddedEntities {
            entities,
            occurrences,
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Value(CreatureActorSpellcastingContext {
                rituals_dc: FactValue::Value(CreatureSourceScalar::Value(27)),
                unsupported_notes: Vec::new(),
            }),
        }
    }

    fn source_identity() -> CreatureEntitySourceIdentity {
        CreatureEntitySourceIdentity {
            nested_source_id: FactValue::Missing,
            stable_source_locator: FactValue::Missing,
            source_locators: Vec::new(),
        }
    }

    fn damage(id: &str, formula: &str) -> CreatureDamage {
        CreatureDamage {
            id: id.to_string(),
            formula: FactValue::Value(formula.to_string()),
            damage_type: FactValue::Value("mental".to_string()),
            category: FactValue::Missing,
            kinds: FactValue::Value(vec![CreatureDamageKind::Damage]),
            apply_modifier: FactValue::Value(CreatureSourceScalar::Value(true)),
        }
    }

    fn save(kind: CreatureSaveKind, value: i64) -> CreatureSave {
        CreatureSave {
            id: component(format!("component:save:{}", kind_slug(kind))),
            kind,
            value: FactValue::Value(value),
            details: FactValue::Missing,
        }
    }

    const fn kind_slug(kind: CreatureSaveKind) -> &'static str {
        match kind {
            CreatureSaveKind::Fortitude => "fortitude",
            CreatureSaveKind::Reflex => "reflex",
            CreatureSaveKind::Will => "will",
        }
    }

    fn speed(id: &str, mode: CreatureMovementMode, value: FactValue<i64>) -> CreatureSpeed {
        CreatureSpeed {
            id: component(id),
            authored_order: 0,
            mode,
            value,
            label: FactValue::Missing,
            details: FactValue::Missing,
        }
    }

    fn unsupported(value: &str) -> UnsupportedSourceValue {
        unsupported_value(value)
    }

    fn unsupported_value(value: &str) -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::String,
            value: value.to_string(),
            reason: UnsupportedSourceReason::OpenVocabulary,
        }
    }

    fn unsupported_note_fixture(source_path: &str, value: &str) -> UnsupportedMechanicNote {
        UnsupportedMechanicNote {
            source_path: source_path.to_string(),
            value: unsupported(value),
        }
    }

    fn source<T>(value: FactValue<T>, field: CreatureSourceField) -> CreatureFact<T> {
        CreatureFact::source(value, field)
    }

    fn component(id: impl Into<String>) -> CreatureComponentId {
        CreatureComponentId::new(id).expect("component id")
    }

    fn occurrence(id: &str) -> CreatureOccurrenceId {
        CreatureOccurrenceId::new(id).expect("occurrence id")
    }

    fn record_key() -> RecordKey {
        RecordKey::new(
            PackName::new("test-pack").expect("pack"),
            RecordId::new("test-creature").expect("record"),
        )
    }

    fn fact<'a>(facts: &'a [MechanicFact], target: &MechanicTarget) -> Option<&'a MechanicFact> {
        facts.iter().find(|fact| &fact.target == target)
    }

    fn activity<'a>(
        activities: &'a [CanonicalMechanicActivity],
        id: &str,
    ) -> &'a CanonicalMechanicActivity {
        activities
            .iter()
            .find(|activity| activity.occurrence_id == occurrence(id))
            .expect("activity")
    }

    fn capability_mut<'a>(
        embedded: &'a mut CreatureEmbeddedEntities,
        id: &str,
    ) -> &'a mut CreatureCapability {
        &mut embedded
            .occurrences
            .iter_mut()
            .find(|entry| entry.id == occurrence(id))
            .expect("fixture occurrence")
            .capability
    }

    fn unsupported_at<'a>(
        projection: &'a super::CanonicalMechanicsProjection,
        source_path: &str,
    ) -> &'a UnsupportedMechanic {
        projection
            .unsupported
            .iter()
            .chain(
                projection
                    .activities
                    .iter()
                    .flat_map(|activity| &activity.unsupported),
            )
            .find(|unsupported| unsupported.source_path == source_path)
            .unwrap_or_else(|| panic!("missing unsupported mechanic at {source_path}"))
    }

    fn assert_activity_target(
        activity: &CanonicalMechanicActivity,
        target: MechanicTarget,
        surface: MechanicSurface,
    ) {
        let fact = fact(&activity.facts, &target).unwrap_or_else(|| panic!("missing {target:?}"));
        assert_eq!(fact.facets.surface, surface);
    }

    fn projection_targets(
        projection: &super::CanonicalMechanicsProjection,
    ) -> (
        Vec<MechanicTarget>,
        Vec<(CreatureOccurrenceId, Vec<MechanicTarget>)>,
    ) {
        (
            projection
                .facts
                .iter()
                .map(|fact| fact.target.clone())
                .collect(),
            projection
                .activities
                .iter()
                .map(|activity| {
                    (
                        activity.occurrence_id.clone(),
                        activity
                            .facts
                            .iter()
                            .map(|fact| fact.target.clone())
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}
