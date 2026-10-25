//! Explicit, pure lifecycle consequences; source preparation and dice stay outside.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::hp_events::{add_condition, clear, has, replace_counter, set_counter};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::*;
use atlas_local_state::{
    ConditionEventChange, ConditionParameters, EncounterParticipant, EncounterParticipantCondition,
    FrightenedRecovery, MortalityState, ParticipantEventChange, PendingConsequence,
    PreparedConditionEvent, SavedPendingConsequence,
};

pub(in crate::encounters) fn prepare_lifecycle<'a>(
    participant: &'a EncounterParticipant,
    action: &ConditionLifecycleActionView,
    pending: Option<&SavedPendingConsequence>,
    request_id: &'a str,
    sources: &super::source_action_conditions::SourceActionConditions,
) -> AppServiceResult<PreparedLifecycle<'a>> {
    let mut changes = PreparedConditionEvent::default();
    let mut effect = None;
    match action {
        ConditionLifecycleActionView::DyingRecovery { .. } => {
            if !participant.track_dying
                || participant.mortality == MortalityState::Dead
                || participant.current_hp != Some(0)
            {
                return Err(AppServiceError::invalid_request(
                    "Dying recovery requires a living participant at 0 HP who tracks dying",
                ));
            }
            let dying = known_effective(participant, ConditionRule::Dying)?;
            if dying < 1 {
                return Err(AppServiceError::invalid_request("participant is not dying"));
            }
            known_effective(participant, ConditionRule::Doomed)?;
            effect = Some(CheckEffect::Dying { dying });
        }
        ConditionLifecycleActionView::Retch { condition_id, .. } => {
            let condition = source(participant, *condition_id, ConditionRule::Sickened)?;
            let value = positive_value(condition)?;
            effect = Some(CheckEffect::Retch { condition, value });
        }
        ConditionLifecycleActionView::FrightenedEndTurn => {
            let sources = participant
                .conditions
                .iter()
                .filter(|c| {
                    condition_rule_for_key(c.condition_key.as_deref())
                        == Some(ConditionRule::Frightened)
                })
                .collect::<Vec<_>>();
            if sources.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "participant is not frightened",
                ));
            }
            for condition in sources {
                let value = positive_value(condition)?;
                let minimum = match &condition.parameters {
                    ConditionParameters::Frightened {
                        recovery: FrightenedRecovery::Held,
                    } => continue,
                    ConditionParameters::Frightened {
                        recovery: FrightenedRecovery::EndTurn { minimum },
                    } => *minimum,
                    // Unknown recovery restrictions stay with the GM. Known sources
                    // still recover without requiring decisions about unrelated entries.
                    ConditionParameters::Frightened {
                        recovery: FrightenedRecovery::SourceDependent,
                    } => continue,
                    ConditionParameters::None => 0,
                    _ => {
                        return Err(AppServiceError::invalid_request(
                            "Frightened recovery parameters are invalid",
                        ));
                    }
                };
                reduce_source(
                    participant,
                    condition,
                    value.saturating_sub(1).max(minimum).min(value),
                    &mut changes,
                );
            }
        }
        ConditionLifecycleActionView::StunnedActionsLost {
            condition_id,
            actions_lost,
        } => {
            let selected = source(participant, *condition_id, ConditionRule::Stunned)?;
            positive_value(selected)?;
            let maximum = super::capabilities::stunned_resolution_loss(participant, sources)
                .ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "Set the missing condition values before applying Stunned recovery",
                    )
                })?;
            if maximum < 1 || *actions_lost != maximum {
                return Err(AppServiceError::invalid_request(
                    "Actions lost must match the available actions and current Stunned value",
                ));
            }
            if let Some(mut resources) = participant
                .turn_resources
                .clone()
                .filter(|r| r.pending_reason.is_none())
            {
                let extra_lost = resources.extra_remaining.min(*actions_lost);
                resources.extra_remaining -= extra_lost;
                resources.regular_remaining -= *actions_lost - extra_lost;
                for condition in &participant.conditions {
                    if condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(ConditionRule::Stunned)
                        && let Some(before) = condition.value
                    {
                        resources
                            .effects
                            .push(atlas_local_state::TurnResourceEffect {
                                condition_id: Some(condition.condition_id),
                                name: condition.name.clone(),
                                before: Some(before),
                                after: Some((before - *actions_lost).max(0)),
                                actions_lost: before.min(*actions_lost),
                            });
                    }
                }
                changes.turn_resources.push(resources);
            }
            // One overlapping action loss reduces every concurrently active numeric source.
            for condition in &participant.conditions {
                if condition_rule_for_key(condition.condition_key.as_deref())
                    == Some(ConditionRule::Stunned)
                    && condition.value.is_some()
                {
                    reduce_source(
                        participant,
                        condition,
                        positive_value(condition)?.saturating_sub(*actions_lost),
                        &mut changes,
                    );
                }
            }
        }
        ConditionLifecycleActionView::ConfusedDamageRecovery {
            consequence_key,
            check,
        } => {
            let saved = pending
                .filter(|p| &p.consequence_key == consequence_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("damage recovery opportunity no longer exists")
                })?;
            let PendingConsequence::ConfusedDamageRecovery {
                participant_key,
                condition_ids: eligible,
                qualifying_damage: known,
            } = &saved.consequence
            else {
                return Err(AppServiceError::invalid_request(
                    "selected consequence is not Confused damage recovery",
                ));
            };
            if participant_key != &participant.participant_key {
                return Err(AppServiceError::invalid_request(
                    "recovery belongs to another participant",
                ));
            }
            if check.is_some() {
                if *known == Some(false) {
                    return Err(AppServiceError::invalid_request(
                        "This damage does not allow Confused recovery",
                    ));
                }
                // Invoking the aid asserts the table trigger. Its source scope is
                // captured by the damage event, never selected by a recovery form.
                let condition_ids = eligible
                    .iter()
                    .copied()
                    .filter(|id| {
                        participant.conditions.iter().any(|condition| {
                            condition.condition_id == *id
                                && condition_rule_for_key(condition.condition_key.as_deref())
                                    == Some(ConditionRule::Confused)
                        })
                    })
                    .collect::<Vec<_>>();
                if condition_ids.is_empty() {
                    return Err(AppServiceError::invalid_request(
                        "Confused sources no longer exist",
                    ));
                }
                effect = Some(CheckEffect::Confused { condition_ids });
            }
            changes.resolved_pending_keys.push(consequence_key.clone());
        }
    }
    Ok(PreparedLifecycle {
        participant,
        request_id,
        changes,
        effect,
    })
}

pub(in crate::encounters) struct PreparedLifecycle<'a> {
    participant: &'a EncounterParticipant,
    request_id: &'a str,
    changes: PreparedConditionEvent,
    effect: Option<CheckEffect<'a>>,
}
enum CheckEffect<'a> {
    Dying {
        dying: i64,
    },
    Retch {
        condition: &'a EncounterParticipantCondition,
        value: i64,
    },
    Confused {
        condition_ids: Vec<i64>,
    },
}
impl PreparedLifecycle<'_> {
    pub fn resolve(
        self,
        degree: Option<CheckDegreeView>,
    ) -> AppServiceResult<PreparedConditionEvent> {
        let Self {
            participant,
            request_id,
            mut changes,
            effect,
        } = self;
        let Some(effect) = effect else {
            return Ok(changes);
        };
        let degree = degree
            .ok_or_else(|| AppServiceError::invalid_request("Record or roll the required check"))?;
        match effect {
            CheckEffect::Dying { dying } => {
                let delta = match degree {
                    CheckDegreeView::CriticalSuccess => -2,
                    CheckDegreeView::Success => -1,
                    CheckDegreeView::Failure => 1,
                    CheckDegreeView::CriticalFailure => 2,
                };
                let next = dying.checked_add(delta).ok_or_else(|| {
                    AppServiceError::invalid_request("Dying value exceeds the supported range")
                })?;
                let threshold = participant
                    .base_death_threshold
                    .saturating_sub(known_effective(participant, ConditionRule::Doomed)?);
                if next >= threshold || threshold <= 0 {
                    let mut update = ParticipantEventChange::preserving(participant);
                    update.mortality = MortalityState::Dead;
                    changes.participant_changes.push(update);
                    clear(participant, ConditionRule::Dying, &mut changes);
                    clear(participant, ConditionRule::Doomed, &mut changes);
                } else if next <= 0 {
                    clear(participant, ConditionRule::Dying, &mut changes);
                    set_counter(
                        participant,
                        ConditionRule::Wounded,
                        known_effective(participant, ConditionRule::Wounded)?
                            .checked_add(1)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "Wounded value exceeds the supported range",
                                )
                            })?,
                        request_id,
                        &mut changes,
                    )?;
                    if !participant.conditions.iter().any(|c| {
                        condition_rule_for_key(c.condition_key.as_deref())
                            == Some(ConditionRule::Unconscious)
                            && !matches!(
                                c.parameters,
                                ConditionParameters::Unconscious {
                                    waking: atlas_local_state::WakingRule::SourceControlled
                                }
                            )
                    }) {
                        add_condition(
                            participant,
                            ConditionRule::Unconscious,
                            None,
                            request_id,
                            "stable-unconscious",
                            &mut changes,
                        )?;
                    }
                } else {
                    set_counter(
                        participant,
                        ConditionRule::Dying,
                        next,
                        request_id,
                        &mut changes,
                    )?;
                }
            }
            CheckEffect::Retch { condition, value } => {
                let reduction = match degree {
                    CheckDegreeView::CriticalSuccess => 2,
                    CheckDegreeView::Success => 1,
                    _ => 0,
                };
                if reduction > 0 {
                    reduce_source(
                        participant,
                        condition,
                        value.saturating_sub(reduction),
                        &mut changes,
                    );
                }
            }
            CheckEffect::Confused { condition_ids } => {
                if matches!(
                    degree,
                    CheckDegreeView::Success | CheckDegreeView::CriticalSuccess
                ) {
                    for id in &condition_ids {
                        changes
                            .condition_changes
                            .push(ConditionEventChange::Remove {
                                participant_key: participant.participant_key.clone(),
                                condition_id: *id,
                            });
                    }
                }
            }
        }
        Ok(changes)
    }
}
pub(in crate::encounters) fn source(
    participant: &EncounterParticipant,
    id: i64,
    rule: ConditionRule,
) -> AppServiceResult<&EncounterParticipantCondition> {
    participant
        .conditions
        .iter()
        .find(|c| {
            c.condition_id == id && condition_rule_for_key(c.condition_key.as_deref()) == Some(rule)
        })
        .ok_or_else(|| {
            AppServiceError::invalid_request("selected condition source no longer exists")
        })
}
fn positive_value(condition: &EncounterParticipantCondition) -> AppServiceResult<i64> {
    condition.value.filter(|value| *value > 0).ok_or_else(|| {
        AppServiceError::invalid_request(
            "Set the condition's positive numeric value before resolving recovery",
        )
    })
}
pub(in crate::encounters) fn known_effective(
    participant: &EncounterParticipant,
    rule: ConditionRule,
) -> AppServiceResult<i64> {
    let mut value = 0;
    for condition in &participant.conditions {
        if condition_rule_for_key(condition.condition_key.as_deref()) == Some(rule) {
            value = value.max(positive_value(condition)?);
        }
    }
    Ok(value)
}
fn reduce_source(
    participant: &EncounterParticipant,
    condition: &EncounterParticipantCondition,
    value: i64,
    changes: &mut PreparedConditionEvent,
) {
    if value <= 0 {
        changes
            .condition_changes
            .push(ConditionEventChange::Remove {
                participant_key: participant.participant_key.clone(),
                condition_id: condition.condition_id,
            });
    } else if condition.value != Some(value) {
        changes
            .condition_changes
            .push(ConditionEventChange::Replace(replace_counter(
                participant,
                condition,
                value,
            )));
    }
}
pub(in crate::encounters) fn available_actions(
    participant: &EncounterParticipant,
    fortitude: Option<i64>,
    pending: &[SavedPendingConsequence],
    claims: &[atlas_local_state::EncounterBoundaryClaim],
    source_actions: &super::source_action_conditions::SourceActionConditions,
) -> Vec<ConditionLifecycleOptionView> {
    use ConditionRecoveryOfferView as Offer;
    let sources = |rule| {
        participant.conditions.iter().filter(move |condition| {
            condition_rule_for_key(condition.condition_key.as_deref()) == Some(rule)
        })
    };
    let mut actions = vec![];
    let mut add = |action, label: &str, scope: Option<&str>| {
        let claimed =
            scope.is_some_and(|scope| claims.iter().any(|claim| claim.scope_key == scope));
        actions.push(ConditionLifecycleOptionView {
            condition_ids: recovery_sources(&action, participant),
            action,
            label: label.into(),
            available: !claimed,
            unavailable_reason: claimed.then(|| "Already resolved this round".into()),
        });
    };
    if has(participant, ConditionRule::Dying)
        && participant.track_dying
        && participant.mortality == MortalityState::Alive
        && participant.current_hp == Some(0)
    {
        add(
            Offer::DyingRecovery {
                check: LifecycleCheckFactsView {
                    dc: known_effective(participant, ConditionRule::Dying)
                        .ok()
                        .and_then(|value| value.checked_add(10)),
                    modifier: Some(0),
                },
            },
            "Recovery check",
            Some("dying"),
        );
    }
    for condition in sources(ConditionRule::Sickened) {
        add(
            Offer::Retch {
                condition_id: condition.condition_id,
                check: LifecycleCheckFactsView {
                    dc: match condition.parameters {
                        ConditionParameters::Sickened { recovery_dc } => recovery_dc,
                        _ => None,
                    },
                    modifier: fortitude,
                },
                issues: vec![],
            },
            "Retch",
            None,
        );
    }
    if sources(ConditionRule::Frightened).any(|condition| {
        condition.value.is_some_and(|value| value > 0)
            && matches!(
                condition.parameters,
                ConditionParameters::None
                    | ConditionParameters::Frightened {
                        recovery: FrightenedRecovery::EndTurn { .. }
                    }
            )
    }) {
        add(
            Offer::FrightenedEndTurn,
            "Reduce Frightened by 1",
            Some("frightened"),
        );
    }
    for condition in &participant.conditions {
        let rule = condition_rule_for_key(condition.condition_key.as_deref());
        let text = match rule {
            Some(ConditionRule::Wounded) => {
                "Cleared by healing from Treat Wounds, or 10 minutes of rest at full HP."
            }
            Some(ConditionRule::Unconscious) => {
                if matches!(
                    condition.parameters,
                    ConditionParameters::Unconscious {
                        waking: atlas_local_state::WakingRule::SourceControlled
                    }
                ) {
                    "Waking depends on the effect causing unconsciousness."
                } else if participant.current_hp == Some(0) {
                    "At 0 HP, healing can wake you. If stable, recover 1 HP after at least 10 minutes; the GM determines the time."
                } else {
                    "Healing, damage, an Interact action, or finishing or interrupting sleep can wake you. Noise allows a Perception check against the noise's DC."
                }
            }
            Some(ConditionRule::Fatigued) => "Cleared by a full night's rest.",
            Some(ConditionRule::Drained) => {
                "Decreases by 1 after a full night's rest. This doesn't restore lost HP."
            }
            Some(ConditionRule::Doomed) => "Decreases by 1 after a full night's rest.",
            Some(ConditionRule::Frightened) => match condition.parameters {
                ConditionParameters::Frightened {
                    recovery: FrightenedRecovery::Held,
                } => "This effect prevents the usual end-of-turn reduction.",
                ConditionParameters::Frightened {
                    recovery: FrightenedRecovery::SourceDependent,
                } => {
                    "Check the causing effect for when Frightened can decrease and its minimum value."
                }
                _ => continue,
            },
            Some(ConditionRule::Broken)
                if !matches!(
                    condition.parameters,
                    ConditionParameters::Broken { threshold: Some(_) }
                ) =>
            {
                "Broken prevents the hazard from functioning. Applying Broken also reduces HP to its Broken Threshold; check its maximum HP if the threshold is unknown."
            }
            _ => continue,
        };
        add(
            Offer::Reminder {
                condition_id: condition.condition_id,
                text: text.into(),
            },
            "Recovery",
            None,
        );
    }
    for condition in sources(ConditionRule::Stunned).filter(|condition| condition.value.is_some()) {
        add(
            Offer::StunnedActionsLost {
                condition_id: condition.condition_id,
                actions_lost: super::capabilities::stunned_resolution_loss(
                    participant,
                    source_actions,
                ),
            },
            "Consume actions lost to stunned",
            Some("stunned-actions"),
        );
    }
    for saved in pending {
        match &saved.consequence {
            PendingConsequence::ConfusedDamageRecovery {
                participant_key,
                condition_ids,
                qualifying_damage,
            } if participant_key == &participant.participant_key
                && *qualifying_damage != Some(false)
                && condition_ids
                    .iter()
                    .any(|id| participant.conditions.iter().any(|c| c.condition_id == *id)) =>
            {
                add(
                    Offer::ConfusedDamageRecovery {
                        consequence_key: saved.consequence_key.clone(),
                        source_condition_ids: condition_ids.clone(),
                        qualifying_damage: *qualifying_damage,
                        check: LifecycleCheckFactsView {
                            dc: Some(11),
                            modifier: Some(0),
                        },
                    },
                    "Check to recover from confused",
                    None,
                );
            }
            _ => {}
        }
    }
    for option in &mut actions {
        if matches!(
            option.action,
            Offer::StunnedActionsLost {
                actions_lost: None | Some(0),
                ..
            }
        ) {
            option.available = false;
            option.unavailable_reason =
                Some("No actions can currently be lost to this Stunned effect.".into());
        }
    }
    actions
}

/// Associate recovery with saved sources without teaching the browser condition rules.
fn recovery_sources(
    offer: &ConditionRecoveryOfferView,
    participant: &EncounterParticipant,
) -> Vec<i64> {
    use ConditionRecoveryOfferView as Offer;
    let rule = match offer {
        Offer::Retch { condition_id, .. }
        | Offer::StunnedActionsLost { condition_id, .. }
        | Offer::Reminder { condition_id, .. } => {
            return vec![*condition_id];
        }
        Offer::ConfusedDamageRecovery {
            source_condition_ids,
            ..
        } => {
            return source_condition_ids.clone();
        }
        Offer::DyingRecovery { .. } => ConditionRule::Dying,
        Offer::FrightenedEndTurn => ConditionRule::Frightened,
    };
    participant
        .conditions
        .iter()
        .filter(|condition| {
            condition_rule_for_key(condition.condition_key.as_deref()) == Some(rule)
                && (rule != ConditionRule::Frightened
                    || matches!(
                        condition.parameters,
                        ConditionParameters::None
                            | ConditionParameters::Frightened {
                                recovery: FrightenedRecovery::EndTurn { .. }
                            }
                    ))
        })
        .map(|condition| condition.condition_id)
        .collect()
}

/// Match a fresh offer to its command without a second parallel action-kind enum.
pub(in crate::encounters) fn matches_action(
    offer: &ConditionRecoveryOfferView,
    action: &ConditionLifecycleActionView,
) -> bool {
    use ConditionLifecycleActionView as A;
    use ConditionRecoveryOfferView as O;
    match (offer, action) {
        (O::DyingRecovery { .. }, A::DyingRecovery { .. })
        | (O::FrightenedEndTurn, A::FrightenedEndTurn) => true,
        (
            O::Retch {
                condition_id: offered,
                ..
            },
            A::Retch { condition_id, .. },
        )
        | (
            O::StunnedActionsLost {
                condition_id: offered,
                ..
            },
            A::StunnedActionsLost { condition_id, .. },
        ) => offered == condition_id,
        (
            O::ConfusedDamageRecovery {
                consequence_key: offered,
                ..
            },
            A::ConfusedDamageRecovery {
                consequence_key, ..
            },
        ) => offered == consequence_key,
        _ => false,
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::super::numerical_tests::{condition, participant};
    use super::*;

    #[test]
    fn duration_stunned_shares_known_action_loss_with_numeric_sources() {
        for quickened in [false, true] {
            let mut actor = participant();
            let mut duration = condition(2, ConditionRule::Stunned, None);
            duration.duration = Some(atlas_local_state::ConditionDuration {
                amount: 1,
                unit: atlas_local_state::ConditionDurationUnit::Minutes,
                timing: None,
            });
            actor.conditions = vec![
                condition(1, ConditionRule::Stunned, Some(4)),
                duration,
                condition(3, ConditionRule::Slowed, Some(2)),
            ];
            if quickened {
                actor
                    .conditions
                    .push(condition(4, ConditionRule::Quickened, None));
            }
            let actions_lost = if quickened { 4 } else { 3 };
            assert_eq!(
                super::super::capabilities::numeric_stunned_actions_lost(
                    &actor,
                    &Default::default()
                ),
                Some(actions_lost)
            );
            let changes = prepare_lifecycle(
                &actor,
                &ConditionLifecycleActionView::StunnedActionsLost {
                    condition_id: 1,
                    actions_lost,
                },
                None,
                "mixed-stun",
                &Default::default(),
            )
            .unwrap()
            .resolve(None)
            .unwrap();
            assert_eq!(changes.condition_changes.len(), 1);
            if quickened {
                assert!(matches!(
                    &changes.condition_changes[0],
                    ConditionEventChange::Remove {
                        condition_id: 1,
                        ..
                    }
                ));
            } else {
                assert!(matches!(
                    &changes.condition_changes[0],
                    ConditionEventChange::Replace(row)
                        if row.condition_id == 1 && row.value == Some(1)
                ));
            }
        }
    }

    #[test]
    fn reminders_are_associated_with_independent_rows_and_offer_no_rest_or_waking_command() {
        let mut actor = participant();
        actor.conditions = vec![
            condition(1, ConditionRule::Wounded, Some(1)),
            condition(2, ConditionRule::Fatigued, None),
            condition(3, ConditionRule::Drained, Some(1)),
            condition(4, ConditionRule::Doomed, Some(1)),
            condition(5, ConditionRule::Unconscious, None),
        ];
        let before = actor.clone();
        let offers = available_actions(&actor, None, &[], &[], &Default::default());
        assert_eq!(offers.len(), 5);
        for (offer, id) in offers.iter().zip(1..=5) {
            assert_eq!(offer.condition_ids, [id]);
            assert!(
                matches!(&offer.action, ConditionRecoveryOfferView::Reminder { condition_id, text }
                if *condition_id == id && !text.is_empty())
            );
        }
        assert_eq!(actor, before);
        actor.current_hp = Some(0);
        let offers = available_actions(&actor, None, &[], &[], &Default::default());
        assert!(
            offers
                .iter()
                .all(|offer| matches!(offer.action, ConditionRecoveryOfferView::Reminder { .. }))
        );
    }

    #[test]
    fn frightened_reduces_known_sources_without_guessing_unknown_restrictions() {
        let mut actor = participant();
        actor.conditions = vec![
            condition(1, ConditionRule::Frightened, Some(3)),
            condition(2, ConditionRule::Frightened, Some(2)),
            condition(3, ConditionRule::Frightened, Some(1)),
        ];
        actor.conditions[1].parameters = ConditionParameters::Frightened {
            recovery: FrightenedRecovery::SourceDependent,
        };
        let offers = available_actions(&actor, None, &[], &[], &Default::default());
        let reduction = offers
            .iter()
            .find(|offer| matches!(offer.action, ConditionRecoveryOfferView::FrightenedEndTurn))
            .unwrap();
        assert_eq!(reduction.condition_ids, [1, 3]);
        let changes = prepare_lifecycle(
            &actor,
            &ConditionLifecycleActionView::FrightenedEndTurn,
            None,
            "fear",
            &Default::default(),
        )
        .unwrap()
        .resolve(None)
        .unwrap();
        assert_eq!(changes.condition_changes.len(), 2);
        assert!(
            matches!(&changes.condition_changes[0], ConditionEventChange::Replace(row) if row.condition_id == 1 && row.value == Some(2))
        );
        assert!(matches!(
            &changes.condition_changes[1],
            ConditionEventChange::Remove {
                condition_id: 3,
                ..
            }
        ));
    }

    #[test]
    fn unknown_broken_threshold_does_not_create_a_recovery_questionnaire() {
        let mut actor = participant();
        actor.participant_kind = atlas_local_state::ParticipantKind::Hazard;
        let definition =
            super::super::super::conditions::modeled_condition_by_rule(ConditionRule::Broken)
                .unwrap();
        let input = atlas_local_state::AddEncounterParticipantCondition {
            participant_key: actor.participant_key.clone(),
            condition_key: Some(definition.condition_ref.into()),
            name: definition.name.into(),
            value: None,
            source_participant_key: None,
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            parameters: ConditionParameters::Broken { threshold: None },
            origin: None,
            duration: None,
            note: None,
        };
        let planned = super::super::events::plan_condition_change(
            &actor,
            ConditionEventChange::Add(input),
            true,
            false,
            None,
            "broken",
        )
        .unwrap();
        assert!(planned.changes.pending.is_empty());
        assert!(planned.changes.participant_changes.is_empty());
    }
}
