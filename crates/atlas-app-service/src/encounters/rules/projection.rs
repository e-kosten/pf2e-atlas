use crate::encounters::rules::modifiers::{Modifier, ModifierType};
// Pure encounter condition evaluation and modifier resolution.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::super::mechanics::{adjusted_npc_level, integer};
use super::super::stat_block::*;
use atlas_app_model::*;
use atlas_local_state::{EncounterParticipant, ParticipantVariant};
use serde_json::Number;

pub(in crate::encounters) fn project_conditions(
    block: StatBlockView,
    participant: &EncounterParticipant,
    additional: &[super::causes::ScopedConditionCause<'_>],
) -> StatBlockView {
    let mut block = block;
    for condition in &participant.conditions {
        if condition_rule_for_key(condition.condition_key.as_deref()).is_none()
            && let Some(note) = condition.note.as_deref()
        {
            block
                .unapplied_effects
                .push(condition_note(condition, note));
        }
    }
    let opponents: std::collections::BTreeSet<_> = participant
        .conditions
        .iter()
        .flat_map(|c| match &c.scope {
            atlas_local_state::ConditionScope::Everyone => vec![],
            atlas_local_state::ConditionScope::SelectedParticipants { participant_keys } => {
                participant_keys.iter().map(String::as_str).collect()
            }
        })
        .chain(additional.iter().filter_map(|cause| cause.against))
        .chain(
            block
                .contextual_modifiers
                .iter()
                .map(|effect| effect.against.as_str()),
        )
        .collect();
    let contextual = opponents
        .into_iter()
        .filter_map(|opponent| {
            let evaluated =
                project_for_opponent(block.clone(), participant, Some(opponent), additional);
            evaluated
                .effective_stat(&StatisticTarget::ArmorClass)
                .map(|ac| EncounterContextualArmorClassView {
                    opponent_key: opponent.to_owned(),
                    value: FactView {
                        state: atlas_domain::QueryFieldState::Value,
                        value: Some(ac.clone()),
                    },
                })
        })
        .collect();
    let mut result = project_for_opponent(block, participant, None, additional);
    result.contextual_armor_class = contextual;
    result
}
fn project_for_opponent(
    mut block: StatBlockView,
    participant: &EncounterParticipant,
    opponent: Option<&str>,
    additional: &[super::causes::ScopedConditionCause<'_>],
) -> StatBlockView {
    for effect in block
        .contextual_modifiers
        .clone()
        .into_iter()
        .filter(|effect| opponent == Some(effect.against.as_str()))
    {
        if let Some(value) = block
            .values
            .iter_mut()
            .find(|value| value.target == effect.target)
        {
            apply(value, effect.modifier);
        }
    }
    if let Some(note) = stupefied_hint(participant, opponent) {
        block.unapplied_effects.push(note);
    }
    let mut movement_restriction = None;
    let deafened = super::causes::resolve_causes(participant, opponent, additional)
        .iter()
        .any(|cause| cause.rule == ConditionRule::Deafened);
    let available_senses = block
        .sense_facts
        .iter()
        .copied()
        .map(|mut facts| {
            if deafened && facts.auditory {
                facts.available = Some(false);
            }
            facts
        })
        .collect::<Vec<_>>();
    let blinded_penalty = block
        .senses_known
        .then(|| super::senses::blinded_penalty(&available_senses, None))
        .flatten();
    for cause in super::causes::resolve_causes(participant, opponent, additional) {
        let c = cause.source;
        let rule = cause.rule;
        let amount = match rule {
            ConditionRule::Frightened
            | ConditionRule::Sickened
            | ConditionRule::Clumsy
            | ConditionRule::Enfeebled
            | ConditionRule::Stupefied
            | ConditionRule::Drained => match cause.value {
                Some(value) if value >= 0 => value,
                _ => {
                    block.unapplied_effects.push(condition_note(
                        c,
                        "Enter the condition value to calculate its penalties.",
                    ));
                    continue;
                }
            },
            _ => 0,
        };
        let mut applied = false;
        for v in &mut block.values {
            let matched = match rule {
                ConditionRule::Blinded => {
                    v.target == StatisticTarget::Perception && blinded_penalty.is_some()
                }
                ConditionRule::Frightened | ConditionRule::Sickened => v.target.check_or_dc(),
                ConditionRule::OffGuard => v.target == StatisticTarget::ArmorClass,
                ConditionRule::Clumsy => v.ability == Some(Ability::Dexterity),
                ConditionRule::Enfeebled => v.ability == Some(Ability::Strength),
                ConditionRule::Stupefied => v.ability.is_some_and(Ability::mental),
                ConditionRule::Drained => {
                    v.ability == Some(Ability::Constitution)
                        || (v.target == StatisticTarget::HitPointMaximum
                            && block.adjusted_level.is_some())
                }
                ConditionRule::Fatigued => {
                    v.target == StatisticTarget::ArmorClass || v.target.save()
                }
                ConditionRule::Fascinated => {
                    v.target == StatisticTarget::Perception || v.target.skill()
                }
                ConditionRule::Unconscious => matches!(
                    v.target,
                    StatisticTarget::ArmorClass
                        | StatisticTarget::Perception
                        | StatisticTarget::Save(StatisticSave::Reflex)
                ),
                _ => false,
            };
            if matched {
                let value = if matches!(rule, ConditionRule::OffGuard) {
                    -2
                } else if rule == ConditionRule::Fatigued {
                    -1
                } else if rule == ConditionRule::Fascinated {
                    -2
                } else if rule == ConditionRule::Unconscious {
                    -4
                } else if rule == ConditionRule::Drained
                    && v.target == StatisticTarget::HitPointMaximum
                {
                    let Some(level) = block
                        .adjusted_level
                        .as_ref()
                        .and_then(|level| level.as_i64())
                    else {
                        continue;
                    };
                    let Some(penalty) =
                        drained_hp_penalty(amount, Some(level)).and_then(i64::checked_neg)
                    else {
                        continue;
                    };
                    penalty
                } else if rule == ConditionRule::Blinded {
                    blinded_penalty.unwrap_or(0)
                } else if rule == ConditionRule::Encumbered {
                    -1
                } else {
                    -amount
                };
                apply(
                    v,
                    Modifier {
                        source: format!("condition:{}", c.condition_id),
                        label: c.name.clone(),
                        modifier_type: if matches!(
                            rule,
                            ConditionRule::OffGuard
                                | ConditionRule::Grabbed
                                | ConditionRule::Restrained
                                | ConditionRule::Prone
                        ) {
                            ModifierType::Circumstance
                        } else {
                            ModifierType::Status
                        },
                        value,
                    },
                );
                applied = true;
            }
        }
        // Pinned PF2e 8.6.1: MeleePF2e.defaultAttribute and
        // getAttackDamageDomains define these selectors from typed strike traits.
        for activity in &mut block.activities {
            let penalty = match rule {
                ConditionRule::Frightened | ConditionRule::Sickened => {
                    Some((-amount, ModifierType::Status))
                }
                ConditionRule::Enfeebled if activity.attack_ability == Some(Ability::Strength) => {
                    Some((-amount, ModifierType::Status))
                }
                ConditionRule::Clumsy if activity.attack_ability == Some(Ability::Dexterity) => {
                    Some((-amount, ModifierType::Status))
                }
                ConditionRule::Stupefied
                    if activity.attack_ability.is_some_and(Ability::mental) =>
                {
                    Some((-amount, ModifierType::Status))
                }
                ConditionRule::Drained
                    if activity.attack_ability == Some(Ability::Constitution) =>
                {
                    Some((-amount, ModifierType::Status))
                }
                ConditionRule::Prone => Some((-2, ModifierType::Circumstance)),
                _ => None,
            };
            if let Some((value, modifier_type)) = penalty {
                for roll in &mut activity.rolls {
                    apply_modifier(
                        &mut roll.adjusted_value,
                        &mut roll.modifiers,
                        &mut roll.suppressed_modifiers,
                        Modifier {
                            source: format!("condition:{}", c.condition_id),
                            label: c.name.clone(),
                            modifier_type,
                            value,
                        },
                    );
                    applied = true;
                }
            }
            if let Some(dc) = &mut activity.difficulty_class {
                let applies = match rule {
                    ConditionRule::Frightened | ConditionRule::Sickened => true,
                    ConditionRule::Stupefied => dc.ability.is_some_and(Ability::mental),
                    ConditionRule::Clumsy => dc.ability == Some(Ability::Dexterity),
                    ConditionRule::Enfeebled => dc.ability == Some(Ability::Strength),
                    ConditionRule::Drained => dc.ability == Some(Ability::Constitution),
                    _ => false,
                };
                if applies {
                    apply(
                        dc,
                        Modifier {
                            source: format!("condition:{}", c.condition_id),
                            label: c.name.clone(),
                            modifier_type: ModifierType::Status,
                            value: -amount,
                        },
                    );
                    applied = true;
                }
            }
            if rule == ConditionRule::Enfeebled {
                for damage in &mut activity.damage {
                    if damage.effect_kind != DamageEffectKindView::Damage {
                        continue;
                    }
                    if !damage.ability_known {
                        damage.unavailable_reason =
                            Some("Damage ability is unavailable for Enfeebled.".into());
                        continue;
                    }
                    if damage.ability != Some(Ability::Strength) {
                        continue;
                    }
                    let mut contribution =
                        Number::from(damage.modifiers.iter().map(|m| m.value).sum::<i64>());
                    apply_modifier(
                        &mut contribution,
                        &mut damage.modifiers,
                        &mut damage.suppressed_modifiers,
                        Modifier {
                            source: format!("condition:{}", c.condition_id),
                            label: c.name.clone(),
                            modifier_type: ModifierType::Status,
                            value: -amount,
                        },
                    );
                    applied = true;
                }
            }
        }
        if rule == ConditionRule::Encumbered {
            movement_restriction = Some((rule, c));
            applied |= !block.speeds.is_empty();
        }
        if let Some(reason) = condition_effect_hint(c, rule, applied) {
            block.unapplied_effects.push(condition_note(c, &reason));
        }
    }
    if let Some((rule, condition)) = movement_restriction {
        for speed in &mut block.speeds {
            let baseline = &speed.base_value_feet;
            let restricted = if rule == ConditionRule::Encumbered {
                if let Some(n) = baseline.as_i64() {
                    Some(Number::from(if n > 0 { (n - 10).max(5) } else { n }))
                } else if baseline.is_u64() {
                    None
                } else {
                    baseline.as_f64().and_then(|n| {
                        Number::from_f64(if n > 0.0 { (n - 10.0).max(5.0) } else { n })
                    })
                }
            } else {
                Some(Number::from(0))
            };
            if let Some(restricted) = restricted {
                speed.adjusted_value_feet = restricted;
                speed.notes.push(RuntimeEffectNoteView {
                    source: condition.name.clone(),
                    label: condition.name.clone(),
                    reason: "Speed reduced by 10 feet, to a minimum of 5 feet.".into(),
                });
            }
        }
    }
    for activity in &mut block.activities {
        for damage in &mut activity.damage {
            let delta: i64 = damage.modifiers.iter().map(|m| m.value).sum();
            if delta != 0 && atlas_dice::prepare(&damage.formula).is_err() {
                damage.unavailable_reason =
                    Some("This damage formula needs table resolution.".into());
            }
            damage.adjusted_formula = (delta != 0 && damage.unavailable_reason.is_none())
                .then(|| format!("({}){delta:+}", damage.formula));
            if let Some(reason) = &damage.unavailable_reason {
                activity.notes.push(RuntimeEffectNoteView {
                    source: damage.damage_id.clone(),
                    label: "Damage".into(),
                    reason: reason.clone(),
                });
            }
        }
    }
    let causes = super::causes::resolve_causes(participant, opponent, additional);
    if effective_hp_maximum(
        participant,
        block.adjusted_level.as_ref().and_then(Number::as_i64),
    )
    .is_none()
        && participant.conditions.iter().any(|condition| {
            condition_rule_for_key(condition.condition_key.as_deref())
                == Some(ConditionRule::Drained)
        })
    {
        block
            .unresolved_stats
            .push(StatisticTarget::HitPointMaximum);
    }
    let blinded = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Blinded);
    if blinded && blinded_penalty.is_none() {
        block.unresolved_stats.push(StatisticTarget::Perception);
    }
    let petrified = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Petrified);
    if let Some(cause) = causes
        .iter()
        .find(|cause| cause.rule == ConditionRule::Petrified)
    {
        for (target, value) in [
            (StatisticTarget::ArmorClass, 9),
            (StatisticTarget::Hardness, 8),
        ] {
            block.replacements.push(StatReplacement {
                target,
                value: Number::from(value),
                condition_id: cause.source.condition_id,
                condition_name: cause.source.name.clone(),
            });
        }
    }
    let perception = block
        .values
        .iter()
        .find(|v| v.target == StatisticTarget::Perception)
        .and_then(|v| v.adjusted_value.as_f64());
    for (sense, facts) in block.senses.iter_mut().zip(&block.sense_facts) {
        // This overview has no target or distance: show intrinsic availability.
        let available = if petrified
            || facts.available == Some(false)
            || (blinded && facts.visual)
            || (deafened && facts.auditory)
        {
            super::senses::Availability::Unavailable
        } else if facts.available == Some(true) {
            super::senses::Availability::Available
        } else {
            super::senses::Availability::NeedsContext
        };
        sense.available = match available {
            super::senses::Availability::Available => Some(true),
            super::senses::Availability::Unavailable => Some(false),
            super::senses::Availability::NeedsContext => None,
        };
        if available == super::senses::Availability::Unavailable {
            sense.perception = None;
            sense.reason = if petrified {
                Some("Petrified".into())
            } else if deafened && facts.auditory {
                Some("Deafened".into())
            } else {
                (blinded && facts.visual).then(|| "Blinded".into())
            };
        } else if blinded {
            let penalty = blinded_penalty;
            sense.perception = perception
                .zip(penalty)
                .and_then(|(value, _)| Number::from_f64(value));
        } else {
            sense.perception = perception.and_then(Number::from_f64);
        }
    }
    block.action_budget = Some(super::capabilities::action_budget(participant));
    block
}
fn condition_note(
    condition: &atlas_local_state::EncounterParticipantCondition,
    reason: &str,
) -> UnappliedEffectView {
    UnappliedEffectView {
        condition_id: Some(condition.condition_id),
        source: condition.name.clone(),
        label: condition.name.clone(),
        reason: reason.into(),
    }
}

fn stupefied_hint(
    participant: &EncounterParticipant,
    opponent: Option<&str>,
) -> Option<UnappliedEffectView> {
    let requirement = super::action_context::stupefied_casting_hint(participant, opponent)?;
    Some(UnappliedEffectView {
        condition_id: None,
        source: "condition:stupefied".into(),
        label: "Stupefied".into(),
        reason: match requirement {
            Ok(dc) => format!("Spellcasting requires a DC {dc} flat check."),
            Err(()) => {
                "Enter the Stupefied value to determine the spellcasting flat-check DC.".into()
            }
        },
    })
}

fn condition_effect_hint(
    _condition: &atlas_local_state::EncounterParticipantCondition,
    rule: ConditionRule,
    applied: bool,
) -> Option<String> {
    match rule {
        ConditionRule::Enfeebled | ConditionRule::Clumsy if applied => None,
        ConditionRule::Enfeebled | ConditionRule::Clumsy => {
            Some("Ability-based effects need a supported statistic.")
        }

        ConditionRule::Petrified => Some(
            "Bulk is doubled. Your body and mind are in stasis: you do not age or notice time passing.",
        ),
        ConditionRule::Prone => Some("Move by Crawling or Stand to end Prone."),
        ConditionRule::Frightened | ConditionRule::Sickened if !applied => {
            Some("Check and DC statistics are unavailable.")
        }
        ConditionRule::OffGuard if !applied => Some("AC is unavailable."),
        ConditionRule::Immobilized if !applied => {
            Some("Cannot use move actions.")
        }
        ConditionRule::Grabbed => Some("Off-Guard and Immobilized. Manipulate actions require a DC 5 flat check."),
        ConditionRule::Restrained => Some("Off-Guard and Immobilized. Only Escape or Force Open your bonds can have the attack or manipulate trait."),

        ConditionRule::Encumbered if !applied => {
            Some("Dexterity-based statistics or Speed are unavailable.")
        }
        _ => None,
    }.map(str::to_owned)
}

pub(in crate::encounters) fn add(n: &Number, delta: i64) -> Option<Number> {
    if let Some(i) = n.as_i64() {
        return i.checked_add(delta).map(Number::from);
    }
    if n.is_u64() {
        return None;
    }
    Number::from_f64(n.as_f64()? + delta as f64)
}
pub(in crate::encounters) fn apply(v: &mut StatValueView, m: Modifier) {
    apply_modifier(
        &mut v.adjusted_value,
        &mut v.modifiers,
        &mut v.suppressed_modifiers,
        m,
    );
}
fn apply_modifier(
    adjusted_value: &mut Number,
    modifiers: &mut Vec<Modifier>,
    suppressed_modifiers: &mut Vec<Modifier>,
    m: Modifier,
) {
    let same = modifiers.iter().position(|old| {
        old.modifier_type == m.modifier_type
            && super::modifiers::competes(m.modifier_type, old.value, m.value)
    });
    let delta = if let Some(i) = same {
        if !super::modifiers::stronger(m.value, modifiers[i].value) {
            suppressed_modifiers.push(m);
            return;
        }
        m.value - modifiers[i].value
    } else {
        m.value
    };
    if let Some(n) = add(adjusted_value, delta) {
        if let Some(i) = same {
            suppressed_modifiers.push(modifiers.remove(i));
        }
        *adjusted_value = n;
        modifiers.push(m);
    }
}

pub(in crate::encounters) fn participant_runtime_block(p: &EncounterParticipant) -> StatBlockView {
    StatBlockView {
        record_key: p.participant_key.clone(),
        title: p.display_name.clone(),
        level: None,
        adjusted_level: p.level_override.map(Number::from),
        values: vec![],
        replacements: vec![],
        unresolved_stats: vec![],
        senses: vec![],
        sense_facts: vec![],
        senses_known: false,
        contextual_modifiers: vec![],
        contextual_armor_class: vec![],
        speeds: vec![],
        action_budget: Some(super::capabilities::action_budget(p)),
        activities: vec![],
        unapplied_effects: super::causes::condition_causes(p, None)
            .into_iter()
            // Parent hints describe their maintained grants; do not duplicate child hints.
            .filter(|cause| {
                condition_rule_for_key(cause.source.condition_key.as_deref()) == Some(cause.rule)
            })
            .filter_map(|cause| {
                condition_effect_hint(cause.source, cause.rule, false)
                    .map(|reason| condition_note(cause.source, &reason))
            })
            .chain(stupefied_hint(p, None))
            .chain(
                p.conditions
                    .iter()
                    .filter(|condition| scope_applies(&condition.scope, None))
                    .filter(|condition| {
                        condition_rule_for_key(condition.condition_key.as_deref()).is_none()
                    })
                    .filter_map(|condition| {
                        condition
                            .note
                            .as_deref()
                            .map(|note| condition_note(condition, note))
                    }),
            )
            .collect(),
    }
}

/// General projections include only universal causes; targeted queries supply an opponent.
pub(in crate::encounters) fn scope_applies(
    scope: &atlas_local_state::ConditionScope,
    opponent: Option<&str>,
) -> bool {
    match scope {
        atlas_local_state::ConditionScope::Everyone => true,
        atlas_local_state::ConditionScope::SelectedParticipants { participant_keys } => {
            opponent.is_some_and(|opponent| participant_keys.iter().any(|key| key == opponent))
        }
    }
}

fn prepare_participant(
    participant: &EncounterParticipant,
    baseline: &StatBlockView,
    adjustment_applicable: bool,
) -> StatBlockView {
    let mut block = baseline.clone();
    if let Some(maximum) = participant.max_hp {
        if let Some(value) = block
            .values
            .iter_mut()
            .find(|value| value.target == StatisticTarget::HitPointMaximum)
        {
            value.base_value = maximum.into();
            value.adjusted_value = maximum.into();
            value.modifiers.clear();
            value.suppressed_modifiers.clear();
        } else {
            block.values.push(StatValueView {
                target: StatisticTarget::HitPointMaximum,
                label: "Maximum HP".into(),
                ability: None,
                base_value: maximum.into(),
                adjusted_value: maximum.into(),
                modifiers: vec![],
                suppressed_modifiers: vec![],
            });
        }
    } else {
        block
            .values
            .retain(|value| value.target != StatisticTarget::HitPointMaximum);
    }
    let variant = participant.participant_variant;
    let delta = match variant {
        ParticipantVariant::Normal => 0,
        ParticipantVariant::Elite => 2,
        ParticipantVariant::Weak => -2,
    };
    if adjustment_applicable
        && participant.variant_origin
            != atlas_local_state::ParticipantVariantOrigin::InheritedUnknown
    {
        if let Some(level) = block.level.as_ref().and_then(integer) {
            block.adjusted_level = Some(adjusted_npc_level(level, variant).into());
        }
        for value in &mut block.values {
            let amount = if value.target == StatisticTarget::HitPointMaximum {
                None
            } else if value.target == StatisticTarget::ArmorClass
                || value.target.save()
                || value.target == StatisticTarget::Perception
                || value.target.skill()
            {
                Some(delta)
            } else {
                None
            };
            if let Some(amount) = amount.filter(|v| *v != 0) {
                apply(
                    value,
                    Modifier {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: ModifierType::Adjustment,
                        value: amount,
                    },
                );
            }
        }
        for activity in &mut block.activities {
            for roll in &mut activity.rolls {
                if delta != 0 {
                    let m = Modifier {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: ModifierType::Adjustment,
                        value: delta,
                    };
                    if let Some(n) = add(&roll.adjusted_value, delta) {
                        roll.adjusted_value = n;
                        roll.modifiers.push(m);
                    }
                }
            }
            if delta != 0
                && let Some(dc) = &mut activity.difficulty_class
            {
                apply(
                    dc,
                    Modifier {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: ModifierType::Adjustment,
                        value: delta,
                    },
                );
            }
            // Pinned strike and spell preparation adjust only authored component zero.
            // Do not promote a later component when the first is unavailable.
            let damage_delta = match activity.kind {
                MechanicActivityKindView::Strike => Some(delta),
                MechanicActivityKindView::Spell => activity.spell_usage.map(|usage| match usage {
                    SpellUsage::AtWill => delta,
                    SpellUsage::Limited => delta * 2,
                }),
                _ => None,
            };
            if delta != 0 && !activity.damage.is_empty() {
                if let Some(amount) = damage_delta
                    && let Some(damage) = activity.damage.first_mut().filter(|damage| {
                        damage.source_ordinal == 0
                            && damage.effect_kind == DamageEffectKindView::Damage
                    })
                {
                    damage.modifiers.push(Modifier {
                        source: "npc-adjustment".into(),
                        label: format!("{variant:?}"),
                        modifier_type: ModifierType::Adjustment,
                        value: amount,
                    });
                } else if !activity.damage.first().is_some_and(|damage| {
                    damage.source_ordinal == 0
                        && damage.effect_kind == DamageEffectKindView::Healing
                }) {
                    for damage in &mut activity.damage {
                        damage.unavailable_reason =
                            Some("Damage adjustment needs source context.".into());
                    }
                }
            }
        }
    } else if adjustment_applicable {
        block.unapplied_effects.push(UnappliedEffectView{condition_id:None,source:"npc-adjustment".into(),label:"Source adjustment unavailable".into(),reason:"Missing or invalid adjustment does not establish Normal; choose an explicit variant to apply an adjustment.".into()});
    }
    if let Some(level) = participant.level_override {
        block.adjusted_level = Some(level.into());
    } else if adjustment_applicable
        && participant.variant_origin
            == atlas_local_state::ParticipantVariantOrigin::InheritedUnknown
    {
        block.adjusted_level = None;
    }
    block
}

pub(in crate::encounters) fn project_participant(
    participant: &EncounterParticipant,
    baseline: &StatBlockView,
    adjustment_applicable: bool,
    additional: &[super::causes::ScopedConditionCause<'_>],
) -> StatBlockView {
    project_conditions(
        prepare_participant(participant, baseline, adjustment_applicable),
        participant,
        additional,
    )
}
/// A numerical consumer evaluates a known opponent without reading projected DTOs.
pub(in crate::encounters) fn project_participant_for_opponent(
    participant: &EncounterParticipant,
    baseline: &StatBlockView,
    adjustment_applicable: bool,
    additional: &[super::causes::ScopedConditionCause<'_>],
    opponent: &str,
) -> StatBlockView {
    project_for_opponent(
        prepare_participant(participant, baseline, adjustment_applicable),
        participant,
        Some(opponent),
        additional,
    )
}

/// Saved maximum HP is the pre-condition capacity; source/variant setup already
/// established it. Reads never charge Drained's current-HP consequence.
pub(in crate::encounters) fn effective_hp_maximum(
    participant: &EncounterParticipant,
    level: Option<i64>,
) -> Option<i64> {
    let level = participant.level_override.or(level);
    let baseline = participant.max_hp?;
    let mut penalty = 0;
    for cause in super::causes::condition_causes(participant, None)
        .into_iter()
        .filter(|cause| cause.rule == ConditionRule::Drained)
    {
        let amount = cause.value.filter(|value| *value >= 0)?;
        penalty = penalty.max(drained_hp_penalty(amount, level)?);
    }
    baseline.checked_sub(penalty)
}
fn drained_hp_penalty(amount: i64, level: Option<i64>) -> Option<i64> {
    if amount == 0 {
        return Some(0);
    }
    level?.max(1).checked_mul(amount)
}

/// Deafened penalizes Perception initiative even when another sense can perceive.
pub(in crate::encounters) fn project_initiative(
    mut block: StatBlockView,
    participant: &EncounterParticipant,
) -> StatBlockView {
    for cause in super::causes::condition_causes(participant, None)
        .into_iter()
        .filter(|cause| cause.rule == ConditionRule::Deafened)
    {
        if let Some(value) = block
            .values
            .iter_mut()
            .find(|value| value.target == StatisticTarget::Perception)
        {
            apply(
                value,
                Modifier {
                    source: format!("condition:{}", cause.source.condition_id),
                    label: cause.source.name.clone(),
                    modifier_type: ModifierType::Status,
                    value: -2,
                },
            );
        }
    }
    block
}

#[cfg(test)]
mod modifier_tests {
    use super::*;
    fn modifier(value: i64) -> Modifier {
        Modifier {
            source: value.to_string(),
            label: value.to_string(),
            modifier_type: ModifierType::Status,
            value,
        }
    }
    #[test]
    fn strongest_bonus_and_penalty_both_apply_in_either_order() {
        for values in [[2, -3, 1, -1], [-1, 1, -3, 2]] {
            let mut value = Number::from(10);
            let mut applied = vec![];
            let mut suppressed = vec![];
            for amount in values {
                apply_modifier(&mut value, &mut applied, &mut suppressed, modifier(amount));
            }
            assert_eq!(value, Number::from(9));
            assert_eq!(applied.len(), 2);
            assert_eq!(suppressed.len(), 2);
        }
    }
    #[test]
    fn selected_opponent_does_not_change_general_projection() {
        let scope = atlas_local_state::ConditionScope::SelectedParticipants {
            participant_keys: vec!["a".into()],
        };
        assert!(!scope_applies(&scope, None));
        assert!(!scope_applies(&scope, Some("b")));
        assert!(scope_applies(&scope, Some("a")));
    }
}
