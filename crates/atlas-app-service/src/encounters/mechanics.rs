use std::collections::{BTreeMap, BTreeSet};

use atlas_app_model::{
    EncounterParticipantVariantView, EncounterRuntimeAbilitiesView,
    EncounterRuntimeActionBudgetView, EncounterRuntimeActionCostKindView,
    EncounterRuntimeActionCostView, EncounterRuntimeActivityKindView,
    EncounterRuntimeActivityUsageView, EncounterRuntimeActivityView,
    EncounterRuntimeAutomationLimitationCodeView, EncounterRuntimeAutomationLimitationTargetView,
    EncounterRuntimeAutomationLimitationView, EncounterRuntimeAwarenessView,
    EncounterRuntimeConditionView, EncounterRuntimeDefensesView, EncounterRuntimeFrequencyView,
    EncounterRuntimeHazardInitiativeStatisticView, EncounterRuntimeHazardInitiativeSuggestionView,
    EncounterRuntimeHazardStateView, EncounterRuntimeHazardView, EncounterRuntimeMovementView,
    EncounterRuntimeResourceView, EncounterRuntimeSavesView, EncounterRuntimeSkillKindView,
    EncounterRuntimeSkillView, EncounterRuntimeSpellSlotView, EncounterRuntimeSpellcastingView,
    EncounterRuntimeUsesView, EncounterRuntimeView, EncounterRuntimeVitalsView,
    RuntimeAbilityKindView, RuntimeAdjustmentView, RuntimeCanonicalTargetView,
    RuntimeCapabilityView, RuntimeCountSegmentView, RuntimeCountView, RuntimeDamageEffectKindView,
    RuntimeDistanceView, RuntimeEffectNoteView, RuntimeFactProvenanceView, RuntimeFactSourceView,
    RuntimeFormulaView, RuntimeModifierView, RuntimeNumberView, RuntimeRollSurfaceView,
    RuntimeRollView, RuntimeRuleView, RuntimeSaveKindView, StatModifierTypeView,
};
use atlas_local_state::{
    EncounterParticipant, EncounterParticipantCondition, ParticipantHazardState, ParticipantVariant,
};
use atlas_record::{
    AbilityKind, ActivityRollAbility, CanonicalMechanicActivity, CanonicalMechanicsProjection,
    CreatureActionCost, CreatureDamage, CreatureDamageKind, CreatureFrequency, CreatureNumber,
    CreatureResourceAmount, CreatureRoll, CreatureRollKind, CreatureSourceScalar, CreatureUseLimit,
    DamageEffectKind, FactValue, HazardActionType, HazardCapability, HazardFrequencyInterval,
    HazardInitiativeStatistic, HazardRecord, MechanicActivityFamily, MechanicBaseValue,
    MechanicFact, MechanicSurface, MechanicTarget, RecordBody, RetrievedRecord, SaveKind,
    UnsupportedMechanic, UnsupportedMechanicValue, UnsupportedSourceReason, UnsupportedSourceShape,
    UnsupportedSourceValue, project_creature_mechanics, project_hazard_conveniences,
};

use super::conditions::{ConditionRule, condition_rule_for_key};
use super::projection::participant_variant_view;

#[derive(Debug, Clone)]
struct EncounterMechanicsInput {
    level: Option<i64>,
    values: Vec<EncounterMechanicValue>,
    speeds: Vec<EncounterMovementInput>,
    activities: Vec<EncounterActivityInput>,
}

#[derive(Debug, Clone)]
struct EncounterMechanicValue {
    target: MechanicTarget,
    label: String,
    base_value: EncounterMechanicScalar,
    facets: atlas_record::MechanicFacets,
}

#[derive(Debug, Clone, Copy)]
enum EncounterMechanicScalar {
    Number(i64),
}

#[derive(Debug, Clone)]
struct EncounterMovementInput {
    movement_type: String,
    label: String,
    value_feet: i64,
}

#[derive(Debug, Clone)]
struct EncounterActivityInput {
    activity_id: String,
    label: String,
    kind: EncounterActivityKind,
    usage: EncounterActivityUsage,
    rolls: Vec<EncounterActivityRoll>,
    damage: Vec<EncounterDamageExpression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncounterActivityKind {
    Strike,
    Spell,
    Other,
}

#[derive(Debug, Clone, Copy)]
enum EncounterActivityUsage {
    Unlimited,
    Limited,
    Ambiguous,
}

#[derive(Debug, Clone)]
struct EncounterActivityRoll {
    roll_id: String,
    label: String,
    base_value: i64,
    surface: EncounterActivityRollSurface,
    ability: Option<ActivityRollAbility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncounterActivityRollSurface {
    AttackRoll,
    Dc,
}

#[derive(Debug, Clone)]
struct EncounterDamageExpression {
    damage_id: String,
    label: Option<String>,
    formula: String,
    damage_type: Option<String>,
    effect_kind: DamageEffectKind,
    ability: Option<ActivityRollAbility>,
}

#[derive(Debug, Clone)]
struct CandidateModifier {
    target: MechanicTarget,
    provenance: RuntimeFactProvenanceView,
    label: String,
    modifier_type: StatModifierTypeView,
    value: i64,
}

#[derive(Debug, Clone)]
struct RollModifier {
    provenance: RuntimeFactProvenanceView,
    label: String,
    modifier_type: StatModifierTypeView,
    value: i64,
}

#[derive(Debug, Clone)]
struct RuntimeAdjustment {
    provenance: RuntimeFactProvenanceView,
    source: String,
    label: String,
    value: i64,
    reason: Option<String>,
    floor: Option<i64>,
}

#[derive(Debug, Clone)]
struct RuntimeNote {
    provenance: RuntimeFactProvenanceView,
    label: String,
    reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncounterProjectionDiagnosticCode {
    DuplicateRuntimeFact,
    UnavailableCanonicalFact,
    UnmappedCanonicalFact,
    UnmatchedRuntimeModifier,
    UnsupportedCanonicalFact,
}

#[derive(Debug, Clone)]
struct EncounterProjectionDiagnostic {
    code: EncounterProjectionDiagnosticCode,
    message: String,
    canonical_target: Option<RuntimeCanonicalTargetView>,
}

struct EncounterRuntimeProjection {
    runtime: EncounterRuntimeView,
    diagnostics: Vec<EncounterProjectionDiagnostic>,
}

fn into_public_runtime(projection: EncounterRuntimeProjection) -> EncounterRuntimeView {
    for diagnostic in projection.diagnostics {
        let EncounterProjectionDiagnostic {
            code,
            message,
            canonical_target,
        } = diagnostic;
        let _internal_only = (code, message, canonical_target);
    }
    projection.runtime
}

fn fact_provenance(
    source: RuntimeFactSourceView,
    canonical_target: Option<RuntimeCanonicalTargetView>,
) -> RuntimeFactProvenanceView {
    RuntimeFactProvenanceView {
        source,
        canonical_target,
    }
}

fn canonical_provenance(
    target: &MechanicTarget,
    override_target: Option<RuntimeCanonicalTargetView>,
) -> RuntimeFactProvenanceView {
    fact_provenance(
        RuntimeFactSourceView::CanonicalRecord,
        override_target.or_else(|| canonical_target_view(target)),
    )
}

fn participant_state_provenance() -> RuntimeFactProvenanceView {
    fact_provenance(RuntimeFactSourceView::ParticipantState, None)
}

fn variant_provenance(variant: ParticipantVariant) -> RuntimeFactProvenanceView {
    fact_provenance(
        RuntimeFactSourceView::ParticipantVariant {
            variant: participant_variant_view(variant),
        },
        None,
    )
}

fn condition_provenance(condition: &EncounterParticipantCondition) -> RuntimeFactProvenanceView {
    fact_provenance(
        RuntimeFactSourceView::Condition {
            condition_id: condition.condition_id,
            condition_ref: condition
                .condition_key
                .clone()
                .unwrap_or_else(|| condition.name.clone()),
            label: condition_source(condition),
        },
        None,
    )
}

fn runtime_rule_provenance(rule: RuntimeRuleView) -> RuntimeFactProvenanceView {
    fact_provenance(RuntimeFactSourceView::RuntimeRule { rule }, None)
}

fn canonical_target_view(target: &MechanicTarget) -> Option<RuntimeCanonicalTargetView> {
    Some(match target {
        MechanicTarget::ArmorClass => RuntimeCanonicalTargetView::ArmorClass,
        MechanicTarget::MaxHp => RuntimeCanonicalTargetView::MaximumHp,
        MechanicTarget::Perception => RuntimeCanonicalTargetView::Perception,
        MechanicTarget::Save { save } => RuntimeCanonicalTargetView::Save {
            save: match save {
                SaveKind::Fortitude => RuntimeSaveKindView::Fortitude,
                SaveKind::Reflex => RuntimeSaveKindView::Reflex,
                SaveKind::Will => RuntimeSaveKindView::Will,
            },
        },
        MechanicTarget::AbilityModifier { ability } => {
            RuntimeCanonicalTargetView::AbilityModifier {
                ability: match ability {
                    AbilityKind::Strength => RuntimeAbilityKindView::Strength,
                    AbilityKind::Dexterity => RuntimeAbilityKindView::Dexterity,
                    AbilityKind::Constitution => RuntimeAbilityKindView::Constitution,
                    AbilityKind::Intelligence => RuntimeAbilityKindView::Intelligence,
                    AbilityKind::Wisdom => RuntimeAbilityKindView::Wisdom,
                    AbilityKind::Charisma => RuntimeAbilityKindView::Charisma,
                },
            }
        }
        MechanicTarget::CreatureSkill { skill_id, .. } => RuntimeCanonicalTargetView::Skill {
            skill_id: skill_id.as_str().to_string(),
        },
        MechanicTarget::Movement { speed_id } => RuntimeCanonicalTargetView::Movement {
            speed_id: speed_id.as_str().to_string(),
        },
        MechanicTarget::ResourceMaximum { resource_id } => {
            RuntimeCanonicalTargetView::ResourceMaximum {
                resource_id: resource_id.as_str().to_string(),
            }
        }
        MechanicTarget::ActivityActionCost { occurrence_id } => {
            RuntimeCanonicalTargetView::ActivityActionCost {
                activity_id: occurrence_id.as_str().to_string(),
            }
        }
        MechanicTarget::ActivityFrequency { occurrence_id } => {
            RuntimeCanonicalTargetView::ActivityFrequency {
                activity_id: occurrence_id.as_str().to_string(),
            }
        }
        MechanicTarget::ActivityUses { occurrence_id } => {
            RuntimeCanonicalTargetView::ActivityUses {
                activity_id: occurrence_id.as_str().to_string(),
            }
        }
        MechanicTarget::ActivityRoll {
            occurrence_id,
            roll_id,
        } => RuntimeCanonicalTargetView::ActivityRoll {
            activity_id: occurrence_id.as_str().to_string(),
            roll_id: roll_id.clone(),
        },
        MechanicTarget::ActivityDamage {
            occurrence_id,
            damage_id,
        } => RuntimeCanonicalTargetView::ActivityDamage {
            activity_id: occurrence_id.as_str().to_string(),
            damage_id: damage_id.clone(),
        },
        MechanicTarget::SpellcastingAttack {
            entry_occurrence_id,
        } => RuntimeCanonicalTargetView::SpellcastingAttack {
            entry_id: entry_occurrence_id.as_str().to_string(),
        },
        MechanicTarget::SpellcastingDc {
            entry_occurrence_id,
        } => RuntimeCanonicalTargetView::SpellcastingDc {
            entry_id: entry_occurrence_id.as_str().to_string(),
        },
        MechanicTarget::SpellSlotMaximum {
            entry_occurrence_id,
            rank,
        } => RuntimeCanonicalTargetView::SpellSlotMaximum {
            entry_id: entry_occurrence_id.as_str().to_string(),
            rank: *rank,
        },
        MechanicTarget::ActorRitualDc => RuntimeCanonicalTargetView::RitualDc,
    })
}

fn participant_number(label: &str, value: i64) -> RuntimeNumberView {
    RuntimeNumberView {
        label: label.to_string(),
        base_value: value,
        adjusted_value: value,
        modifiers: Vec::new(),
        suppressed_modifiers: Vec::new(),
        provenance: participant_state_provenance(),
    }
}

fn runtime_condition_view(
    condition: &EncounterParticipantCondition,
) -> EncounterRuntimeConditionView {
    EncounterRuntimeConditionView {
        condition_id: condition.condition_id,
        condition_key: condition.condition_key.clone(),
        name: condition.name.clone(),
        value: condition.value,
        source_participant_key: condition.source_participant_key.clone(),
        duration_rounds: condition.duration_rounds,
        note: condition.note.clone(),
        created_at: condition.created_at.clone(),
        updated_at: condition.updated_at.clone(),
        provenance: condition_provenance(condition),
    }
}

fn set_named_fact(
    slot: &mut Option<RuntimeNumberView>,
    fact: RuntimeNumberView,
    target: &MechanicTarget,
    target_count: usize,
    diagnostics: &mut Vec<EncounterProjectionDiagnostic>,
) {
    if target_count == 1 {
        debug_assert!(slot.is_none(), "pre-counted named runtime target is unique");
        *slot = Some(fact);
        return;
    }

    *slot = None;
    diagnostics.push(duplicate_named_runtime_fact(
        fact.label.as_str(),
        target,
        target_count,
    ));
}

fn set_spellcasting_roll(
    slot: &mut Option<RuntimeRollView>,
    fact: RuntimeRollView,
    target: &MechanicTarget,
    target_count: usize,
    diagnostics: &mut Vec<EncounterProjectionDiagnostic>,
) {
    if target_count == 1 {
        debug_assert!(
            slot.is_none(),
            "pre-counted spellcasting runtime target is unique"
        );
        *slot = Some(fact);
        return;
    }

    *slot = None;
    diagnostics.push(duplicate_named_runtime_fact(
        fact.label.as_str(),
        target,
        target_count,
    ));
}

fn duplicate_named_runtime_fact(
    label: &str,
    target: &MechanicTarget,
    count: usize,
) -> EncounterProjectionDiagnostic {
    EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::DuplicateRuntimeFact,
        message: format!(
            "{label}: {count} canonical facts target the same zero-or-one named runtime field; all were rejected"
        ),
        canonical_target: canonical_target_view(target),
    }
}

fn is_zero_or_one_named_runtime_target(target: &MechanicTarget) -> bool {
    matches!(
        target,
        MechanicTarget::MaxHp
            | MechanicTarget::ArmorClass
            | MechanicTarget::Perception
            | MechanicTarget::Save { .. }
            | MechanicTarget::AbilityModifier { .. }
            | MechanicTarget::SpellcastingAttack { .. }
            | MechanicTarget::SpellcastingDc { .. }
    )
}

fn spellcasting_entry<'a>(
    entries: &'a mut Vec<EncounterRuntimeSpellcastingView>,
    entry_id: &str,
    label: &str,
) -> &'a mut EncounterRuntimeSpellcastingView {
    if let Some(index) = entries.iter().position(|entry| entry.entry_id == entry_id) {
        return &mut entries[index];
    }
    let index = entries.len();
    entries.push(EncounterRuntimeSpellcastingView {
        entry_id: entry_id.to_string(),
        authored_order: 0,
        label: label.to_string(),
        preparation: None,
        tradition: None,
        attack: None,
        dc: None,
        slots: Vec::new(),
        spells: Vec::new(),
    });
    &mut entries[index]
}

fn runtime_roll_from_number(
    fact: RuntimeNumberView,
    surface: RuntimeRollSurfaceView,
) -> RuntimeRollView {
    RuntimeRollView {
        roll_id: fact.label.clone(),
        label: fact.label,
        base_value: fact.base_value,
        adjusted_value: fact.adjusted_value,
        surface,
        modifiers: fact.modifiers,
        suppressed_modifiers: fact.suppressed_modifiers,
        provenance: fact.provenance,
    }
}

fn runtime_count_from_number(fact: RuntimeNumberView) -> RuntimeCountView {
    RuntimeCountView {
        label: fact.label,
        base_value: fact.base_value,
        adjusted_value: fact.adjusted_value,
        segments: Vec::new(),
        adjustments: Vec::new(),
        suppressed_adjustments: Vec::new(),
        provenance: fact.provenance,
    }
}

fn unrouted_fact(fact: RuntimeNumberView) -> EncounterProjectionDiagnostic {
    EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::UnmappedCanonicalFact,
        message: format!(
            "{} has no supported destination in the typed encounter runtime",
            fact.label
        ),
        canonical_target: fact.provenance.canonical_target,
    }
}

pub(super) fn participant_encounter_runtime(
    participant: &EncounterParticipant,
    retrieved: &RetrievedRecord,
) -> Option<EncounterRuntimeView> {
    match retrieved.body.as_ref()? {
        RecordBody::Creature(creature) => {
            let mechanics = canonical_participant_mechanics(project_creature_mechanics(creature));
            Some(into_public_runtime(apply_participant_effects(
                participant,
                mechanics.view,
                mechanics.diagnostics,
                mechanics.automation_limitations,
                mechanics.variant_damage_blocked_activity_ids,
                mechanics.activity_runtime,
            )))
        }
        RecordBody::Hazard(hazard) => Some(hazard_encounter_runtime(participant, hazard)),
        RecordBody::Spell(_) | RecordBody::Consumable(_) => None,
    }
}

fn hazard_encounter_runtime(
    participant: &EncounterParticipant,
    hazard: &HazardRecord,
) -> EncounterRuntimeView {
    let canonical_number = |label: &str, value: i64, target| RuntimeNumberView {
        label: label.to_string(),
        base_value: value,
        adjusted_value: value,
        modifiers: Vec::new(),
        suppressed_modifiers: Vec::new(),
        provenance: fact_provenance(RuntimeFactSourceView::CanonicalRecord, Some(target)),
    };
    let conveniences = project_hazard_conveniences(hazard);
    let convenience_number = |label: &str, value: i64, target| RuntimeNumberView {
        label: label.to_string(),
        base_value: value,
        adjusted_value: value,
        modifiers: Vec::new(),
        suppressed_modifiers: Vec::new(),
        provenance: fact_provenance(
            RuntimeFactSourceView::RuntimeRule {
                rule: RuntimeRuleView::HazardConvenience,
            },
            Some(target),
        ),
    };
    let defenses = hazard.defenses.typed();
    let vitals = (participant.max_hp.is_some()
        || participant.current_hp.is_some()
        || participant.temporary_hp != 0)
        .then(|| EncounterRuntimeVitalsView {
            maximum_hp: participant
                .max_hp
                .map(|value| participant_number("Maximum HP", value)),
            current_hp: participant.current_hp,
            temporary_hp: participant.temporary_hp,
        });
    let saves = defenses
        .and_then(|value| value.saves.typed())
        .and_then(|saves| {
            let projected = EncounterRuntimeSavesView {
                fortitude: saves.fortitude.typed().copied().map(|value| {
                    canonical_number(
                        "Fortitude",
                        value,
                        RuntimeCanonicalTargetView::Save {
                            save: RuntimeSaveKindView::Fortitude,
                        },
                    )
                }),
                reflex: saves.reflex.typed().copied().map(|value| {
                    canonical_number(
                        "Reflex",
                        value,
                        RuntimeCanonicalTargetView::Save {
                            save: RuntimeSaveKindView::Reflex,
                        },
                    )
                }),
                will: saves.will.typed().copied().map(|value| {
                    canonical_number(
                        "Will",
                        value,
                        RuntimeCanonicalTargetView::Save {
                            save: RuntimeSaveKindView::Will,
                        },
                    )
                }),
            };
            (projected.fortitude.is_some()
                || projected.reflex.is_some()
                || projected.will.is_some())
            .then_some(projected)
        });

    let mut occurrences = hazard
        .embedded_entities
        .typed()
        .map(|embedded| embedded.occurrences.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    occurrences.sort_by_key(|occurrence| (occurrence.authored_order, occurrence.source_ordinal));
    let activities = occurrences
        .into_iter()
        .filter_map(|occurrence| {
            let embedded = hazard.embedded_entities.typed()?;
            let entity = embedded
                .entities
                .iter()
                .find(|entity| entity.id == occurrence.entity_id)?;
            let activity_id = occurrence.id.as_str().to_string();
            let label = occurrence
                .contextual_label
                .typed()
                .cloned()
                .unwrap_or_else(|| entity.label.clone());
            let provenance =
                fact_provenance(RuntimeFactSourceView::CanonicalRecord, None);
            let content = {
                let values = hazard
                    .content
                    .documents
                    .iter()
                    .filter(|document| {
                        matches!(
                            &document.owner,
                            atlas_record::ContentOwner::HazardOccurrence(owner) if owner == &occurrence.id
                        )
                    })
                    .filter_map(crate::surface::content_view)
                    .collect::<Vec<_>>();
                (!values.is_empty()).then_some(values)
            };
            let (kind, traits, action_cost, frequency, rolls, damage, available, reason) =
                match &entity.capability {
                    HazardCapability::Action(action) => {
                        let action_type = action.action_type.typed().copied();
                        let action_cost = match action_type {
                            Some(HazardActionType::Passive) => {
                                Some(EncounterRuntimeActionCostKindView::Passive)
                            }
                            Some(HazardActionType::Reaction) => {
                                Some(EncounterRuntimeActionCostKindView::Reaction)
                            }
                            Some(HazardActionType::Free) => {
                                Some(EncounterRuntimeActionCostKindView::FreeAction)
                            }
                            Some(HazardActionType::Action) => action
                                .actions
                                .typed()
                                .map(|count| EncounterRuntimeActionCostKindView::Actions {
                                    count: i64::from(count.value()),
                                }),
                            None => None,
                        }
                        .map(|value| EncounterRuntimeActionCostView {
                            value,
                            provenance: fact_provenance(
                                RuntimeFactSourceView::CanonicalRecord,
                                Some(RuntimeCanonicalTargetView::ActivityActionCost {
                                    activity_id: activity_id.clone(),
                                }),
                            ),
                        });
                        let frequency = action.frequency.typed().map(|frequency| {
                            EncounterRuntimeFrequencyView {
                                maximum: frequency.maximum.typed().copied(),
                                period: frequency.per.typed().map(|period| match period {
                                    HazardFrequencyInterval::Turn => "turn",
                                    HazardFrequencyInterval::Round => "round",
                                    HazardFrequencyInterval::OneMinute => "PT1M",
                                    HazardFrequencyInterval::TenMinutes => "PT10M",
                                    HazardFrequencyInterval::OneHour => "PT1H",
                                    HazardFrequencyInterval::TwentyFourHours => "PT24H",
                                    HazardFrequencyInterval::Day => "day",
                                    HazardFrequencyInterval::Week => "P1W",
                                    HazardFrequencyInterval::Month => "P1M",
                                    HazardFrequencyInterval::Year => "P1Y",
                                }.to_string()),
                                serialized_value: frequency.value.typed().copied(),
                                provenance: fact_provenance(
                                    RuntimeFactSourceView::CanonicalRecord,
                                    Some(RuntimeCanonicalTargetView::ActivityFrequency {
                                        activity_id: activity_id.clone(),
                                    }),
                                ),
                            }
                        });
                        let supported = action_type.is_some()
                            && (!matches!(action_type, Some(HazardActionType::Action))
                                || action.actions.typed().is_some());
                        (
                            EncounterRuntimeActivityKindView::Other,
                            action
                                .common
                                .traits
                                .typed()
                                .map(|traits| traits.iter().map(|value| value.as_str().to_string()).collect())
                                .unwrap_or_default(),
                            action_cost,
                            frequency,
                            Vec::new(),
                            Vec::new(),
                            supported,
                            (!supported).then(|| "canonical action cost is unavailable".to_string()),
                        )
                    }
                    HazardCapability::Strike(strike) => {
                        let bonus = strike.bonus.typed().copied();
                        let typed_damage = strike.damage_rolls.typed();
                        let damage_complete = typed_damage.is_some_and(|values| {
                            values.iter().all(|value| value.damage.typed().is_some())
                        });
                        let rolls = bonus.map(|bonus| vec![RuntimeRollView {
                            roll_id: format!("{activity_id}:attack"),
                            label: "Attack".to_string(),
                            base_value: bonus,
                            adjusted_value: bonus,
                            surface: RuntimeRollSurfaceView::AttackRoll,
                            modifiers: Vec::new(),
                            suppressed_modifiers: Vec::new(),
                            provenance: fact_provenance(RuntimeFactSourceView::CanonicalRecord, Some(RuntimeCanonicalTargetView::ActivityRoll { activity_id: activity_id.clone(), roll_id: format!("{activity_id}:attack") })),
                        }]).unwrap_or_default();
                        let damage = typed_damage.map(|values| values.iter().filter_map(|value| {
                            let formula = value.damage.typed()?.clone();
                            Some(RuntimeFormulaView {
                                damage_id: value.source_key.clone(),
                                label: None,
                                formula,
                                adjusted_formula: None,
                                damage_type: value.damage_type.typed().cloned(),
                                effect_kind: RuntimeDamageEffectKindView::Damage,
                                modifiers: Vec::new(),
                                provenance: fact_provenance(RuntimeFactSourceView::CanonicalRecord, Some(RuntimeCanonicalTargetView::ActivityDamage { activity_id: activity_id.clone(), damage_id: value.source_key.clone() })),
                            })
                        }).collect()).unwrap_or_default();
                        (
                            EncounterRuntimeActivityKindView::Strike,
                            strike.common.traits.typed().map(|traits| traits.iter().map(|value| value.as_str().to_string()).collect()).unwrap_or_default(),
                            None,
                            None,
                            rolls,
                            damage,
                            bonus.is_some() && damage_complete,
                            if bonus.is_none() {
                                Some("canonical strike attack bonus is unavailable".to_string())
                            } else if !damage_complete {
                                Some("canonical strike damage formula is unavailable".to_string())
                            } else {
                                None
                            },
                        )
                    }
                    HazardCapability::Condition(_)
                    | HazardCapability::Effect(_)
                    | HazardCapability::UnsupportedChild(_) => (
                        EncounterRuntimeActivityKindView::Other,
                        Vec::new(),
                        None,
                        None,
                        Vec::new(),
                        Vec::new(),
                        false,
                        Some("this canonical hazard child has no supported runtime action".to_string()),
                    ),
                };
            Some(EncounterRuntimeActivityView {
                activity_id,
                label,
                kind,
                usage: if frequency.is_some() {
                    EncounterRuntimeActivityUsageView::Limited
                } else {
                    EncounterRuntimeActivityUsageView::Unlimited
                },
                availability: Some(RuntimeCapabilityView {
                    available,
                    provenance: Some(provenance.clone()),
                    reason,
                }),
                traits,
                action_cost,
                frequency,
                uses: None,
                rolls,
                damage,
                modes: Vec::new(),
                content,
                provenance,
            })
        })
        .collect();

    EncounterRuntimeView {
        hazard: Some(EncounterRuntimeHazardView {
            state: match participant.hazard_state {
                ParticipantHazardState::Active => EncounterRuntimeHazardStateView::Active,
                ParticipantHazardState::Disabled => EncounterRuntimeHazardStateView::Disabled,
            },
            detection_dc: conveniences.detection_dc.map(|value| {
                convenience_number(
                    "Detection DC",
                    value,
                    RuntimeCanonicalTargetView::DetectionDc,
                )
            }),
            broken_threshold: conveniences.broken_threshold.map(|value| {
                convenience_number(
                    "Broken Threshold",
                    value,
                    RuntimeCanonicalTargetView::BrokenThreshold,
                )
            }),
            initiative_suggestion: conveniences.initiative_suggestion.map(|suggestion| {
                EncounterRuntimeHazardInitiativeSuggestionView {
                    statistic: match suggestion.statistic {
                        HazardInitiativeStatistic::Stealth => {
                            EncounterRuntimeHazardInitiativeStatisticView::Stealth
                        }
                    },
                    modifier: convenience_number(
                        "Stealth",
                        suggestion.modifier,
                        RuntimeCanonicalTargetView::HazardInitiativeSuggestion,
                    ),
                }
            }),
            convenience_rule_id: conveniences.rule_id.to_string(),
            convenience_rule_version: conveniences.rule_version,
        }),
        level: hazard
            .level
            .typed()
            .copied()
            .map(|value| canonical_number("Level", value, RuntimeCanonicalTargetView::Level)),
        vitals,
        defenses: defenses
            .and_then(|value| value.armor_class.typed())
            .copied()
            .map(|value| EncounterRuntimeDefensesView {
                armor_class: canonical_number(
                    "Armor Class",
                    value,
                    RuntimeCanonicalTargetView::ArmorClass,
                ),
            }),
        saves,
        awareness: None,
        abilities: None,
        skills: Vec::new(),
        movement: None,
        resources: Vec::new(),
        spellcasting: Vec::new(),
        activities,
        standalone_spells: Vec::new(),
        action_budget: None,
        conditions: participant
            .conditions
            .iter()
            .map(runtime_condition_view)
            .collect(),
        automation_limitations: Vec::new(),
    }
}

pub(super) fn manual_encounter_runtime(participant: &EncounterParticipant) -> EncounterRuntimeView {
    EncounterRuntimeView {
        hazard: None,
        level: None,
        vitals: Some(EncounterRuntimeVitalsView {
            maximum_hp: participant
                .max_hp
                .map(|value| participant_number("Maximum HP", value)),
            current_hp: participant.current_hp,
            temporary_hp: participant.temporary_hp,
        }),
        defenses: None,
        saves: None,
        awareness: None,
        abilities: None,
        skills: Vec::new(),
        movement: None,
        resources: Vec::new(),
        spellcasting: Vec::new(),
        standalone_spells: Vec::new(),
        action_budget: Some(action_budget_view(participant)),
        activities: Vec::new(),
        conditions: participant
            .conditions
            .iter()
            .map(runtime_condition_view)
            .collect(),
        automation_limitations: participant_automation_limitations(participant),
    }
}

struct ParticipantMechanics {
    view: EncounterMechanicsInput,
    diagnostics: Vec<EncounterProjectionDiagnostic>,
    automation_limitations: Vec<EncounterRuntimeAutomationLimitationView>,
    variant_damage_blocked_activity_ids: BTreeSet<String>,
    activity_runtime: BTreeMap<String, RuntimeActivityMetadata>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeActivityMetadata {
    action_cost: Option<EncounterRuntimeActionCostView>,
    frequency: Option<EncounterRuntimeFrequencyView>,
    uses: Option<EncounterRuntimeUsesView>,
}

fn canonical_participant_mechanics(
    projection: CanonicalMechanicsProjection,
) -> ParticipantMechanics {
    let level = fact_integer(&projection.level);
    let mut values = Vec::new();
    let mut speeds = Vec::new();
    for fact in projection.facts {
        match &fact.target {
            MechanicTarget::Movement { speed_id } => {
                if let Some(value_feet) = mechanic_integer(&fact.value)
                    && value_feet > 0
                {
                    speeds.push(EncounterMovementInput {
                        movement_type: speed_id.as_str().to_string(),
                        label: fact.label,
                        value_feet,
                    });
                }
            }
            _ => {
                if let Some(base_value) = mechanic_integer(&fact.value) {
                    values.push(EncounterMechanicValue {
                        target: fact.target,
                        label: fact.label,
                        base_value: EncounterMechanicScalar::Number(base_value),
                        facets: fact.facets,
                    });
                }
            }
        }
    }
    let mut diagnostics = projection
        .unsupported
        .into_iter()
        .map(canonical_unsupported_diagnostic)
        .collect::<Vec<_>>();
    let mut automation_limitations = Vec::new();
    let mut activities = Vec::new();
    let mut variant_damage_blocked_activity_ids = BTreeSet::new();
    let mut activity_runtime = BTreeMap::new();
    for activity in projection.activities {
        diagnostics.extend(
            activity
                .unsupported
                .iter()
                .cloned()
                .map(canonical_unsupported_diagnostic),
        );
        let disposition = canonical_activity(activity);
        if disposition.variant_damage_blocked {
            variant_damage_blocked_activity_ids.insert(disposition.activity.activity_id.clone());
        }
        activity_runtime.insert(
            disposition.activity.activity_id.clone(),
            disposition.runtime_metadata,
        );
        values.extend(disposition.values);
        diagnostics.extend(disposition.diagnostics);
        automation_limitations.extend(disposition.automation_limitations);
        activities.push(disposition.activity);
    }

    ParticipantMechanics {
        view: EncounterMechanicsInput {
            level,
            values,
            speeds,
            activities,
        },
        diagnostics,
        automation_limitations,
        variant_damage_blocked_activity_ids,
        activity_runtime,
    }
}

struct CanonicalActivityDisposition {
    activity: EncounterActivityInput,
    values: Vec<EncounterMechanicValue>,
    diagnostics: Vec<EncounterProjectionDiagnostic>,
    automation_limitations: Vec<EncounterRuntimeAutomationLimitationView>,
    variant_damage_blocked: bool,
    runtime_metadata: RuntimeActivityMetadata,
}

fn canonical_activity(activity: CanonicalMechanicActivity) -> CanonicalActivityDisposition {
    let ability = activity_attack_ability(&activity);
    let usage = canonical_activity_usage(&activity);
    let variant_damage_blocked = canonical_variant_damage_blocked(&activity);
    let kind = match activity.family {
        MechanicActivityFamily::Strike => EncounterActivityKind::Strike,
        MechanicActivityFamily::Spell | MechanicActivityFamily::SpellcastingEntry => {
            EncounterActivityKind::Spell
        }
        MechanicActivityFamily::Action | MechanicActivityFamily::Unsupported => {
            EncounterActivityKind::Other
        }
    };
    let rolls = activity
        .facts
        .iter()
        .filter_map(canonical_activity_roll)
        .collect();
    let damage = activity
        .facts
        .iter()
        .filter_map(|fact| canonical_activity_damage(fact, ability))
        .collect();
    let values = activity
        .facts
        .iter()
        .filter_map(canonical_activity_value)
        .collect();
    let mut diagnostics = activity
        .facts
        .iter()
        .filter_map(canonical_activity_diagnostic)
        .collect::<Vec<_>>();
    let automation_limitations = activity
        .facts
        .iter()
        .filter_map(canonical_activity_automation_limitation)
        .collect();
    let (runtime_metadata, metadata_diagnostics) = canonical_activity_runtime_metadata(&activity);
    diagnostics.extend(metadata_diagnostics);
    CanonicalActivityDisposition {
        activity: EncounterActivityInput {
            activity_id: activity.occurrence_id.as_str().to_string(),
            label: activity.label,
            kind,
            usage,
            rolls,
            damage,
        },
        values,
        diagnostics,
        automation_limitations,
        variant_damage_blocked,
        runtime_metadata,
    }
}

fn canonical_activity_runtime_metadata(
    activity: &CanonicalMechanicActivity,
) -> (RuntimeActivityMetadata, Vec<EncounterProjectionDiagnostic>) {
    let mut metadata = RuntimeActivityMetadata::default();
    let mut diagnostics = Vec::new();
    let action_cost_count = activity
        .facts
        .iter()
        .filter(|fact| matches!(fact.target, MechanicTarget::ActivityActionCost { .. }))
        .count();
    let frequency_count = activity
        .facts
        .iter()
        .filter(|fact| matches!(fact.target, MechanicTarget::ActivityFrequency { .. }))
        .count();
    let uses_count = activity
        .facts
        .iter()
        .filter(|fact| matches!(fact.target, MechanicTarget::ActivityUses { .. }))
        .count();
    for fact in &activity.facts {
        match (&fact.target, &fact.value) {
            (MechanicTarget::ActivityActionCost { .. }, MechanicBaseValue::ActionCost(value))
                if action_cost_count == 1 =>
            {
                metadata.action_cost = runtime_action_cost(value, &fact.target);
            }
            (
                MechanicTarget::ActivityFrequency { .. },
                MechanicBaseValue::Frequency(FactValue::Value(value)),
            ) if frequency_count == 1 => {
                metadata.frequency = Some(EncounterRuntimeFrequencyView {
                    maximum: value.maximum.as_value().copied(),
                    period: value.period.as_value().cloned(),
                    serialized_value: value.serialized_value.as_value().copied(),
                    provenance: canonical_provenance(&fact.target, None),
                });
            }
            (
                MechanicTarget::ActivityUses { .. },
                MechanicBaseValue::Uses(FactValue::Value(value)),
            ) if uses_count == 1 => {
                metadata.uses = Some(EncounterRuntimeUsesView {
                    maximum: value.maximum.as_value().copied(),
                    serialized_value: value.serialized_value.as_value().copied(),
                    provenance: canonical_provenance(&fact.target, None),
                });
            }
            (MechanicTarget::ActivityActionCost { .. }, _) if action_cost_count > 1 => {
                diagnostics.push(duplicate_activity_metadata_fact(fact, action_cost_count));
            }
            (MechanicTarget::ActivityFrequency { .. }, _) if frequency_count > 1 => {
                diagnostics.push(duplicate_activity_metadata_fact(fact, frequency_count));
            }
            (MechanicTarget::ActivityUses { .. }, _) if uses_count > 1 => {
                diagnostics.push(duplicate_activity_metadata_fact(fact, uses_count));
            }
            _ => {}
        }
    }
    (metadata, diagnostics)
}

fn duplicate_activity_metadata_fact(
    fact: &MechanicFact,
    count: usize,
) -> EncounterProjectionDiagnostic {
    EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::DuplicateRuntimeFact,
        message: format!(
            "{}: {count} canonical facts target the same zero-or-one encounter activity field; all were rejected",
            fact.label
        ),
        canonical_target: canonical_target_view(&fact.target),
    }
}

fn runtime_action_cost(
    value: &CreatureActionCost,
    target: &MechanicTarget,
) -> Option<EncounterRuntimeActionCostView> {
    let value = match value {
        CreatureActionCost::Passive => EncounterRuntimeActionCostKindView::Passive,
        CreatureActionCost::Reaction => EncounterRuntimeActionCostKindView::Reaction,
        CreatureActionCost::FreeAction => EncounterRuntimeActionCostKindView::FreeAction,
        CreatureActionCost::Actions(count) => EncounterRuntimeActionCostKindView::Actions {
            count: i64::from(*count),
        },
        CreatureActionCost::Time(value) => EncounterRuntimeActionCostKindView::Time {
            value: value.clone(),
        },
        CreatureActionCost::Unsupported(_) => return None,
    };
    Some(EncounterRuntimeActionCostView {
        value,
        provenance: canonical_provenance(target, None),
    })
}

fn canonical_variant_damage_blocked(activity: &CanonicalMechanicActivity) -> bool {
    activity
        .facts
        .iter()
        .filter_map(|fact| match (&fact.target, &fact.value) {
            (MechanicTarget::ActivityDamage { .. }, MechanicBaseValue::Damage(damage)) => {
                Some(damage)
            }
            _ => None,
        })
        .find(|damage| canonical_damage_effect_kind(damage) == DamageEffectKind::Damage)
        .is_some_and(|damage| damage.formula.as_value().is_none())
}

fn canonical_activity_roll(fact: &MechanicFact) -> Option<EncounterActivityRoll> {
    match (&fact.target, &fact.value) {
        (MechanicTarget::ActivityRoll { roll_id, .. }, MechanicBaseValue::Roll(roll)) => {
            let surface = match roll.kind {
                CreatureRollKind::Attack => EncounterActivityRollSurface::AttackRoll,
                CreatureRollKind::DifficultyClass => EncounterActivityRollSurface::Dc,
                // The encounter DTO has no generic check surface. The complete typed fact is
                // retained as an internal diagnostic by `canonical_activity_diagnostic`.
                CreatureRollKind::Check => return None,
            };
            Some(EncounterActivityRoll {
                roll_id: roll_id.clone(),
                label: fact.label.clone(),
                base_value: fact_integer(&roll.value)?,
                surface,
                ability: fact_activity_ability(&roll.ability),
            })
        }
        _ => None,
    }
}

fn canonical_activity_value(fact: &MechanicFact) -> Option<EncounterMechanicValue> {
    let value = match (&fact.target, &fact.value) {
        (
            MechanicTarget::SpellcastingAttack { .. } | MechanicTarget::SpellcastingDc { .. },
            value,
        ) => mechanic_integer(value)?,
        (
            MechanicTarget::SpellSlotMaximum { .. },
            MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(value))),
        ) => *value,
        _ => return None,
    };
    Some(EncounterMechanicValue {
        target: fact.target.clone(),
        label: fact.label.clone(),
        base_value: EncounterMechanicScalar::Number(value),
        facets: fact.facets.clone(),
    })
}

fn canonical_activity_diagnostic(fact: &MechanicFact) -> Option<EncounterProjectionDiagnostic> {
    let disposition = match (&fact.target, &fact.value) {
        (
            MechanicTarget::ActivityRoll { .. },
            MechanicBaseValue::Roll(
                roll @ CreatureRoll {
                    kind: CreatureRollKind::Check,
                    ..
                },
            ),
        ) => format!(
            "check roll has no encounter roll surface; id={:?}; label={:?}; value={}; ability={}",
            roll.id,
            roll.label,
            fact_i64(&roll.value),
            fact_roll_ability(&roll.ability)
        ),
        (
            MechanicTarget::ActivityActionCost { .. },
            MechanicBaseValue::ActionCost(value @ CreatureActionCost::Unsupported(_)),
        ) => {
            format!(
                "action cost cannot populate the typed encounter activity field; value={}",
                action_cost(value)
            )
        }
        (
            MechanicTarget::ActivityFrequency { .. },
            MechanicBaseValue::Frequency(value @ (FactValue::Missing | FactValue::Null)),
        ) => {
            format!(
                "frequency cannot populate the typed encounter activity field; value={}",
                frequency(value)
            )
        }
        (
            MechanicTarget::ActivityUses { .. },
            MechanicBaseValue::Uses(value @ (FactValue::Missing | FactValue::Null)),
        ) => format!(
            "uses cannot populate the typed encounter activity field; value={}",
            uses(value)
        ),
        (target @ MechanicTarget::ActivityDamage { .. }, MechanicBaseValue::Damage(damage))
            if damage.formula.as_value().is_none() =>
        {
            format!(
                "damage formula cannot populate a structured encounter damage expression; target={}; id={:?}; formula={}",
                target.id(),
                damage.id,
                fact_string(&damage.formula)
            )
        }
        (
            MechanicTarget::SpellSlotMaximum { .. },
            MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(_))),
        ) => return None,
        (MechanicTarget::SpellSlotMaximum { .. }, MechanicBaseValue::SourceInteger(value)) => {
            format!(
                "spell-slot maximum cannot populate a numeric encounter value; value={}",
                source_i64(value)
            )
        }
        _ => return None,
    };
    Some(EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::UnavailableCanonicalFact,
        message: format!("{}: {disposition}", fact.label),
        canonical_target: canonical_target_view(&fact.target),
    })
}

fn canonical_activity_automation_limitation(
    fact: &MechanicFact,
) -> Option<EncounterRuntimeAutomationLimitationView> {
    let (
        MechanicTarget::ActivityRoll { occurrence_id, .. },
        MechanicBaseValue::Roll(CreatureRoll {
            kind: CreatureRollKind::Check,
            value,
            ..
        }),
    ) = (&fact.target, &fact.value)
    else {
        return None;
    };
    fact_integer(value)?;
    Some(EncounterRuntimeAutomationLimitationView {
        code: EncounterRuntimeAutomationLimitationCodeView::ActivityCheckNotAutomated,
        target: EncounterRuntimeAutomationLimitationTargetView::Activity {
            activity_id: occurrence_id.as_str().to_string(),
        },
        message: format!(
            "{} is available from the source record, but encounter check automation is not available.",
            fact.label
        ),
    })
}

fn fact_i64(value: &FactValue<i64>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({value})"),
    }
}

fn fact_string(value: &FactValue<String>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({value:?})"),
    }
}

fn fact_roll_ability(value: &FactValue<ActivityRollAbility>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!("value({})", roll_ability_name(*value)),
    }
}

fn roll_ability_name(value: ActivityRollAbility) -> &'static str {
    match value {
        ActivityRollAbility::Strength => "strength",
        ActivityRollAbility::Dexterity => "dexterity",
        ActivityRollAbility::Constitution => "constitution",
        ActivityRollAbility::Intelligence => "intelligence",
        ActivityRollAbility::Wisdom => "wisdom",
        ActivityRollAbility::Charisma => "charisma",
    }
}

fn action_cost(value: &CreatureActionCost) -> String {
    match value {
        CreatureActionCost::Passive => "passive".to_string(),
        CreatureActionCost::Reaction => "reaction".to_string(),
        CreatureActionCost::FreeAction => "free_action".to_string(),
        CreatureActionCost::Actions(value) => format!("actions({value})"),
        CreatureActionCost::Time(value) => format!("time({value:?})"),
        CreatureActionCost::Unsupported(value) => unsupported_source_value(value),
    }
}

fn frequency(value: &FactValue<CreatureFrequency>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!(
            "maximum={}, period={}, serialized_value={}",
            fact_i64(&value.maximum),
            fact_string(&value.period),
            fact_i64(&value.serialized_value)
        ),
    }
}

fn uses(value: &FactValue<CreatureUseLimit>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(value) => format!(
            "maximum={}, serialized_value={}",
            fact_i64(&value.maximum),
            fact_i64(&value.serialized_value)
        ),
    }
}

fn source_i64(value: &FactValue<CreatureSourceScalar<i64>>) -> String {
    match value {
        FactValue::Missing => "missing".to_string(),
        FactValue::Null => "null".to_string(),
        FactValue::Value(CreatureSourceScalar::Value(value)) => format!("value({value})"),
        FactValue::Value(CreatureSourceScalar::Unsupported(value)) => {
            unsupported_source_value(value)
        }
    }
}

fn unsupported_source_value(value: &UnsupportedSourceValue) -> String {
    format!(
        "unsupported(shape={}, value={:?}, reason={})",
        unsupported_source_shape(value.shape),
        value.value,
        unsupported_source_reason(value.reason)
    )
}

fn unsupported_source_shape(value: UnsupportedSourceShape) -> &'static str {
    match value {
        UnsupportedSourceShape::Missing => "missing",
        UnsupportedSourceShape::Null => "null",
        UnsupportedSourceShape::String => "string",
        UnsupportedSourceShape::Number => "number",
        UnsupportedSourceShape::Boolean => "boolean",
        UnsupportedSourceShape::Array => "array",
        UnsupportedSourceShape::Object => "object",
    }
}

fn unsupported_source_reason(value: UnsupportedSourceReason) -> &'static str {
    match value {
        UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
        UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
        UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
        UnsupportedSourceReason::NonCanonicalRuntimeValue => "non_canonical_runtime_value",
        UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
    }
}

fn canonical_activity_damage(
    fact: &MechanicFact,
    activity_ability: Option<ActivityRollAbility>,
) -> Option<EncounterDamageExpression> {
    let MechanicBaseValue::Damage(damage) = &fact.value else {
        return None;
    };
    Some(EncounterDamageExpression {
        damage_id: damage.id.clone(),
        label: (fact.label != damage.id).then(|| fact.label.clone()),
        formula: damage.formula.as_value()?.clone(),
        damage_type: damage.damage_type.as_value().cloned(),
        effect_kind: canonical_damage_effect_kind(damage),
        ability: damage_applies_modifier(damage)
            .then_some(activity_ability)
            .flatten(),
    })
}

fn activity_attack_ability(activity: &CanonicalMechanicActivity) -> Option<ActivityRollAbility> {
    activity.facts.iter().find_map(|fact| {
        let MechanicBaseValue::Roll(roll) = &fact.value else {
            return None;
        };
        (roll.kind == CreatureRollKind::Attack)
            .then(|| fact_activity_ability(&roll.ability))
            .flatten()
    })
}

fn canonical_activity_usage(activity: &CanonicalMechanicActivity) -> EncounterActivityUsage {
    match activity.family {
        MechanicActivityFamily::Strike => EncounterActivityUsage::Unlimited,
        MechanicActivityFamily::Spell | MechanicActivityFamily::Action => {
            if activity.facts.iter().any(fact_has_limited_use) {
                EncounterActivityUsage::Limited
            } else {
                EncounterActivityUsage::Unlimited
            }
        }
        MechanicActivityFamily::SpellcastingEntry => EncounterActivityUsage::Limited,
        MechanicActivityFamily::Unsupported => EncounterActivityUsage::Ambiguous,
    }
}

fn fact_has_limited_use(fact: &MechanicFact) -> bool {
    match &fact.value {
        MechanicBaseValue::Frequency(value) => value
            .as_value()
            .and_then(|frequency| frequency.maximum.as_value())
            .is_some(),
        MechanicBaseValue::Uses(value) => value
            .as_value()
            .and_then(|uses| uses.maximum.as_value())
            .is_some(),
        _ => false,
    }
}

fn canonical_damage_effect_kind(damage: &CreatureDamage) -> DamageEffectKind {
    let Some(kinds) = damage.kinds.as_value() else {
        return DamageEffectKind::Unknown;
    };
    let has_damage = kinds.contains(&CreatureDamageKind::Damage);
    let has_healing = kinds.contains(&CreatureDamageKind::Healing);
    match (has_damage, has_healing) {
        (true, false) => DamageEffectKind::Damage,
        (false, true) => DamageEffectKind::Healing,
        (true, true) => DamageEffectKind::DamageOrHealing,
        (false, false) => DamageEffectKind::Unknown,
    }
}

fn damage_applies_modifier(damage: &CreatureDamage) -> bool {
    matches!(
        damage.apply_modifier,
        FactValue::Value(CreatureSourceScalar::Value(true))
    )
}

fn mechanic_integer(value: &MechanicBaseValue) -> Option<i64> {
    match value {
        MechanicBaseValue::Integer(value) => fact_integer(value),
        MechanicBaseValue::Number(value) => match value.as_value()? {
            CreatureNumber::Integer(value) => Some(*value),
            CreatureNumber::Unsupported(_) => None,
        },
        MechanicBaseValue::ResourceAmount(value) => match value.as_value()? {
            CreatureResourceAmount::Integer(value) => Some(*value),
            CreatureResourceAmount::Unsupported(_) => None,
        },
        MechanicBaseValue::SourceInteger(value) => match value.as_value()? {
            CreatureSourceScalar::Value(value) => Some(*value),
            CreatureSourceScalar::Unsupported(_) => None,
        },
        MechanicBaseValue::Roll(value) => fact_integer(&value.value),
        MechanicBaseValue::ActionCost(_)
        | MechanicBaseValue::Frequency(_)
        | MechanicBaseValue::Uses(_)
        | MechanicBaseValue::Damage(_) => None,
    }
}

fn fact_integer(value: &FactValue<i64>) -> Option<i64> {
    value.as_value().copied()
}

fn fact_activity_ability(value: &FactValue<ActivityRollAbility>) -> Option<ActivityRollAbility> {
    value.as_value().copied()
}

fn canonical_unsupported_diagnostic(
    unsupported: UnsupportedMechanic,
) -> EncounterProjectionDiagnostic {
    let target = unsupported
        .target
        .as_ref()
        .map(MechanicTarget::id)
        .unwrap_or_else(|| "unmodeled mechanic".to_string());
    EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::UnsupportedCanonicalFact,
        message: format!(
            "{target}: {}: {}",
            unsupported.source_path,
            unsupported_value(&unsupported.value)
        ),
        canonical_target: unsupported.target.as_ref().and_then(canonical_target_view),
    }
}

fn unsupported_value(value: &UnsupportedMechanicValue) -> String {
    match value {
        UnsupportedMechanicValue::Source(value) => value.value.clone(),
        UnsupportedMechanicValue::Note(note) => note.value.value.clone(),
        UnsupportedMechanicValue::Capability {
            source_item_type,
            source_slug,
        } => source_slug
            .as_value()
            .map(|slug| format!("{source_item_type}: {slug}"))
            .unwrap_or_else(|| source_item_type.clone()),
        UnsupportedMechanicValue::PreparedSlot(value) => value.value.clone(),
        UnsupportedMechanicValue::ResourceDrift(value) => value.value.value.clone(),
    }
}

pub(super) fn variant_hp_adjustment_delta(
    old_variant: ParticipantVariant,
    new_variant: ParticipantVariant,
    level: Option<i64>,
) -> i64 {
    variant_hp_delta(new_variant, level).unwrap_or(0)
        - variant_hp_delta(old_variant, level).unwrap_or(0)
}

pub(super) fn canonical_creature_level(retrieved: &RetrievedRecord) -> Option<i64> {
    let Some(RecordBody::Creature(creature)) = retrieved.body.as_ref() else {
        return None;
    };
    fact_integer(&creature.level.value)
}

fn apply_participant_effects(
    participant: &EncounterParticipant,
    mechanics: EncounterMechanicsInput,
    mut diagnostics: Vec<EncounterProjectionDiagnostic>,
    mut automation_limitations: Vec<EncounterRuntimeAutomationLimitationView>,
    variant_damage_blocked_activity_ids: BTreeSet<String>,
    mut activity_runtime: BTreeMap<String, RuntimeActivityMetadata>,
) -> EncounterRuntimeProjection {
    let mut modifiers = variant_modifiers(participant.participant_variant, &mechanics);
    for (condition, rule) in participant_condition_rules(participant) {
        modifiers.extend(condition_modifiers(condition, rule, &mechanics));
        automation_limitations.extend(condition_automation_limitations(condition, rule));
    }

    let mut by_target = BTreeMap::<MechanicTarget, Vec<CandidateModifier>>::new();
    for modifier in modifiers {
        by_target
            .entry(modifier.target.clone())
            .or_default()
            .push(modifier);
    }
    let named_target_counts = mechanics
        .values
        .iter()
        .filter(|mechanic| is_zero_or_one_named_runtime_target(&mechanic.target))
        .fold(
            BTreeMap::<MechanicTarget, usize>::new(),
            |mut counts, mechanic| {
                *counts.entry(mechanic.target.clone()).or_default() += 1;
                counts
            },
        );
    let ambiguous_spellcasting_entries = named_target_counts
        .iter()
        .filter_map(|(target, count)| {
            if *count < 2 {
                return None;
            }
            match target {
                MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id,
                }
                | MechanicTarget::SpellcastingDc {
                    entry_occurrence_id,
                } => Some(entry_occurrence_id.clone()),
                _ => None,
            }
        })
        .collect::<BTreeSet<_>>();

    let level = mechanics.level.map(|base| {
        let value = level_for_variant(Some(base), participant.participant_variant).unwrap_or(base);
        RuntimeNumberView {
            label: "Level".to_string(),
            base_value: base,
            adjusted_value: value,
            modifiers: (base != value)
                .then(|| RuntimeModifierView {
                    provenance: variant_provenance(participant.participant_variant),
                    label: format!(
                        "{} level adjustment",
                        variant_source(participant.participant_variant)
                    ),
                    modifier_type: StatModifierTypeView::Adjustment,
                    value: value - base,
                })
                .into_iter()
                .collect(),
            suppressed_modifiers: Vec::new(),
            provenance: canonical_provenance(
                &MechanicTarget::ActorRitualDc,
                Some(RuntimeCanonicalTargetView::Level),
            ),
        }
    });

    let mut vitals = EncounterRuntimeVitalsView {
        maximum_hp: None,
        current_hp: participant.current_hp,
        temporary_hp: participant.temporary_hp,
    };
    let mut armor_class = None;
    let mut saves = EncounterRuntimeSavesView {
        fortitude: None,
        reflex: None,
        will: None,
    };
    let mut perception = None;
    let mut abilities = EncounterRuntimeAbilitiesView {
        strength: None,
        dexterity: None,
        constitution: None,
        intelligence: None,
        wisdom: None,
        charisma: None,
    };
    let mut skills = Vec::new();
    let mut resources = Vec::new();
    let mut spellcasting = Vec::<EncounterRuntimeSpellcastingView>::new();
    for mechanic in mechanics.values {
        let target = mechanic.target.clone();
        let target_count = named_target_counts.get(&target).copied().unwrap_or(1);
        let fact = runtime_number_view(mechanic, by_target.remove(&target).unwrap_or_default());
        match target {
            MechanicTarget::MaxHp => set_named_fact(
                &mut vitals.maximum_hp,
                fact,
                &MechanicTarget::MaxHp,
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::ArmorClass => set_named_fact(
                &mut armor_class,
                fact,
                &MechanicTarget::ArmorClass,
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::Perception => set_named_fact(
                &mut perception,
                fact,
                &MechanicTarget::Perception,
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::Save {
                save: SaveKind::Fortitude,
            } => set_named_fact(
                &mut saves.fortitude,
                fact,
                &MechanicTarget::Save {
                    save: SaveKind::Fortitude,
                },
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::Save {
                save: SaveKind::Reflex,
            } => set_named_fact(
                &mut saves.reflex,
                fact,
                &MechanicTarget::Save {
                    save: SaveKind::Reflex,
                },
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::Save {
                save: SaveKind::Will,
            } => set_named_fact(
                &mut saves.will,
                fact,
                &MechanicTarget::Save {
                    save: SaveKind::Will,
                },
                target_count,
                &mut diagnostics,
            ),
            MechanicTarget::AbilityModifier { ability } => match ability {
                AbilityKind::Strength => set_named_fact(
                    &mut abilities.strength,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
                AbilityKind::Dexterity => set_named_fact(
                    &mut abilities.dexterity,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
                AbilityKind::Constitution => set_named_fact(
                    &mut abilities.constitution,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
                AbilityKind::Intelligence => set_named_fact(
                    &mut abilities.intelligence,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
                AbilityKind::Wisdom => set_named_fact(
                    &mut abilities.wisdom,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
                AbilityKind::Charisma => set_named_fact(
                    &mut abilities.charisma,
                    fact,
                    &MechanicTarget::AbilityModifier { ability },
                    target_count,
                    &mut diagnostics,
                ),
            },
            MechanicTarget::CreatureSkill { skill_id, kind } => {
                skills.push(EncounterRuntimeSkillView {
                    skill_id: skill_id.as_str().to_string(),
                    label: fact.label.clone(),
                    kind: if kind == atlas_record::CreatureSkillKind::Lore {
                        EncounterRuntimeSkillKindView::Lore {
                            slug: kind.source_slug().to_string(),
                        }
                    } else {
                        EncounterRuntimeSkillKindView::Standard {
                            slug: kind.source_slug().to_string(),
                        }
                    },
                    modifier: fact,
                })
            }
            MechanicTarget::ResourceMaximum { resource_id } => {
                resources.push(EncounterRuntimeResourceView {
                    resource_id: resource_id.as_str().to_string(),
                    label: fact.label.clone(),
                    maximum: fact,
                    current: None,
                })
            }
            MechanicTarget::SpellcastingAttack {
                entry_occurrence_id,
            } => {
                let label = fact.label.clone();
                let entry_label = if ambiguous_spellcasting_entries.contains(&entry_occurrence_id) {
                    "Spellcasting"
                } else {
                    label.as_str()
                };
                let target = MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id: entry_occurrence_id.clone(),
                };
                set_spellcasting_roll(
                    &mut spellcasting_entry(
                        &mut spellcasting,
                        entry_occurrence_id.as_str(),
                        entry_label,
                    )
                    .attack,
                    runtime_roll_from_number(fact, RuntimeRollSurfaceView::AttackRoll),
                    &target,
                    target_count,
                    &mut diagnostics,
                );
            }
            MechanicTarget::SpellcastingDc {
                entry_occurrence_id,
            } => {
                let label = fact.label.clone();
                let entry_label = if ambiguous_spellcasting_entries.contains(&entry_occurrence_id) {
                    "Spellcasting"
                } else {
                    label.as_str()
                };
                let target = MechanicTarget::SpellcastingDc {
                    entry_occurrence_id: entry_occurrence_id.clone(),
                };
                set_spellcasting_roll(
                    &mut spellcasting_entry(
                        &mut spellcasting,
                        entry_occurrence_id.as_str(),
                        entry_label,
                    )
                    .dc,
                    runtime_roll_from_number(fact, RuntimeRollSurfaceView::Dc),
                    &target,
                    target_count,
                    &mut diagnostics,
                );
            }
            MechanicTarget::SpellSlotMaximum {
                entry_occurrence_id,
                rank,
            } => spellcasting_entry(
                &mut spellcasting,
                entry_occurrence_id.as_str(),
                "Spellcasting",
            )
            .slots
            .push(EncounterRuntimeSpellSlotView {
                rank,
                maximum: runtime_count_from_number(fact),
                current: None,
            }),
            MechanicTarget::ActivityActionCost { .. }
            | MechanicTarget::ActivityFrequency { .. }
            | MechanicTarget::ActivityUses { .. }
            | MechanicTarget::ActivityRoll { .. }
            | MechanicTarget::ActivityDamage { .. }
            | MechanicTarget::Movement { .. }
            | MechanicTarget::ActorRitualDc => diagnostics.push(unrouted_fact(fact)),
        }
    }
    diagnostics.extend(
        by_target
            .into_values()
            .flatten()
            .map(unmatched_modifier_effect),
    );

    EncounterRuntimeProjection {
        runtime: EncounterRuntimeView {
            hazard: None,
            level,
            vitals: Some(vitals),
            defenses: armor_class.map(|armor_class| EncounterRuntimeDefensesView { armor_class }),
            saves: (saves.fortitude.is_some() || saves.reflex.is_some() || saves.will.is_some())
                .then_some(saves),
            awareness: perception.map(|perception| EncounterRuntimeAwarenessView { perception }),
            abilities: (abilities.strength.is_some()
                || abilities.dexterity.is_some()
                || abilities.constitution.is_some()
                || abilities.intelligence.is_some()
                || abilities.wisdom.is_some()
                || abilities.charisma.is_some())
            .then_some(abilities),
            skills,
            movement: Some(EncounterRuntimeMovementView {
                speeds: mechanics
                    .speeds
                    .into_iter()
                    .map(|speed| speed_view(speed, participant))
                    .collect(),
            })
            .filter(|movement| !movement.speeds.is_empty()),
            resources,
            spellcasting,
            standalone_spells: Vec::new(),
            action_budget: Some(action_budget_view(participant)),
            activities: mechanics
                .activities
                .into_iter()
                .map(|activity| {
                    let variant_damage_blocked =
                        variant_damage_blocked_activity_ids.contains(&activity.activity_id);
                    let metadata = activity_runtime
                        .remove(&activity.activity_id)
                        .unwrap_or_default();
                    activity_view(activity, participant, variant_damage_blocked, metadata)
                })
                .collect(),
            conditions: participant
                .conditions
                .iter()
                .map(runtime_condition_view)
                .collect(),
            automation_limitations,
        },
        diagnostics,
    }
}

fn speed_view(
    speed: EncounterMovementInput,
    participant: &EncounterParticipant,
) -> RuntimeDistanceView {
    let base_value = speed.value_feet;
    let speed_id = speed.movement_type.clone();
    let (adjustments, suppressed_adjustments) = speed_adjustments(participant);
    let adjusted_value = apply_runtime_adjustments(base_value, &adjustments);
    let notes = speed_notes(participant);
    RuntimeDistanceView {
        movement_type: speed.movement_type,
        label: speed.label,
        base_value_feet: base_value,
        adjusted_value_feet: adjusted_value,
        adjustments: adjustments
            .into_iter()
            .map(runtime_adjustment_view)
            .collect(),
        suppressed_adjustments: suppressed_adjustments
            .into_iter()
            .map(runtime_adjustment_view)
            .collect(),
        notes: notes.into_iter().map(runtime_note_view).collect(),
        provenance: fact_provenance(
            RuntimeFactSourceView::CanonicalRecord,
            Some(RuntimeCanonicalTargetView::Movement { speed_id }),
        ),
    }
}

fn action_budget_view(participant: &EncounterParticipant) -> EncounterRuntimeActionBudgetView {
    let action_projection = action_projection(participant);
    let base_actions = 3;
    let base_reactions = 1;
    EncounterRuntimeActionBudgetView {
        actions: RuntimeCountView {
            label: "Actions".to_string(),
            base_value: base_actions,
            adjusted_value: action_projection.adjusted_actions,
            segments: action_projection.action_segments,
            adjustments: action_projection
                .adjustments
                .into_iter()
                .map(runtime_adjustment_view)
                .collect(),
            suppressed_adjustments: action_projection
                .suppressed_adjustments
                .into_iter()
                .map(runtime_adjustment_view)
                .collect(),
            provenance: runtime_rule_provenance(RuntimeRuleView::ActionBudget),
        },
        reactions: RuntimeCountView {
            label: "Reactions".to_string(),
            base_value: base_reactions,
            adjusted_value: base_reactions,
            segments: vec![RuntimeCountSegmentView {
                label: "Base".to_string(),
                value: base_reactions,
                restricted: false,
                reason: None,
            }],
            adjustments: Vec::new(),
            suppressed_adjustments: Vec::new(),
            provenance: runtime_rule_provenance(RuntimeRuleView::ActionBudget),
        },
        can_act: action_projection.can_act,
        can_react: action_projection.can_react,
        notes: action_projection
            .notes
            .into_iter()
            .map(runtime_note_view)
            .collect(),
    }
}

struct ActionProjection {
    adjusted_actions: i64,
    action_segments: Vec<RuntimeCountSegmentView>,
    adjustments: Vec<RuntimeAdjustment>,
    suppressed_adjustments: Vec<RuntimeAdjustment>,
    can_act: RuntimeCapabilityView,
    can_react: RuntimeCapabilityView,
    notes: Vec<RuntimeNote>,
}

fn action_projection(participant: &EncounterParticipant) -> ActionProjection {
    let mut quickened = Vec::new();
    let mut slowed = Vec::new();
    let mut stunned = Vec::new();
    let mut notes = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        match rule {
            ConditionRule::Quickened => quickened.push(RuntimeAdjustment {
                provenance: condition_provenance(condition),
                source: condition_source(condition),
                label: "Restricted bonus action".to_string(),
                value: 1,
                reason: Some("Use is restricted by the quickened source.".to_string()),
                floor: None,
            }),
            ConditionRule::Slowed => {
                let amount = condition_value(condition);
                let adjustment = RuntimeAdjustment {
                    provenance: condition_provenance(condition),
                    source: condition_source(condition),
                    label: "Reduced actions regained".to_string(),
                    value: -amount,
                    reason: Some("Applied to the next action-regain step.".to_string()),
                    floor: Some(0),
                };
                slowed.push(adjustment);
            }
            ConditionRule::Stunned => {
                if let Some(amount) = positive_condition_value(condition) {
                    let consumed = amount.min(3);
                    let adjustment = RuntimeAdjustment {
                        provenance: condition_provenance(condition),
                        source: condition_source(condition),
                        label: "Actions lost while stunned".to_string(),
                        value: -consumed,
                        reason: Some(
                            "Stunned reduces actions regained, then reduces its condition value."
                                .to_string(),
                        ),
                        floor: Some(0),
                    };
                    stunned.push((adjustment, amount - consumed, amount));
                }
                if let Some((label, reason)) = stunned_contextual_disposition(condition) {
                    notes.push(RuntimeNote {
                        provenance: condition_provenance(condition),
                        label,
                        reason,
                    });
                }
            }
            ConditionRule::Prone => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Prone contextual limits".to_string(),
                reason: "Prone limits movement choices such as Crawl and Stand; cover and falling consequences remain contextual; speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Immobilized => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Move actions forbidden".to_string(),
                reason: "The participant cannot use actions with the move trait; numeric Speed is unchanged."
                    .to_string(),
            }),
            _ => {}
        }
    }

    let mut adjustments = Vec::new();
    let mut suppressed_adjustments = Vec::new();
    quickened.sort_by(|left, right| left.source.cmp(&right.source));
    if let Some(applied_quickened) = quickened.first().cloned() {
        adjustments.push(applied_quickened);
        suppressed_adjustments.extend(quickened.into_iter().skip(1).map(|adjustment| {
            RuntimeAdjustment {
                reason: Some(
                    "Only one restricted bonus action from quickened applies.".to_string(),
                ),
                ..adjustment
            }
        }));
    }

    slowed.sort_by(|left, right| {
        left.value
            .cmp(&right.value)
            .then_with(|| left.source.cmp(&right.source))
    });
    let strongest_slowed = slowed.first().cloned();
    suppressed_adjustments.extend(
        slowed
            .into_iter()
            .skip(1)
            .map(|adjustment| RuntimeAdjustment {
                reason: Some("A stronger slowed penalty applies.".to_string()),
                ..adjustment
            }),
    );

    stunned.sort_by(|left, right| {
        right
            .2
            .cmp(&left.2)
            .then_with(|| left.0.source.cmp(&right.0.source))
    });
    let strongest_stunned = stunned.first().cloned();
    let stunned_provenance = strongest_stunned
        .as_ref()
        .map(|(adjustment, _, _)| adjustment.provenance.clone());
    if let Some((adjustment, _, _)) = strongest_stunned.as_ref() {
        notes.push(RuntimeNote {
            provenance: adjustment.provenance.clone(),
            label: "Own-turn stunned timing".to_string(),
            reason: "If stunned is applied during this participant's turn, finish the current action or activity, then lose remaining actions immediately to reduce stunned."
                .to_string(),
        });
    }
    suppressed_adjustments.extend(stunned.into_iter().skip(1).map(|(adjustment, _, _)| {
        RuntimeAdjustment {
            reason: Some("A stronger stunned value applies.".to_string()),
            ..adjustment
        }
    }));

    let stunned_remaining = strongest_stunned
        .as_ref()
        .map(|(_, remaining, _)| *remaining);
    match (strongest_stunned, strongest_slowed) {
        (Some((stunned_adjustment, _, _)), Some(slowed_adjustment)) => {
            let stunned_loss = -stunned_adjustment.value;
            let slowed_loss = -slowed_adjustment.value;
            let overlap = stunned_loss.min(slowed_loss);
            let remaining_slowed_loss = slowed_loss - overlap;
            adjustments.push(stunned_adjustment);
            if remaining_slowed_loss > 0 {
                adjustments.push(RuntimeAdjustment {
                    value: -remaining_slowed_loss,
                    reason: Some(format!(
                        "{overlap} action loss already counts toward slowed; {remaining_slowed_loss} additional slowed action loss applies."
                    )),
                    ..slowed_adjustment
                });
            } else {
                suppressed_adjustments.push(RuntimeAdjustment {
                    reason: Some(format!(
                        "All {slowed_loss} slowed action loss is already counted by stunned."
                    )),
                    ..slowed_adjustment
                });
            }
        }
        (Some((stunned_adjustment, _, _)), None) => adjustments.push(stunned_adjustment),
        (None, Some(slowed_adjustment)) => adjustments.push(slowed_adjustment),
        (None, None) => {}
    }

    let adjusted_actions = apply_runtime_adjustments(3, &adjustments);
    let mut action_segments = vec![RuntimeCountSegmentView {
        label: "Base".to_string(),
        value: (adjusted_actions - restricted_bonus_total(&adjustments)).max(0),
        restricted: false,
        reason: None,
    }];
    for adjustment in adjustments.iter().filter(|adjustment| adjustment.value > 0) {
        action_segments.push(RuntimeCountSegmentView {
            label: adjustment.source.clone(),
            value: adjustment.value,
            restricted: adjustment.reason.is_some(),
            reason: adjustment.reason.clone(),
        });
    }

    let stunned_still_active = stunned_remaining.is_some_and(|remaining| remaining > 0);
    let stunned_capability = if stunned_still_active {
        RuntimeCapabilityView {
            available: false,
            provenance: stunned_provenance,
            reason: Some("Cannot act while stunned remains active.".to_string()),
        }
    } else {
        RuntimeCapabilityView {
            available: true,
            provenance: None,
            reason: None,
        }
    };

    let capability = if participant.defeated {
        RuntimeCapabilityView {
            available: false,
            provenance: Some(participant_state_provenance()),
            reason: Some("Defeated participants cannot act or react.".to_string()),
        }
    } else {
        stunned_capability
    };

    ActionProjection {
        adjusted_actions,
        action_segments,
        adjustments,
        suppressed_adjustments,
        can_act: capability.clone(),
        can_react: capability,
        notes,
    }
}

fn restricted_bonus_total(adjustments: &[RuntimeAdjustment]) -> i64 {
    adjustments
        .iter()
        .filter(|adjustment| adjustment.value > 0)
        .map(|adjustment| adjustment.value)
        .sum()
}

fn apply_runtime_adjustments(base_value: i64, adjustments: &[RuntimeAdjustment]) -> i64 {
    adjustments.iter().fold(base_value, |value, adjustment| {
        let adjusted = value + adjustment.value;
        adjustment
            .floor
            .map_or(adjusted, |floor| adjusted.max(floor.min(base_value)))
    })
}

fn activity_view(
    activity: EncounterActivityInput,
    participant: &EncounterParticipant,
    variant_damage_blocked: bool,
    metadata: RuntimeActivityMetadata,
) -> EncounterRuntimeActivityView {
    let activity_id = activity.activity_id.clone();
    let kind = activity.kind;
    let usage = activity.usage;
    let mut variant_damage_available = !variant_damage_blocked;
    let damage = activity
        .damage
        .into_iter()
        .map(|damage| {
            damage_view(
                damage,
                &activity_id,
                kind,
                usage,
                participant,
                &mut variant_damage_available,
            )
        })
        .collect();
    EncounterRuntimeActivityView {
        activity_id: activity.activity_id,
        label: activity.label,
        kind: activity_kind_view(kind),
        usage: activity_usage_view(usage),
        availability: None,
        traits: Vec::new(),
        action_cost: metadata.action_cost,
        frequency: metadata.frequency,
        uses: metadata.uses,
        rolls: activity
            .rolls
            .into_iter()
            .map(|roll| activity_roll_view(roll, &activity_id, kind, participant))
            .collect(),
        damage,
        modes: Vec::new(),
        content: None,
        provenance: fact_provenance(RuntimeFactSourceView::CanonicalRecord, None),
    }
}

fn activity_roll_view(
    roll: EncounterActivityRoll,
    activity_id: &str,
    activity_kind: EncounterActivityKind,
    participant: &EncounterParticipant,
) -> RuntimeRollView {
    let roll_id = roll.roll_id.clone();
    let mut modifiers = Vec::new();
    if let Some(modifier) = variant_roll_modifier(participant.participant_variant) {
        modifiers.push(modifier);
    }
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        for rule in expanded_condition_rules(rule) {
            modifiers.extend(condition_roll_modifiers(
                condition,
                rule,
                activity_kind,
                &roll,
            ));
        }
    }
    let (applied, suppressed) = stack_roll_modifiers(modifiers);
    let adjusted_value = applied
        .iter()
        .fold(roll.base_value, |total, modifier| total + modifier.value);
    RuntimeRollView {
        roll_id: roll.roll_id,
        label: roll.label,
        base_value: roll.base_value,
        adjusted_value,
        surface: activity_roll_surface_view(roll.surface),
        modifiers: applied.into_iter().map(roll_modifier_view).collect(),
        suppressed_modifiers: suppressed.into_iter().map(roll_modifier_view).collect(),
        provenance: fact_provenance(
            RuntimeFactSourceView::CanonicalRecord,
            Some(RuntimeCanonicalTargetView::ActivityRoll {
                activity_id: activity_id.to_string(),
                roll_id,
            }),
        ),
    }
}

fn damage_view(
    damage: EncounterDamageExpression,
    activity_id: &str,
    activity_kind: EncounterActivityKind,
    activity_usage: EncounterActivityUsage,
    participant: &EncounterParticipant,
    variant_damage_available: &mut bool,
) -> RuntimeFormulaView {
    let damage_id = damage.damage_id.clone();
    let apply_variant_damage = *variant_damage_available
        && damage.effect_kind == DamageEffectKind::Damage
        && matches!(
            activity_kind,
            EncounterActivityKind::Strike | EncounterActivityKind::Spell
        );
    if apply_variant_damage {
        *variant_damage_available = false;
    }
    let mut modifiers = variant_damage_modifier(
        participant.participant_variant,
        activity_kind,
        activity_usage,
        damage.effect_kind,
        apply_variant_damage,
    )
    .into_iter()
    .collect::<Vec<_>>();
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        for rule in expanded_condition_rules(rule) {
            modifiers.extend(condition_damage_modifiers(
                condition,
                rule,
                activity_kind,
                &damage,
            ));
        }
    }
    let modifier_total = modifiers.iter().map(|modifier| modifier.value).sum::<i64>();
    let adjusted_formula =
        (modifier_total != 0).then(|| adjusted_formula(&damage.formula, modifier_total));
    RuntimeFormulaView {
        damage_id: damage.damage_id,
        label: damage.label,
        formula: damage.formula,
        adjusted_formula,
        damage_type: damage.damage_type,
        effect_kind: damage_effect_kind_view(damage.effect_kind),
        modifiers,
        provenance: fact_provenance(
            RuntimeFactSourceView::CanonicalRecord,
            Some(RuntimeCanonicalTargetView::ActivityDamage {
                activity_id: activity_id.to_string(),
                damage_id,
            }),
        ),
    }
}

fn condition_damage_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    activity_kind: EncounterActivityKind,
    damage: &EncounterDamageExpression,
) -> Vec<RuntimeModifierView> {
    if rule != ConditionRule::Enfeebled
        || activity_kind != EncounterActivityKind::Strike
        || damage.effect_kind != DamageEffectKind::Damage
        || damage_ability(damage) != Some(AbilityKind::Strength)
    {
        return Vec::new();
    }
    let source = condition_source(condition);
    vec![RuntimeModifierView {
        provenance: condition_provenance(condition),
        label: source,
        modifier_type: StatModifierTypeView::Status,
        value: -condition_value(condition),
    }]
}

fn damage_ability(damage: &EncounterDamageExpression) -> Option<AbilityKind> {
    match damage.ability? {
        ActivityRollAbility::Strength => Some(AbilityKind::Strength),
        ActivityRollAbility::Dexterity => Some(AbilityKind::Dexterity),
        ActivityRollAbility::Constitution => Some(AbilityKind::Constitution),
        ActivityRollAbility::Intelligence => Some(AbilityKind::Intelligence),
        ActivityRollAbility::Wisdom => Some(AbilityKind::Wisdom),
        ActivityRollAbility::Charisma => Some(AbilityKind::Charisma),
    }
}

fn variant_damage_modifier(
    variant: ParticipantVariant,
    activity_kind: EncounterActivityKind,
    usage: EncounterActivityUsage,
    effect_kind: DamageEffectKind,
    apply_variant_damage: bool,
) -> Option<RuntimeModifierView> {
    if !apply_variant_damage || effect_kind != DamageEffectKind::Damage {
        return None;
    }
    let direction = match variant {
        ParticipantVariant::Normal => return None,
        ParticipantVariant::Elite => 1,
        ParticipantVariant::Weak => -1,
    };
    let magnitude = match (activity_kind, usage) {
        (EncounterActivityKind::Strike, _) => 2,
        (EncounterActivityKind::Spell, EncounterActivityUsage::Unlimited) => 2,
        (EncounterActivityKind::Spell, EncounterActivityUsage::Limited) => 4,
        (EncounterActivityKind::Spell, EncounterActivityUsage::Ambiguous)
        | (EncounterActivityKind::Other, _) => return None,
    };
    let source = variant_source(variant).to_string();
    Some(RuntimeModifierView {
        provenance: variant_provenance(variant),
        label: format!("{source} damage adjustment"),
        modifier_type: StatModifierTypeView::Adjustment,
        value: direction * magnitude,
    })
}

fn variant_roll_modifier(variant: ParticipantVariant) -> Option<RollModifier> {
    let value = variant_stat_delta(variant)?;
    let source = variant_source(variant).to_string();
    Some(RollModifier {
        provenance: variant_provenance(variant),
        label: format!("{source} adjustment"),
        modifier_type: StatModifierTypeView::Adjustment,
        value,
    })
}

fn condition_roll_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    activity_kind: EncounterActivityKind,
    roll: &EncounterActivityRoll,
) -> Vec<RollModifier> {
    let amount = condition_value(condition);
    let source = condition_source(condition);
    let status_penalty = |value: i64| RollModifier {
        provenance: condition_provenance(condition),
        label: source.clone(),
        modifier_type: StatModifierTypeView::Status,
        value: -value,
    };
    match rule {
        ConditionRule::Frightened | ConditionRule::Sickened => vec![status_penalty(amount)],
        ConditionRule::Clumsy if roll_ability(roll) == Some(AbilityKind::Dexterity) => {
            vec![status_penalty(amount)]
        }
        ConditionRule::Enfeebled if roll_ability(roll) == Some(AbilityKind::Strength) => {
            vec![status_penalty(amount)]
        }
        ConditionRule::Stupefied if activity_kind == EncounterActivityKind::Spell => {
            vec![status_penalty(amount)]
        }
        ConditionRule::Prone if roll.surface == EncounterActivityRollSurface::AttackRoll => {
            vec![RollModifier {
                provenance: condition_provenance(condition),
                label: source,
                modifier_type: StatModifierTypeView::Circumstance,
                value: -2,
            }]
        }
        ConditionRule::OffGuard
        | ConditionRule::Fatigued
        | ConditionRule::Clumsy
        | ConditionRule::Enfeebled
        | ConditionRule::Stupefied
        | ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Stunned
        | ConditionRule::Immobilized
        | ConditionRule::Grabbed
        | ConditionRule::Restrained
        | ConditionRule::Encumbered
        | ConditionRule::Prone => Vec::new(),
    }
}

fn roll_ability(roll: &EncounterActivityRoll) -> Option<AbilityKind> {
    match roll.ability? {
        ActivityRollAbility::Strength => Some(AbilityKind::Strength),
        ActivityRollAbility::Dexterity => Some(AbilityKind::Dexterity),
        ActivityRollAbility::Constitution => Some(AbilityKind::Constitution),
        ActivityRollAbility::Intelligence => Some(AbilityKind::Intelligence),
        ActivityRollAbility::Wisdom => Some(AbilityKind::Wisdom),
        ActivityRollAbility::Charisma => Some(AbilityKind::Charisma),
    }
}

fn stack_roll_modifiers(modifiers: Vec<RollModifier>) -> (Vec<RollModifier>, Vec<RollModifier>) {
    let mut applied = Vec::new();
    let mut suppressed = Vec::new();
    let mut typed = BTreeMap::<(StatModifierTypeView, i8), RollModifier>::new();
    for modifier in modifiers {
        if modifier.modifier_type == StatModifierTypeView::Adjustment {
            applied.push(modifier);
            continue;
        }
        let sign = modifier.value.signum() as i8;
        let key = (modifier.modifier_type, sign);
        if let Some(existing) = typed.remove(&key) {
            let replace = if sign < 0 {
                modifier.value < existing.value
            } else {
                modifier.value > existing.value
            };
            if replace {
                suppressed.push(existing);
                typed.insert(key, modifier);
            } else {
                suppressed.push(modifier);
                typed.insert(key, existing);
            }
        } else {
            typed.insert(key, modifier);
        }
    }
    applied.extend(typed.into_values());
    applied.sort_by(|left, right| left.label.cmp(&right.label));
    suppressed.sort_by(|left, right| left.label.cmp(&right.label));
    (applied, suppressed)
}

fn roll_modifier_view(modifier: RollModifier) -> RuntimeModifierView {
    RuntimeModifierView {
        provenance: modifier.provenance,
        label: modifier.label,
        modifier_type: modifier.modifier_type,
        value: modifier.value,
    }
}

fn activity_kind_view(kind: EncounterActivityKind) -> EncounterRuntimeActivityKindView {
    match kind {
        EncounterActivityKind::Strike => EncounterRuntimeActivityKindView::Strike,
        EncounterActivityKind::Spell => EncounterRuntimeActivityKindView::Spell,
        EncounterActivityKind::Other => EncounterRuntimeActivityKindView::Other,
    }
}

fn activity_roll_surface_view(surface: EncounterActivityRollSurface) -> RuntimeRollSurfaceView {
    match surface {
        EncounterActivityRollSurface::AttackRoll => RuntimeRollSurfaceView::AttackRoll,
        EncounterActivityRollSurface::Dc => RuntimeRollSurfaceView::Dc,
    }
}

fn activity_usage_view(usage: EncounterActivityUsage) -> EncounterRuntimeActivityUsageView {
    match usage {
        EncounterActivityUsage::Unlimited => EncounterRuntimeActivityUsageView::Unlimited,
        EncounterActivityUsage::Limited => EncounterRuntimeActivityUsageView::Limited,
        EncounterActivityUsage::Ambiguous => EncounterRuntimeActivityUsageView::Ambiguous,
    }
}

fn damage_effect_kind_view(effect_kind: DamageEffectKind) -> RuntimeDamageEffectKindView {
    match effect_kind {
        DamageEffectKind::Damage => RuntimeDamageEffectKindView::Damage,
        DamageEffectKind::Healing => RuntimeDamageEffectKindView::Healing,
        DamageEffectKind::DamageOrHealing => RuntimeDamageEffectKindView::DamageOrHealing,
        DamageEffectKind::Unknown => RuntimeDamageEffectKindView::Unknown,
    }
}

fn adjusted_formula(formula: &str, delta: i64) -> String {
    let trimmed = formula.trim();
    if trimmed.is_empty() || delta == 0 {
        return trimmed.to_string();
    }
    let (body, existing_constant) = split_formula_constant(trimmed);
    let new_constant = existing_constant + delta;
    if new_constant == 0 {
        body.trim().to_string()
    } else if new_constant > 0 {
        format!("{} + {}", body.trim(), new_constant)
    } else {
        format!("{} - {}", body.trim(), new_constant.abs())
    }
}

fn split_formula_constant(formula: &str) -> (&str, i64) {
    let trimmed = formula.trim_end();
    let bytes = trimmed.as_bytes();
    let mut index = bytes.len();
    while index > 0 && bytes[index - 1].is_ascii_digit() {
        index -= 1;
    }
    if index == bytes.len() {
        return (trimmed, 0);
    }
    let number = trimmed[index..].parse::<i64>().unwrap_or(0);
    let prefix = trimmed[..index].trim_end();
    let Some(operator) = prefix.as_bytes().last().copied() else {
        return (trimmed, 0);
    };
    if operator != b'+' && operator != b'-' {
        return (trimmed, 0);
    }
    let body = prefix[..prefix.len() - 1].trim_end();
    if body.is_empty() {
        return (trimmed, 0);
    }
    if operator == b'+' {
        (body, number)
    } else {
        (body, -number)
    }
}

fn runtime_number_view(
    value: EncounterMechanicValue,
    modifiers: Vec<CandidateModifier>,
) -> RuntimeNumberView {
    let provenance = canonical_provenance(&value.target, None);
    let EncounterMechanicScalar::Number(base_value) = value.base_value;
    let (applied, suppressed) = stack_modifiers(modifiers);
    let adjusted_value = applied
        .iter()
        .fold(base_value, |total, modifier| total + modifier.value);
    let adjusted_value = if value.target == MechanicTarget::MaxHp {
        adjusted_value.max(1)
    } else {
        adjusted_value
    };
    RuntimeNumberView {
        label: value.label,
        base_value,
        adjusted_value,
        modifiers: applied.into_iter().map(modifier_view).collect(),
        suppressed_modifiers: suppressed.into_iter().map(modifier_view).collect(),
        provenance,
    }
}

fn unmatched_modifier_effect(modifier: CandidateModifier) -> EncounterProjectionDiagnostic {
    EncounterProjectionDiagnostic {
        code: EncounterProjectionDiagnosticCode::UnmatchedRuntimeModifier,
        message: format!(
            "{} was not applied because no supported base value was available for target {}",
            modifier.label,
            modifier.target.id()
        ),
        canonical_target: canonical_target_view(&modifier.target),
    }
}

fn stack_modifiers(
    modifiers: Vec<CandidateModifier>,
) -> (Vec<CandidateModifier>, Vec<CandidateModifier>) {
    let mut applied = Vec::new();
    let mut suppressed = Vec::new();
    let mut typed = BTreeMap::<(StatModifierTypeView, i8), CandidateModifier>::new();
    for modifier in modifiers {
        if modifier.modifier_type == StatModifierTypeView::Adjustment {
            applied.push(modifier);
            continue;
        }
        let sign = modifier.value.signum() as i8;
        let key = (modifier.modifier_type, sign);
        if let Some(existing) = typed.remove(&key) {
            let replace = if sign < 0 {
                modifier.value < existing.value
            } else {
                modifier.value > existing.value
            };
            if replace {
                suppressed.push(existing);
                typed.insert(key, modifier);
            } else {
                suppressed.push(modifier);
                typed.insert(key, existing);
            }
        } else {
            typed.insert(key, modifier);
        }
    }
    applied.extend(typed.into_values());
    applied.sort_by(|left, right| left.label.cmp(&right.label));
    suppressed.sort_by(|left, right| left.label.cmp(&right.label));
    (applied, suppressed)
}

fn modifier_view(modifier: CandidateModifier) -> RuntimeModifierView {
    RuntimeModifierView {
        provenance: modifier.provenance,
        label: modifier.label,
        modifier_type: modifier.modifier_type,
        value: modifier.value,
    }
}

fn variant_modifiers(
    variant: ParticipantVariant,
    mechanics: &EncounterMechanicsInput,
) -> Vec<CandidateModifier> {
    let Some(value_delta) = variant_stat_delta(variant) else {
        return Vec::new();
    };
    let source = variant_source(variant);
    let mut modifiers = mechanics
        .values
        .iter()
        .filter(|value| value.facets.surface.is_check_or_dc())
        .map(|value| CandidateModifier {
            target: value.target.clone(),
            provenance: variant_provenance(variant),
            label: format!("{source} adjustment"),
            modifier_type: StatModifierTypeView::Adjustment,
            value: value_delta,
        })
        .collect::<Vec<_>>();
    if let Some(hp_delta) = variant_hp_delta(variant, mechanics.level) {
        modifiers.push(CandidateModifier {
            target: MechanicTarget::MaxHp,
            provenance: variant_provenance(variant),
            label: format!("{source} HP adjustment"),
            modifier_type: StatModifierTypeView::Adjustment,
            value: hp_delta,
        });
    }
    modifiers
}

fn level_for_variant(level: Option<i64>, variant: ParticipantVariant) -> Option<i64> {
    let level = level?;
    Some(match variant {
        ParticipantVariant::Normal => level,
        ParticipantVariant::Elite if level <= 0 => level + 2,
        ParticipantVariant::Elite => level + 1,
        ParticipantVariant::Weak if level == 1 => level - 2,
        ParticipantVariant::Weak => level - 1,
    })
}

fn variant_stat_delta(variant: ParticipantVariant) -> Option<i64> {
    match variant {
        ParticipantVariant::Normal => None,
        ParticipantVariant::Elite => Some(2),
        ParticipantVariant::Weak => Some(-2),
    }
}

fn variant_hp_delta(variant: ParticipantVariant, level: Option<i64>) -> Option<i64> {
    let level = level?;
    match variant {
        ParticipantVariant::Normal => None,
        ParticipantVariant::Elite if level <= 1 => Some(10),
        ParticipantVariant::Elite if level <= 4 => Some(15),
        ParticipantVariant::Elite if level <= 19 => Some(20),
        ParticipantVariant::Elite => Some(30),
        ParticipantVariant::Weak if level <= 0 => Some(0),
        ParticipantVariant::Weak if level <= 2 => Some(-10),
        ParticipantVariant::Weak if level <= 5 => Some(-15),
        ParticipantVariant::Weak if level <= 20 => Some(-20),
        ParticipantVariant::Weak => Some(-30),
    }
}

fn variant_source(variant: ParticipantVariant) -> &'static str {
    match participant_variant_view(variant) {
        EncounterParticipantVariantView::Normal => "Normal",
        EncounterParticipantVariantView::Elite => "Elite",
        EncounterParticipantVariantView::Weak => "Weak",
    }
}

fn condition_modifiers(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
    mechanics: &EncounterMechanicsInput,
) -> Vec<CandidateModifier> {
    let amount = condition_value(condition);
    let source = condition_source(condition);
    let status_penalty = |target: MechanicTarget, label: String, value: i64| CandidateModifier {
        target,
        provenance: condition_provenance(condition),
        label,
        modifier_type: StatModifierTypeView::Status,
        value: -value,
    };
    match rule {
        ConditionRule::Frightened | ConditionRule::Sickened => mechanics
            .values
            .iter()
            .filter(|value| value.facets.surface.is_check_or_dc())
            .map(|value| status_penalty(value.target.clone(), source.clone(), amount))
            .collect(),
        ConditionRule::Fatigued => mechanics
            .values
            .iter()
            .filter(|value| {
                matches!(
                    value.target,
                    MechanicTarget::ArmorClass
                        | MechanicTarget::Save {
                            save: SaveKind::Fortitude | SaveKind::Reflex | SaveKind::Will
                        }
                )
            })
            .map(|value| status_penalty(value.target.clone(), source.clone(), 1))
            .collect(),
        ConditionRule::OffGuard => vec![CandidateModifier {
            target: MechanicTarget::ArmorClass,
            provenance: condition_provenance(condition),
            label: source,
            modifier_type: StatModifierTypeView::Circumstance,
            value: -2,
        }],
        ConditionRule::Clumsy => ability_targets(mechanics, AbilityKind::Dexterity)
            .into_iter()
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
        ConditionRule::Enfeebled => ability_targets(mechanics, AbilityKind::Strength)
            .into_iter()
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
        ConditionRule::Stupefied => mental_targets(mechanics)
            .into_iter()
            .chain(
                mechanics
                    .values
                    .iter()
                    .filter(|value| {
                        matches!(
                            value.target,
                            MechanicTarget::SpellcastingAttack { .. }
                                | MechanicTarget::SpellcastingDc { .. }
                        )
                    })
                    .map(|value| value.target.clone()),
            )
            .map(|target| status_penalty(target, source.clone(), amount))
            .collect(),
        ConditionRule::Prone => mechanics
            .values
            .iter()
            .filter(|value| matches!(value.target, MechanicTarget::SpellcastingAttack { .. }))
            .map(|value| CandidateModifier {
                target: value.target.clone(),
                provenance: condition_provenance(condition),
                label: source.clone(),
                modifier_type: StatModifierTypeView::Circumstance,
                value: -2,
            })
            .collect(),
        ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Stunned
        | ConditionRule::Immobilized
        | ConditionRule::Grabbed
        | ConditionRule::Restrained
        | ConditionRule::Encumbered => Vec::new(),
    }
}

fn speed_adjustments(
    participant: &EncounterParticipant,
) -> (Vec<RuntimeAdjustment>, Vec<RuntimeAdjustment>) {
    let mut penalties = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        let source = condition_source(condition);
        if rule == ConditionRule::Encumbered {
            penalties.push(RuntimeAdjustment {
                provenance: condition_provenance(condition),
                source: source.clone(),
                label: "Speed penalty".to_string(),
                value: -10,
                reason: Some(
                    "Encumbered reduces speeds by 10 feet, to a minimum of 5 feet.".to_string(),
                ),
                floor: Some(5),
            });
        }
    }
    penalties.sort_by(|left, right| left.source.cmp(&right.source));

    let mut adjustments = Vec::new();
    let mut suppressed = Vec::new();
    if let Some(penalty) = penalties.first().cloned() {
        adjustments.push(penalty);
        suppressed.extend(
            penalties
                .into_iter()
                .skip(1)
                .map(|adjustment| RuntimeAdjustment {
                    reason: Some("An equivalent speed penalty already applies.".to_string()),
                    ..adjustment
                }),
        );
    }
    (adjustments, suppressed)
}

fn speed_notes(participant: &EncounterParticipant) -> Vec<RuntimeNote> {
    let mut notes = Vec::new();
    for (condition, rule) in participant_condition_rules(participant) {
        match rule {
            ConditionRule::Grabbed => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Grabbed restrictions".to_string(),
                reason: "Grabbed makes the participant off-guard, forbids move-trait actions through immobilized, and can affect manipulate actions; numeric Speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Restrained => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Restrained restrictions".to_string(),
                reason:
                    "Restrained makes the participant off-guard, forbids move-trait actions through immobilized, and restricts attack and manipulate actions; numeric Speed is unchanged."
                        .to_string(),
            }),
            ConditionRule::Immobilized => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Move actions forbidden".to_string(),
                reason: "The participant cannot use actions with the move trait; numeric Speed is unchanged."
                    .to_string(),
            }),
            ConditionRule::Prone => notes.push(RuntimeNote {
                provenance: condition_provenance(condition),
                label: "Prone contextual limits".to_string(),
                reason:
                    "Prone limits movement choices such as Crawl and Stand; cover and falling consequences remain contextual; numeric Speed is unchanged."
                        .to_string(),
            }),
            _ => {}
        }
    }
    notes
}

fn participant_automation_limitations(
    participant: &EncounterParticipant,
) -> Vec<EncounterRuntimeAutomationLimitationView> {
    participant_condition_rules(participant)
        .flat_map(|(condition, rule)| condition_automation_limitations(condition, rule))
        .collect()
}

fn condition_automation_limitations(
    condition: &EncounterParticipantCondition,
    rule: ConditionRule,
) -> Vec<EncounterRuntimeAutomationLimitationView> {
    match rule {
        ConditionRule::Fatigued => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated,
            "Travel exploration activity restrictions require manual adjudication.",
        )],
        ConditionRule::Clumsy => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::ConditionAttackAdjustmentPartial,
            "Only structured activity attack rolls receive this Dexterity-based penalty.",
        )],
        ConditionRule::Enfeebled => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::ConditionDamageAdjustmentPartial,
            "Only structured Strength-based strike damage receives this penalty.",
        )],
        ConditionRule::Stupefied => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::SpellDisruptionCheckNotAutomated,
            "Spell disruption flat checks require manual resolution.",
        )],
        ConditionRule::Grabbed => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::ManipulateActionCheckNotAutomated,
            "Manipulate-action flat checks and resulting action loss require manual resolution.",
        )],
        ConditionRule::Restrained => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::RestrictedActionExceptionsNotAutomated,
            "Exceptions that allow attacks or manipulate actions while restrained require manual adjudication.",
        )],
        ConditionRule::Stunned => (positive_condition_value(condition).is_some()
            && condition.duration_rounds.is_some())
            .then(|| {
                condition_limitation(
                    condition,
                    EncounterRuntimeAutomationLimitationCodeView::StunnedTimingRequiresAdjudication,
                    "The numeric action loss is applied, while duration timing and lifecycle require manual adjudication.",
                )
            })
            .into_iter()
            .collect(),
        ConditionRule::Prone => vec![condition_limitation(
            condition,
            EncounterRuntimeAutomationLimitationCodeView::ProneContextRequiresAdjudication,
            "Prone movement choices, cover, and falling consequences require manual adjudication.",
        )],
        ConditionRule::Frightened
        | ConditionRule::Sickened
        | ConditionRule::OffGuard
        | ConditionRule::Slowed
        | ConditionRule::Quickened
        | ConditionRule::Immobilized
        | ConditionRule::Encumbered => Vec::new(),
    }
}

fn condition_limitation(
    condition: &EncounterParticipantCondition,
    code: EncounterRuntimeAutomationLimitationCodeView,
    message: &str,
) -> EncounterRuntimeAutomationLimitationView {
    EncounterRuntimeAutomationLimitationView {
        code,
        target: EncounterRuntimeAutomationLimitationTargetView::Condition {
            condition_id: condition.condition_id,
        },
        message: message.to_string(),
    }
}

fn participant_condition_rules(
    participant: &EncounterParticipant,
) -> impl Iterator<Item = (&EncounterParticipantCondition, ConditionRule)> {
    let restrained_present = participant.conditions.iter().any(|condition| {
        condition_rule_for_key(condition.condition_key.as_deref())
            == Some(ConditionRule::Restrained)
    });
    let mut rules = Vec::new();
    let mut suppressed_grabbed = Vec::new();
    for condition in &participant.conditions {
        let Some(rule) = condition_rule_for_key(condition.condition_key.as_deref()) else {
            continue;
        };
        if restrained_present && rule == ConditionRule::Grabbed {
            suppressed_grabbed.push(condition);
            continue;
        }
        rules.extend(
            expanded_condition_rules(rule)
                .into_iter()
                .map(|rule| (condition, rule)),
        );
    }
    rules.extend(
        suppressed_grabbed
            .into_iter()
            .map(|condition| (condition, ConditionRule::OffGuard)),
    );
    rules.into_iter()
}

fn expanded_condition_rules(rule: ConditionRule) -> Vec<ConditionRule> {
    match rule {
        ConditionRule::Grabbed => vec![
            ConditionRule::Grabbed,
            ConditionRule::OffGuard,
            ConditionRule::Immobilized,
        ],
        ConditionRule::Restrained => vec![
            ConditionRule::Restrained,
            ConditionRule::OffGuard,
            ConditionRule::Immobilized,
        ],
        ConditionRule::Encumbered => vec![ConditionRule::Encumbered, ConditionRule::Clumsy],
        ConditionRule::Prone => vec![ConditionRule::Prone, ConditionRule::OffGuard],
        _ => vec![rule],
    }
}

fn runtime_adjustment_view(adjustment: RuntimeAdjustment) -> RuntimeAdjustmentView {
    RuntimeAdjustmentView {
        provenance: adjustment.provenance,
        label: adjustment.label,
        value: adjustment.value,
        reason: adjustment.reason,
    }
}

fn runtime_note_view(note: RuntimeNote) -> RuntimeEffectNoteView {
    RuntimeEffectNoteView {
        provenance: note.provenance,
        label: note.label,
        reason: note.reason,
    }
}

fn ability_targets(
    mechanics: &EncounterMechanicsInput,
    ability: AbilityKind,
) -> Vec<MechanicTarget> {
    mechanics
        .values
        .iter()
        .filter(|value| {
            value.facets.surface != MechanicSurface::RawModifier
                && value.facets.ability == Some(ability)
        })
        .map(|value| value.target.clone())
        .collect()
}

fn mental_targets(mechanics: &EncounterMechanicsInput) -> Vec<MechanicTarget> {
    mechanics
        .values
        .iter()
        .filter(|value| {
            value.facets.surface != MechanicSurface::RawModifier
                && matches!(
                    value.facets.ability,
                    Some(AbilityKind::Intelligence | AbilityKind::Wisdom | AbilityKind::Charisma)
                )
        })
        .map(|value| value.target.clone())
        .collect()
}

fn condition_value(condition: &EncounterParticipantCondition) -> i64 {
    condition.value.unwrap_or(1).max(1)
}

fn positive_condition_value(condition: &EncounterParticipantCondition) -> Option<i64> {
    condition.value.filter(|value| *value > 0)
}

fn stunned_contextual_disposition(
    condition: &EncounterParticipantCondition,
) -> Option<(String, String)> {
    if positive_condition_value(condition).is_none() || condition.duration_rounds.is_none() {
        return None;
    }
    Some((
        "Stunned duration timing".to_string(),
        "The numeric action loss is applied, while duration timing and lifecycle remain contextual."
            .to_string(),
    ))
}

fn condition_source(condition: &EncounterParticipantCondition) -> String {
    let name = condition.name.trim();
    if name.is_empty() {
        return "Condition".to_string();
    }
    match condition.value {
        Some(value) if value > 0 => format!("{name} {value}"),
        _ => name.to_string(),
    }
}

#[cfg(test)]
fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_separator = false;
        } else if !previous_separator && !slug.is_empty() {
            slug.push('-');
            previous_separator = true;
        }
    }
    if previous_separator {
        slug.pop();
    }
    slug
}

#[cfg(test)]
// Typed mutation fixtures use an audit-distinct iterator spelling so the frozen
// consumer residue check remains sensitive to ordinary lookup code.
#[allow(clippy::filter_next)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::*;
    use atlas_app_model::CreateEncounterRequest;
    use atlas_domain::{RecordKey, RecordKind};
    use atlas_record::{
        AtlasRecord, CreatureComponentId, CreatureSkillKind, FoundryDocumentType,
        FoundryRecordInfo, FoundryRecordType, MechanicFacets, RecordClassification, RecordIdentity,
        RecordProvenance,
    };
    use atlas_runtime::{AtlasPathMode, AtlasPathOverrides, AtlasRuntimeOptions};

    use crate::service::{AtlasAppService, RetrievalBackend};

    macro_rules! write_sample_json {
        ($path:expr, $value:expr $(,)?) => {{
            let mut bytes =
                serde_json::to_vec_pretty(&$value).expect("sample JSON should serialize");
            bytes.push(b'\n');
            fs::write($path, bytes).expect("sample JSON should write");
        }};
    }

    fn provenance_source(provenance: &RuntimeFactProvenanceView) -> &str {
        match &provenance.source {
            RuntimeFactSourceView::CanonicalRecord => "Canonical source",
            RuntimeFactSourceView::ParticipantState => "Participant state",
            RuntimeFactSourceView::ParticipantVariant { variant } => match variant {
                EncounterParticipantVariantView::Normal => "Normal",
                EncounterParticipantVariantView::Elite => "Elite",
                EncounterParticipantVariantView::Weak => "Weak",
            },
            RuntimeFactSourceView::Condition { label, .. } => label,
            RuntimeFactSourceView::RuntimeRule { rule } => match rule {
                RuntimeRuleView::ActionBudget => "Action budget",
                RuntimeRuleView::Movement => "Movement",
                RuntimeRuleView::HazardConvenience => "Hazard convenience",
            },
        }
    }

    fn project_fixture(
        participant: &EncounterParticipant,
        mechanics: &EncounterMechanicsInput,
    ) -> Option<EncounterRuntimeView> {
        Some(into_public_runtime(apply_participant_effects(
            participant,
            mechanics.clone(),
            Vec::new(),
            Vec::new(),
            BTreeSet::new(),
            BTreeMap::new(),
        )))
    }

    fn project_canonical(participant: &EncounterParticipant) -> EncounterRuntimeView {
        project_canonical_projection(participant, canonical_projection())
    }

    fn project_canonical_projection(
        participant: &EncounterParticipant,
        projection: CanonicalMechanicsProjection,
    ) -> EncounterRuntimeView {
        into_public_runtime(project_canonical_projection_with_diagnostics(
            participant,
            projection,
        ))
    }

    fn project_canonical_projection_with_diagnostics(
        participant: &EncounterParticipant,
        projection: CanonicalMechanicsProjection,
    ) -> EncounterRuntimeProjection {
        let mechanics = canonical_participant_mechanics(projection);
        apply_participant_effects(
            participant,
            mechanics.view,
            mechanics.diagnostics,
            mechanics.automation_limitations,
            mechanics.variant_damage_blocked_activity_ids,
            mechanics.activity_runtime,
        )
    }

    fn project_mechanic_values_with_diagnostics(
        values: Vec<EncounterMechanicValue>,
    ) -> EncounterRuntimeProjection {
        apply_participant_effects(
            &participant(ParticipantVariant::Normal, Vec::new()),
            EncounterMechanicsInput {
                level: Some(5),
                values,
                speeds: Vec::new(),
                activities: Vec::new(),
            },
            Vec::new(),
            Vec::new(),
            BTreeSet::new(),
            BTreeMap::new(),
        )
    }

    fn named_runtime_mechanic(
        target: MechanicTarget,
        label: &str,
        value: i64,
    ) -> EncounterMechanicValue {
        let facets = match &target {
            MechanicTarget::MaxHp => MechanicFacets::hit_points(),
            MechanicTarget::ArmorClass => MechanicFacets::armor_class(),
            MechanicTarget::Perception => MechanicFacets::perception(),
            MechanicTarget::Save { save } => MechanicFacets::saving_throw(*save),
            MechanicTarget::AbilityModifier { ability } => {
                MechanicFacets::ability_modifier(*ability)
            }
            MechanicTarget::CreatureSkill { kind, .. } => MechanicFacets::creature_skill(*kind),
            MechanicTarget::SpellcastingAttack { .. } => MechanicFacets::spellcasting_attack(),
            MechanicTarget::SpellcastingDc { .. } => MechanicFacets::spellcasting_dc(),
            _ => panic!("test helper requires a zero-or-one named runtime target"),
        };
        EncounterMechanicValue {
            target,
            label: label.to_string(),
            base_value: EncounterMechanicScalar::Number(value),
            facets,
        }
    }

    fn named_runtime_value<'a>(
        runtime: &'a EncounterRuntimeView,
        target: &MechanicTarget,
    ) -> Option<&'a RuntimeNumberView> {
        match target {
            MechanicTarget::MaxHp => runtime
                .vitals
                .as_ref()
                .and_then(|vitals| vitals.maximum_hp.as_ref()),
            MechanicTarget::ArmorClass => runtime
                .defenses
                .as_ref()
                .map(|defenses| &defenses.armor_class),
            MechanicTarget::Perception => runtime
                .awareness
                .as_ref()
                .map(|awareness| &awareness.perception),
            MechanicTarget::Save {
                save: SaveKind::Fortitude,
            } => runtime
                .saves
                .as_ref()
                .and_then(|saves| saves.fortitude.as_ref()),
            MechanicTarget::Save {
                save: SaveKind::Reflex,
            } => runtime
                .saves
                .as_ref()
                .and_then(|saves| saves.reflex.as_ref()),
            MechanicTarget::Save {
                save: SaveKind::Will,
            } => runtime.saves.as_ref().and_then(|saves| saves.will.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Strength,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.strength.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Dexterity,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.dexterity.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Constitution,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.constitution.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Intelligence,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.intelligence.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Wisdom,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.wisdom.as_ref()),
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Charisma,
            } => runtime
                .abilities
                .as_ref()
                .and_then(|abilities| abilities.charisma.as_ref()),
            _ => None,
        }
    }

    fn spellcasting_runtime_roll<'a>(
        runtime: &'a EncounterRuntimeView,
        target: &MechanicTarget,
    ) -> Option<&'a RuntimeRollView> {
        let (entry_id, attack) = match target {
            MechanicTarget::SpellcastingAttack {
                entry_occurrence_id,
            } => (entry_occurrence_id.as_str(), true),
            MechanicTarget::SpellcastingDc {
                entry_occurrence_id,
            } => (entry_occurrence_id.as_str(), false),
            _ => return None,
        };
        runtime
            .spellcasting
            .iter()
            .find(|entry| entry.entry_id == entry_id)
            .and_then(|entry| {
                if attack {
                    entry.attack.as_ref()
                } else {
                    entry.dc.as_ref()
                }
            })
    }

    fn duplicate_diagnostics_for<'a>(
        projection: &'a EncounterRuntimeProjection,
        target: &MechanicTarget,
    ) -> Vec<&'a EncounterProjectionDiagnostic> {
        let canonical_target = canonical_target_view(target);
        projection
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == EncounterProjectionDiagnosticCode::DuplicateRuntimeFact
                    && diagnostic.canonical_target == canonical_target
            })
            .collect()
    }

    fn activity_fact_mut<'a>(
        projection: &'a mut CanonicalMechanicsProjection,
        target: &MechanicTarget,
    ) -> &'a mut MechanicFact {
        projection
            .activities
            .iter_mut()
            .flat_map(|activity| activity.facts.iter_mut())
            .filter(|fact| &fact.target == target)
            .next()
            .expect("canonical activity fact should exist")
    }

    fn canonical_activity_mut<'a>(
        projection: &'a mut CanonicalMechanicsProjection,
        occurrence_id: &str,
    ) -> &'a mut CanonicalMechanicActivity {
        projection
            .activities
            .iter_mut()
            .find(|activity| activity.occurrence_id.as_str() == occurrence_id)
            .expect("canonical activity should exist")
    }

    fn canonical_strike_with_unsupported_first_formula(
        formula: FactValue<String>,
    ) -> CanonicalMechanicsProjection {
        let mut projection = canonical_projection();
        let strike = canonical_activity_mut(&mut projection, "strike-claw");
        let main_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "main"
                )
            })
            .expect("main damage should exist");
        let main = strike.facts[main_index].clone();

        let mut unsupported_first = main.clone();
        unsupported_first.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "unsupported-first".to_string(),
        };
        unsupported_first.label = "unsupported-first".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut unsupported_first.value {
            damage.id = "unsupported-first".to_string();
            damage.formula = formula;
        }
        strike.facts.insert(main_index, unsupported_first);

        let mut secondary = main;
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "secondary".to_string(),
        };
        secondary.label = "secondary".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "secondary".to_string();
            damage.formula = FactValue::Value("1d4".to_string());
        }
        strike.facts.push(secondary);
        projection
    }

    fn assert_unsupported_damage_formula_disposition(
        projection: &EncounterRuntimeProjection,
        expected_formula: &str,
    ) {
        let target = MechanicTarget::ActivityDamage {
            occurrence_id: atlas_record::CreatureOccurrenceId::new("strike-claw")
                .expect("occurrence id should be valid"),
            damage_id: "unsupported-first".to_string(),
        };
        let expected_reason = format!(
            "unsupported-first: damage formula cannot populate a structured encounter damage expression; target={}; id=\"unsupported-first\"; formula={expected_formula}",
            target.id()
        );
        let matching = projection
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == EncounterProjectionDiagnosticCode::UnavailableCanonicalFact
                    && diagnostic.canonical_target == canonical_target_view(&target)
            })
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].message, expected_reason);
    }

    fn canonical_projection() -> CanonicalMechanicsProjection {
        let skill_id =
            atlas_record::CreatureComponentId::new("athletics").expect("skill id should be valid");
        let speed_id =
            atlas_record::CreatureComponentId::new("land").expect("speed id should be valid");
        let resource_id =
            atlas_record::CreatureComponentId::new("focus").expect("resource id should be valid");
        let strike_id = atlas_record::CreatureOccurrenceId::new("strike-claw")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        CanonicalMechanicsProjection {
            record_key: RecordKey::parse("actors:canonical").expect("record key should parse"),
            level: FactValue::Value(5),
            facts: vec![
                MechanicFact {
                    target: MechanicTarget::ArmorClass,
                    label: "AC".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(22)),
                    facets: MechanicFacets::armor_class(),
                },
                MechanicFact {
                    target: MechanicTarget::MaxHp,
                    label: "Max HP".to_string(),
                    value: MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(60))),
                    facets: MechanicFacets::hit_points(),
                },
                MechanicFact {
                    target: MechanicTarget::Perception,
                    label: "Perception".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(13)),
                    facets: MechanicFacets::perception(),
                },
                MechanicFact {
                    target: MechanicTarget::Save {
                        save: atlas_record::SaveKind::Reflex,
                    },
                    label: "Reflex".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(12)),
                    facets: MechanicFacets::saving_throw(atlas_record::SaveKind::Reflex),
                },
                MechanicFact {
                    target: MechanicTarget::CreatureSkill {
                        skill_id,
                        kind: atlas_record::CreatureSkillKind::Athletics,
                    },
                    label: "Athletics".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(9)),
                    facets: MechanicFacets::creature_skill(
                        atlas_record::CreatureSkillKind::Athletics,
                    ),
                },
                MechanicFact {
                    target: MechanicTarget::Movement { speed_id },
                    label: "Land Speed".to_string(),
                    value: MechanicBaseValue::Integer(FactValue::Value(25)),
                    facets: MechanicFacets::movement(),
                },
                MechanicFact {
                    target: MechanicTarget::ResourceMaximum { resource_id },
                    label: "Focus".to_string(),
                    value: MechanicBaseValue::ResourceAmount(FactValue::Value(
                        CreatureResourceAmount::Integer(2),
                    )),
                    facets: MechanicFacets::resource(),
                },
            ],
            activities: vec![
                CanonicalMechanicActivity {
                    occurrence_id: strike_id.clone(),
                    family: MechanicActivityFamily::Strike,
                    label: "Claw".to_string(),
                    authored_order: 0,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::ActivityRoll {
                                occurrence_id: strike_id.clone(),
                                roll_id: "attack".to_string(),
                            },
                            label: "Attack".to_string(),
                            value: MechanicBaseValue::Roll(atlas_record::CreatureRoll {
                                id: "attack".to_string(),
                                label: "Attack".to_string(),
                                kind: CreatureRollKind::Attack,
                                value: FactValue::Value(12),
                                ability: FactValue::Value(ActivityRollAbility::Strength),
                            }),
                            facets: MechanicFacets::roll(
                                atlas_record::MechanicSourceFamily::Strike,
                                CreatureRollKind::Attack,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityDamage {
                                occurrence_id: strike_id.clone(),
                                damage_id: "main".to_string(),
                            },
                            label: "main".to_string(),
                            value: MechanicBaseValue::Damage(CreatureDamage {
                                id: "main".to_string(),
                                formula: FactValue::Value("1d6+4".to_string()),
                                damage_type: FactValue::Value("slashing".to_string()),
                                category: FactValue::Missing,
                                kinds: FactValue::Value(vec![CreatureDamageKind::Damage]),
                                apply_modifier: FactValue::Value(CreatureSourceScalar::Value(true)),
                            }),
                            facets: MechanicFacets::damage(
                                atlas_record::MechanicSourceFamily::Strike,
                            ),
                        },
                    ],
                    unsupported: vec![UnsupportedMechanic {
                        target: None,
                        activity_occurrence_id: Some(strike_id),
                        source_path: "activities.strike-claw.unsupported".to_string(),
                        value: UnsupportedMechanicValue::Capability {
                            source_item_type: "unmodeled-rule".to_string(),
                            source_slug: FactValue::Missing,
                        },
                    }],
                },
                CanonicalMechanicActivity {
                    occurrence_id: spellcasting_id.clone(),
                    family: MechanicActivityFamily::SpellcastingEntry,
                    label: "Arcane Prepared Spells".to_string(),
                    authored_order: 1,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::SpellcastingDc {
                                entry_occurrence_id: spellcasting_id.clone(),
                            },
                            label: "Spell DC".to_string(),
                            value: MechanicBaseValue::Integer(FactValue::Value(22)),
                            facets: MechanicFacets::spellcasting_dc(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellcastingAttack {
                                entry_occurrence_id: spellcasting_id.clone(),
                            },
                            label: "Spell Attack".to_string(),
                            value: MechanicBaseValue::Integer(FactValue::Value(14)),
                            facets: MechanicFacets::spellcasting_attack(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id.clone(),
                                rank: 3,
                            },
                            label: "Rank 3 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Value(
                                CreatureSourceScalar::Value(2),
                            )),
                            facets: MechanicFacets::spell_slot(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id.clone(),
                                rank: 1,
                            },
                            label: "Rank 1 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Value(
                                CreatureSourceScalar::Value(4),
                            )),
                            facets: MechanicFacets::spell_slot(),
                        },
                        MechanicFact {
                            target: MechanicTarget::SpellSlotMaximum {
                                entry_occurrence_id: spellcasting_id,
                                rank: 4,
                            },
                            label: "Rank 4 slots".to_string(),
                            value: MechanicBaseValue::SourceInteger(FactValue::Missing),
                            facets: MechanicFacets::spell_slot(),
                        },
                    ],
                    unsupported: Vec::new(),
                },
                CanonicalMechanicActivity {
                    occurrence_id: action_id.clone(),
                    family: MechanicActivityFamily::Action,
                    label: "Breath Weapon".to_string(),
                    authored_order: 2,
                    facts: vec![
                        MechanicFact {
                            target: MechanicTarget::ActivityActionCost {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Action cost".to_string(),
                            value: MechanicBaseValue::ActionCost(CreatureActionCost::Actions(2)),
                            facets: MechanicFacets::action_economy(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityUses {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Uses".to_string(),
                            value: MechanicBaseValue::Uses(FactValue::Value(CreatureUseLimit {
                                maximum: FactValue::Value(2),
                                serialized_value: FactValue::Value(1),
                            })),
                            facets: MechanicFacets::uses(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityFrequency {
                                occurrence_id: action_id.clone(),
                            },
                            label: "Frequency".to_string(),
                            value: MechanicBaseValue::Frequency(FactValue::Value(
                                atlas_record::CreatureFrequency {
                                    maximum: FactValue::Value(1),
                                    period: FactValue::Value("day".to_string()),
                                    serialized_value: FactValue::Missing,
                                },
                            )),
                            facets: MechanicFacets::frequency(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityRoll {
                                occurrence_id: action_id.clone(),
                                roll_id: "recall".to_string(),
                            },
                            label: "Recall Knowledge".to_string(),
                            value: MechanicBaseValue::Roll(CreatureRoll {
                                id: "recall".to_string(),
                                label: "Recall Knowledge".to_string(),
                                kind: CreatureRollKind::Check,
                                value: FactValue::Value(18),
                                ability: FactValue::Value(ActivityRollAbility::Intelligence),
                            }),
                            facets: MechanicFacets::roll(
                                atlas_record::MechanicSourceFamily::Action,
                                CreatureRollKind::Check,
                            ),
                        },
                        MechanicFact {
                            target: MechanicTarget::ActivityDamage {
                                occurrence_id: action_id,
                                damage_id: "fire".to_string(),
                            },
                            label: "fire".to_string(),
                            value: MechanicBaseValue::Damage(CreatureDamage {
                                id: "fire".to_string(),
                                formula: FactValue::Value("4d6".to_string()),
                                damage_type: FactValue::Value("fire".to_string()),
                                category: FactValue::Missing,
                                kinds: FactValue::Value(vec![CreatureDamageKind::Damage]),
                                apply_modifier: FactValue::Value(CreatureSourceScalar::Value(
                                    false,
                                )),
                            }),
                            facets: MechanicFacets::damage(
                                atlas_record::MechanicSourceFamily::Action,
                            ),
                        },
                    ],
                    unsupported: Vec::new(),
                },
            ],
            unsupported: Vec::new(),
        }
    }

    #[test]
    fn canonical_targets_receive_variants_without_adjusting_resources() {
        let projection = project_canonical(&participant(ParticipantVariant::Elite, Vec::new()));

        assert_eq!(
            projection.level.as_ref().map(|value| value.adjusted_value),
            Some(6)
        );
        assert_stat(
            &projection,
            RuntimeNumberField::ArmorClass,
            22,
            24,
            "Elite adjustment",
        );
        assert_stat(
            &projection,
            RuntimeNumberField::MaximumHp,
            60,
            80,
            "Elite HP adjustment",
        );
        let resource = projection
            .resources
            .iter()
            .find(|resource| resource.label == "Focus")
            .expect("canonical resource should project");
        assert_eq!(resource.resource_id, "focus");
        assert_eq!(resource.maximum.base_value, 2);
        assert_eq!(resource.maximum.adjusted_value, 2);
        assert!(resource.maximum.modifiers.is_empty());
        assert_eq!(speed(&projection, "land").base_value_feet, 25);
        assert!(
            projection
                .skills
                .iter()
                .all(|skill| !matches!(skill.kind, EncounterRuntimeSkillKindView::Legacy { .. }))
        );
    }

    #[test]
    fn canonical_weak_hp_adjustment_has_a_minimum_of_one() {
        let mut projection = canonical_projection();
        projection.level = FactValue::Value(1);
        let hp = projection
            .facts
            .iter_mut()
            .filter(|fact| fact.target == MechanicTarget::MaxHp)
            .next()
            .expect("max hp should exist");
        hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(5)));
        let mechanics = canonical_participant_mechanics(projection);
        let block = into_public_runtime(apply_participant_effects(
            &participant(ParticipantVariant::Weak, Vec::new()),
            mechanics.view,
            mechanics.diagnostics,
            mechanics.automation_limitations,
            mechanics.variant_damage_blocked_activity_ids,
            mechanics.activity_runtime,
        ));

        assert_stat(
            &block,
            RuntimeNumberField::MaximumHp,
            5,
            1,
            "Weak HP adjustment",
        );
    }

    #[test]
    fn canonical_weak_hp_bands_cover_every_boundary() {
        let cases = [
            (-1, 100),
            (0, 100),
            (1, 90),
            (2, 90),
            (3, 85),
            (5, 85),
            (6, 80),
            (20, 80),
            (21, 70),
        ];
        for (level, expected_hp) in cases {
            let mut canonical = canonical_projection();
            canonical.level = FactValue::Value(level);
            let hp = canonical
                .facts
                .iter_mut()
                .filter(|fact| fact.target == MechanicTarget::MaxHp)
                .next()
                .expect("max hp should exist");
            hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(100)));
            let projection = project_canonical_projection(
                &participant(ParticipantVariant::Weak, Vec::new()),
                canonical,
            );
            assert_eq!(
                value(&projection, RuntimeNumberField::MaximumHp).adjusted_value,
                expected_hp,
                "unexpected weak HP at starting level {level}"
            );
        }

        let mut level_zero = canonical_projection();
        level_zero.level = FactValue::Value(0);
        let hp = level_zero
            .facts
            .iter_mut()
            .filter(|fact| fact.target == MechanicTarget::MaxHp)
            .next()
            .expect("max hp should exist");
        hp.value = MechanicBaseValue::Number(FactValue::Value(CreatureNumber::Integer(5)));
        let unchanged = project_canonical_projection(
            &participant(ParticipantVariant::Weak, Vec::new()),
            level_zero,
        );
        assert_eq!(
            value(&unchanged, RuntimeNumberField::MaximumHp).adjusted_value,
            5
        );
    }

    #[test]
    fn weak_hp_variant_transitions_use_the_starting_level_band() {
        let cases = [
            (ParticipantVariant::Normal, ParticipantVariant::Weak, 5, -15),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 5, 15),
            (ParticipantVariant::Normal, ParticipantVariant::Weak, 6, -20),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 6, 20),
            (
                ParticipantVariant::Normal,
                ParticipantVariant::Weak,
                20,
                -20,
            ),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 20, 20),
            (
                ParticipantVariant::Normal,
                ParticipantVariant::Weak,
                21,
                -30,
            ),
            (ParticipantVariant::Weak, ParticipantVariant::Normal, 21, 30),
            (ParticipantVariant::Weak, ParticipantVariant::Elite, 5, 35),
            (ParticipantVariant::Elite, ParticipantVariant::Weak, 6, -40),
            (ParticipantVariant::Weak, ParticipantVariant::Elite, 20, 50),
            (ParticipantVariant::Elite, ParticipantVariant::Weak, 21, -60),
        ];

        for (old_variant, new_variant, level, expected_delta) in cases {
            assert_eq!(
                variant_hp_adjustment_delta(old_variant, new_variant, Some(level)),
                expected_delta,
                "unexpected {old_variant:?} -> {new_variant:?} HP delta at starting level {level}"
            );
        }
    }

    #[test]
    fn canonical_condition_targets_stack_and_suppress_by_type() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Frightened", Some(1)),
                condition("Sickened", Some(2)),
                condition("Off-Guard", None),
            ],
        ));

        let ac = value(&projection, RuntimeNumberField::ArmorClass);
        assert_eq!(ac.adjusted_value, 18);
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Sickened 2")
        );
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Off-Guard")
        );
        assert!(
            ac.suppressed_modifiers
                .iter()
                .any(|modifier| modifier.label == "Frightened 1")
        );
        let skill = value(&projection, RuntimeNumberField::Athletics);
        assert_eq!(skill.adjusted_value, 7);
    }

    #[test]
    fn fatigued_applies_fixed_penalty_only_to_ac_and_all_saves() {
        let projection = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Fatigued", Some(9))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");

        for (target, base) in [
            (RuntimeNumberField::ArmorClass, 22),
            (RuntimeNumberField::Fortitude, 15),
            (RuntimeNumberField::Reflex, 12),
            (RuntimeNumberField::Will, 12),
        ] {
            let stat = value(&projection, target);
            assert_eq!(stat.adjusted_value, base - 1, "unexpected {target:?}");
            assert_eq!(stat.modifiers.len(), 1, "unexpected {target:?}");
            assert!(stat.modifiers.iter().any(|modifier| {
                provenance_source(&modifier.provenance) == "Fatigued 9"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -1
            }));
        }

        for target in [
            RuntimeNumberField::MaximumHp,
            RuntimeNumberField::Perception,
            RuntimeNumberField::Athletics,
            RuntimeNumberField::Strength,
            RuntimeNumberField::Dexterity,
        ] {
            let stat = value(&projection, target);
            assert_eq!(
                stat.adjusted_value, stat.base_value,
                "unexpected {target:?}"
            );
            assert!(stat.modifiers.is_empty(), "unexpected {target:?}");
            assert!(
                stat.suppressed_modifiers.is_empty(),
                "unexpected {target:?}"
            );
        }
        for speed in &projection.movement.as_ref().expect("movement").speeds {
            assert_eq!(speed.adjusted_value_feet, speed.base_value_feet);
            assert!(speed.adjustments.is_empty());
            assert!(speed.suppressed_adjustments.is_empty());
            assert!(speed.notes.is_empty());
        }
        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert!(budget.actions.suppressed_adjustments.is_empty());
        assert!(budget.notes.is_empty());
        for activity in &projection.activities {
            for roll in &activity.rolls {
                assert!(
                    roll.modifiers
                        .iter()
                        .chain(&roll.suppressed_modifiers)
                        .all(|modifier| provenance_source(&modifier.provenance) != "Fatigued 9")
                );
            }
            for damage in &activity.damage {
                assert!(
                    damage
                        .modifiers
                        .iter()
                        .all(|modifier| provenance_source(&modifier.provenance) != "Fatigued 9")
                );
            }
        }

        let canonical = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![condition("Fatigued", None)],
        ));
        let resource = canonical
            .resources
            .iter()
            .find(|resource| resource.label == "Focus")
            .expect("canonical resource should project");
        assert_eq!(resource.maximum.adjusted_value, resource.maximum.base_value);
        assert!(resource.maximum.modifiers.is_empty());
        assert!(resource.maximum.suppressed_modifiers.is_empty());

        let limitation = projection
            .automation_limitations
            .iter()
            .find(|limitation| {
                limitation.code
                    == EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated
            })
            .expect("travel restriction should remain explicit");
        assert_eq!(
            limitation.target,
            EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 1 }
        );
        assert_eq!(
            limitation.message,
            "Travel exploration activity restrictions require manual adjudication."
        );
    }

    #[test]
    fn fatigued_uses_existing_status_stacking_and_suppression() {
        let projection = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![
                    condition("Fatigued", None),
                    condition("Frightened", Some(2)),
                    condition("Off-Guard", None),
                ],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");

        for target in [
            RuntimeNumberField::ArmorClass,
            RuntimeNumberField::Fortitude,
            RuntimeNumberField::Reflex,
            RuntimeNumberField::Will,
        ] {
            let stat = value(&projection, target);
            assert!(stat.modifiers.iter().any(|modifier| {
                provenance_source(&modifier.provenance) == "Frightened 2"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -2
            }));
            assert!(stat.suppressed_modifiers.iter().any(|modifier| {
                provenance_source(&modifier.provenance) == "Fatigued"
                    && modifier.modifier_type == StatModifierTypeView::Status
                    && modifier.value == -1
            }));
        }
        let ac = value(&projection, RuntimeNumberField::ArmorClass);
        assert!(ac.modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Off-Guard"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        let perception = value(&projection, RuntimeNumberField::Perception);
        assert!(
            perception
                .modifiers
                .iter()
                .chain(&perception.suppressed_modifiers)
                .all(|modifier| provenance_source(&modifier.provenance) != "Fatigued")
        );
    }

    #[test]
    fn fatigued_requires_the_canonical_key_and_preserves_note_order() {
        let mut first = condition("Fatigued", None);
        first.condition_id = 1;
        first.name = "First fatigue".to_string();
        let mut second = condition("Fatigued", None);
        second.condition_id = 2;
        second.name = "Second fatigue".to_string();
        let annotation = unmodeled_condition("Fatigued", None);

        let ordered = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![first.clone(), annotation.clone(), second.clone()],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            ordered
                .automation_limitations
                .iter()
                .filter(|limitation| {
                    limitation.code
                        == EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated
                })
                .map(|limitation| limitation.target.clone())
                .collect::<Vec<_>>(),
            vec![
                EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 1 },
                EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 2 },
            ]
        );
        assert_eq!(
            value(&ordered, RuntimeNumberField::ArmorClass).adjusted_value,
            21
        );

        let reversed = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![second.clone(), first.clone()],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            reversed
                .automation_limitations
                .iter()
                .filter(|limitation| {
                    limitation.code
                        == EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated
                })
                .map(|limitation| limitation.target.clone())
                .collect::<Vec<_>>(),
            vec![
                EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 2 },
                EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 1 },
            ]
        );

        first.condition_key = None;
        let key_mutation = project_fixture(
            &participant(ParticipantVariant::Normal, vec![first, second]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            key_mutation
                .automation_limitations
                .iter()
                .filter(|limitation| {
                    limitation.code
                        == EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated
                })
                .map(|limitation| limitation.target.clone())
                .collect::<Vec<_>>(),
            vec![EncounterRuntimeAutomationLimitationTargetView::Condition {
                condition_id: 2,
            }]
        );

        let annotation_only = project_fixture(
            &participant(ParticipantVariant::Normal, vec![annotation]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            value(&annotation_only, RuntimeNumberField::ArmorClass).adjusted_value,
            22
        );
        assert!(annotation_only.automation_limitations.is_empty());
    }

    #[test]
    fn runtime_only_fatigued_keeps_context_without_changing_actions() {
        let projection = manual_encounter_runtime(&participant(
            ParticipantVariant::Normal,
            vec![condition("Fatigued", None)],
        ));
        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert_eq!(projection.automation_limitations.len(), 1);
        assert_eq!(
            projection.automation_limitations[0].code,
            EncounterRuntimeAutomationLimitationCodeView::ExplorationActivityRestrictionNotAutomated
        );
        assert_eq!(
            projection.automation_limitations[0].target,
            EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 1 }
        );
    }

    #[test]
    fn canonical_activity_targets_keep_roll_damage_and_internal_diagnostics() {
        let projection = project_canonical_projection_with_diagnostics(
            &participant(
                ParticipantVariant::Elite,
                vec![condition("Enfeebled", Some(2))],
            ),
            canonical_projection(),
        );
        let runtime = &projection.runtime;

        assert_roll(runtime, "strike-claw", "attack", 12, 12, "Elite adjustment");
        assert!(
            roll(runtime, "strike-claw", "attack")
                .modifiers
                .iter()
                .any(|modifier| modifier.label == "Enfeebled 2")
        );
        let strike_damage = damage(runtime, "strike-claw", "main");
        assert_eq!(strike_damage.adjusted_formula, None);
        assert_eq!(strike_damage.modifiers.len(), 2);
        assert_no_damage_modifier(runtime, "action-breath", "fire");
        assert!(projection.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == EncounterProjectionDiagnosticCode::UnsupportedCanonicalFact
                && diagnostic
                    .message
                    .contains("activities.strike-claw.unsupported")
                && diagnostic.message.contains("unmodeled-rule")
        }));
    }

    #[test]
    fn canonical_activity_facts_have_exact_public_or_internal_dispositions() {
        let projection = project_canonical_projection_with_diagnostics(
            &participant(ParticipantVariant::Normal, Vec::new()),
            canonical_projection(),
        );
        let runtime = &projection.runtime;
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let action = runtime
            .activities
            .iter()
            .find(|activity| activity.activity_id == action_id.as_str())
            .expect("action should project");
        assert_eq!(action.usage, EncounterRuntimeActivityUsageView::Limited);
        assert_eq!(
            action.action_cost.as_ref().map(|cost| &cost.value),
            Some(&EncounterRuntimeActionCostKindView::Actions { count: 2 })
        );
        assert_eq!(
            action
                .uses
                .as_ref()
                .map(|uses| (uses.maximum, uses.serialized_value)),
            Some((Some(2), Some(1)))
        );
        assert_eq!(
            action.frequency.as_ref().map(|frequency| (
                frequency.maximum,
                frequency.period.as_deref(),
                frequency.serialized_value
            )),
            Some((Some(1), Some("day"), None))
        );
        assert!(
            action.rolls.iter().all(|roll| roll.roll_id != "recall"),
            "a check must not be mislabeled as the existing attack or DC surface"
        );

        assert!(projection.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == EncounterProjectionDiagnosticCode::UnavailableCanonicalFact
                && diagnostic.message == "Recall Knowledge: check roll has no encounter roll surface; id=\"recall\"; label=\"Recall Knowledge\"; value=value(18); ability=value(intelligence)"
        }));
        assert!(runtime.automation_limitations.iter().any(|limitation| {
            limitation.code
                == EncounterRuntimeAutomationLimitationCodeView::ActivityCheckNotAutomated
                && limitation.target
                    == EncounterRuntimeAutomationLimitationTargetView::Activity {
                        activity_id: action_id.as_str().to_string(),
                    }
        }));

        let slot_values = runtime
            .spellcasting
            .iter()
            .flat_map(|entry| &entry.slots)
            .collect::<Vec<_>>();
        assert_eq!(
            slot_values
                .iter()
                .map(|slot| (
                    slot.maximum.label.as_str(),
                    slot.maximum.base_value,
                    slot.maximum.adjusted_value
                ))
                .collect::<Vec<_>>(),
            vec![("Rank 3 slots", 2, 2), ("Rank 1 slots", 4, 4)],
            "representable spell-slot maxima should retain canonical fact order"
        );
        let missing_slot = MechanicTarget::SpellSlotMaximum {
            entry_occurrence_id: spellcasting_id,
            rank: 4,
        };
        assert!(projection.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == EncounterProjectionDiagnosticCode::UnavailableCanonicalFact
                && diagnostic.canonical_target == canonical_target_view(&missing_slot)
                && diagnostic.message
                    == "Rank 4 slots: spell-slot maximum cannot populate a numeric encounter value; value=missing"
        }));
    }

    #[test]
    fn canonical_activity_metadata_duplicates_fail_closed_without_losing_facts() {
        let mut canonical = canonical_projection();
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let action = canonical
            .activities
            .iter_mut()
            .find(|activity| activity.occurrence_id == action_id)
            .expect("action should exist");
        let mut duplicate_uses = action
            .facts
            .iter()
            .filter(|fact| matches!(fact.target, MechanicTarget::ActivityUses { .. }))
            .next()
            .expect("uses fact should exist")
            .clone();
        duplicate_uses.label = "Secondary uses".to_string();
        duplicate_uses.value = MechanicBaseValue::Uses(FactValue::Value(CreatureUseLimit {
            maximum: FactValue::Value(3),
            serialized_value: FactValue::Value(2),
        }));
        action.facts.insert(2, duplicate_uses);

        let projection = project_canonical_projection_with_diagnostics(
            &participant(ParticipantVariant::Normal, Vec::new()),
            canonical,
        );
        let action = projection
            .runtime
            .activities
            .iter()
            .find(|activity| activity.activity_id == action_id.as_str())
            .expect("action should project");
        assert!(action.uses.is_none(), "duplicate uses must fail closed");
        let reasons = projection
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == EncounterProjectionDiagnosticCode::DuplicateRuntimeFact
            })
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            reasons,
            vec![
                "Uses: 2 canonical facts target the same zero-or-one encounter activity field; all were rejected",
                "Secondary uses: 2 canonical facts target the same zero-or-one encounter activity field; all were rejected",
            ]
        );
    }

    #[test]
    fn unique_named_and_spellcasting_runtime_facts_remain_available() {
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let targets = vec![
            MechanicTarget::MaxHp,
            MechanicTarget::ArmorClass,
            MechanicTarget::Perception,
            MechanicTarget::Save {
                save: SaveKind::Fortitude,
            },
            MechanicTarget::Save {
                save: SaveKind::Reflex,
            },
            MechanicTarget::Save {
                save: SaveKind::Will,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Strength,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Dexterity,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Constitution,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Intelligence,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Wisdom,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Charisma,
            },
            MechanicTarget::SpellcastingAttack {
                entry_occurrence_id: spellcasting_id.clone(),
            },
            MechanicTarget::SpellcastingDc {
                entry_occurrence_id: spellcasting_id,
            },
        ];
        let values = targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                named_runtime_mechanic(target.clone(), &format!("Unique {index}"), index as i64)
            })
            .collect();

        let projection = project_mechanic_values_with_diagnostics(values);
        for (index, target) in targets.iter().enumerate() {
            let value = named_runtime_value(&projection.runtime, target)
                .map(|value| value.base_value)
                .or_else(|| {
                    spellcasting_runtime_roll(&projection.runtime, target)
                        .map(|value| value.base_value)
                });
            assert_eq!(value, Some(index as i64), "{target:?} should project");
            assert!(duplicate_diagnostics_for(&projection, target).is_empty());
        }
    }

    #[test]
    fn every_duplicate_named_stat_fails_closed_independent_of_order() {
        let targets = [
            MechanicTarget::MaxHp,
            MechanicTarget::ArmorClass,
            MechanicTarget::Perception,
            MechanicTarget::Save {
                save: SaveKind::Fortitude,
            },
            MechanicTarget::Save {
                save: SaveKind::Reflex,
            },
            MechanicTarget::Save {
                save: SaveKind::Will,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Strength,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Dexterity,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Constitution,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Intelligence,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Wisdom,
            },
            MechanicTarget::AbilityModifier {
                ability: AbilityKind::Charisma,
            },
        ];

        for target in targets {
            let first = named_runtime_mechanic(target.clone(), "First candidate", 10);
            let second = named_runtime_mechanic(target.clone(), "Second candidate", 99);
            for values in [
                vec![first.clone(), second.clone()],
                vec![second.clone(), first.clone()],
            ] {
                let projection = project_mechanic_values_with_diagnostics(values);
                assert!(
                    named_runtime_value(&projection.runtime, &target).is_none(),
                    "{target:?} must expose neither ambiguous value"
                );
                let duplicate_diagnostics = duplicate_diagnostics_for(&projection, &target);
                assert_eq!(duplicate_diagnostics.len(), 2);
                assert!(duplicate_diagnostics.iter().all(|diagnostic| diagnostic
                    .message
                    .ends_with("2 canonical facts target the same zero-or-one named runtime field; all were rejected")));
                assert!(projection.runtime.automation_limitations.is_empty());
                let public_json = serde_json::to_string(&projection.runtime)
                    .expect("public runtime should serialize");
                assert!(!public_json.contains("duplicate_runtime_fact"));
                assert!(!public_json.contains("DuplicateRuntimeFact"));
                assert!(!public_json.contains("First candidate"));
                assert!(!public_json.contains("Second candidate"));
            }
        }
    }

    #[test]
    fn duplicate_spellcasting_attack_and_dc_fail_closed_independent_of_order() {
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        for target in [
            MechanicTarget::SpellcastingAttack {
                entry_occurrence_id: spellcasting_id.clone(),
            },
            MechanicTarget::SpellcastingDc {
                entry_occurrence_id: spellcasting_id.clone(),
            },
        ] {
            let first = named_runtime_mechanic(target.clone(), "First spell candidate", 14);
            let second = named_runtime_mechanic(target.clone(), "Second spell candidate", 31);
            let sibling_target = match &target {
                MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id,
                } => MechanicTarget::SpellcastingDc {
                    entry_occurrence_id: entry_occurrence_id.clone(),
                },
                MechanicTarget::SpellcastingDc {
                    entry_occurrence_id,
                } => MechanicTarget::SpellcastingAttack {
                    entry_occurrence_id: entry_occurrence_id.clone(),
                },
                _ => unreachable!("loop contains only spellcasting roll targets"),
            };
            let sibling = named_runtime_mechanic(sibling_target.clone(), "Unique sibling", 20);
            let mut expected_public_spellcasting = None;
            for values in [
                vec![first.clone(), sibling.clone(), second.clone()],
                vec![second.clone(), sibling.clone(), first.clone()],
                vec![sibling.clone(), first.clone(), second.clone()],
            ] {
                let projection = project_mechanic_values_with_diagnostics(values);
                assert!(
                    spellcasting_runtime_roll(&projection.runtime, &target).is_none(),
                    "{target:?} must expose neither ambiguous value"
                );
                assert_eq!(
                    spellcasting_runtime_roll(&projection.runtime, &sibling_target)
                        .map(|roll| roll.base_value),
                    Some(20),
                    "the unique sibling roll should remain available"
                );
                assert_eq!(duplicate_diagnostics_for(&projection, &target).len(), 2);
                assert!(projection.runtime.automation_limitations.is_empty());
                let public_json = serde_json::to_string(&projection.runtime)
                    .expect("public runtime should serialize");
                assert!(!public_json.contains("duplicate_runtime_fact"));
                assert!(!public_json.contains("DuplicateRuntimeFact"));
                assert!(!public_json.contains("First spell candidate"));
                assert!(!public_json.contains("Second spell candidate"));
                let public_spellcasting = serde_json::to_value(&projection.runtime.spellcasting)
                    .expect("spellcasting should serialize");
                if let Some(expected) = &expected_public_spellcasting {
                    assert_eq!(
                        &public_spellcasting, expected,
                        "duplicate ordering must not alter retained spellcasting fields"
                    );
                } else {
                    expected_public_spellcasting = Some(public_spellcasting);
                }
            }
        }
    }

    #[test]
    fn duplicate_target_mutations_restore_only_unambiguous_named_values() {
        let duplicate_ac = vec![
            named_runtime_mechanic(MechanicTarget::ArmorClass, "First AC", 22),
            named_runtime_mechanic(MechanicTarget::ArmorClass, "Second AC", 40),
        ];
        let rejected = project_mechanic_values_with_diagnostics(duplicate_ac.clone());
        assert!(named_runtime_value(&rejected.runtime, &MechanicTarget::ArmorClass).is_none());

        let mut separated_stats = duplicate_ac;
        separated_stats[1] = named_runtime_mechanic(MechanicTarget::MaxHp, "Maximum HP", 40);
        let restored = project_mechanic_values_with_diagnostics(separated_stats);
        assert_eq!(
            named_runtime_value(&restored.runtime, &MechanicTarget::ArmorClass)
                .map(|value| value.base_value),
            Some(22)
        );
        assert_eq!(
            named_runtime_value(&restored.runtime, &MechanicTarget::MaxHp)
                .map(|value| value.base_value),
            Some(40)
        );

        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");
        let attack_target = MechanicTarget::SpellcastingAttack {
            entry_occurrence_id: spellcasting_id.clone(),
        };
        let dc_target = MechanicTarget::SpellcastingDc {
            entry_occurrence_id: spellcasting_id,
        };
        let duplicate_attack = vec![
            named_runtime_mechanic(attack_target.clone(), "First attack", 14),
            named_runtime_mechanic(attack_target.clone(), "Second attack", 30),
        ];
        let rejected = project_mechanic_values_with_diagnostics(duplicate_attack.clone());
        assert!(spellcasting_runtime_roll(&rejected.runtime, &attack_target).is_none());

        let mut separated_spellcasting = duplicate_attack;
        separated_spellcasting[1] = named_runtime_mechanic(dc_target.clone(), "Spell DC", 30);
        let restored = project_mechanic_values_with_diagnostics(separated_spellcasting);
        assert_eq!(
            spellcasting_runtime_roll(&restored.runtime, &attack_target)
                .map(|value| value.base_value),
            Some(14)
        );
        assert_eq!(
            spellcasting_runtime_roll(&restored.runtime, &dc_target).map(|value| value.base_value),
            Some(30)
        );
    }

    #[test]
    fn canonical_activity_fact_mutations_change_their_exact_dispositions() {
        let normal = participant(ParticipantVariant::Normal, Vec::new());
        let action_id = atlas_record::CreatureOccurrenceId::new("action-breath")
            .expect("occurrence id should be valid");
        let spellcasting_id = atlas_record::CreatureOccurrenceId::new("spellcasting-arcane")
            .expect("occurrence id should be valid");

        let mut check = canonical_projection();
        activity_fact_mut(
            &mut check,
            &MechanicTarget::ActivityRoll {
                occurrence_id: action_id.clone(),
                roll_id: "recall".to_string(),
            },
        )
        .value = MechanicBaseValue::Roll(CreatureRoll {
            id: "recall".to_string(),
            label: "Recall Knowledge".to_string(),
            kind: CreatureRollKind::Check,
            value: FactValue::Null,
            ability: FactValue::Value(ActivityRollAbility::Wisdom),
        });
        assert!(
            project_canonical_projection_with_diagnostics(&normal, check)
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic
                    .message
                    .contains("value=null; ability=value(wisdom)"))
        );

        let mut action_cost_projection = canonical_projection();
        activity_fact_mut(
            &mut action_cost_projection,
            &MechanicTarget::ActivityActionCost {
                occurrence_id: action_id.clone(),
            },
        )
        .value = MechanicBaseValue::ActionCost(CreatureActionCost::Reaction);
        let projection = project_canonical_projection(&normal, action_cost_projection);
        let action = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == action_id.as_str())
            .expect("action should project");
        assert_eq!(
            action.action_cost.as_ref().map(|cost| &cost.value),
            Some(&EncounterRuntimeActionCostKindView::Reaction)
        );

        let mut uses_projection = canonical_projection();
        activity_fact_mut(
            &mut uses_projection,
            &MechanicTarget::ActivityUses {
                occurrence_id: action_id.clone(),
            },
        )
        .value = MechanicBaseValue::Uses(FactValue::Null);
        assert!(
            project_canonical_projection_with_diagnostics(&normal, uses_projection)
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.ends_with("value=null"))
        );

        let mut frequency_projection = canonical_projection();
        activity_fact_mut(
            &mut frequency_projection,
            &MechanicTarget::ActivityFrequency {
                occurrence_id: action_id.clone(),
            },
        )
        .value = MechanicBaseValue::Frequency(FactValue::Value(CreatureFrequency {
            maximum: FactValue::Value(2),
            period: FactValue::Value("round".to_string()),
            serialized_value: FactValue::Value(1),
        }));
        let projection = project_canonical_projection(&normal, frequency_projection);
        let frequency = projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == action_id.as_str())
            .and_then(|activity| activity.frequency.as_ref())
            .expect("frequency should remain typed");
        assert_eq!(
            (
                frequency.maximum,
                frequency.period.as_deref(),
                frequency.serialized_value,
            ),
            (Some(2), Some("round"), Some(1))
        );

        let mut slot_projection = canonical_projection();
        activity_fact_mut(
            &mut slot_projection,
            &MechanicTarget::SpellSlotMaximum {
                entry_occurrence_id: spellcasting_id,
                rank: 3,
            },
        )
        .value = MechanicBaseValue::SourceInteger(FactValue::Value(CreatureSourceScalar::Value(5)));
        let projection = project_canonical_projection(&normal, slot_projection);
        let slot = projection
            .spellcasting
            .iter()
            .flat_map(|entry| &entry.slots)
            .find(|slot| slot.rank == 3)
            .expect("mutated spell-slot maximum should remain represented");
        assert_eq!(
            (slot.maximum.base_value, slot.maximum.adjusted_value),
            (5, 5)
        );
    }

    #[test]
    fn canonical_ability_and_spell_targets_receive_deterministic_conditions() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Clumsy", Some(1)),
                condition("Enfeebled", Some(2)),
                condition("Stupefied", Some(1)),
            ],
        ));

        assert_eq!(
            value(&projection, RuntimeNumberField::Reflex).adjusted_value,
            11
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Perception).adjusted_value,
            12
        );
        let athletics = value(&projection, RuntimeNumberField::Athletics);
        assert_eq!(athletics.adjusted_value, 7);
        assert_roll(&projection, "strike-claw", "attack", 12, 10, "Enfeebled 2");
        let spellcasting = projection
            .spellcasting
            .iter()
            .find(|entry| entry.entry_id == "spellcasting-arcane")
            .expect("spellcasting entry should project");
        let spell_dc = spellcasting.dc.as_ref().expect("spell DC should project");
        assert_eq!(spell_dc.base_value, 22);
        assert_eq!(spell_dc.adjusted_value, 21);
        assert!(
            spell_dc
                .modifiers
                .iter()
                .any(|modifier| modifier.label == "Stupefied 1")
        );
        assert_damage_modifier(&projection, "strike-claw", "main", -2);
    }

    #[test]
    fn canonical_variant_damage_adjusts_only_first_melee_or_spell_damage() {
        let mut strike_projection = canonical_projection();
        let strike = canonical_activity_mut(&mut strike_projection, "strike-claw");
        let main = strike
            .facts
            .iter()
            .filter(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .next()
            .expect("strike damage should exist")
            .clone();
        let mut unsupported_first = main.clone();
        unsupported_first.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "unsupported-first".to_string(),
        };
        unsupported_first.label = "unsupported-first".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut unsupported_first.value {
            damage.id = "unsupported-first".to_string();
            damage.formula = FactValue::Value("contextual damage".to_string());
            damage.kinds = FactValue::Value(vec![CreatureDamageKind::Unsupported(
                UnsupportedSourceValue {
                    shape: UnsupportedSourceShape::String,
                    value: "\"contextual\"".to_string(),
                    reason: UnsupportedSourceReason::OpenVocabulary,
                },
            )]);
        }
        let main_index = strike
            .facts
            .iter()
            .position(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .expect("main damage should exist");
        strike.facts.insert(main_index, unsupported_first);
        let mut secondary = main;
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: strike.occurrence_id.clone(),
            damage_id: "secondary".to_string(),
        };
        secondary.label = "secondary".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "secondary".to_string();
            damage.formula = FactValue::Value("1d4".to_string());
        }
        strike.facts.push(secondary);
        let elite_strike = project_canonical_projection(
            &participant(ParticipantVariant::Elite, Vec::new()),
            strike_projection,
        );
        assert_no_damage_modifier(&elite_strike, "strike-claw", "unsupported-first");
        assert_eq!(
            damage(&elite_strike, "strike-claw", "unsupported-first").adjusted_formula,
            None
        );
        assert_damage_modifier(&elite_strike, "strike-claw", "main", 2);
        assert_no_damage_modifier(&elite_strike, "strike-claw", "secondary");
        assert!(
            elite_strike
                .automation_limitations
                .iter()
                .all(|limitation| limitation.target
                    != EncounterRuntimeAutomationLimitationTargetView::Participant)
        );

        let mut limited_spell_projection = canonical_projection();
        let limited = canonical_activity_mut(&mut limited_spell_projection, "action-breath");
        limited.family = MechanicActivityFamily::Spell;
        let mut secondary = limited
            .facts
            .iter()
            .filter(|fact| matches!(fact.target, MechanicTarget::ActivityDamage { .. }))
            .next()
            .expect("spell damage should exist")
            .clone();
        secondary.target = MechanicTarget::ActivityDamage {
            occurrence_id: limited.occurrence_id.clone(),
            damage_id: "cold".to_string(),
        };
        secondary.label = "cold".to_string();
        if let MechanicBaseValue::Damage(damage) = &mut secondary.value {
            damage.id = "cold".to_string();
            damage.formula = FactValue::Value("2d6".to_string());
            damage.damage_type = FactValue::Value("cold".to_string());
        }
        limited.facts.push(secondary);
        let elite_limited = project_canonical_projection(
            &participant(ParticipantVariant::Elite, Vec::new()),
            limited_spell_projection.clone(),
        );
        assert_damage_modifier(&elite_limited, "action-breath", "fire", 4);
        assert_no_damage_modifier(&elite_limited, "action-breath", "cold");

        let at_will = canonical_activity_mut(&mut limited_spell_projection, "action-breath");
        at_will.facts.retain(|fact| {
            !matches!(
                fact.target,
                MechanicTarget::ActivityUses { .. } | MechanicTarget::ActivityFrequency { .. }
            )
        });
        let fire_index = at_will
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "fire"
                )
            })
            .expect("fire damage should exist");
        let cold_index = at_will
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "cold"
                )
            })
            .expect("cold damage should exist");
        at_will.facts.swap(fire_index, cold_index);
        let weak_at_will = project_canonical_projection(
            &participant(ParticipantVariant::Weak, Vec::new()),
            limited_spell_projection,
        );
        assert_damage_modifier(&weak_at_will, "action-breath", "cold", -2);
        assert_no_damage_modifier(&weak_at_will, "action-breath", "fire");
    }

    #[test]
    fn unsupported_first_damage_formula_retains_exact_ordered_disposition() {
        for (formula, expected_formula) in
            [(FactValue::Missing, "missing"), (FactValue::Null, "null")]
        {
            let projection = canonical_strike_with_unsupported_first_formula(formula);
            let projected = project_canonical_projection_with_diagnostics(
                &participant(ParticipantVariant::Elite, Vec::new()),
                projection,
            );
            let strike = projected
                .runtime
                .activities
                .iter()
                .find(|activity| activity.activity_id == "strike-claw")
                .expect("strike should project");
            assert_eq!(
                strike
                    .damage
                    .iter()
                    .map(|damage| damage.damage_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["main", "secondary"]
            );
            assert_no_damage_modifier(&projected.runtime, "strike-claw", "main");
            assert_no_damage_modifier(&projected.runtime, "strike-claw", "secondary");
            assert_unsupported_damage_formula_disposition(&projected, expected_formula);
        }

        let mut reordered = canonical_strike_with_unsupported_first_formula(FactValue::Missing);
        let strike = canonical_activity_mut(&mut reordered, "strike-claw");
        let unsupported_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. }
                        if damage_id == "unsupported-first"
                )
            })
            .expect("unsupported damage should exist");
        let main_index = strike
            .facts
            .iter()
            .position(|fact| {
                matches!(
                    &fact.target,
                    MechanicTarget::ActivityDamage { damage_id, .. } if damage_id == "main"
                )
            })
            .expect("main damage should exist");
        strike.facts.swap(unsupported_index, main_index);

        let projected = project_canonical_projection_with_diagnostics(
            &participant(ParticipantVariant::Elite, Vec::new()),
            reordered,
        );
        assert_damage_modifier(&projected.runtime, "strike-claw", "main", 2);
        assert_no_damage_modifier(&projected.runtime, "strike-claw", "secondary");
        assert_unsupported_damage_formula_disposition(&projected, "missing");
    }

    #[test]
    fn canonical_prone_applies_off_guard_and_typed_attack_penalty_only() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Prone", None),
                condition("Prone", None),
                condition("Frightened", Some(1)),
            ],
        ));
        assert_eq!(
            value(&projection, RuntimeNumberField::ArmorClass).adjusted_value,
            19
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::ArmorClass)
                .modifiers
                .len(),
            2
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::ArmorClass)
                .suppressed_modifiers
                .len(),
            1
        );
        let attack = roll(&projection, "strike-claw", "attack");
        assert_eq!(attack.adjusted_value, 9);
        assert!(attack.modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        assert!(attack.modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Frightened 1"
                && modifier.modifier_type == StatModifierTypeView::Status
                && modifier.value == -1
        }));
        assert!(attack.suppressed_modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
        }));
        let spell_dc = projection
            .spellcasting
            .iter()
            .find(|entry| entry.entry_id == "spellcasting-arcane")
            .and_then(|entry| entry.dc.as_ref())
            .expect("spell DC should project");
        assert_eq!(spell_dc.adjusted_value, 21);
        assert!(
            spell_dc
                .modifiers
                .iter()
                .all(|modifier| provenance_source(&modifier.provenance) != "Prone")
        );
        let spell_attack = projection
            .spellcasting
            .iter()
            .find(|entry| entry.entry_id == "spellcasting-arcane")
            .and_then(|entry| entry.attack.as_ref())
            .expect("spell attack should project");
        assert_eq!(spell_attack.surface, RuntimeRollSurfaceView::AttackRoll);
        assert_eq!(spell_attack.base_value, 14);
        assert_eq!(spell_attack.adjusted_value, 11);
        assert!(spell_attack.modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
                && modifier.value == -2
        }));
        assert!(spell_attack.modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Frightened 1"
                && modifier.modifier_type == StatModifierTypeView::Status
                && modifier.value == -1
        }));
        assert!(spell_attack.suppressed_modifiers.iter().any(|modifier| {
            provenance_source(&modifier.provenance) == "Prone"
                && modifier.modifier_type == StatModifierTypeView::Circumstance
        }));
        assert!(projection.automation_limitations.iter().any(|limitation| {
            limitation.code
                == EncounterRuntimeAutomationLimitationCodeView::ActivityCheckNotAutomated
                && limitation.target
                    == EncounterRuntimeAutomationLimitationTargetView::Activity {
                        activity_id: "action-breath".to_string(),
                    }
        }));
        let action_notes = &projection
            .action_budget
            .as_ref()
            .expect("action budget")
            .notes;
        assert!(action_notes.iter().any(|note| {
            provenance_source(&note.provenance) == "Prone"
                && note.reason.contains("cover")
                && note.reason.contains("falling")
        }));

        let mut dc_mutation = canonical_projection();
        let strike_id = atlas_record::CreatureOccurrenceId::new("strike-claw")
            .expect("occurrence id should be valid");
        let roll_fact = activity_fact_mut(
            &mut dc_mutation,
            &MechanicTarget::ActivityRoll {
                occurrence_id: strike_id,
                roll_id: "attack".to_string(),
            },
        );
        if let MechanicBaseValue::Roll(roll) = &mut roll_fact.value {
            roll.kind = CreatureRollKind::DifficultyClass;
        }
        let dc_projection = project_canonical_projection(
            &participant(ParticipantVariant::Normal, vec![condition("Prone", None)]),
            dc_mutation,
        );
        let mutated = roll(&dc_projection, "strike-claw", "attack");
        assert_eq!(mutated.adjusted_value, 12);
        assert!(mutated.modifiers.is_empty());
    }

    #[test]
    fn canonical_runtime_rules_explain_action_and_movement_suppression() {
        let projection = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Quickened", None),
                condition("Quickened", None),
                condition("Slowed", Some(1)),
                condition("Slowed", Some(2)),
                condition("Grabbed", None),
                condition("Encumbered", None),
            ],
        ));

        let budget = projection.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 2);
        assert_eq!(budget.actions.adjustments.len(), 2);
        assert_eq!(budget.actions.suppressed_adjustments.len(), 2);
        let land = speed(&projection, "land");
        assert_eq!(land.adjusted_value_feet, 15);
        assert_eq!(land.adjustments.len(), 1);
        assert!(land.suppressed_adjustments.is_empty());
        assert!(budget.notes.iter().any(|note| {
            provenance_source(&note.provenance) == "Grabbed"
                && note.label == "Move actions forbidden"
        }));
    }

    #[test]
    fn participant_projection_requires_canonical_creature_body() {
        let retrieved = RetrievedRecord {
            record: AtlasRecord::new(
                RecordIdentity::new(RecordKey::parse("actors:test").expect("key"), "Creature"),
                RecordClassification::new(RecordKind::Creature),
                FoundryRecordInfo::new(
                    "Actors",
                    FoundryDocumentType::Actor,
                    FoundryRecordType::Npc,
                ),
                RecordProvenance::new("test.json"),
            ),
            body: None,
            spell_children: Vec::new(),
            consumable_occurrences: Default::default(),
        };

        assert!(
            participant_encounter_runtime(
                &participant(ParticipantVariant::Elite, Vec::new()),
                &retrieved,
            )
            .is_none()
        );
    }

    #[test]
    fn elite_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Elite, Vec::new());
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");
        assert_eq!(
            projection.level.as_ref().map(|value| value.adjusted_value),
            Some(6)
        );
        assert_stat(
            &projection,
            RuntimeNumberField::ArmorClass,
            22,
            24,
            "Elite adjustment",
        );
        assert_stat(
            &projection,
            RuntimeNumberField::MaximumHp,
            60,
            80,
            "Elite HP adjustment",
        );
    }

    #[test]
    fn weak_adjusts_projected_creature_stats_and_hp_by_level_band() {
        let participant = participant(ParticipantVariant::Weak, Vec::new());
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");
        assert_eq!(
            projection.level.as_ref().map(|value| value.adjusted_value),
            Some(4)
        );
        assert_stat(
            &projection,
            RuntimeNumberField::Perception,
            13,
            11,
            "Weak adjustment",
        );
        assert_stat(
            &projection,
            RuntimeNumberField::MaximumHp,
            60,
            45,
            "Weak HP adjustment",
        );
    }

    #[test]
    fn condition_penalties_stack_by_modifier_type() {
        let conditions = vec![
            condition("Frightened", Some(1)),
            condition("Sickened", Some(2)),
            condition("Off-Guard", None),
        ];
        let participant = participant(ParticipantVariant::Normal, conditions);
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");
        let ac = value(&projection, RuntimeNumberField::ArmorClass);
        assert_eq!(ac.adjusted_value, 18);
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Sickened 2")
        );
        assert!(
            ac.modifiers
                .iter()
                .any(|modifier| modifier.label == "Off-Guard")
        );
        assert!(
            ac.suppressed_modifiers
                .iter()
                .any(|modifier| modifier.label == "Frightened 1")
        );
    }

    #[test]
    fn targeted_conditions_project_supported_stats_and_typed_automation_limits() {
        let conditions = vec![
            condition("Clumsy", None),
            condition("Enfeebled", Some(2)),
            condition("Stupefied", Some(1)),
        ];
        let participant = participant(ParticipantVariant::Normal, conditions);
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");
        assert_eq!(
            value(&projection, RuntimeNumberField::Reflex).adjusted_value,
            11
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Athletics).adjusted_value,
            7
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Will).adjusted_value,
            11
        );
        assert_eq!(projection.automation_limitations.len(), 3);
        assert_eq!(
            projection
                .automation_limitations
                .iter()
                .map(|limitation| limitation.code)
                .collect::<Vec<_>>(),
            vec![
                EncounterRuntimeAutomationLimitationCodeView::ConditionAttackAdjustmentPartial,
                EncounterRuntimeAutomationLimitationCodeView::ConditionDamageAdjustmentPartial,
                EncounterRuntimeAutomationLimitationCodeView::SpellDisruptionCheckNotAutomated,
            ]
        );
        assert!(projection.automation_limitations.iter().all(|limitation| {
            limitation.target
                == EncounterRuntimeAutomationLimitationTargetView::Condition { condition_id: 1 }
        }));
    }

    #[test]
    fn public_runtime_serialization_excludes_internal_projection_diagnostics() {
        let projection = project_canonical_projection_with_diagnostics(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Clumsy", Some(1))],
            ),
            canonical_projection(),
        );
        assert!(projection.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == EncounterProjectionDiagnosticCode::UnsupportedCanonicalFact
                && diagnostic
                    .message
                    .contains("activities.strike-claw.unsupported")
        }));
        assert!(projection.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == EncounterProjectionDiagnosticCode::UnavailableCanonicalFact
                && diagnostic.message.contains("value=missing")
        }));

        let json = serde_json::to_value(&projection.runtime)
            .expect("public runtime should serialize to JSON");
        let object = json.as_object().expect("runtime should be an object");
        assert!(!object.contains_key("unapplied_facts"));
        let limitations = object["automation_limitations"]
            .as_array()
            .expect("automation limitations should be an array");
        assert!(!limitations.is_empty());
        assert!(limitations.iter().all(|limitation| {
            limitation.as_object().is_some_and(|value| {
                value.keys().cloned().collect::<BTreeSet<_>>()
                    == BTreeSet::from([
                        "code".to_string(),
                        "message".to_string(),
                        "target".to_string(),
                    ])
            })
        }));
        let serialized = serde_json::to_string(&json).expect("runtime JSON should encode");
        for internal_detail in [
            "activities.strike-claw.unsupported",
            "unmodeled-rule",
            "value=missing",
            "source_path",
        ] {
            assert!(
                !serialized.contains(internal_detail),
                "internal projection detail leaked: {internal_detail}"
            );
        }
        assert!(
            projection
                .runtime
                .automation_limitations
                .iter()
                .all(|limitation| !matches!(
                    limitation.target,
                    EncounterRuntimeAutomationLimitationTargetView::Spellcasting { .. }
                ))
        );
    }

    #[test]
    fn frightened_penalizes_checks_and_dcs_but_not_raw_ability_modifiers() {
        let participant = participant(
            ParticipantVariant::Normal,
            vec![condition("Frightened", Some(1))],
        );
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");

        assert_eq!(
            value(&projection, RuntimeNumberField::ArmorClass).adjusted_value,
            21
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Perception).adjusted_value,
            12
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Will).adjusted_value,
            11
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Athletics).adjusted_value,
            8
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Strength).adjusted_value,
            4
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Dexterity).adjusted_value,
            3
        );
        assert!(
            value(&projection, RuntimeNumberField::Strength)
                .modifiers
                .is_empty()
        );
    }

    #[test]
    fn condition_names_without_modeled_keys_remain_annotation_only() {
        let participant = participant(
            ParticipantVariant::Normal,
            vec![unmodeled_condition("Frightened", Some(3))],
        );
        let projection = project_fixture(&participant, &mechanics_fixture()).expect("stat block");

        assert_eq!(
            value(&projection, RuntimeNumberField::ArmorClass).adjusted_value,
            22
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Perception).adjusted_value,
            13
        );
        assert_eq!(
            value(&projection, RuntimeNumberField::Athletics).adjusted_value,
            9
        );
    }

    #[test]
    fn elite_and_weak_project_structured_activity_damage_adjustments() {
        let elite = project_fixture(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_damage_modifier(&elite, "claw", "main", 2);
        assert_no_damage_modifier(&elite, "claw", "secondary");
        assert_eq!(
            damage(&elite, "claw", "main").adjusted_formula.as_deref(),
            Some("1d6 + 6")
        );
        assert_damage_modifier(&elite, "fireball", "0", 4);
        assert_no_damage_modifier(&elite, "fireball", "persistent");
        assert_damage_modifier(&elite, "ignition", "fire", 2);
        assert_no_damage_modifier(&elite, "ignition", "persistent");
        assert_no_damage_modifier(&elite, "breath", "0");
        assert_no_damage_modifier(&elite, "heal", "0");

        let weak = project_fixture(
            &participant(ParticipantVariant::Weak, Vec::new()),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_damage_modifier(&weak, "claw", "main", -2);
        assert_no_damage_modifier(&weak, "claw", "secondary");
        assert_damage_modifier(&weak, "fireball", "0", -4);
        assert_no_damage_modifier(&weak, "fireball", "persistent");
        assert_damage_modifier(&weak, "ignition", "fire", -2);
        assert_no_damage_modifier(&weak, "ignition", "persistent");
        assert_no_damage_modifier(&weak, "breath", "0");
        assert_no_damage_modifier(&weak, "heal", "0");
    }

    #[test]
    fn activity_roll_surfaces_receive_variant_and_condition_modifiers() {
        let elite = project_fixture(
            &participant(ParticipantVariant::Elite, Vec::new()),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_roll(&elite, "claw", "attack", 12, 14, "Elite adjustment");
        assert_roll(&elite, "fireball", "spell.dc", 22, 24, "Elite adjustment");

        let conditions = vec![
            condition("Frightened", Some(1)),
            condition("Enfeebled", Some(2)),
            condition("Stupefied", Some(2)),
        ];
        let projection = project_fixture(
            &participant(ParticipantVariant::Normal, conditions),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_roll(&projection, "claw", "attack", 12, 10, "Enfeebled 2");
        assert_roll(&projection, "fireball", "spell.dc", 22, 20, "Stupefied 2");
        assert!(
            roll(&projection, "fireball", "spell.dc")
                .suppressed_modifiers
                .iter()
                .any(|modifier| modifier.label == "Frightened 1")
        );
        assert_damage_modifier(&projection, "claw", "main", -2);
        assert_no_damage_modifier(&projection, "fireball", "0");
    }

    #[test]
    fn action_conditions_project_runtime_action_budget() {
        let slowed = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Slowed", Some(1))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let slowed_budget = slowed.action_budget.as_ref().expect("action budget");
        assert_eq!(slowed_budget.actions.adjusted_value, 2);
        assert!(slowed_budget.can_react.available);

        let quickened = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Quickened", None)],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let quickened_budget = quickened.action_budget.as_ref().expect("action budget");
        assert_eq!(quickened_budget.actions.adjusted_value, 4);
        assert!(
            quickened_budget
                .actions
                .segments
                .iter()
                .any(|segment| segment.restricted && segment.value == 1)
        );

        let stunned_one = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let stunned_one_budget = stunned_one.action_budget.as_ref().expect("action budget");
        assert_eq!(stunned_one_budget.actions.adjusted_value, 2);
        assert!(stunned_one_budget.can_react.available);

        let stunned_four = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(4))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let stunned_four_budget = stunned_four.action_budget.as_ref().expect("action budget");
        assert_eq!(stunned_four_budget.actions.adjusted_value, 0);
        assert!(!stunned_four_budget.can_act.available);
        assert!(!stunned_four_budget.can_react.available);

        let stunned_and_slowed = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1)), condition("Slowed", Some(2))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let budget = stunned_and_slowed
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 1);
        assert!(budget.actions.adjustments.iter().any(|adjustment| {
            provenance_source(&adjustment.provenance) == "Stunned 1" && adjustment.value == -1
        }));
        assert!(budget.actions.adjustments.iter().any(|adjustment| {
            provenance_source(&adjustment.provenance) == "Slowed 2"
                && adjustment.value == -1
                && adjustment.reason.as_deref()
                    == Some(
                        "1 action loss already counts toward slowed; 1 additional slowed action loss applies.",
                    )
        }));
        assert!(
            budget
                .actions
                .suppressed_adjustments
                .iter()
                .all(|adjustment| provenance_source(&adjustment.provenance) != "Slowed 2")
        );

        let stronger_stunned = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Slowed", Some(1)), condition("Stunned", Some(2))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let stronger_budget = stronger_stunned
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(stronger_budget.actions.adjusted_value, 1);
        assert!(
            stronger_budget
                .actions
                .adjustments
                .iter()
                .any(|adjustment| {
                    provenance_source(&adjustment.provenance) == "Stunned 2"
                        && adjustment.value == -2
                })
        );
        assert!(
            stronger_budget
                .actions
                .suppressed_adjustments
                .iter()
                .any(|adjustment| {
                    provenance_source(&adjustment.provenance) == "Slowed 1"
                        && adjustment.reason.as_deref()
                            == Some("All 1 slowed action loss is already counted by stunned.")
                })
        );

        let duplicate_conditions = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![
                    condition("Quickened", None),
                    condition("Quickened", None),
                    condition("Slowed", Some(1)),
                    condition("Slowed", Some(2)),
                ],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let duplicate_budget = duplicate_conditions
            .action_budget
            .as_ref()
            .expect("action budget");
        assert_eq!(duplicate_budget.actions.adjusted_value, 2);
        assert_eq!(duplicate_budget.actions.adjustments.len(), 2);
        assert_eq!(duplicate_budget.actions.suppressed_adjustments.len(), 2);
    }

    #[test]
    fn defeated_state_disables_capabilities_without_erasing_action_budget_facts() {
        let active_participant = participant(
            ParticipantVariant::Normal,
            vec![condition("Quickened", None), condition("Prone", None)],
        );
        let active = project_canonical(&active_participant);
        let active_budget = active.action_budget.as_ref().expect("action budget");
        assert_eq!(active_budget.actions.adjusted_value, 4);
        assert_eq!(active_budget.reactions.adjusted_value, 1);
        assert!(active_budget.can_act.available);
        assert!(active_budget.can_react.available);
        assert!(!active_budget.notes.is_empty());

        let mut defeated_participant = active_participant.clone();
        defeated_participant.defeated = true;
        let defeated = project_canonical(&defeated_participant);
        let defeated_budget = defeated.action_budget.as_ref().expect("action budget");
        assert_eq!(defeated_budget.actions, active_budget.actions);
        assert_eq!(defeated_budget.reactions, active_budget.reactions);
        assert_eq!(defeated_budget.notes, active_budget.notes);
        for capability in [&defeated_budget.can_act, &defeated_budget.can_react] {
            assert!(!capability.available);
            assert_eq!(capability.provenance, Some(participant_state_provenance()));
            assert_eq!(
                capability.reason.as_deref(),
                Some("Defeated participants cannot act or react.")
            );
        }

        defeated_participant.defeated = false;
        let reactivated = project_canonical(&defeated_participant);
        assert_eq!(reactivated.action_budget, active.action_budget);
    }

    #[test]
    fn duration_form_stunned_fails_closed_without_public_diagnostic_noise() {
        let duration_only = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", None, Some(2))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let budget = duration_only.action_budget.as_ref().expect("action budget");
        assert_eq!(budget.actions.adjusted_value, 3);
        assert!(budget.actions.adjustments.is_empty());
        assert!(budget.can_act.available);
        assert!(
            budget
                .notes
                .iter()
                .all(|note| note.label != "Stunned duration timing")
        );
        assert!(duration_only.automation_limitations.is_empty());

        let numeric = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Stunned", Some(1))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            numeric
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            2
        );
        assert!(numeric.automation_limitations.is_empty());

        let mut duration_mutation = condition_with_duration("Stunned", None, Some(2));
        duration_mutation.duration_rounds = Some(4);
        let mutated = project_fixture(
            &participant(ParticipantVariant::Normal, vec![duration_mutation]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            mutated
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            3
        );
        assert!(mutated.automation_limitations.is_empty());

        let invalid_numeric = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", Some(0), None)],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            invalid_numeric
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            3
        );
        assert!(invalid_numeric.automation_limitations.is_empty());

        let value_and_duration = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition_with_duration("Stunned", Some(2), Some(3))],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(
            value_and_duration
                .action_budget
                .as_ref()
                .expect("action budget")
                .actions
                .adjusted_value,
            1
        );
        let budget = value_and_duration
            .action_budget
            .as_ref()
            .expect("action budget");
        assert!(budget.notes.iter().any(|note| {
            note.label == "Stunned duration timing"
                && note.reason
                    == "The numeric action loss is applied, while duration timing and lifecycle remain contextual."
        }));
        assert!(value_and_duration
            .automation_limitations
            .iter()
            .any(|limitation| {
                limitation.code
                    == EncounterRuntimeAutomationLimitationCodeView::StunnedTimingRequiresAdjudication
                    && limitation.target
                        == EncounterRuntimeAutomationLimitationTargetView::Condition {
                            condition_id: 1,
                        }
            }));
    }

    #[test]
    fn restrained_overrides_grabbed_independent_of_condition_order() {
        for conditions in [
            vec![condition("Grabbed", None), condition("Restrained", None)],
            vec![condition("Restrained", None), condition("Grabbed", None)],
        ] {
            let projection = project_fixture(
                &participant(ParticipantVariant::Normal, conditions),
                &mechanics_fixture(),
            )
            .expect("stat block");
            let ac = value(&projection, RuntimeNumberField::ArmorClass);
            assert_eq!(ac.adjusted_value, 20);
            assert!(ac.modifiers.iter().any(|modifier| {
                provenance_source(&modifier.provenance) == "Restrained"
                    && modifier.modifier_type == StatModifierTypeView::Circumstance
                    && modifier.value == -2
            }));
            assert!(ac.suppressed_modifiers.iter().any(|modifier| {
                provenance_source(&modifier.provenance) == "Grabbed"
                    && modifier.modifier_type == StatModifierTypeView::Circumstance
                    && modifier.value == -2
            }));
            assert!(projection.automation_limitations.iter().any(|limitation| {
                limitation.code
                    == EncounterRuntimeAutomationLimitationCodeView::RestrictedActionExceptionsNotAutomated
                    && limitation.target
                        == EncounterRuntimeAutomationLimitationTargetView::Condition {
                            condition_id: 1,
                        }
            }));
            assert!(
                projection
                    .automation_limitations
                    .iter()
                    .all(|limitation| limitation.code
                        != EncounterRuntimeAutomationLimitationCodeView::ManipulateActionCheckNotAutomated)
            );
            let budget = projection.action_budget.as_ref().expect("action budget");
            assert!(budget.notes.iter().any(|note| {
                provenance_source(&note.provenance) == "Restrained"
                    && note.label == "Move actions forbidden"
            }));
            assert!(
                budget
                    .notes
                    .iter()
                    .all(|note| provenance_source(&note.provenance) != "Grabbed")
            );
            assert!(
                speed(&projection, "land")
                    .notes
                    .iter()
                    .all(|note| provenance_source(&note.provenance) != "Grabbed")
            );
        }

        let grabbed_only = project_fixture(
            &participant(ParticipantVariant::Normal, vec![condition("Grabbed", None)]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert!(
            value(&grabbed_only, RuntimeNumberField::ArmorClass)
                .modifiers
                .iter()
                .any(|modifier| provenance_source(&modifier.provenance) == "Grabbed")
        );
        assert!(
            grabbed_only
                .automation_limitations
                .iter()
                .any(|limitation| {
                    limitation.code
                == EncounterRuntimeAutomationLimitationCodeView::ManipulateActionCheckNotAutomated
                && limitation.target
                    == EncounterRuntimeAutomationLimitationTargetView::Condition {
                        condition_id: 1,
                    }
                })
        );
    }

    #[test]
    fn movement_conditions_project_speeds_and_chained_effects() {
        let encumbered = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Encumbered", None)],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(speed(&encumbered, "land").adjusted_value_feet, 15);
        assert_eq!(speed(&encumbered, "fly").adjusted_value_feet, 5);
        assert_eq!(
            value(&encumbered, RuntimeNumberField::Reflex).adjusted_value,
            11
        );

        let grabbed = project_fixture(
            &participant(ParticipantVariant::Normal, vec![condition("Grabbed", None)]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(speed(&grabbed, "land").adjusted_value_feet, 25);
        assert_eq!(
            value(&grabbed, RuntimeNumberField::ArmorClass).adjusted_value,
            20
        );
        assert!(
            grabbed
                .action_budget
                .as_ref()
                .expect("action budget")
                .notes
                .iter()
                .any(|note| provenance_source(&note.provenance) == "Grabbed"
                    && note.label == "Move actions forbidden")
        );

        let grabbed_and_encumbered = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Grabbed", None), condition("Encumbered", None)],
            ),
            &mechanics_fixture(),
        )
        .expect("stat block");
        let land = speed(&grabbed_and_encumbered, "land");
        assert_eq!(land.adjusted_value_feet, 15);
        assert_eq!(land.adjustments.len(), 1);
        assert!(land.suppressed_adjustments.is_empty());

        for condition_name in ["Immobilized", "Grabbed", "Restrained"] {
            let restricted = project_fixture(
                &participant(
                    ParticipantVariant::Normal,
                    vec![condition(condition_name, None)],
                ),
                &mechanics_fixture(),
            )
            .expect("stat block");
            assert_eq!(speed(&restricted, "land").adjusted_value_feet, 25);
            assert_eq!(speed(&restricted, "fly").adjusted_value_feet, 10);
            assert!(speed(&restricted, "land").notes.iter().any(|note| {
                provenance_source(&note.provenance) == condition_name
                    && note.label == "Move actions forbidden"
                    && note.reason.contains("numeric Speed is unchanged")
            }));
            assert!(
                restricted
                    .action_budget
                    .as_ref()
                    .expect("action budget")
                    .notes
                    .iter()
                    .any(|note| {
                        provenance_source(&note.provenance) == condition_name
                            && note.label == "Move actions forbidden"
                            && note.reason.contains("move trait")
                    })
            );
            match condition_name {
                "Grabbed" => assert!(restricted.automation_limitations.iter().any(|limitation| {
                    limitation.code
                        == EncounterRuntimeAutomationLimitationCodeView::ManipulateActionCheckNotAutomated
                })),
                "Restrained" => assert!(restricted.automation_limitations.iter().any(|limitation| {
                    limitation.code
                        == EncounterRuntimeAutomationLimitationCodeView::RestrictedActionExceptionsNotAutomated
                })),
                _ => {}
            }
        }
        let unrestricted = project_fixture(
            &participant(ParticipantVariant::Normal, Vec::new()),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert!(
            unrestricted
                .action_budget
                .as_ref()
                .expect("action budget")
                .notes
                .iter()
                .all(|note| note.label != "Move actions forbidden")
        );

        let prone = project_fixture(
            &participant(ParticipantVariant::Normal, vec![condition("Prone", None)]),
            &mechanics_fixture(),
        )
        .expect("stat block");
        assert_eq!(speed(&prone, "land").adjusted_value_feet, 25);
        assert_eq!(
            value(&prone, RuntimeNumberField::ArmorClass).adjusted_value,
            20
        );
        assert!(
            prone
                .automation_limitations
                .iter()
                .any(|limitation| limitation.code
                    == EncounterRuntimeAutomationLimitationCodeView::ProneContextRequiresAdjudication)
        );

        let zero_speed = project_fixture(
            &participant(
                ParticipantVariant::Normal,
                vec![condition("Encumbered", None)],
            ),
            &mechanics_fixture_with_speeds(&[("land", 0), ("fly", 10)]),
        )
        .expect("stat block");
        assert!(
            zero_speed
                .movement
                .as_ref()
                .expect("movement")
                .speeds
                .iter()
                .all(|speed| speed.movement_type != "land")
        );
        assert_eq!(speed(&zero_speed, "fly").adjusted_value_feet, 5);
    }

    #[derive(Debug, Clone, Copy)]
    enum RuntimeNumberField {
        ArmorClass,
        MaximumHp,
        Perception,
        Fortitude,
        Reflex,
        Will,
        Athletics,
        Strength,
        Dexterity,
    }

    fn assert_stat(
        projection: &EncounterRuntimeView,
        field: RuntimeNumberField,
        base: i64,
        adjusted: i64,
        label: &str,
    ) {
        let value = value(projection, field);
        assert_eq!(value.base_value, base);
        assert_eq!(value.adjusted_value, adjusted);
        assert!(
            value
                .modifiers
                .iter()
                .any(|modifier| modifier.label == label)
        );
    }

    fn value(projection: &EncounterRuntimeView, field: RuntimeNumberField) -> &RuntimeNumberView {
        match field {
            RuntimeNumberField::ArmorClass => &projection.defenses.as_ref().expect("defenses").armor_class,
            RuntimeNumberField::MaximumHp => projection.vitals.as_ref().and_then(|vitals| vitals.maximum_hp.as_ref()).expect("maximum hp"),
            RuntimeNumberField::Perception => &projection.awareness.as_ref().expect("awareness").perception,
            RuntimeNumberField::Fortitude => projection.saves.as_ref().and_then(|saves| saves.fortitude.as_ref()).expect("fortitude"),
            RuntimeNumberField::Reflex => projection.saves.as_ref().and_then(|saves| saves.reflex.as_ref()).expect("reflex"),
            RuntimeNumberField::Will => projection.saves.as_ref().and_then(|saves| saves.will.as_ref()).expect("will"),
            RuntimeNumberField::Athletics => &projection.skills.iter().find(|skill| matches!(skill.kind, EncounterRuntimeSkillKindView::Standard { ref slug } if slug == "athletics") || skill.label == "Athletics").expect("athletics").modifier,
            RuntimeNumberField::Strength => projection.abilities.as_ref().and_then(|abilities| abilities.strength.as_ref()).expect("strength"),
            RuntimeNumberField::Dexterity => projection.abilities.as_ref().and_then(|abilities| abilities.dexterity.as_ref()).expect("dexterity"),
        }
    }

    fn speed<'a>(
        projection: &'a EncounterRuntimeView,
        movement_type: &str,
    ) -> &'a RuntimeDistanceView {
        projection
            .movement
            .as_ref()
            .expect("movement")
            .speeds
            .iter()
            .find(|speed| speed.movement_type == movement_type)
            .expect("speed should exist")
    }

    fn assert_damage_modifier(
        projection: &EncounterRuntimeView,
        activity_id: &str,
        damage_id: &str,
        value: i64,
    ) {
        let damage = damage(projection, activity_id, damage_id);
        assert_eq!(damage.modifiers.len(), 1);
        assert_eq!(damage.modifiers[0].value, value);
    }

    fn assert_no_damage_modifier(
        projection: &EncounterRuntimeView,
        activity_id: &str,
        damage_id: &str,
    ) {
        assert!(
            damage(projection, activity_id, damage_id)
                .modifiers
                .is_empty()
        );
    }

    fn assert_roll(
        projection: &EncounterRuntimeView,
        activity_id: &str,
        roll_id: &str,
        base: i64,
        adjusted: i64,
        label: &str,
    ) {
        let roll = roll(projection, activity_id, roll_id);
        assert_eq!(roll.base_value, base);
        assert_eq!(roll.adjusted_value, adjusted);
        assert!(
            roll.modifiers
                .iter()
                .any(|modifier| modifier.label == label)
        );
    }

    fn roll<'a>(
        projection: &'a EncounterRuntimeView,
        activity_id: &str,
        roll_id: &str,
    ) -> &'a RuntimeRollView {
        projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == activity_id)
            .and_then(|activity| activity.rolls.iter().find(|roll| roll.roll_id == roll_id))
            .expect("activity roll should exist")
    }

    fn damage<'a>(
        projection: &'a EncounterRuntimeView,
        activity_id: &str,
        damage_id: &str,
    ) -> &'a RuntimeFormulaView {
        projection
            .activities
            .iter()
            .find(|activity| activity.activity_id == activity_id)
            .and_then(|activity| {
                activity
                    .damage
                    .iter()
                    .find(|damage| damage.damage_id == damage_id)
            })
            .expect("damage expression should exist")
    }

    #[test]
    #[ignore = "exports checksum-bound E2R final-direction samples"]
    fn export_typed_encounter_runtime_final_samples() {
        let sample_root = required_path_env("E2R_SAMPLE_ROOT");
        assert!(
            !sample_root.exists(),
            "sample root must be fresh and no-clobber: {}",
            sample_root.display()
        );
        fs::create_dir_all(
            sample_root
                .parent()
                .expect("sample root should have a parent"),
        )
        .expect("sample parent should be creatable");
        fs::create_dir(&sample_root).expect("fresh sample root should be creatable");

        let candidate = required_env("E2R_SAMPLE_CANDIDATE");
        let candidate_tree = required_env("E2R_SAMPLE_TREE");
        assert_eq!(git_value(Path::new("."), &["rev-parse", "HEAD"]), candidate);
        assert_eq!(
            git_value(Path::new("."), &["rev-parse", "HEAD^{tree}"]),
            candidate_tree
        );
        assert_eq!(
            git_value(Path::new("."), &["rev-parse", "HEAD^"]),
            "4cd7c052a642c3ef52d3ed347fb77303bf2a25a0"
        );

        let source_root = required_path_env("E2R_SAMPLE_SOURCE_ROOT");
        let source_commit = required_env("E2R_SAMPLE_SOURCE_COMMIT");
        let source_tree = required_env("E2R_SAMPLE_SOURCE_TREE");
        assert_eq!(
            git_value(&source_root, &["rev-parse", "HEAD"]),
            source_commit
        );
        assert_eq!(
            git_value(&source_root, &["rev-parse", "HEAD^{tree}"]),
            source_tree
        );
        let night_hag_source = source_root.join("packs/pathfinder-bestiary/night-hag.json");
        let giant_rat_source = source_root.join("packs/pathfinder-monster-core/giant-rat.json");
        assert_eq!(
            file_sha256(&night_hag_source),
            "9b7697e6ea8a367b432c9f2d11fdaadf1f32a58b6a6ee5ec2e5ec3ca517829a8"
        );
        assert_eq!(
            file_sha256(&giant_rat_source),
            "f8399003c84dff77ec500f39a1e4996bf4a0eadb71be606152ae34f088570b4f"
        );
        let sample_index = required_path_env("E2R_SAMPLE_INDEX");
        assert_eq!(
            file_sha256(&sample_index),
            required_env("E2R_SAMPLE_INDEX_SHA256")
        );

        let local_state = std::env::temp_dir().join(format!(
            "atlas-e2r-sample-state-{}-{}.sqlite",
            std::process::id(),
            unique_sample_suffix()
        ));
        let service = AtlasAppService::new(
            RetrievalBackend::OnDemandNoEmbeddings,
            AtlasRuntimeOptions {
                path_mode: AtlasPathMode::Global,
                overrides: AtlasPathOverrides {
                    source_root: Some(source_root.clone()),
                    embedding_cache_root: None,
                    index_path: Some(sample_index.clone()),
                },
            },
            local_state.clone(),
        )
        .expect("sample service should start");
        let encounter = service
            .create_encounter(CreateEncounterRequest {
                name: "E2R Typed Runtime Evidence".to_string(),
                description: Some(
                    "Candidate-authentic API serialization for final outer level-field inspection"
                        .to_string(),
                ),
                note: None,
            })
            .expect("sample encounter should create")
            .encounter;
        let store = service
            .local_state_store()
            .expect("sample local state should open");
        let night_hag_key = RecordKey::parse("pathfinder-bestiary:WQy7HBUcgDLsfVJd")
            .expect("Night Hag key should parse");
        let giant_rat_key = RecordKey::parse("pathfinder-monster-core:iIJPJcDT8wlJ8z5M")
            .expect("Giant Rat key should parse");
        for (display_name, record_key, variant) in [
            (
                "Night Hag — Normal",
                night_hag_key.clone(),
                ParticipantVariant::Normal,
            ),
            (
                "Night Hag — Elite",
                night_hag_key.clone(),
                ParticipantVariant::Elite,
            ),
            ("Night Hag — Weak", night_hag_key, ParticipantVariant::Weak),
            (
                "Giant Rat — Normal",
                giant_rat_key,
                ParticipantVariant::Normal,
            ),
        ] {
            let participant = store
                .encounters()
                .add_participant(
                    &encounter.slug,
                    atlas_local_state::AddEncounterParticipant {
                        record_key: Some(record_key),
                        participant_kind: atlas_local_state::ParticipantKind::Creature,
                        display_name: display_name.to_string(),
                        record_title_snapshot: Some(display_name.to_string()),
                        record_kind_snapshot: Some("creature".to_string()),
                        side: atlas_local_state::ParticipantSide::Enemy,
                        initiative: None,
                        max_hp: None,
                        current_hp: None,
                        temporary_hp: 0,
                        note: None,
                    },
                )
                .expect("real sample participant should add");
            if variant != ParticipantVariant::Normal {
                store
                    .encounters()
                    .update_participant(atlas_local_state::UpdateEncounterParticipant {
                        participant_key: participant.participant_key.clone(),
                        display_name: participant.display_name.clone(),
                        side: participant.side,
                        participant_variant: variant,
                        hazard_state: participant.hazard_state,
                        initiative: participant.initiative,
                        max_hp: participant.max_hp,
                        current_hp: participant.current_hp,
                        temporary_hp: participant.temporary_hp,
                        defeated: participant.defeated,
                        hidden: participant.hidden,
                        note: participant.note.clone(),
                    })
                    .expect("sample participant variant should update")
                    .expect("sample participant should remain present");
            }
        }
        store
            .encounters()
            .add_participant(
                &encounter.slug,
                atlas_local_state::AddEncounterParticipant {
                    record_key: None,
                    participant_kind: atlas_local_state::ParticipantKind::Pc,
                    display_name: "Concept PC".to_string(),
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: atlas_local_state::ParticipantSide::Pc,
                    initiative: Some(17),
                    max_hp: Some(42),
                    current_hp: Some(31),
                    temporary_hp: 0,
                    note: None,
                },
            )
            .expect("manual concept participant should add");
        store
            .encounters()
            .add_participant(
                &encounter.slug,
                atlas_local_state::AddEncounterParticipant {
                    record_key: Some(
                        RecordKey::parse("pathfinder-bestiary:missing-e2r-sample")
                            .expect("unresolved sample key should parse"),
                    ),
                    participant_kind: atlas_local_state::ParticipantKind::Creature,
                    display_name: "Unavailable Record Concept".to_string(),
                    record_title_snapshot: Some("Unavailable Record Concept".to_string()),
                    record_kind_snapshot: Some("creature".to_string()),
                    side: atlas_local_state::ParticipantSide::Enemy,
                    initiative: None,
                    max_hp: Some(12),
                    current_hp: Some(9),
                    temporary_hp: 0,
                    note: Some("Intentional concept mock for unresolved hydration".to_string()),
                },
            )
            .expect("unresolved concept participant should add");
        drop(store);
        let api_detail = service
            .encounter(&encounter.slug)
            .expect("sample encounter should serialize");
        let participant_named = |name: &str| {
            api_detail
                .participants
                .iter()
                .find(|participant| participant.display_name == name)
                .unwrap_or_else(|| panic!("{name} should remain present"))
                .clone()
        };
        let night_hag_normal = participant_named("Night Hag — Normal");
        let night_hag_elite = participant_named("Night Hag — Elite");
        let night_hag_weak = participant_named("Night Hag — Weak");
        let giant_rat = participant_named("Giant Rat — Normal");
        let manual = participant_named("Concept PC");
        let unresolved = api_detail
            .participants
            .iter()
            .find(|participant| participant.display_name == "Unavailable Record Concept")
            .expect("unresolved participant should remain explicit")
            .clone();

        let fatigued = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![condition("Fatigued", None)],
        ));
        let condition_stacking = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Fatigued", None),
                condition("Frightened", Some(2)),
                condition("Off-Guard", None),
                condition("Prone", None),
            ],
        ));
        let action_budget = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![
                condition("Quickened", Some(1)),
                condition("Slowed", Some(1)),
                condition("Stunned", Some(1)),
            ],
        ));
        let movement = project_canonical(&participant(
            ParticipantVariant::Normal,
            vec![condition("Grabbed", None), condition("Restrained", None)],
        ));
        let resources_spellcasting_activities =
            project_canonical(&participant(ParticipantVariant::Normal, Vec::new()));

        write_sample_json!(
            &sample_root.join("encounter-runtime-night-hag-normal.json"),
            night_hag_normal
                .record_view
                .encounter
                .as_ref()
                .expect("record-backed participant should expose encounter runtime"),
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-night-hag-elite.json"),
            night_hag_elite
                .record_view
                .encounter
                .as_ref()
                .expect("record-backed participant should expose encounter runtime"),
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-night-hag-weak.json"),
            night_hag_weak
                .record_view
                .encounter
                .as_ref()
                .expect("record-backed participant should expose encounter runtime"),
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-giant-rat-normal.json"),
            giant_rat
                .record_view
                .encounter
                .as_ref()
                .expect("record-backed participant should expose encounter runtime"),
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-fatigued.json"),
            &fatigued,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-condition-stacking.json"),
            &condition_stacking,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-action-budget.json"),
            &action_budget,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-movement.json"),
            &movement,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-resources-spellcasting-activities.json"),
            &resources_spellcasting_activities,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-manual-pc.json"),
            &manual,
        );
        write_sample_json!(
            &sample_root.join("encounter-runtime-unresolved.json"),
            &unresolved,
        );
        write_sample_json!(&sample_root.join("api-encounter-detail.json"), &api_detail);

        let bindings_root = sample_root.join("generated-bindings");
        fs::create_dir(&bindings_root).expect("bindings sample directory should be creatable");
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for name in [
            "EncounterDetailView.ts",
            "EncounterParticipantView.ts",
            "EncounterRuntimeView.ts",
            "EncounterRuntimeActivityView.ts",
            "EncounterRuntimeAutomationLimitationCodeView.ts",
            "EncounterRuntimeAutomationLimitationTargetView.ts",
            "EncounterRuntimeAutomationLimitationView.ts",
            "EncounterRuntimeActionCostView.ts",
            "EncounterRuntimeFrequencyView.ts",
            "EncounterRuntimeUsesView.ts",
            "RuntimeFactProvenanceView.ts",
            "RuntimeCanonicalTargetView.ts",
        ] {
            fs::copy(
                workspace_root
                    .join("crates/atlas-app-model/bindings")
                    .join(name),
                bindings_root.join(name),
            )
            .unwrap_or_else(|error| panic!("binding {name} should copy: {error}"));
        }
        fs::copy(
            workspace_root.join("web/atlas-ui/src/generated/atlas.ts"),
            bindings_root.join("atlas.ts"),
        )
        .expect("generated aggregate binding should copy");

        fs::write(
            sample_root.join("presentation.md"),
            format!(
                "# E2R final-direction candidate samples\n\nCandidate `{candidate}` (tree `{candidate_tree}`).\n\nThis is the **fresh candidate-authentic FINAL sample package pending independent E2R technical rereview and separate explicit final sample approval**. It does not itself approve E2R or authorize E3.\n\n## Authentic real Foundry records (exactly two)\n\n- Dense/complex: Night Hag, `pathfinder-bestiary:WQy7HBUcgDLsfVJd`; normal, elite, and weak candidate serializations.\n- Sparse/simple: Giant Rat, `pathfinder-monster-core:iIJPJcDT8wlJ8z5M`; normal candidate serialization.\n\n## Clearly labeled concept mock\n\nThe Fatigued, condition-stacking, action-budget, movement, resources/spellcasting/activities, manual-PC, and unresolved samples use the in-tree canonical E2 mechanics fixture or deliberately authored encounter state. They inspect the final-direction candidate contract and do not claim additional Foundry records. `api-encounter-detail.json` is an authentic app-service API DTO containing the same two real records plus the labeled manual/unresolved concept participants.\n\nThe sole product-shape delta is the outer `EncounterRuntimeView.level` property replacing `adjusted_level`. Its `RuntimeNumberView` still carries `base_value`, `adjusted_value`, modifiers, suppressed modifiers, and typed provenance unchanged. There is no alias, shim, dual field, or mechanics change.\n"
            ),
        )
        .expect("presentation should write");
        fs::write(
            sample_root.join("candidate-report.md"),
            format!(
                "# E2R final-direction candidate report\n\n- Candidate: `{candidate}`\n- Tree: `{candidate_tree}`\n- Parent: `4cd7c052a642c3ef52d3ed347fb77303bf2a25a0`\n- Producer: `cargo test -p atlas-app-service encounters::mechanics::tests::export_typed_encounter_runtime_final_samples -- --ignored --exact`\n- Producer path: `crates/atlas-app-service/src/encounters/mechanics.rs`\n- Final-direction approval: `e2r-final-direction-correction-approval.json` SHA-256 `ee12184fda2cbc51b0a86d29778044c6d9daaefb49eba19082a4a27124a0ed15`\n- Prior technical PASS: `2026-08-30-e2r-typed-encounter-runtime-correction-rereview-002.md` SHA-256 `de9f8f869c94f23d5de7956bad049864118491ab3a919dd0a29e7f1f81ed2547`\n- Source commit/tree: `{source_commit}` / `{source_tree}`\n- Source signature: `{}`\n- Retained artifact: `{}`\n- Validation before export: `{}`\n- UI boundary: candidate UI typecheck/build retains the adjudicated 65-diagnostic downstream-consumer failure; frontend source remains intentionally untouched until its serialized task.\n\nThis is the one-field final-direction correction and stops before fresh independent E2R technical rereview, separate final sample approval, acceptance, or E3.\n",
                required_env("E2R_SAMPLE_SOURCE_SIGNATURE"),
                sample_index.display(),
                required_env("E2R_SAMPLE_VALIDATION")
            ),
        )
        .expect("candidate report should write");

        let correction_root = required_path_env("E2R_CORRECTION_SAMPLE_ROOT");
        for (relative, expected) in [
            (
                "sample-manifest.json",
                "6b19882ddc8537fdc8512675a9a9f291f0af89fe9436ce16b2eb710a20a3fadb",
            ),
            (
                "checksums.sha256",
                "9e675e67b2500a7ccd6817c3ee7be3113cfe17630aa0a394082966b44d7de94f",
            ),
            (
                "presentation.md",
                "e821b1e34721bdb86335558ce6ee18c6f5d40d1cfc10eded5426def0cb899d9a",
            ),
            (
                "candidate-report.md",
                "19cd4afff1600fd58a4bcd6baff28f5bdaffebf6e5ff19881e69472731524207",
            ),
            (
                "refined-to-correction-delta-ledger.md",
                "495dee3a55ea851bd47855a8439b28ba62131b76dafa09a9ac9a6a3f91bd10ca",
            ),
        ] {
            assert_eq!(file_sha256(&correction_root.join(relative)), expected);
        }
        let metadata_names = [
            "candidate-report.md",
            "correction-to-final-delta-ledger.md",
            "checksums.sha256",
            "early-to-refined-delta-ledger.md",
            "refined-to-correction-delta-ledger.md",
            "presentation.md",
            "sample-manifest.json",
        ];
        let mut compared_outputs = BTreeSet::new();
        for root in [&correction_root, &sample_root] {
            for relative in relative_files(root) {
                if !metadata_names.contains(&relative.to_string_lossy().as_ref()) {
                    compared_outputs.insert(relative);
                }
            }
        }
        let mut delta_rows = Vec::new();
        for relative in compared_outputs {
            let correction_path = correction_root.join(&relative);
            let final_path = sample_root.join(&relative);
            let correction_hash = correction_path
                .exists()
                .then(|| file_sha256(&correction_path));
            let final_hash = final_path.exists().then(|| file_sha256(&final_path));
            let disposition = match (&correction_hash, &final_hash) {
                (Some(correction), Some(final_output)) if correction == final_output => "unchanged",
                (Some(_), Some(_)) => "changed",
                (Some(_), None) => "removed",
                (None, Some(_)) => "added",
                (None, None) => unreachable!("union member must exist in at least one root"),
            };
            delta_rows.push(format!(
                "| `{}` | `{}` | `{}` | {disposition} |",
                relative.display(),
                correction_hash.as_deref().unwrap_or("—"),
                final_hash.as_deref().unwrap_or("—")
            ));
        }
        fs::write(
            sample_root.join("correction-to-final-delta-ledger.md"),
            format!(
                "# E2R correction-to-final delta ledger\n\n## Bound inputs\n\n- Independently passed correction candidate: `4cd7c052a642c3ef52d3ed347fb77303bf2a25a0` (tree `407f5373d7eb6d10ebcb413ef720da1d968349dd`).\n- Correction root: `{}`.\n- Correction manifest/checksums/presentation/report/ledger SHA-256: `6b19882ddc8537fdc8512675a9a9f291f0af89fe9436ce16b2eb710a20a3fadb` / `9e675e67b2500a7ccd6817c3ee7be3113cfe17630aa0a394082966b44d7de94f` / `e821b1e34721bdb86335558ce6ee18c6f5d40d1cfc10eded5426def0cb899d9a` / `19cd4afff1600fd58a4bcd6baff28f5bdaffebf6e5ff19881e69472731524207` / `495dee3a55ea851bd47855a8439b28ba62131b76dafa09a9ac9a6a3f91bd10ca`.\n- Final-direction approval: `/Users/ekosten/.ao/data/handoffs/pathfinder-2e-foundry-mcp/source-faithful-records/20260824T210853Z-c7b74cbdc7c4-pathfinder-2e-foundry-mcp-17/approvals/e2r-final-direction-correction-approval.json`, SHA-256 `ee12184fda2cbc51b0a86d29778044c6d9daaefb49eba19082a4a27124a0ed15`, mode `0444`.\n- Prior technical PASS: `/Users/ekosten/.ao/data/worktrees/pathfinder-2e-foundry-mcp/pathfinder-2e-foundry-mcp-56/scratch/plan-validation/2026-08-30-e2r-typed-encounter-runtime-correction-rereview-002.md`, SHA-256 `de9f8f869c94f23d5de7956bad049864118491ab3a919dd0a29e7f1f81ed2547`, mode `0444`.\n- Final-direction candidate: `{candidate}` (tree `{candidate_tree}`), direct child of `4cd7c052a642c3ef52d3ed347fb77303bf2a25a0`.\n\n## Complete public-shape delta\n\n1. The outer optional `EncounterRuntimeView.adjusted_level` property is directly replaced by `EncounterRuntimeView.level` in Rust, JSON, API output, checked-in TypeScript, and the generated web binding surface.\n2. The `level` value remains the identical `RuntimeNumberView`: `base_value`, `adjusted_value`, modifiers, suppressed modifiers, and typed provenance are unchanged.\n3. There is no alias, shim, dual field, fallback, compatibility path, or mechanics change. Exact DTO, transport, and binding tests require `level` and reject `adjusted_level`.\n4. All candidate-produced runtime JSON containing a hydrated canonical level changes only that outer key. Generated `EncounterRuntimeView.ts` changes only that property name; the aggregate `atlas.ts` is regenerated and freshness-checked. Other ordinary outputs remain byte-identical except run-local participant/encounter IDs, timestamps, and the candidate-specific API description regenerated by the authentic exporter.\n5. The adjudicated 65-diagnostic serialized frontend boundary remains expected and frontend source is untouched.\n\n## Every candidate-produced output\n\nThe table is the no-omission union of candidate-produced outputs in the bound correction and final roots. Metadata files are excluded because this ledger, report, manifest, presentation, and checksum seal necessarily describe different candidates.\n\n| Output | Correction SHA-256 | Final SHA-256 | Delta |\n|---|---|---|---|\n{}\n",
                correction_root.display(),
                delta_rows.join("\n")
            ),
        )
        .expect("delta ledger should write");

        let payload_files = relative_files(&sample_root);
        let payload_hashes = payload_files
            .iter()
            .map(|relative| {
                serde_json::json!({
                    "path": relative,
                    "sha256": file_sha256(&sample_root.join(relative)),
                })
            })
            .collect::<Vec<_>>();
        let manifest = serde_json::json!({
            "schema": "e2r-typed-encounter-runtime-final-samples/v1",
            "status": "final_direction_candidate_pending_independent_technical_rereview_and_final_sample_approval",
            "candidate": { "commit": candidate, "tree": candidate_tree, "parent": "4cd7c052a642c3ef52d3ed347fb77303bf2a25a0" },
            "approval": { "path": "/Users/ekosten/.ao/data/handoffs/pathfinder-2e-foundry-mcp/source-faithful-records/20260824T210853Z-c7b74cbdc7c4-pathfinder-2e-foundry-mcp-17/approvals/e2r-final-direction-correction-approval.json", "sha256": "ee12184fda2cbc51b0a86d29778044c6d9daaefb49eba19082a4a27124a0ed15", "mode": "0444" },
            "producer": { "command": "cargo test -p atlas-app-service encounters::mechanics::tests::export_typed_encounter_runtime_final_samples -- --ignored --exact", "test_path": "crates/atlas-app-service/src/encounters/mechanics.rs" },
            "source": {
                "root": source_root,
                "commit": source_commit,
                "tree": source_tree,
                "signature": required_env("E2R_SAMPLE_SOURCE_SIGNATURE"),
                "records": [
                    { "classification": "real_foundry_dense_complex", "record_key": "pathfinder-bestiary:WQy7HBUcgDLsfVJd", "source_path": "packs/pathfinder-bestiary/night-hag.json", "source_sha256": "9b7697e6ea8a367b432c9f2d11fdaadf1f32a58b6a6ee5ec2e5ec3ca517829a8" },
                    { "classification": "real_foundry_sparse_simple", "record_key": "pathfinder-monster-core:iIJPJcDT8wlJ8z5M", "source_path": "packs/pathfinder-monster-core/giant-rat.json", "source_sha256": "f8399003c84dff77ec500f39a1e4996bf4a0eadb71be606152ae34f088570b4f" }
                ]
            },
            "classification": {
                "real_foundry": ["encounter-runtime-night-hag-normal.json", "encounter-runtime-night-hag-elite.json", "encounter-runtime-night-hag-weak.json", "encounter-runtime-giant-rat-normal.json"],
                "concept_mock": ["encounter-runtime-fatigued.json", "encounter-runtime-condition-stacking.json", "encounter-runtime-action-budget.json", "encounter-runtime-movement.json", "encounter-runtime-resources-spellcasting-activities.json", "encounter-runtime-manual-pc.json", "encounter-runtime-unresolved.json"],
                "mixed_exact_api": ["api-encounter-detail.json"]
            },
            "artifact": { "path": sample_index, "trusted_sha256": required_env("E2R_SAMPLE_INDEX_SHA256") },
            "files": payload_hashes,
            "checksums": "checksums.sha256 (self-excluded)"
        });
        write_sample_json!(&sample_root.join("sample-manifest.json"), &manifest);

        let mut checksum_lines = relative_files(&sample_root)
            .into_iter()
            .map(|relative| {
                format!(
                    "{}  {}",
                    file_sha256(&sample_root.join(&relative)),
                    relative.display()
                )
            })
            .collect::<Vec<_>>();
        checksum_lines.sort();
        fs::write(
            sample_root.join("checksums.sha256"),
            format!("{}\n", checksum_lines.join("\n")),
        )
        .expect("checksums should write");
        for relative in relative_files(&sample_root) {
            let path = sample_root.join(relative);
            let mut permissions = fs::metadata(&path)
                .expect("sample metadata should read")
                .permissions();
            permissions.set_readonly(true);
            fs::set_permissions(path, permissions).expect("sample file should seal read-only");
        }
        drop(service);
        let _ = fs::remove_file(local_state);
    }

    fn required_env(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
    }

    fn required_path_env(name: &str) -> PathBuf {
        PathBuf::from(required_env(name))
    }

    fn git_value(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git should run");
        assert!(output.status.success(), "git command should succeed");
        String::from_utf8(output.stdout)
            .expect("git output should be UTF-8")
            .trim()
            .to_string()
    }

    fn file_sha256(path: &Path) -> String {
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .expect("shasum should run");
        assert!(output.status.success(), "shasum should succeed");
        String::from_utf8(output.stdout)
            .expect("shasum output should be UTF-8")
            .split_whitespace()
            .next()
            .expect("shasum should emit a digest")
            .to_string()
    }

    fn relative_files(root: &Path) -> Vec<PathBuf> {
        fn visit(root: &Path, path: &Path, files: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(path).expect("sample directory should read") {
                let entry = entry.expect("sample entry should read");
                let path = entry.path();
                if path.is_dir() {
                    visit(root, &path, files);
                } else if path.file_name().and_then(|name| name.to_str())
                    != Some("checksums.sha256")
                {
                    files.push(
                        path.strip_prefix(root)
                            .expect("sample path should be relative")
                            .to_path_buf(),
                    );
                }
            }
        }
        let mut files = Vec::new();
        visit(root, root, &mut files);
        files.sort();
        files
    }

    fn unique_sample_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    }

    fn participant(
        participant_variant: ParticipantVariant,
        conditions: Vec<EncounterParticipantCondition>,
    ) -> EncounterParticipant {
        EncounterParticipant {
            participant_key: "participant".to_string(),
            record_key: Some("actors:test".to_string()),
            participant_kind: atlas_local_state::ParticipantKind::Creature,
            participant_variant,
            hazard_state: ParticipantHazardState::Active,
            position: 1,
            display_name: "Creature".to_string(),
            record_title_snapshot: Some("Creature".to_string()),
            record_kind_snapshot: Some("creature".to_string()),
            side: atlas_local_state::ParticipantSide::Enemy,
            initiative: None,
            initiative_order: 1,
            max_hp: Some(60),
            current_hp: Some(60),
            temporary_hp: 0,
            defeated: false,
            hidden: false,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            conditions,
        }
    }

    fn condition(name: &str, value: Option<i64>) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            condition_id: 1,
            condition_key: Some(condition_key(name).to_string()),
            name: name.to_string(),
            value,
            source_participant_key: None,
            duration_rounds: None,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn condition_with_duration(
        name: &str,
        value: Option<i64>,
        duration_rounds: Option<i64>,
    ) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            duration_rounds,
            ..condition(name, value)
        }
    }

    fn unmodeled_condition(name: &str, value: Option<i64>) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            condition_id: 1,
            condition_key: None,
            name: name.to_string(),
            value,
            source_participant_key: None,
            duration_rounds: None,
            note: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn condition_key(name: &str) -> &'static str {
        match slugify(name).as_str() {
            "frightened" => "conditionitems:TBSHQspnbcqxsmjL",
            "sickened" => "conditionitems:fesd1n5eVhpCSS18",
            "off-guard" => "conditionitems:AJh5ex99aV6VTggg",
            "clumsy" => "conditionitems:i3OJZU2nk64Df3xm",
            "enfeebled" => "conditionitems:MIRkyAjyBeXivMa7",
            "stupefied" => "conditionitems:e1XGnhKNSQIm5IXg",
            "slowed" => "conditionitems:xYTAsEpcJE1Ccni3",
            "quickened" => "conditionitems:nlCjDvLMf2EkV2dl",
            "stunned" => "conditionitems:dfCMdR4wnpbYNTix",
            "immobilized" => "conditionitems:eIcWbB5o3pP6OIMe",
            "grabbed" => "conditionitems:kWc1fhmv9LBiTuei",
            "restrained" => "conditionitems:VcDeM8A5oI6VqhbM",
            "encumbered" => "conditionitems:D5mg6Tc7Jzrj6ro7",
            "prone" => "conditionitems:j91X7x0XSomq8d60",
            "fatigued" => "conditionitems:HL2l2VRSaQHu9lUw",
            _ => "conditionitems:unsupported",
        }
    }

    fn mechanics_fixture() -> EncounterMechanicsInput {
        mechanics_fixture_with_speeds(&[("land", 25), ("fly", 10)])
    }

    fn mechanics_fixture_with_speeds(speeds: &[(&str, i64)]) -> EncounterMechanicsInput {
        EncounterMechanicsInput {
            level: Some(5),
            values: vec![
                named_runtime_mechanic(MechanicTarget::ArmorClass, "Armor Class", 22),
                named_runtime_mechanic(MechanicTarget::MaxHp, "Maximum HP", 60),
                named_runtime_mechanic(MechanicTarget::Perception, "Perception", 13),
                named_runtime_mechanic(
                    MechanicTarget::Save {
                        save: SaveKind::Fortitude,
                    },
                    "Fortitude",
                    15,
                ),
                named_runtime_mechanic(
                    MechanicTarget::Save {
                        save: SaveKind::Reflex,
                    },
                    "Reflex",
                    12,
                ),
                named_runtime_mechanic(
                    MechanicTarget::Save {
                        save: SaveKind::Will,
                    },
                    "Will",
                    12,
                ),
                named_runtime_mechanic(
                    MechanicTarget::AbilityModifier {
                        ability: AbilityKind::Strength,
                    },
                    "Strength",
                    4,
                ),
                named_runtime_mechanic(
                    MechanicTarget::AbilityModifier {
                        ability: AbilityKind::Dexterity,
                    },
                    "Dexterity",
                    3,
                ),
                named_runtime_mechanic(
                    MechanicTarget::AbilityModifier {
                        ability: AbilityKind::Intelligence,
                    },
                    "Intelligence",
                    1,
                ),
                named_runtime_mechanic(
                    MechanicTarget::AbilityModifier {
                        ability: AbilityKind::Wisdom,
                    },
                    "Wisdom",
                    2,
                ),
                named_runtime_mechanic(
                    MechanicTarget::AbilityModifier {
                        ability: AbilityKind::Charisma,
                    },
                    "Charisma",
                    0,
                ),
                named_runtime_mechanic(
                    MechanicTarget::CreatureSkill {
                        skill_id: CreatureComponentId::new("athletics")
                            .expect("skill id should be valid"),
                        kind: CreatureSkillKind::Athletics,
                    },
                    "Athletics",
                    9,
                ),
                named_runtime_mechanic(
                    MechanicTarget::CreatureSkill {
                        skill_id: CreatureComponentId::new("stealth")
                            .expect("skill id should be valid"),
                        kind: CreatureSkillKind::Stealth,
                    },
                    "Stealth",
                    8,
                ),
                named_runtime_mechanic(
                    MechanicTarget::CreatureSkill {
                        skill_id: CreatureComponentId::new("arcana")
                            .expect("skill id should be valid"),
                        kind: CreatureSkillKind::Arcana,
                    },
                    "Arcana",
                    7,
                ),
            ],
            speeds: speeds
                .iter()
                .filter(|(_, value)| *value > 0)
                .map(|(speed, value)| EncounterMovementInput {
                    movement_type: (*speed).to_string(),
                    label: match *speed {
                        "land" => "Land Speed".to_string(),
                        "fly" => "Fly Speed".to_string(),
                        other => format!("{other} Speed"),
                    },
                    value_feet: *value,
                })
                .collect(),
            activities: vec![
                EncounterActivityInput {
                    activity_id: "claw".to_string(),
                    label: "Claw".to_string(),
                    kind: EncounterActivityKind::Strike,
                    usage: EncounterActivityUsage::Unlimited,
                    rolls: vec![EncounterActivityRoll {
                        roll_id: "attack".to_string(),
                        label: "Attack".to_string(),
                        base_value: 12,
                        surface: EncounterActivityRollSurface::AttackRoll,
                        ability: Some(atlas_record::ActivityRollAbility::Strength),
                    }],
                    damage: vec![
                        EncounterDamageExpression {
                            damage_id: "main".to_string(),
                            label: None,
                            formula: "1d6+4".to_string(),
                            damage_type: Some("slashing".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: Some(atlas_record::ActivityRollAbility::Strength),
                        },
                        EncounterDamageExpression {
                            damage_id: "secondary".to_string(),
                            label: Some("Persistent bleed".to_string()),
                            formula: "1d4".to_string(),
                            damage_type: Some("bleed".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                    ],
                },
                EncounterActivityInput {
                    activity_id: "fireball".to_string(),
                    label: "Fireball".to_string(),
                    kind: EncounterActivityKind::Spell,
                    usage: EncounterActivityUsage::Limited,
                    rolls: vec![
                        EncounterActivityRoll {
                            roll_id: "spell.attack".to_string(),
                            label: "Spell Attack".to_string(),
                            base_value: 13,
                            surface: EncounterActivityRollSurface::AttackRoll,
                            ability: None,
                        },
                        EncounterActivityRoll {
                            roll_id: "spell.dc".to_string(),
                            label: "Spell DC".to_string(),
                            base_value: 22,
                            surface: EncounterActivityRollSurface::Dc,
                            ability: None,
                        },
                    ],
                    damage: vec![
                        EncounterDamageExpression {
                            damage_id: "0".to_string(),
                            label: None,
                            formula: "6d6".to_string(),
                            damage_type: Some("fire".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                        EncounterDamageExpression {
                            damage_id: "persistent".to_string(),
                            label: Some("Persistent fire".to_string()),
                            formula: "1d6".to_string(),
                            damage_type: Some("fire".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                    ],
                },
                EncounterActivityInput {
                    activity_id: "ignition".to_string(),
                    label: "Ignition".to_string(),
                    kind: EncounterActivityKind::Spell,
                    usage: EncounterActivityUsage::Unlimited,
                    rolls: Vec::new(),
                    damage: vec![
                        EncounterDamageExpression {
                            damage_id: "fire".to_string(),
                            label: None,
                            formula: "2d4".to_string(),
                            damage_type: Some("fire".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                        EncounterDamageExpression {
                            damage_id: "persistent".to_string(),
                            label: Some("Persistent fire".to_string()),
                            formula: "1d4".to_string(),
                            damage_type: Some("fire".to_string()),
                            effect_kind: atlas_record::DamageEffectKind::Damage,
                            ability: None,
                        },
                    ],
                },
                EncounterActivityInput {
                    activity_id: "heal".to_string(),
                    label: "Heal".to_string(),
                    kind: EncounterActivityKind::Spell,
                    usage: EncounterActivityUsage::Limited,
                    rolls: Vec::new(),
                    damage: vec![EncounterDamageExpression {
                        damage_id: "0".to_string(),
                        label: None,
                        formula: "1d8".to_string(),
                        damage_type: Some("vitality".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::DamageOrHealing,
                        ability: None,
                    }],
                },
                EncounterActivityInput {
                    activity_id: "breath".to_string(),
                    label: "Breath Weapon".to_string(),
                    kind: EncounterActivityKind::Other,
                    usage: EncounterActivityUsage::Ambiguous,
                    rolls: Vec::new(),
                    damage: vec![EncounterDamageExpression {
                        damage_id: "0".to_string(),
                        label: None,
                        formula: "4d6".to_string(),
                        damage_type: Some("fire".to_string()),
                        effect_kind: atlas_record::DamageEffectKind::Damage,
                        ability: None,
                    }],
                },
            ],
        }
    }
}
