//! Pure condition arrival/change consequences. Reading condition state never runs these.
use super::super::conditions::{ConditionRule, condition_rule_for_key, modeled_condition_by_rule};
use crate::{AppServiceError, AppServiceResult};
use atlas_local_state::{
    AddEncounterParticipantCondition, ConditionEventChange, ConditionOrigin, ConditionParameters,
    EncounterParticipant, PendingConsequence, PreparedConditionEvent, SavedPendingConsequence,
};

pub(in crate::encounters) struct ConditionConsequences {
    pub changes: PreparedConditionEvent,
    pub participant_key: String,
    pub effects: Vec<ConditionConsequence>,
    pub hp_change: Option<atlas_app_model::ConditionHpChangeView>,
}

pub(in crate::encounters) enum ConditionConsequence {
    DirectHpLoss(i64),
    StunnedActionsLost {
        before: i64,
        after: i64,
        stunned_after: Option<i64>,
    },
    BecomeProne,
    DropHeldItems,
    DrainedLevelRequired(i64),
}

pub(in crate::encounters) fn plan_condition_change(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    change: ConditionEventChange,
    apply_consequences: bool,
    keep_upright: bool,
    level: Option<i64>,
    request_id: &str,
) -> AppServiceResult<ConditionConsequences> {
    plan_condition_transition(
        participant,
        sources,
        PreparedConditionEvent {
            condition_changes: vec![change],
            ..Default::default()
        },
        apply_consequences,
        keep_upright,
        level,
        request_id,
    )
}

pub(in crate::encounters) fn plan_condition_transition(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
    mut changes: PreparedConditionEvent,
    apply_consequences: bool,
    keep_upright: bool,
    level: Option<i64>,
    request_id: &str,
) -> AppServiceResult<ConditionConsequences> {
    let after = subject_after_event(participant, &changes)?;
    let before_conditions = super::causes::condition_causes(participant, None, sources);
    let after_conditions = super::causes::condition_causes(&after, None, sources);
    let mut effects = vec![];
    if apply_consequences {
        if before_conditions.may_increase_from(&after_conditions, ConditionRule::Dying)
            && before_conditions.known_presence(ConditionRule::Dying)?
            && !after_conditions.known_presence(ConditionRule::Dying)?
            && participant.mortality != atlas_local_state::MortalityState::Dead
        {
            let wounded = super::causes::condition_causes(&after, None, sources)
                .known_value(ConditionRule::Wounded)?
                .checked_add(1)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Wounded value exceeds the supported range")
                })?;
            super::hp_events::set_counter(
                &after,
                ConditionRule::Wounded,
                wounded,
                request_id,
                &mut changes,
            )?;
            if participant.current_hp == Some(0)
                && !after_conditions.known_presence(ConditionRule::Unconscious)?
            {
                super::hp_events::add_condition(
                    &after,
                    ConditionRule::Unconscious,
                    None,
                    request_id,
                    "stable-unconscious",
                    &mut changes,
                )?;
            }
        }
        let after = subject_after_event(participant, &changes)?;
        let after_conditions = super::causes::condition_causes(&after, None, sources);
        let became_unconscious =
            if after_conditions.may_increase_from(&before_conditions, ConditionRule::Unconscious) {
                after_conditions.known_presence(ConditionRule::Unconscious)?
                    && !before_conditions.known_presence(ConditionRule::Unconscious)?
            } else {
                false
            };
        if became_unconscious {
            effects.insert(0, ConditionConsequence::DropHeldItems);
        }
        if !keep_upright
            && super::causes::application_grants(ConditionRule::Unconscious, ConditionRule::Prone)
            && became_unconscious
            && !after_conditions.known_presence(ConditionRule::Prone)?
        {
            let prone = modeled_condition_by_rule(ConditionRule::Prone).ok_or_else(|| {
                AppServiceError::invalid_request("Prone definition is unavailable")
            })?;
            changes.condition_changes.push(ConditionEventChange::Add(
                AddEncounterParticipantCondition {
                    participant_key: participant.participant_key.clone(),
                    condition_key: Some(prone.condition_ref.into()),
                    name: prone.name.into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: ConditionParameters::None,
                    origin: Some(ConditionOrigin {
                        application_id: request_id.into(),
                        grant_slot: "unconscious-prone".into(),
                        participant_key: None,
                        record_key: None,
                        activity_id: None,
                        artifact_generation: None,
                    }),
                    duration: None,
                    note: None,
                },
            ));
            effects.push(ConditionConsequence::BecomeProne);
        }
        let after = subject_after_event(participant, &changes)?;
        let after_conditions = super::causes::condition_causes(&after, None, sources);
        let checks_mortality = [ConditionRule::Dying, ConditionRule::Doomed]
            .into_iter()
            .any(|rule| after_conditions.may_increase_from(&before_conditions, rule));
        let changes_drained =
            after_conditions.may_increase_from(&before_conditions, ConditionRule::Drained);
        let increase = if changes_drained {
            effective_drained_value(&after, sources)?
                .saturating_sub(effective_drained_value(participant, sources)?)
                .max(0)
        } else {
            0
        };
        if increase > 0 {
            match (level, participant.current_hp) {
                (Some(level), Some(_)) => {
                    let amount = super::condition_effects::hp_loss(
                        ConditionRule::Drained,
                        increase,
                        Some(level),
                    )
                    .ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Drained HP loss exceeds the supported range",
                        )
                    })?;
                    let hp_changes = super::hp_events::plan_hp_event(
                        &after,
                        sources,
                        &crate::encounters::rules::hp_events::HpEventInput::DirectLoss { amount },
                        &Default::default(),
                        None,
                        request_id,
                    )?;
                    changes
                        .participant_changes
                        .extend(hp_changes.participant_changes);
                    changes
                        .condition_changes
                        .extend(hp_changes.condition_changes);
                    changes.pending.extend(hp_changes.pending);
                    effects.push(ConditionConsequence::DirectHpLoss(amount));
                }
                _ => {
                    effects.push(ConditionConsequence::DrainedLevelRequired(increase));
                    changes.pending.push(SavedPendingConsequence {
                        consequence_key: format!("{request_id}:drained-hp"),
                        consequence: PendingConsequence::DrainedHpLoss {
                            participant_key: participant.participant_key.clone(),
                            magnitude_increase: increase,
                        },
                    });
                }
            }
        }
        let threshold = broken_threshold(&after);
        if participant
            .current_hp
            .zip(threshold)
            .is_some_and(|(hp, threshold)| hp > threshold)
            && (after_conditions.may_increase_from(&before_conditions, ConditionRule::Broken)
                || broken_threshold(participant) != threshold)
            && after_conditions.known_presence(ConditionRule::Broken)?
            && !after_conditions.known_presence(ConditionRule::Petrified)?
            && let (Some(hp), Some(threshold)) = (participant.current_hp, threshold)
        {
            let mut update = changes.participant_changes.pop().unwrap_or_else(|| {
                atlas_local_state::ParticipantEventChange::preserving(participant)
            });
            update.current_hp = Some(update.current_hp.unwrap_or(hp).min(threshold));
            effects.push(ConditionConsequence::DirectHpLoss(hp - threshold));
            changes.participant_changes.push(update);
        }
        if checks_mortality {
            // Either known counter can prove death without demanding an unrelated
            // unknown counter. Otherwise their combined threshold needs both values.
            let independently_fatal = after_conditions.active.iter().any(|cause| {
                matches!(cause.rule, ConditionRule::Dying | ConditionRule::Doomed)
                    && cause
                        .value
                        .is_some_and(|value| value >= participant.base_death_threshold)
            });
            let fatal = if independently_fatal {
                true
            } else {
                let dying = after_conditions.known_value(ConditionRule::Dying)?;
                after_conditions.known_at_least(
                    ConditionRule::Doomed,
                    participant.base_death_threshold.saturating_sub(dying),
                )?
            };
            if fatal {
                let mut update = changes.participant_changes.pop().unwrap_or_else(|| {
                    atlas_local_state::ParticipantEventChange::preserving(participant)
                });
                update.mortality = atlas_local_state::MortalityState::Dead;
                changes.participant_changes.push(update);
                super::hp_events::clear(participant, ConditionRule::Dying, &mut changes);
                // Death removes Doomed, including the grant currently being applied.
                changes.condition_changes.retain(|change| match change {
                    ConditionEventChange::Add(input) => !matches!(
                        condition_rule_for_key(input.condition_key.as_deref()),
                        Some(ConditionRule::Doomed | ConditionRule::Dying)
                    ),
                    ConditionEventChange::Replace(input) => !matches!(
                        condition_rule_for_key(input.condition_key.as_deref()),
                        Some(ConditionRule::Doomed | ConditionRule::Dying)
                    ),
                    ConditionEventChange::Remove { .. } => true,
                });
                super::hp_events::clear(participant, ConditionRule::Doomed, &mut changes);
            }
        }
    }
    let after = subject_after_event(participant, &changes)?;
    let maximum_before = super::effective_hp_maximum(participant, sources, level);
    let maximum_after = super::effective_hp_maximum(&after, sources, level);
    let hp_change = (maximum_before != maximum_after
        || effects.iter().any(|effect| {
            matches!(
                effect,
                ConditionConsequence::DirectHpLoss(_)
                    | ConditionConsequence::DrainedLevelRequired(_)
            )
        }))
    .then(|| atlas_app_model::ConditionHpChangeView {
        current_before: participant.current_hp,
        current_after: changes
            .participant_changes
            .iter()
            .find(|change| change.participant_key == participant.participant_key)
            .map_or(participant.current_hp, |change| change.current_hp),
        maximum_before,
        maximum_after,
    });
    Ok(ConditionConsequences {
        hp_change,
        changes,
        participant_key: participant.participant_key.clone(),
        effects,
    })
}
pub(in crate::encounters) fn effective_drained_value(
    participant: &EncounterParticipant,
    sources: &super::source_conditions::SourceConditions,
) -> AppServiceResult<i64> {
    super::causes::condition_causes(participant, None, sources).known_value(ConditionRule::Drained)
}

fn broken_threshold(participant: &EncounterParticipant) -> Option<i64> {
    participant
        .conditions
        .iter()
        .filter(|condition| {
            condition_rule_for_key(condition.condition_key.as_deref())
                == Some(ConditionRule::Broken)
        })
        .find_map(|condition| match condition.parameters {
            ConditionParameters::Broken { threshold } => threshold,
            _ => None,
        })
}

pub(in crate::encounters) fn subject_after_event(
    participant: &EncounterParticipant,
    changes: &PreparedConditionEvent,
) -> AppServiceResult<EncounterParticipant> {
    let mut after = subject_after_changes(participant, &changes.condition_changes)?;
    for change in &changes.ability_states {
        after.ability_states.retain(|prior| {
            prior.owner != change.state.owner || prior.rule_ordinal != change.state.rule_ordinal
        });
        after.ability_states.push(change.state.clone());
    }
    Ok(after)
}

pub(in crate::encounters) fn subject_after_change(
    participant: &EncounterParticipant,
    change: &ConditionEventChange,
) -> AppServiceResult<EncounterParticipant> {
    subject_after_changes(participant, std::slice::from_ref(change))
}

pub(in crate::encounters) fn subject_after_changes(
    participant: &EncounterParticipant,
    changes: &[ConditionEventChange],
) -> AppServiceResult<EncounterParticipant> {
    let mut after = participant.clone();
    for change in changes {
        match change {
            ConditionEventChange::Add(input) => {
                after
                    .conditions
                    .push(atlas_local_state::EncounterParticipantCondition {
                        condition_id: after
                            .conditions
                            .iter()
                            .map(|c| c.condition_id)
                            .min()
                            .unwrap_or(0)
                            .min(0)
                            - 1,
                        condition_key: input.condition_key.clone(),
                        name: input.name.clone(),
                        value: input.value,
                        source_participant_key: input.source_participant_key.clone(),
                        scope: input.scope.clone(),
                        extra_action: input.extra_action.clone(),
                        immunity_override: input.immunity_override,
                        parameters: input.parameters.clone(),
                        origin: input.origin.clone(),
                        duration: input.duration.clone(),
                        note: input.note.clone(),
                        created_at: String::new(),
                        updated_at: String::new(),
                    })
            }
            ConditionEventChange::Replace(input) => {
                let row = after
                    .conditions
                    .iter_mut()
                    .find(|c| c.condition_id == input.condition_id)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("condition no longer exists")
                    })?;
                row.name = input.name.clone();
                row.source_participant_key = input.source_participant_key.clone();
                row.scope = input.scope.clone();
                row.extra_action = input.extra_action.clone();
                row.immunity_override = input.immunity_override;
                row.origin = input.origin.clone();
                row.duration = input.duration.clone();
                row.note = input.note.clone();
                row.condition_key = input.condition_key.clone();
                row.value = input.value;
                row.parameters = input.parameters.clone();
            }
            ConditionEventChange::Remove { condition_id, .. } => {
                after.conditions.retain(|c| c.condition_id != *condition_id)
            }
        }
    }
    Ok(after)
}
