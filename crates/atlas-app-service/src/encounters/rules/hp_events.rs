//! Pure HP and mortality consequences for amounts already resolved by the caller.
use super::super::conditions::{ConditionRule, condition_rule_for_key, modeled_condition_by_rule};
use crate::{AppServiceError, AppServiceResult};
#[derive(Debug, Clone, Copy)]
pub(in crate::encounters) enum HpEventInput {
    ResolvedDamage { amount: i64 },
    ResolvedHealing { amount: i64 },
    DirectLoss { amount: i64 },
}
use atlas_local_state::{
    ConditionEventChange, ConditionOrigin, ConditionParameters, DamageLethality,
    EncounterParticipant, EncounterParticipantCondition, MortalityState, ParticipantEventChange,
    PendingConsequence, PreparedConditionEvent, SavedHpEventContext, SavedPendingConsequence,
};

pub(in crate::encounters) fn plan_hp_event(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    input: &HpEventInput,
    context: &SavedHpEventContext,
    effective_maximum: Option<i64>,
    request_id: &str,
) -> AppServiceResult<PreparedConditionEvent> {
    let current = participant.current_hp.ok_or_else(|| {
        AppServiceError::invalid_request("Set current HP before recording this event")
    })?;
    let amount = match input {
        HpEventInput::ResolvedDamage { amount }
        | HpEventInput::ResolvedHealing { amount }
        | HpEventInput::DirectLoss { amount } => *amount,
    };
    if !(0..=9_007_199_254_740_991).contains(&amount) {
        return Err(AppServiceError::invalid_request(
            "HP amount must be a nonnegative safe integer",
        ));
    }
    if participant.temporary_hp < 0 {
        return Err(AppServiceError::invalid_request(
            "Correct temporary HP before recording this event",
        ));
    }
    let conditions = super::causes::condition_causes(participant, None, sources);

    let mut update = ParticipantEventChange::preserving(participant);
    let mut changes = PreparedConditionEvent::default();
    match input {
        HpEventInput::DirectLoss { .. } => {
            update.current_hp = Some(current.saturating_sub(amount).max(0));
            if current > 0 && update.current_hp == Some(0) {
                if conditions.known_presence(ConditionRule::Petrified)? {
                    update.mortality = MortalityState::Dead;
                    clear(participant, ConditionRule::Dying, &mut changes);
                    clear(participant, ConditionRule::Doomed, &mut changes);
                } else if participant.track_dying && participant.mortality != MortalityState::Dead {
                    changes.pending.push(SavedPendingConsequence {
                        consequence_key: format!("{request_id}:hp-lifecycle"),
                        consequence: PendingConsequence::HpLifecycle {
                            participant_key: participant.participant_key.clone(),
                            before_hp: current,
                            after_hp: 0,
                            damage_amount: 0,
                            was_dying: conditions.known_value(ConditionRule::Dying)?,
                            was_wounded: conditions.known_value(ConditionRule::Wounded)?,
                            was_doomed: conditions.known_value(ConditionRule::Doomed)?,
                            context: context.clone(),
                        },
                    });
                } else if participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
                {
                    update.hazard_disabled = true;
                } else if !participant.track_dying {
                    update.defeated = true;
                }
            }
        }
        HpEventInput::ResolvedHealing { .. } => {
            let healed = current.checked_add(amount).ok_or_else(|| {
                AppServiceError::invalid_request("Healing would overflow current HP")
            })?;
            update.current_hp =
                Some(effective_maximum.map_or(healed, |max| healed.min(max.max(0))));
            if amount > 0
                && update.current_hp.is_some_and(|hp| hp >= 1)
                && participant.mortality != MortalityState::Dead
                && !conditions.known_presence(ConditionRule::Petrified)?
            {
                if conditions.known_value(ConditionRule::Dying)? > 0 {
                    clear(participant, ConditionRule::Dying, &mut changes);
                    let after = super::events::subject_after_changes(
                        participant,
                        &changes.condition_changes,
                    )?;
                    if super::causes::condition_causes(&after, None, sources)
                        .known_value(ConditionRule::Dying)?
                        == 0
                    {
                        set_counter(
                            participant,
                            ConditionRule::Wounded,
                            super::causes::condition_causes(&after, None, sources)
                                .known_value(ConditionRule::Wounded)?
                                .checked_add(1)
                                .ok_or_else(|| {
                                    AppServiceError::invalid_request(
                                        "Wounded value exceeds the supported range",
                                    )
                                })?,
                            request_id,
                            &mut changes,
                        )?;
                    }
                }
                if context.natural_rest_healing != Some(true)
                    || conditions.known_value(ConditionRule::Dying)? > 0
                {
                    clear_normal_unconscious(participant, &mut changes);
                }
            }
        }
        HpEventInput::ResolvedDamage { .. } => {
            let temporary_damage = participant.temporary_hp.min(amount);
            update.temporary_hp -= temporary_damage;
            update.current_hp = Some(current.saturating_sub(amount - temporary_damage).max(0));
            let after = update.current_hp.unwrap_or(0);
            let petrified_changes_consequences = participant.mortality != MortalityState::Dead
                && (after == 0
                    || participant.track_dying
                    || participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
                        && context
                            .broken_threshold
                            .is_some_and(|threshold| after <= threshold)
                    || participant.conditions.iter().any(|row| {
                        matches!(
                            condition_rule_for_key(row.condition_key.as_deref()),
                            Some(ConditionRule::Unconscious | ConditionRule::Confused)
                        )
                    }));
            let petrified = if petrified_changes_consequences {
                conditions.known_presence(ConditionRule::Petrified)?
            } else {
                conditions
                    .active
                    .iter()
                    .any(|cause| cause.rule == ConditionRule::Petrified)
            };
            if petrified {
                if after == 0 {
                    update.mortality = MortalityState::Dead;
                    clear(participant, ConditionRule::Dying, &mut changes);
                    clear(participant, ConditionRule::Doomed, &mut changes);
                }
                changes.participant_changes.push(update);
                return Ok(changes);
            }
            if amount > 0
                && participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
                && !conditions.known_presence(ConditionRule::Broken)?
                && context
                    .broken_threshold
                    .is_some_and(|threshold| after <= threshold)
            {
                add_condition(
                    participant,
                    ConditionRule::Broken,
                    None,
                    request_id,
                    "damage-broken",
                    &mut changes,
                )?;
                if let Some(ConditionEventChange::Add(condition)) =
                    changes.condition_changes.last_mut()
                {
                    condition.parameters = ConditionParameters::Broken {
                        threshold: context.broken_threshold,
                    };
                }
            }
            if amount > 0
                && context.attack_or_spell_damage != Some(false)
                && participant.mortality != MortalityState::Dead
            {
                let condition_ids = participant
                    .conditions
                    .iter()
                    .filter(|condition| {
                        condition_rule_for_key(condition.condition_key.as_deref())
                            == Some(ConditionRule::Confused)
                    })
                    .map(|condition| condition.condition_id)
                    .collect::<Vec<_>>();
                if !condition_ids.is_empty() {
                    changes.pending.push(SavedPendingConsequence {
                        consequence_key: format!("{request_id}:confused-damage"),
                        consequence: PendingConsequence::ConfusedDamageRecovery {
                            participant_key: participant.participant_key.clone(),
                            condition_ids,
                            qualifying_damage: context.attack_or_spell_damage,
                        },
                    });
                }
            }
            if amount > 0 && after >= 1 && participant.mortality != MortalityState::Dead {
                clear_normal_unconscious(participant, &mut changes);
            }
            if participant.track_dying
                && participant.mortality != MortalityState::Dead
                && amount > 0
            {
                let dying = conditions.known_value(ConditionRule::Dying)?;
                if dying > 0 || after == 0 {
                    if lifecycle_needs_context(dying, context) {
                        changes.pending.push(SavedPendingConsequence {
                            consequence_key: format!("{request_id}:hp-lifecycle"),
                            consequence: PendingConsequence::HpLifecycle {
                                participant_key: participant.participant_key.clone(),
                                before_hp: current,
                                after_hp: after,
                                damage_amount: amount,
                                was_dying: dying,
                                was_wounded: conditions.known_value(ConditionRule::Wounded)?,
                                was_doomed: conditions.known_value(ConditionRule::Doomed)?,
                                context: context.clone(),
                            },
                        });
                    } else {
                        plan_damage_lifecycle(
                            participant,
                            sources,
                            context,
                            request_id,
                            &mut changes,
                            &mut update,
                            current > 0,
                        )?;
                    }
                }
            } else if after == 0 {
                if participant.participant_kind == atlas_local_state::ParticipantKind::Hazard {
                    update.hazard_disabled = true;
                } else if !participant.track_dying {
                    update.defeated = true;
                }
            }
        }
    }
    changes.participant_changes.push(update);
    Ok(changes)
}
pub(in crate::encounters) fn required_lifecycle_facts(
    dying: i64,
    context: &SavedHpEventContext,
) -> Vec<atlas_app_model::HpLifecycleRequiredFactView> {
    use atlas_app_model::HpLifecycleRequiredFactView as Fact;
    let mut required = vec![];
    if dying > 0 {
        if context.critical.is_none() {
            required.push(Fact::Critical);
        } else if context.critical == Some(true) && context.enemy_source.is_none() {
            required.push(Fact::EnemySource);
        }
    } else if context.lethality.is_none() {
        required.push(Fact::Lethality);
    } else if context.lethality == Some(DamageLethality::Lethal) && context.critical.is_none() {
        required.push(Fact::Critical);
    }
    required
}
fn lifecycle_needs_context(dying: i64, context: &SavedHpEventContext) -> bool {
    !required_lifecycle_facts(dying, context).is_empty()
}
fn plan_damage_lifecycle(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    context: &SavedHpEventContext,
    request_id: &str,
    changes: &mut PreparedConditionEvent,
    update: &mut ParticipantEventChange,
    relocate: bool,
) -> AppServiceResult<()> {
    let conditions = super::causes::condition_causes(participant, None, sources);
    let dying = conditions.known_value(ConditionRule::Dying)?;
    if relocate && context.keep_current_initiative != Some(true) {
        if let Some(source) = &context.responsible_participant_key {
            changes
                .initiative_moves
                .push(atlas_local_state::ReorderEncounterParticipant {
                    participant_key: participant.participant_key.clone(),
                    target_participant_key: source.clone(),
                    placement: atlas_local_state::ReorderPlacement::Before,
                });
        } else {
            changes.pending.push(SavedPendingConsequence {
                consequence_key: format!("{request_id}:initiative-placement"),
                consequence: PendingConsequence::InitiativePlacement {
                    context: context.clone(),
                    participant_key: participant.participant_key.clone(),
                    before_hp: participant.current_hp.unwrap_or(0),
                    after_hp: update.current_hp.unwrap_or(0),
                },
            });
        }
    }
    let next = if dying > 0 {
        dying.checked_add(
            if context.critical == Some(true) && context.enemy_source == Some(true) {
                2
            } else {
                1
            },
        )
    } else if context.lethality == Some(DamageLethality::Nonlethal) {
        None
    } else {
        (if context.critical == Some(true) {
            2_i64
        } else {
            1_i64
        })
        .checked_add(
            super::causes::condition_causes(participant, None, sources)
                .known_value(ConditionRule::Wounded)?,
        )
    };
    if next.is_none() && (dying > 0 || context.lethality == Some(DamageLethality::Lethal)) {
        return Err(AppServiceError::invalid_request(
            "Dying value exceeds the supported range",
        ));
    }
    let base = context
        .base_death_threshold
        .unwrap_or(participant.base_death_threshold);
    if base < 1 {
        return Err(AppServiceError::invalid_request(
            "base death threshold must be positive",
        ));
    }
    let threshold = base.saturating_sub(
        super::causes::condition_causes(participant, None, sources)
            .known_value(ConditionRule::Doomed)?,
    );
    if next.is_some_and(|value| value >= threshold) || threshold <= 0 {
        update.mortality = MortalityState::Dead;
        clear(participant, ConditionRule::Dying, changes);
        clear(participant, ConditionRule::Doomed, changes);
    } else if let Some(value) = next {
        set_counter(
            participant,
            ConditionRule::Dying,
            value,
            request_id,
            changes,
        )?;
    } else if !conditions.known_presence(ConditionRule::Unconscious)? {
        add_condition(
            participant,
            ConditionRule::Unconscious,
            None,
            request_id,
            "knockout-unconscious",
            changes,
        )?;
    }
    if context.keep_upright != Some(true)
        && super::causes::application_grants(ConditionRule::Unconscious, ConditionRule::Prone)
        && !conditions.known_presence(ConditionRule::Prone)?
    {
        add_condition(
            participant,
            ConditionRule::Prone,
            None,
            request_id,
            "knockout-prone",
            changes,
        )?;
    }
    Ok(())
}
pub(in crate::encounters) fn merge_pending_context(
    saved: &SavedHpEventContext,
    incoming: &SavedHpEventContext,
) -> AppServiceResult<SavedHpEventContext> {
    fn fill<T: Clone + PartialEq>(
        saved: &Option<T>,
        incoming: &Option<T>,
        name: &str,
    ) -> AppServiceResult<Option<T>> {
        if saved.is_some() && incoming.is_some() && saved != incoming {
            return Err(AppServiceError::invalid_request(format!(
                "The recorded {name} cannot be changed by applying this pending event"
            )));
        }
        Ok(saved.clone().or_else(|| incoming.clone()))
    }
    Ok(SavedHpEventContext {
        lethality: fill(&saved.lethality, &incoming.lethality, "damage lethality")?,
        critical: fill(&saved.critical, &incoming.critical, "critical result")?,
        enemy_source: fill(&saved.enemy_source, &incoming.enemy_source, "enemy source")?,
        responsible_participant_key: incoming
            .responsible_participant_key
            .clone()
            .or_else(|| saved.responsible_participant_key.clone()),
        base_death_threshold: fill(
            &saved.base_death_threshold,
            &incoming.base_death_threshold,
            "death threshold",
        )?,
        keep_current_initiative: incoming
            .keep_current_initiative
            .or(saved.keep_current_initiative),
        keep_upright: fill(
            &saved.keep_upright,
            &incoming.keep_upright,
            "upright decision",
        )?,
        natural_rest_healing: fill(
            &saved.natural_rest_healing,
            &incoming.natural_rest_healing,
            "healing qualification",
        )?,
        attack_or_spell_damage: fill(
            &saved.attack_or_spell_damage,
            &incoming.attack_or_spell_damage,
            "damage qualification",
        )?,
        broken_threshold: fill(
            &saved.broken_threshold,
            &incoming.broken_threshold,
            "Broken Threshold",
        )?,
    })
}
/// Pending HP decisions describe one recorded event, never a later knockout.
pub(in crate::encounters) fn pending_is_current(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    pending: &PendingConsequence,
) -> AppServiceResult<bool> {
    if !participant.track_dying || participant.mortality == MortalityState::Dead {
        return Ok(false);
    }
    match pending {
        PendingConsequence::HpLifecycle {
            after_hp,
            was_dying,
            was_wounded,
            was_doomed,
            ..
        } => {
            if participant.current_hp != Some(*after_hp) {
                return Ok(false);
            }
            let conditions = super::causes::condition_causes(participant, None, sources);
            for (rule, expected) in [
                (ConditionRule::Dying, *was_dying),
                (ConditionRule::Wounded, *was_wounded),
                (ConditionRule::Doomed, *was_doomed),
            ] {
                if conditions.known_value(rule)? != expected {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        PendingConsequence::InitiativePlacement { after_hp, .. } => {
            Ok(participant.current_hp == Some(*after_hp))
        }
        _ => Ok(true),
    }
}

pub(in crate::encounters) fn resolve_pending_lifecycle(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    pending: &PendingConsequence,
    context: &SavedHpEventContext,
    request_id: &str,
) -> AppServiceResult<PreparedConditionEvent> {
    let PendingConsequence::HpLifecycle {
        before_hp,
        was_dying,
        context: saved_context,
        ..
    } = pending
    else {
        return Err(AppServiceError::invalid_request(
            "selected consequence is not an HP lifecycle decision",
        ));
    };
    let merged_context = merge_pending_context(saved_context, context)?;
    let context = &merged_context;
    if !pending_is_current(participant, sources, pending)? {
        return Err(AppServiceError::invalid_request(
            "This knockout is no longer current. Refresh the encounter.",
        ));
    }
    if lifecycle_needs_context(*was_dying, context) {
        return Err(AppServiceError::invalid_request(
            "supply the damage facts needed for dying",
        ));
    }
    let mut changes = PreparedConditionEvent::default();
    let mut update = ParticipantEventChange::preserving(participant);
    let mut historical = participant.clone();
    historical.current_hp = Some(*before_hp);
    plan_damage_lifecycle(
        &historical,
        sources,
        context,
        request_id,
        &mut changes,
        &mut update,
        *before_hp > 0,
    )?;
    changes.participant_changes.push(update);
    Ok(changes)
}
/// Explicitly choosing dying rules at 0 HP never replays the damage event.
pub(in crate::encounters) fn plan_knockout(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    context: &SavedHpEventContext,
    request_id: &str,
) -> AppServiceResult<PreparedConditionEvent> {
    let mut subject = participant.clone();
    subject.track_dying = true;
    subject.mortality = MortalityState::Alive;
    let mut update = ParticipantEventChange::preserving(&subject);
    update.defeated = false;
    let mut changes = PreparedConditionEvent::default();
    if !participant.track_dying
        && super::causes::condition_causes(participant, None, sources)
            .known_value(ConditionRule::Dying)?
            > 0
    {
        changes.participant_changes.push(update);
        return Ok(changes);
    }
    if lifecycle_needs_context(
        super::causes::condition_causes(&subject, None, sources)
            .known_value(ConditionRule::Dying)?,
        context,
    ) {
        return Err(AppServiceError::invalid_request(
            "Choose a knockout outcome",
        ));
    }
    plan_damage_lifecycle(
        &subject,
        sources,
        context,
        request_id,
        &mut changes,
        &mut update,
        true,
    )?;
    if update.mortality == MortalityState::Dead {
        update.defeated = participant.defeated;
    }
    changes.participant_changes.push(update);
    Ok(changes)
}

#[derive(Default)]
pub(in crate::encounters) struct KnockoutOptions {
    pub options: Vec<atlas_app_model::KnockoutOptionView>,
    pub unavailable_reason: Option<String>,
}

pub(in crate::encounters) fn knockout_options(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    known: Option<&SavedHpEventContext>,
) -> KnockoutOptions {
    if participant.participant_kind == atlas_local_state::ParticipantKind::Hazard {
        return KnockoutOptions::default();
    }
    if !participant.track_dying
        && let Ok(value) = super::causes::condition_causes(participant, None, sources)
            .known_value(ConditionRule::Dying)
        && value > 0
    {
        return KnockoutOptions {
            options: vec![atlas_app_model::KnockoutOptionView {
                label: "Track existing Dying".into(),
                result: format!("Dying {value}"),
                context: atlas_app_model::HpEventContextView {
                    lethality: Some(atlas_app_model::DamageLethalityView::Lethal),
                    critical: Some(false),
                    keep_current_initiative: Some(true),
                    ..Default::default()
                },
            }],
            unavailable_reason: None,
        };
    }
    let mut unavailable_reason = None;
    let options = [
        ("Normal", false, false),
        ("Critical", true, false),
        ("Nonlethal", false, true),
        ("Critical (other)", true, false),
        ("Nonlethal critical", true, true),
        ("Nonlethal critical (other)", true, true),
    ]
    .into_iter()
    .filter_map(|(label, critical, nonlethal)| {
        let already_dying = match super::causes::condition_causes(participant, None, sources)
            .known_value(ConditionRule::Dying)
        {
            Ok(value) => value > 0,
            Err(error) => {
                unavailable_reason = Some(error.into_app_error().message);
                return None;
            }
        };
        if label.ends_with("(other)") && !already_dying {
            return None;
        }
        if nonlethal
            && critical
            && known.is_none_or(|facts| {
                facts.critical != Some(true)
                    && !(already_dying && facts.lethality == Some(DamageLethality::Nonlethal))
            })
        {
            return None;
        }
        let mut context = SavedHpEventContext {
            lethality: Some(if nonlethal {
                DamageLethality::Nonlethal
            } else {
                DamageLethality::Lethal
            }),
            critical: Some(critical),
            enemy_source: (critical && already_dying).then_some(!label.ends_with("(other)")),
            keep_current_initiative: Some(true),
            ..Default::default()
        };
        if let Some(known) = known {
            if known
                .lethality
                .is_some_and(|value| Some(value) != context.lethality)
                || known
                    .critical
                    .is_some_and(|value| Some(value) != context.critical)
                || (context.enemy_source.is_some()
                    && known
                        .enemy_source
                        .is_some_and(|value| Some(value) != context.enemy_source))
            {
                return None;
            }
            context.enemy_source = known.enemy_source.or(context.enemy_source);
            context.base_death_threshold = known.base_death_threshold;
            context.keep_upright = known.keep_upright;
        }
        let changes = match plan_knockout(participant, sources, &context, "knockout-preview") {
            Ok(changes) => changes,
            Err(error) => {
                unavailable_reason = Some(error.into_app_error().message);
                return None;
            }
        };
        let result = if changes
            .participant_changes
            .iter()
            .any(|change| change.mortality == MortalityState::Dead)
        {
            "Dead".to_owned()
        } else {
            changes
                .condition_changes
                .iter()
                .filter_map(|change| match change {
                    ConditionEventChange::Add(c)
                        if matches!(
                            condition_rule_for_key(c.condition_key.as_deref()),
                            Some(ConditionRule::Dying | ConditionRule::Unconscious)
                        ) =>
                    {
                        Some(match c.value {
                            Some(value) => format!("{} {value}", c.name),
                            None => c.name.clone(),
                        })
                    }
                    ConditionEventChange::Replace(c)
                        if matches!(
                            condition_rule_for_key(c.condition_key.as_deref()),
                            Some(ConditionRule::Dying | ConditionRule::Unconscious)
                        ) =>
                    {
                        Some(match c.value {
                            Some(value) => format!("{} {value}", c.name),
                            None => c.name.clone(),
                        })
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" · ")
        };
        Some(atlas_app_model::KnockoutOptionView {
            label: label.into(),
            result,
            context: crate::encounters::hp_events::hp_context_view(context),
        })
    })
    .collect::<Vec<_>>();
    KnockoutOptions {
        unavailable_reason: options.is_empty().then_some(unavailable_reason).flatten(),
        options,
    }
}

pub(in crate::encounters) fn has_saved_condition(
    participant: &EncounterParticipant,
    rule: ConditionRule,
) -> bool {
    participant
        .conditions
        .iter()
        .any(|c| condition_rule_for_key(c.condition_key.as_deref()) == Some(rule))
}
pub(super) fn clear(
    participant: &EncounterParticipant,
    rule: ConditionRule,
    changes: &mut PreparedConditionEvent,
) {
    for condition in &participant.conditions {
        if condition_rule_for_key(condition.condition_key.as_deref()) == Some(rule) {
            changes
                .condition_changes
                .push(ConditionEventChange::Remove {
                    participant_key: participant.participant_key.clone(),
                    condition_id: condition.condition_id,
                });
        }
    }
}
pub(super) fn clear_normal_unconscious(
    participant: &EncounterParticipant,
    changes: &mut PreparedConditionEvent,
) {
    for condition in &participant.conditions {
        if condition_rule_for_key(condition.condition_key.as_deref())
            == Some(ConditionRule::Unconscious)
            && !matches!(
                condition.parameters,
                ConditionParameters::Unconscious {
                    waking: atlas_local_state::WakingRule::SourceControlled
                }
            )
        {
            changes
                .condition_changes
                .push(ConditionEventChange::Remove {
                    participant_key: participant.participant_key.clone(),
                    condition_id: condition.condition_id,
                });
        }
    }
}
pub(super) fn set_counter(
    participant: &EncounterParticipant,
    rule: ConditionRule,
    value: i64,
    request_id: &str,
    changes: &mut PreparedConditionEvent,
) -> AppServiceResult<()> {
    let rows = participant
        .conditions
        .iter()
        .filter(|c| condition_rule_for_key(c.condition_key.as_deref()) == Some(rule))
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return add_condition(
            participant,
            rule,
            Some(value),
            request_id,
            if rule == ConditionRule::Dying {
                "dying-counter"
            } else {
                "wounded-counter"
            },
            changes,
        );
    }
    for condition in rows {
        changes
            .condition_changes
            .push(ConditionEventChange::Replace(replace_counter(
                participant,
                condition,
                value,
            )));
    }
    Ok(())
}
pub(super) fn replace_counter(
    participant: &EncounterParticipant,
    condition: &EncounterParticipantCondition,
    value: i64,
) -> atlas_local_state::UpdateEncounterParticipantCondition {
    atlas_local_state::UpdateEncounterParticipantCondition {
        expected_encounter_revision: 0,
        participant_key: participant.participant_key.clone(),
        condition_id: condition.condition_id,
        condition_key: condition.condition_key.clone(),
        name: condition.name.clone(),
        value: Some(value),
        source_participant_key: condition.source_participant_key.clone(),
        scope: condition.scope.clone(),
        extra_action: condition.extra_action.clone(),
        immunity_override: condition.immunity_override,
        parameters: condition.parameters.clone(),
        origin: condition.origin.clone(),
        duration: condition.duration.clone(),
        note: condition.note.clone(),
    }
}
pub(super) fn add_condition(
    participant: &EncounterParticipant,
    rule: ConditionRule,
    value: Option<i64>,
    request_id: &str,
    slot: &str,
    changes: &mut PreparedConditionEvent,
) -> AppServiceResult<()> {
    let condition = modeled_condition_by_rule(rule)
        .ok_or_else(|| AppServiceError::invalid_request("condition definition is unavailable"))?;
    changes.condition_changes.push(ConditionEventChange::Add(
        atlas_local_state::AddEncounterParticipantCondition {
            participant_key: participant.participant_key.clone(),
            condition_key: Some(condition.condition_ref.into()),
            name: condition.name.into(),
            value,
            source_participant_key: None,
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            parameters: if rule == ConditionRule::Unconscious {
                ConditionParameters::Unconscious {
                    waking: atlas_local_state::WakingRule::Normal,
                }
            } else {
                ConditionParameters::None
            },
            origin: Some(ConditionOrigin {
                application_id: request_id.into(),
                grant_slot: slot.into(),
                participant_key: None,
                record_key: None,
                activity_id: None,
                artifact_generation: None,
            }),
            duration: None,
            note: None,
        },
    ));
    Ok(())
}
