//! Pure condition arrival/change consequences. Reading condition state never runs these.
use super::super::conditions::{ConditionRule, condition_rule_for_key, modeled_condition_by_rule};
use crate::{AppServiceError, AppServiceResult};
use atlas_local_state::{
    AddEncounterParticipantCondition, ConditionEventChange, ConditionOrigin, ConditionParameters,
    EncounterParticipant, PendingConsequence, PreparedConditionEvent, SavedPendingConsequence,
};

pub(in crate::encounters) struct ConditionConsequences {
    pub changes: PreparedConditionEvent,
    pub hp_loss: Option<i64>,
    pub becomes_prone: bool,
    pub drop_held_items: bool,
}

pub(in crate::encounters) fn plan_condition_change(
    participant: &EncounterParticipant,
    change: ConditionEventChange,
    apply_consequences: bool,
    keep_upright: bool,
    level: Option<i64>,
    request_id: &str,
) -> AppServiceResult<ConditionConsequences> {
    let (prior_rule, prior_value, next_rule, next_value) = match &change {
        ConditionEventChange::Add(input) => (
            None,
            None,
            condition_rule_for_key(input.condition_key.as_deref()),
            input.value,
        ),
        ConditionEventChange::Replace(input) => {
            let prior = participant
                .conditions
                .iter()
                .find(|condition| condition.condition_id == input.condition_id)
                .ok_or_else(|| AppServiceError::invalid_request("condition no longer exists"))?;
            (
                condition_rule_for_key(prior.condition_key.as_deref()),
                prior.value,
                condition_rule_for_key(input.condition_key.as_deref()),
                input.value,
            )
        }
        ConditionEventChange::Remove { condition_id, .. } => {
            let prior = participant
                .conditions
                .iter()
                .find(|condition| condition.condition_id == *condition_id)
                .ok_or_else(|| AppServiceError::invalid_request("condition no longer exists"))?;
            (
                condition_rule_for_key(prior.condition_key.as_deref()),
                prior.value,
                None,
                None,
            )
        }
    };
    let magnitude_changed = prior_rule != next_rule || prior_value != next_value;
    let changes_drained = magnitude_changed && next_rule == Some(ConditionRule::Drained);
    let checks_mortality = magnitude_changed
        && matches!(
            next_rule,
            Some(ConditionRule::Dying | ConditionRule::Doomed)
        );
    let mut after = participant.clone();
    match &change {
        ConditionEventChange::Add(input) => {
            after
                .conditions
                .push(atlas_local_state::EncounterParticipantCondition {
                    condition_id: -1,
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
                .ok_or_else(|| AppServiceError::invalid_request("condition no longer exists"))?;
            row.condition_key = input.condition_key.clone();
            row.value = input.value;
            row.parameters = input.parameters.clone();
        }
        ConditionEventChange::Remove { condition_id, .. } => {
            after.conditions.retain(|c| c.condition_id != *condition_id)
        }
    }
    let mut changes = PreparedConditionEvent {
        condition_changes: vec![change],
        ..Default::default()
    };
    let mut hp_loss = None;
    let mut becomes_prone = false;
    let mut drop_held_items = false;
    if apply_consequences {
        let increase = if changes_drained {
            effective_drained_value(&after)?
                .saturating_sub(effective_drained_value(participant)?)
                .max(0)
        } else {
            0
        };
        if increase > 0 {
            match (level, participant.current_hp) {
                (Some(level), Some(_)) => {
                    let amount = level.max(1).checked_mul(increase).ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "Drained HP loss exceeds the supported range",
                        )
                    })?;
                    let hp_changes = super::hp_events::plan_hp_event(
                        participant,
                        &atlas_app_model::HpEventInputView::DirectLoss { amount },
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
                    hp_loss = Some(amount);
                }
                _ => changes.pending.push(SavedPendingConsequence {
                    consequence_key: format!("{request_id}:drained-hp"),
                    consequence: PendingConsequence::DrainedHpLoss {
                        participant_key: participant.participant_key.clone(),
                        magnitude_increase: increase,
                    },
                }),
            }
        }
        if !has_condition(participant, ConditionRule::Broken)
            && has_condition(&after, ConditionRule::Broken)
            && !has_condition(&after, ConditionRule::Petrified)
        {
            let threshold = after
                .conditions
                .iter()
                .filter(|condition| {
                    condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(ConditionRule::Broken)
                })
                .find_map(|condition| match condition.parameters {
                    ConditionParameters::Broken { threshold } => threshold,
                    _ => None,
                });
            match (participant.current_hp, threshold) {
                (Some(hp), Some(threshold)) => {
                    let mut update =
                        atlas_local_state::ParticipantEventChange::preserving(participant);
                    update.current_hp = Some(hp.min(threshold));
                    if hp > threshold {
                        hp_loss = Some(hp - threshold);
                    }
                    changes.participant_changes.push(update);
                }
                _ => changes.pending.push(SavedPendingConsequence {
                    consequence_key: format!("{request_id}:broken-threshold"),
                    consequence: PendingConsequence::BrokenHpThreshold {
                        participant_key: participant.participant_key.clone(),
                    },
                }),
            }
        }
        if has_condition(participant, ConditionRule::Dying)
            && !has_condition(&after, ConditionRule::Dying)
            && participant.mortality != atlas_local_state::MortalityState::Dead
        {
            let wounded = super::lifecycle::known_effective(&after, ConditionRule::Wounded)?
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
                && !has_condition(&after, ConditionRule::Unconscious)
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
        if checks_mortality {
            // Either known counter can prove death without demanding an unrelated
            // unknown counter. Otherwise their combined threshold needs both values.
            let independently_fatal = after.conditions.iter().any(|condition| {
                matches!(
                    condition_rule_for_key(condition.condition_key.as_deref()),
                    Some(ConditionRule::Dying | ConditionRule::Doomed)
                ) && condition
                    .value
                    .is_some_and(|value| value >= participant.base_death_threshold)
            });
            let fatal = if independently_fatal {
                true
            } else {
                let doomed = super::lifecycle::known_effective(&after, ConditionRule::Doomed)?;
                let dying = super::lifecycle::known_effective(&after, ConditionRule::Dying)?;
                doomed >= participant.base_death_threshold
                    || dying > 0 && dying >= participant.base_death_threshold.saturating_sub(doomed)
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
        if !is_unconscious(participant) && is_unconscious(&after) {
            drop_held_items = true;
        }
        if !keep_upright
            && !is_unconscious(participant)
            && is_unconscious(&after)
            && !has_condition(&after, ConditionRule::Prone)
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
            becomes_prone = true;
        }
    }
    Ok(ConditionConsequences {
        changes,
        hp_loss,
        becomes_prone,
        drop_held_items,
    })
}
fn effective_drained_value(participant: &EncounterParticipant) -> AppServiceResult<i64> {
    let mut maximum = 0;
    for condition in participant.conditions.iter().filter(|condition| {
        condition_rule_for_key(condition.condition_key.as_deref()) == Some(ConditionRule::Drained)
    }) {
        let value = condition.value.filter(|value| *value > 0).ok_or_else(|| {
            AppServiceError::invalid_request(
                "Set the existing Drained value before calculating its HP loss",
            )
        })?;
        maximum = maximum.max(value);
    }
    Ok(maximum)
}
fn has_condition(participant: &EncounterParticipant, rule: ConditionRule) -> bool {
    participant
        .conditions
        .iter()
        .any(|c| condition_rule_for_key(c.condition_key.as_deref()) == Some(rule))
}
fn is_unconscious(participant: &EncounterParticipant) -> bool {
    has_condition(participant, ConditionRule::Unconscious)
        || has_condition(participant, ConditionRule::Dying)
}
