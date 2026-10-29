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
            block.unapplied_effects.push(condition_note(
                super::causes::ConditionSource::Saved(condition),
                note,
            ));
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
    let sources = block.source_conditions.clone();
    if let Some(note) = stupefied_hint(participant, opponent, &sources) {
        block.unapplied_effects.push(note);
    }
    let mut movement_restriction = None;
    let conditions = super::causes::resolve_causes(participant, opponent, additional, &sources);
    let deafened = conditions.known_presence(ConditionRule::Deafened).ok();
    let available_senses = block
        .sense_facts
        .iter()
        .copied()
        .map(|mut facts| {
            if facts.auditory && facts.available != Some(false) {
                facts.available = match deafened {
                    Some(true) => Some(false),
                    Some(false) => facts.available,
                    None => None,
                };
            }
            facts
        })
        .collect::<Vec<_>>();
    let blinded_applies = block
        .senses_known
        .then(|| super::senses::blinded_penalty_applies(&available_senses, None))
        .flatten();
    for cause in super::causes::resolve_causes(participant, opponent, additional, &sources).active {
        let c = cause.source;
        let rule = cause.rule;
        use super::condition_effects::{Context, Target, modifiers};
        let level = block.adjusted_level.as_ref().and_then(Number::as_i64);
        if rule.definition().modifiers.iter().any(|effect| {
            !matches!(
                effect.value,
                atlas_foundry_model::generated::condition_definitions::ConditionValue::Fixed(_)
            )
        }) && cause.value.is_none_or(|value| value < 0)
        {
            block.unapplied_effects.push(condition_note(
                c,
                if c.saved().is_some() {
                    "Enter the condition value to calculate its penalties."
                } else {
                    "The source does not supply a condition value."
                },
            ));
            continue;
        }
        let mut applied = false;
        for stat in &mut block.values {
            if stat.target == StatisticTarget::HitPointMaximum {
                continue;
            }
            let context = Context::statistic(&stat.target, stat.ability);
            for (modifier_type, value) in modifiers(rule, cause.value, level, &context) {
                let value = if rule == ConditionRule::Blinded {
                    blinded_applies.and_then(|applies| if applies { value } else { Some(0) })
                } else {
                    value
                };
                if let Some(value) = value {
                    apply_modifier(
                        &mut stat.adjusted_value,
                        &mut stat.modifiers,
                        &mut stat.suppressed_modifiers,
                        Modifier {
                            source: c.explanation_key(),
                            label: c.name().to_owned(),
                            modifier_type,
                            value,
                        },
                    );
                    applied = true;
                } else if rule != ConditionRule::Blinded {
                    if !block.unresolved_stats.contains(&stat.target) {
                        block.unresolved_stats.push(stat.target.clone());
                    }
                    stat.notes.push(RuntimeEffectNoteView {
                        source: c.explanation_key(),
                        label: c.name().into(),
                        reason: "The statistic's attribute or traits are unavailable.".into(),
                    });
                }
            }
        }
        for activity in &mut block.activities {
            let context = Context {
                target: Target::Attack,
                ability: activity.attack_ability,
                initiative: false,
                auditory: activity
                    .traits_known
                    .then(|| activity.traits.iter().any(|t| t == "auditory")),
            };
            for (modifier_type, value) in modifiers(rule, cause.value, level, &context) {
                if let Some(value) = value {
                    for roll in &mut activity.rolls {
                        apply_modifier(
                            &mut roll.adjusted_value,
                            &mut roll.modifiers,
                            &mut roll.suppressed_modifiers,
                            Modifier {
                                source: c.explanation_key(),
                                label: c.name().to_owned(),
                                modifier_type,
                                value,
                            },
                        );
                        applied = true;
                    }
                } else if !activity.rolls.is_empty() {
                    activity.calculation_unavailable =
                        Some("The attack's attribute or traits are unavailable.".into());
                }
            }
            if let Some(dc) = &mut activity.difficulty_class {
                let context = Context::statistic(&dc.target, dc.ability);
                for (modifier_type, value) in modifiers(rule, cause.value, level, &context) {
                    if let Some(value) = value {
                        apply_modifier(
                            &mut dc.adjusted_value,
                            &mut dc.modifiers,
                            &mut dc.suppressed_modifiers,
                            Modifier {
                                source: c.explanation_key(),
                                label: c.name().to_owned(),
                                modifier_type,
                                value,
                            },
                        );
                        applied = true;
                    } else {
                        activity.calculation_unavailable =
                            Some("The DC's attribute or traits are unavailable.".into());
                    }
                }
            }
            for damage in &mut activity.damage {
                if damage.effect_kind != DamageEffectKindView::Damage {
                    continue;
                }
                let context = Context {
                    target: Target::Damage {
                        ability_known: damage.ability_known,
                    },
                    ability: damage.ability,
                    initiative: false,
                    auditory: Some(false),
                };
                for (modifier_type, value) in modifiers(rule, cause.value, level, &context) {
                    if let Some(value) = value {
                        let mut contribution =
                            Number::from(damage.modifiers.iter().map(|m| m.value).sum::<i64>());
                        apply_modifier(
                            &mut contribution,
                            &mut damage.modifiers,
                            &mut damage.suppressed_modifiers,
                            Modifier {
                                source: c.explanation_key(),
                                label: c.name().to_owned(),
                                modifier_type,
                                value,
                            },
                        );
                        applied = true;
                    } else {
                        damage.unavailable_reason =
                            Some("The damage's attribute contribution is unavailable.".into());
                    }
                }
            }
        }
        let context = Context {
            target: Target::Speed,
            ability: None,
            initiative: false,
            auditory: Some(false),
        };
        for (_, value) in modifiers(rule, cause.value, level, &context) {
            if let Some(value) = value {
                movement_restriction = Some((c, value));
                applied |= !block.speeds.is_empty();
            }
        }
        if let Some(reason) = condition_effect_hint(rule, applied) {
            block.unapplied_effects.push(condition_note(c, &reason));
        }
    }
    if let Some((condition, adjustment)) = movement_restriction {
        for speed in &mut block.speeds {
            let baseline = &speed.base_value_feet;
            let restricted = if let Some(n) = baseline.as_i64() {
                if n > 0 {
                    n.checked_add(adjustment).map(|n| Number::from(n.max(5)))
                } else {
                    Some(Number::from(n))
                }
            } else if baseline.is_u64() {
                None
            } else {
                baseline.as_f64().and_then(|n| {
                    Number::from_f64(if n > 0.0 {
                        (n + adjustment as f64).max(5.0)
                    } else {
                        n
                    })
                })
            };
            if let Some(restricted) = restricted {
                speed.adjusted_value_feet = restricted;
                speed.notes.push(RuntimeEffectNoteView {
                    source: condition.name().to_owned(),
                    label: condition.name().to_owned(),
                    reason: format!(
                        "Speed reduced by {} feet, to a minimum of 5 feet.",
                        adjustment.unsigned_abs()
                    ),
                });
            }
        }
    }
    for activity in &mut block.activities {
        if let Some(reason) = &activity.calculation_unavailable {
            activity.rolls.clear();
            activity.difficulty_class = None;
            if !activity.notes.iter().any(|note| &note.reason == reason) {
                activity.notes.push(RuntimeEffectNoteView {
                    source: "condition-calculation".into(),
                    label: "Calculation".into(),
                    reason: reason.clone(),
                });
            }
        }
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
    let causes = super::causes::resolve_causes(participant, opponent, additional, &sources).active;
    if let Some((maximum, uncertain)) = projected_hp_maximum(
        participant,
        &sources,
        block.adjusted_level.as_ref().and_then(Number::as_i64),
    ) && let Some(stat) = block
        .values
        .iter_mut()
        .find(|stat| stat.target == StatisticTarget::HitPointMaximum)
    {
        *stat = maximum;
        if uncertain
            && !block
                .unresolved_stats
                .contains(&StatisticTarget::HitPointMaximum)
        {
            block
                .unresolved_stats
                .push(StatisticTarget::HitPointMaximum);
        }
    }
    if effective_hp_maximum(
        participant,
        &sources,
        block.adjusted_level.as_ref().and_then(Number::as_i64),
    )
    .is_none()
        && causes
            .iter()
            .any(|cause| cause.rule == ConditionRule::Drained)
    {
        block
            .unresolved_stats
            .push(StatisticTarget::HitPointMaximum);
    }
    let blinded = conditions.known_presence(ConditionRule::Blinded).ok();
    if let Some(cause) = causes
        .iter()
        .find(|cause| cause.rule == ConditionRule::Blinded)
        && let Some(perception) = block
            .values
            .iter_mut()
            .find(|v| v.target == StatisticTarget::Perception)
    {
        // A conditional penalty is not a missing source statistic. Keep the known
        // value and explain the context instead of assuming a target or distance.
        perception.notes.push(RuntimeEffectNoteView {
            source: cause.source.explanation_key(),
            label: cause.source.name().to_owned(),
            reason: if blinded_applies.is_none() {
                "Vision-based Perception checks automatically critically fail. If vision is your only precise sense, you take a –4 status penalty to Perception checks."
            } else {
                "Vision-based Perception checks automatically critically fail."
            }.into(),
        });
    }
    let petrified = conditions.known_presence(ConditionRule::Petrified).ok();
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
                source: cause.source.explanation_key(),
                condition_name: cause.source.name().to_owned(),
            });
        }
    }
    project_uncertain_sources(
        &mut block,
        participant,
        opponent,
        additional,
        &sources,
        blinded_applies,
    );
    let perception = block
        .effective_stat(&StatisticTarget::Perception)
        .and_then(Number::as_f64);
    for (sense, facts) in block.senses.iter_mut().zip(&block.sense_facts) {
        // This overview has no target or distance: show intrinsic availability.
        let available = if petrified == Some(true)
            || facts.available == Some(false)
            || (blinded == Some(true) && facts.visual)
            || (deafened == Some(true) && facts.auditory)
        {
            super::senses::Availability::Unavailable
        } else if petrified.is_none()
            || blinded.is_none() && facts.visual
            || deafened.is_none() && facts.auditory
        {
            super::senses::Availability::NeedsContext
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
            sense.reason = if petrified == Some(true) {
                Some("Petrified".into())
            } else if deafened == Some(true) && facts.auditory {
                Some("Deafened".into())
            } else {
                (blinded == Some(true) && facts.visual).then(|| "Blinded".into())
            };
        } else {
            sense.perception = perception.and_then(Number::from_f64);
            if available == super::senses::Availability::NeedsContext {
                sense.reason = Some("Check source conditions and sense availability.".into());
            }
        }
    }
    block.action_budget = Some(super::capabilities::action_budget(
        participant,
        &block.source_conditions,
    ));
    block
}

fn modifier_can_change(modifiers: &[Modifier], kind: ModifierType, value: Option<i64>) -> bool {
    let Some(value) = value else {
        return true;
    };
    value != 0
        && !modifiers.iter().any(|known| {
            known.modifier_type == kind
                && super::modifiers::competes(kind, known.value, value)
                && !super::modifiers::stronger(value, known.value)
        })
}

fn project_uncertain_sources(
    block: &mut StatBlockView,
    participant: &EncounterParticipant,
    opponent: Option<&str>,
    additional: &[super::causes::ScopedConditionCause<'_>],
    sources: &super::source_conditions::SourceConditions,
    blinded_applies: Option<bool>,
) {
    use super::condition_effects::{Context, Target, modifiers};
    let level = participant
        .level_override
        .or_else(|| block.adjusted_level.as_ref().and_then(Number::as_i64));
    for cause in super::causes::resolve_causes(participant, opponent, additional, sources).uncertain
    {
        let reason = if cause.rule.definition().has_value && cause.value.is_none() {
            "The source condition's value is unavailable."
        } else {
            "Condition activation needs context; check the source rule."
        };
        let note = RuntimeEffectNoteView {
            source: cause.source.explanation_key(),
            label: cause.source.name().into(),
            reason: reason.into(),
        };
        for stat in &mut block.values {
            if block
                .replacements
                .iter()
                .any(|replacement| replacement.target == stat.target)
            {
                continue;
            }
            if cause.rule == ConditionRule::Blinded
                && stat.target == StatisticTarget::Perception
                && blinded_applies != Some(true)
            {
                stat.notes.push(RuntimeEffectNoteView {
                    source: cause.source.explanation_key(),
                    label: cause.source.name().into(),
                    reason: "If Blinded applies, vision-based Perception checks automatically critically fail. If vision is your only precise sense, you take a –4 status penalty to Perception checks.".into(),
                });
                continue;
            }
            if modifiers(
                cause.rule,
                cause.value,
                level,
                &Context::statistic(&stat.target, stat.ability),
            )
            .any(|(kind, value)| modifier_can_change(&stat.modifiers, kind, value))
            {
                if !block.unresolved_stats.contains(&stat.target) {
                    block.unresolved_stats.push(stat.target.clone());
                }
                if !stat.notes.contains(&note) {
                    stat.notes.push(note.clone());
                }
            }
        }
        if cause.rule == ConditionRule::Petrified {
            for (target, replacement) in [
                (StatisticTarget::ArmorClass, 9),
                (StatisticTarget::Hardness, 8),
            ] {
                if block.effective_stat(&target).and_then(Number::as_i64) != Some(replacement)
                    && !block.unresolved_stats.contains(&target)
                {
                    block.unresolved_stats.push(target.clone());
                    if let Some(stat) = block.values.iter_mut().find(|stat| stat.target == target) {
                        stat.notes.push(note.clone());
                    }
                }
            }
        }
        for activity in &mut block.activities {
            let context = Context {
                target: Target::Attack,
                ability: activity.attack_ability,
                initiative: false,
                auditory: activity
                    .traits_known
                    .then(|| activity.traits.iter().any(|value| value == "auditory")),
            };
            let attack_uncertain = activity.rolls.iter().any(|roll| {
                modifiers(cause.rule, cause.value, level, &context)
                    .any(|(kind, value)| modifier_can_change(&roll.modifiers, kind, value))
            });
            let dc_uncertain = activity.difficulty_class.as_ref().is_some_and(|dc| {
                modifiers(
                    cause.rule,
                    cause.value,
                    level,
                    &Context::statistic(&dc.target, dc.ability),
                )
                .any(|(kind, value)| modifier_can_change(&dc.modifiers, kind, value))
            });
            if attack_uncertain || dc_uncertain {
                activity
                    .calculation_unavailable
                    .get_or_insert_with(|| reason.into());
                if !activity.notes.contains(&note) {
                    activity.notes.push(note.clone());
                }
            }
            for damage in &mut activity.damage {
                if damage.effect_kind == DamageEffectKindView::Damage
                    && modifiers(
                        cause.rule,
                        cause.value,
                        level,
                        &Context {
                            target: Target::Damage {
                                ability_known: damage.ability_known,
                            },
                            ability: damage.ability,
                            initiative: false,
                            auditory: Some(false),
                        },
                    )
                    .any(|(kind, value)| modifier_can_change(&damage.modifiers, kind, value))
                {
                    damage
                        .unavailable_reason
                        .get_or_insert_with(|| reason.into());
                    if !activity.notes.contains(&note) {
                        activity.notes.push(note.clone());
                    }
                }
            }
        }
    }
}
fn condition_note(
    condition: super::causes::ConditionSource<'_>,
    reason: &str,
) -> UnappliedEffectView {
    UnappliedEffectView {
        condition_id: condition.condition_id(),
        source: condition.explanation_key(),
        label: condition.name().to_owned(),
        reason: reason.into(),
    }
}

fn stupefied_hint(
    participant: &EncounterParticipant,
    opponent: Option<&str>,
    sources: &super::source_conditions::SourceConditions,
) -> Option<UnappliedEffectView> {
    let requirement =
        super::action_context::stupefied_casting_hint(participant, opponent, sources)?;
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

/// Resolve display subjects only where the encounter roster is available. Numerical
/// evaluation and action-specific checks do not need this presentation context.
pub(in crate::encounters) fn resolve_condition_subject_hints(
    block: &mut StatBlockView,
    participant: &EncounterParticipant,
    roster: &[EncounterParticipant],
) {
    for note in &mut block.unapplied_effects {
        let Some(condition) = participant
            .conditions
            .iter()
            .find(|condition| Some(condition.condition_id) == note.condition_id)
        else {
            continue;
        };
        let (subject, fleeing) = match &condition.parameters {
            atlas_local_state::ConditionParameters::Fascinated { subject } => (subject, false),
            atlas_local_state::ConditionParameters::Fleeing { subject } => (subject, true),
            _ => continue,
        };
        let subject = subject.as_ref().and_then(|subject| match subject {
            atlas_local_state::ConditionSubject::Participant { participant_key } => roster
                .iter()
                .find(|participant| &participant.participant_key == participant_key)
                .map(|participant| participant.display_name.as_str()),
            atlas_local_state::ConditionSubject::Other { description } => {
                Some(description.as_str())
            }
        });
        note.reason = if fleeing {
            fleeing_hint(subject)
        } else {
            fascination_hint(subject)
        };
    }
}

fn fleeing_hint(subject: Option<&str>) -> String {
    format!(
        "Spend your actions escaping {}. Can't Delay or Ready.",
        subject.unwrap_or("the feared subject")
    )
}

fn fascination_hint(subject: Option<&str>) -> String {
    format!(
        "Concentrate actions must relate to {}.",
        subject.unwrap_or("the subject of fascination")
    )
}

fn condition_effect_hint(rule: ConditionRule, applied: bool) -> Option<String> {
    match rule {
        ConditionRule::Petrified => Some(
            "Bulk is doubled. Your body and mind are in stasis: you do not age or notice time passing.",
        ),
        ConditionRule::Prone => Some("Move by Crawling or Stand to end Prone."),
        ConditionRule::Fatigued => Some("Can't use exploration activities while traveling."),
        ConditionRule::Fascinated => return Some(fascination_hint(None)),
        ConditionRule::Fleeing => return Some(fleeing_hint(None)),
        ConditionRule::Confused => Some("Randomly attacks with Strikes or offensive cantrips."),
        ConditionRule::Immobilized if !applied => {
            Some("Cannot use move actions.")
        }
        ConditionRule::Grabbed => Some("Off-Guard and Immobilized. Manipulate actions require a DC 5 flat check."),
        ConditionRule::Restrained => Some("You can't use any attack or manipulate actions except to attempt to Escape or Force Open your bonds."),

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
        source_conditions: Default::default(),
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
        contextual_armor_class: vec![],
        speeds: vec![],
        action_budget: Some(super::capabilities::action_budget(p, &Default::default())),
        activities: vec![],
        unapplied_effects: super::causes::condition_causes(
            p,
            None,
            crate::encounters::rules::source_conditions::SourceConditions::empty(),
        )
        .active
        .into_iter()
        // Parent hints describe their maintained grants; do not duplicate child hints.
        .filter(|cause| {
            cause
                .source
                .saved()
                .and_then(|source| condition_rule_for_key(source.condition_key.as_deref()))
                == Some(cause.rule)
        })
        .filter_map(|cause| {
            condition_effect_hint(cause.rule, false)
                .map(|reason| condition_note(cause.source, &reason))
        })
        .chain(stupefied_hint(
            p,
            None,
            super::source_conditions::SourceConditions::empty(),
        ))
        .chain(
            p.conditions
                .iter()
                .filter(|condition| scope_applies(&condition.scope, None))
                .filter(|condition| {
                    condition_rule_for_key(condition.condition_key.as_deref()).is_none()
                })
                .filter_map(|condition| {
                    condition.note.as_deref().map(|note| {
                        condition_note(super::causes::ConditionSource::Saved(condition), note)
                    })
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

/// The same effective level drives runtime statistics and planned encounter XP.
/// Retain nonintegral authored values for display; arithmetic consumers require integers.
pub(in crate::encounters) fn effective_participant_level(
    participant: &EncounterParticipant,
    authored: Option<&Number>,
    adjustment_applicable: bool,
) -> Option<Number> {
    effective_level(
        participant.level_override,
        participant.participant_variant,
        participant.variant_origin,
        authored,
        adjustment_applicable,
    )
}

pub(in crate::encounters) fn effective_level(
    level_override: Option<i64>,
    variant: atlas_local_state::ParticipantVariant,
    origin: atlas_local_state::ParticipantVariantOrigin,
    authored: Option<&Number>,
    adjustment_applicable: bool,
) -> Option<Number> {
    if let Some(level) = level_override {
        return Some(level.into());
    }
    if !adjustment_applicable {
        return authored.cloned();
    }
    if origin == atlas_local_state::ParticipantVariantOrigin::InheritedUnknown {
        return None;
    }
    authored
        .and_then(integer)
        .map(|level| Number::from(adjusted_npc_level(level, variant)))
        .or_else(|| authored.cloned())
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
                notes: vec![],
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
    block.adjusted_level =
        effective_participant_level(participant, baseline.level.as_ref(), adjustment_applicable);
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
/// Saved maximum HP is the pre-condition capacity; source/variant setup already
/// established it. Reads never charge Drained's current-HP consequence.
pub(in crate::encounters) fn effective_hp_maximum(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    level: Option<i64>,
) -> Option<i64> {
    hp_maximum_stat(participant, sources, level)?
        .adjusted_value
        .as_i64()
}

/// The same attributed maximum feeds saved HP presentation and healing limits.
pub(in crate::encounters) fn hp_maximum_stat(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    level: Option<i64>,
) -> Option<StatValueView> {
    let (stat, uncertain) = projected_hp_maximum(participant, sources, level)?;
    (!uncertain).then_some(stat)
}

fn projected_hp_maximum(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    level: Option<i64>,
) -> Option<(StatValueView, bool)> {
    use super::condition_effects::{Context, modifiers};
    let maximum = participant.max_hp?;
    let mut stat = StatValueView {
        target: StatisticTarget::HitPointMaximum,
        label: "Maximum HP".into(),
        ability: None,
        base_value: maximum.into(),
        adjusted_value: maximum.into(),
        modifiers: vec![],
        suppressed_modifiers: vec![],
        notes: vec![],
    };
    let context = Context::statistic(&stat.target, None);
    let conditions = super::causes::condition_causes(participant, None, sources);
    let mut uncertain = false;
    if let Err(error) = conditions.known_value(ConditionRule::Drained) {
        uncertain = true;
        stat.notes.push(RuntimeEffectNoteView {
            source: "source-conditions".into(),
            label: "Source conditions".into(),
            reason: error.into_app_error().message,
        });
    }
    for cause in conditions.active {
        for (modifier_type, value) in modifiers(
            cause.rule,
            cause.value,
            participant.level_override.or(level),
            &context,
        ) {
            let Some(value) = value else {
                uncertain = true;
                stat.notes.push(RuntimeEffectNoteView {
                    source: cause.source.explanation_key(),
                    label: cause.source.name().into(),
                    reason: "Level or condition value needed to calculate maximum HP.".into(),
                });
                continue;
            };
            apply_modifier(
                &mut stat.adjusted_value,
                &mut stat.modifiers,
                &mut stat.suppressed_modifiers,
                Modifier {
                    source: cause.source.explanation_key(),
                    label: cause.source.name().to_owned(),
                    modifier_type,
                    value,
                },
            );
        }
    }
    for cause in conditions.uncertain {
        if modifiers(
            cause.rule,
            cause.value,
            participant.level_override.or(level),
            &context,
        )
        .any(|(kind, value)| modifier_can_change(&stat.modifiers, kind, value))
        {
            uncertain = true;
            let note = RuntimeEffectNoteView {
                source: cause.source.explanation_key(),
                label: cause.source.name().into(),
                reason: "Condition value or activation needed to calculate maximum HP.".into(),
            };
            if !stat.notes.contains(&note) {
                stat.notes.push(note);
            }
        }
    }
    if stat.adjusted_value.as_i64()? < 0 {
        stat.notes.push(RuntimeEffectNoteView {
            source: "hp-minimum".into(),
            label: "Minimum".into(),
            reason: "0 HP".into(),
        });
        stat.adjusted_value = 0.into();
    }
    Some((stat, uncertain))
}

/// Deafened penalizes Perception initiative even when another sense can perceive.
pub(in crate::encounters) fn project_initiative(
    mut block: StatBlockView,
    participant: &EncounterParticipant,
) -> StatBlockView {
    let sources = block.source_conditions.clone();
    use super::condition_effects::{Context, modifiers};
    let target = StatisticTarget::Perception;
    let context = Context {
        initiative: true,
        ..Context::statistic(&target, None)
    };
    let level = block.adjusted_level.as_ref().and_then(Number::as_i64);
    let conditions = super::causes::condition_causes(participant, None, &sources);
    for cause in conditions.active {
        if !cause.rule.definition().modifiers.iter().any(|effect| {
            !matches!(
                effect.predicate,
                atlas_foundry_model::generated::condition_definitions::ConditionPredicate::All([])
            )
        }) {
            continue;
        }
        for (modifier_type, value) in modifiers(cause.rule, cause.value, level, &context) {
            if let (Some(value), Some(stat)) =
                (value, block.values.iter_mut().find(|v| v.target == target))
            {
                apply(
                    stat,
                    Modifier {
                        source: cause.source.explanation_key(),
                        label: cause.source.name().to_owned(),
                        modifier_type,
                        value,
                    },
                );
            }
        }
    }
    for cause in conditions.uncertain {
        if let Some(stat) = block.values.iter_mut().find(|stat| stat.target == target)
            && modifiers(cause.rule, cause.value, level, &context)
                .any(|(kind, value)| modifier_can_change(&stat.modifiers, kind, value))
        {
            if !block.unresolved_stats.contains(&target) {
                block.unresolved_stats.push(target.clone());
            }
            stat.notes.push(RuntimeEffectNoteView {
                source: cause.source.explanation_key(),
                label: cause.source.name().into(),
                reason: "Check whether this source condition changes initiative.".into(),
            });
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
        for values in [[2, -7, 5, -1], [-1, 5, -7, 2]] {
            let mut value = Number::from(10);
            let mut applied = vec![];
            let mut suppressed = vec![];
            for amount in values {
                apply_modifier(&mut value, &mut applied, &mut suppressed, modifier(amount));
            }
            assert_eq!(value, Number::from(8));
            applied.sort_by_key(|row| row.value);
            suppressed.sort_by_key(|row| row.value);
            assert_eq!(
                applied.iter().map(|row| row.value).collect::<Vec<_>>(),
                [-7, 5]
            );
            assert_eq!(
                suppressed.iter().map(|row| row.value).collect::<Vec<_>>(),
                [-1, 2]
            );
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
