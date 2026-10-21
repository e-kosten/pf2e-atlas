//! Explicit, pure lifecycle consequences; source preparation and dice stay outside.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::hp_events::{
    add_condition, clear, clear_normal_unconscious, has, replace_counter, set_counter,
};
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
    effective_maximum: Option<i64>,
    pending: Option<&SavedPendingConsequence>,
    request_id: &'a str,
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
        ConditionLifecycleActionView::Retch {
            condition_id,
            actions_spent,
            ..
        } => {
            if !actions_spent {
                return Err(AppServiceError::invalid_request(
                    "Confirm the Retch action was spent at the table",
                ));
            }
            let condition = source(participant, *condition_id, ConditionRule::Sickened)?;
            let value = positive_value(condition)?;
            effect = Some(CheckEffect::Retch { condition, value });
        }
        ConditionLifecycleActionView::FrightenedEndTurn { decisions } => {
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
                    ConditionParameters::Frightened {
                        recovery: FrightenedRecovery::SourceDependent,
                    } => {
                        let decision = decisions
                            .iter()
                            .find(|decision| decision.condition_id == condition.condition_id)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "Decide whether this Frightened source can decrease",
                                )
                            })?;
                        if !decision.reduce {
                            continue;
                        }
                        decision
                            .minimum
                            .filter(|minimum| *minimum >= 0)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "Specify the Frightened minimum for this source",
                                )
                            })?
                    }
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
        ConditionLifecycleActionView::TreatWounds { healing_received } => {
            if *healing_received < 1 {
                return Err(AppServiceError::invalid_request(
                    "Treat Wounds must have restored HP to remove Wounded",
                ));
            }
            if !has(participant, ConditionRule::Wounded) {
                return Err(AppServiceError::invalid_request(
                    "participant is not wounded",
                ));
            }
            clear(participant, ConditionRule::Wounded, &mut changes);
        }
        ConditionLifecycleActionView::FullHpRest { minutes } => {
            if *minutes < 10 {
                return Err(AppServiceError::invalid_request(
                    "Rest at full HP must last at least 10 minutes",
                ));
            }
            let maximum = effective_maximum.ok_or_else(|| {
                AppServiceError::invalid_request(
                    "Establish effective maximum HP before recording full-HP rest",
                )
            })?;
            if participant.current_hp != Some(maximum) {
                return Err(AppServiceError::invalid_request(
                    "Full-HP rest requires the participant to be at full HP",
                ));
            }
            if !has(participant, ConditionRule::Wounded) {
                return Err(AppServiceError::invalid_request(
                    "participant is not wounded",
                ));
            }
            clear(participant, ConditionRule::Wounded, &mut changes);
        }
        ConditionLifecycleActionView::OvernightRest => {
            for condition in &participant.conditions {
                if matches!(
                    condition_rule_for_key(condition.condition_key.as_deref()),
                    Some(ConditionRule::Drained | ConditionRule::Doomed)
                ) {
                    reduce_source(
                        participant,
                        condition,
                        positive_value(condition)?.saturating_sub(1),
                        &mut changes,
                    );
                }
            }
            clear(participant, ConditionRule::Fatigued, &mut changes);
            if changes.condition_changes.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "No Fatigued, Drained or Doomed source needs overnight recovery",
                ));
            }
        }
        ConditionLifecycleActionView::StunnedActionsLost {
            condition_id,
            actions_lost,
        } => {
            let selected = source(participant, *condition_id, ConditionRule::Stunned)?;
            positive_value(selected)?;
            let maximum = super::capabilities::numeric_stunned_actions_lost(participant).ok_or_else(||AppServiceError::invalid_request("Resolve the duration-based or unknown action loss before consuming numeric Stunned"))?;
            if maximum < 1 || *actions_lost != maximum {
                return Err(AppServiceError::invalid_request(
                    "Actions lost must match the current Stunned action-regain forecast",
                ));
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
        ConditionLifecycleActionView::StableRecovery { minutes } => {
            if *minutes < 10
                || participant.current_hp != Some(0)
                || known_effective(participant, ConditionRule::Dying)? > 0
                || participant.mortality == MortalityState::Dead
                || has(participant, ConditionRule::Petrified)
            {
                return Err(AppServiceError::invalid_request(
                    "Recovery requires at least 10 minutes, 0 HP and no Dying condition",
                ));
            }
            clear_normal_unconscious(participant, &mut changes);
            if changes.condition_changes.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "This unconscious source does not allow ordinary recovery",
                ));
            }
            let mut update = ParticipantEventChange::preserving(participant);
            update.current_hp = Some(1);
            changes.participant_changes.push(update);
        }
        ConditionLifecycleActionView::Wake { .. } => {
            require_waking(participant)?;
            clear_normal_unconscious(participant, &mut changes);
            if changes.condition_changes.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "Only the causing effect can end this unconscious source",
                ));
            }
        }
        ConditionLifecycleActionView::WakeFromNoise { dc, .. } => {
            if *dc < 1 {
                return Err(AppServiceError::invalid_request(
                    "Noise DC must be positive",
                ));
            }
            require_waking(participant)?;
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
                return Err(AppServiceError::invalid_request(
                    "This unconscious source does not allow ordinary noise waking",
                ));
            }
            effect = Some(CheckEffect::WakeFromNoise);
        }
        ConditionLifecycleActionView::ConfusedDamageRecovery {
            consequence_key,
            condition_ids,
            qualifying_damage,
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
            if known.is_some_and(|known| known != *qualifying_damage) {
                return Err(AppServiceError::invalid_request(
                    "damage origin contradicts the recorded event",
                ));
            }
            if *qualifying_damage && check.is_some() {
                if condition_ids.is_empty() || condition_ids.iter().any(|id| !eligible.contains(id))
                {
                    return Err(AppServiceError::invalid_request(
                        "Select the Confused sources affected by this recovery",
                    ));
                }
                for id in condition_ids {
                    source(participant, *id, ConditionRule::Confused)?;
                }
                effect = Some(CheckEffect::Confused {
                    condition_ids: condition_ids.clone(),
                });
            }
            changes.resolved_pending_keys.push(consequence_key.clone());
        }
        ConditionLifecycleActionView::ResolveBroken {
            consequence_key,
            threshold,
        } => {
            if *threshold < 0 {
                return Err(AppServiceError::invalid_request(
                    "Broken threshold must be nonnegative",
                ));
            }
            if participant.participant_kind != atlas_local_state::ParticipantKind::Hazard
                || has(participant, ConditionRule::Petrified)
            {
                return Err(AppServiceError::invalid_request(
                    "Broken threshold applies to physical hazards, not Petrified creatures",
                ));
            }
            let saved = pending
                .filter(|p| &p.consequence_key == consequence_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Broken threshold decision no longer exists")
                })?;
            if !matches!(&saved.consequence,PendingConsequence::BrokenHpThreshold{participant_key} if participant_key==&participant.participant_key)
            {
                return Err(AppServiceError::invalid_request(
                    "selected consequence is not this participant's Broken HP adjustment",
                ));
            }
            let current = participant.current_hp.ok_or_else(|| {
                AppServiceError::invalid_request("Set current HP before applying Broken")
            })?;
            let mut update = ParticipantEventChange::preserving(participant);
            update.current_hp = Some(current.min(*threshold));
            changes.participant_changes.push(update);
            for condition in &participant.conditions {
                if condition_rule_for_key(condition.condition_key.as_deref())
                    == Some(ConditionRule::Broken)
                {
                    let mut replacement = replace_counter(participant, condition, 0);
                    replacement.value = None;
                    replacement.parameters = ConditionParameters::Broken {
                        threshold: Some(*threshold),
                    };
                    changes
                        .condition_changes
                        .push(ConditionEventChange::Replace(replacement));
                }
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
    WakeFromNoise,
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
            CheckEffect::WakeFromNoise => {
                if matches!(
                    degree,
                    CheckDegreeView::Success | CheckDegreeView::CriticalSuccess
                ) {
                    clear_normal_unconscious(participant, &mut changes);
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
fn require_waking(participant: &EncounterParticipant) -> AppServiceResult<()> {
    if participant.current_hp.is_none_or(|hp| hp < 1)
        || participant.mortality == MortalityState::Dead
        || has(participant, ConditionRule::Petrified)
        || known_effective(participant, ConditionRule::Dying)? > 0
    {
        return Err(AppServiceError::invalid_request(
            "Ordinary waking requires a living, non-Petrified participant with at least 1 HP",
        ));
    }
    Ok(())
}

pub(in crate::encounters) fn available_actions(
    participant: &EncounterParticipant,
    fortitude: Option<i64>,
    perception: Option<i64>,
    maximum: Option<i64>,
    pending: &[SavedPendingConsequence],
    claims: &[atlas_local_state::EncounterBoundaryClaim],
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
    if has(participant, ConditionRule::Frightened) {
        add(
            Offer::FrightenedEndTurn {
                decision_source_ids: sources(ConditionRule::Frightened)
                    .filter(|condition| {
                        matches!(
                            condition.parameters,
                            ConditionParameters::Frightened {
                                recovery: FrightenedRecovery::SourceDependent
                            }
                        )
                    })
                    .map(|condition| condition.condition_id)
                    .collect(),
            },
            "End turn: reduce frightened",
            Some("frightened"),
        );
    }
    if has(participant, ConditionRule::Wounded) {
        add(Offer::TreatWounds, "Recover after Treat Wounds", None);
        if maximum.is_some() && participant.current_hp == maximum {
            add(Offer::FullHpRest, "Rest at full HP", None);
        }
    }
    if [
        ConditionRule::Drained,
        ConditionRule::Doomed,
        ConditionRule::Fatigued,
    ]
    .iter()
    .any(|rule| has(participant, *rule))
    {
        add(Offer::OvernightRest, "Rest overnight", None);
    }
    for condition in sources(ConditionRule::Stunned).filter(|condition| condition.value.is_some()) {
        add(
            Offer::StunnedActionsLost {
                condition_id: condition.condition_id,
                actions_lost: super::capabilities::numeric_stunned_actions_lost(participant),
            },
            "Consume actions lost to stunned",
            Some("stunned-actions"),
        );
    }
    let normal_unconscious = sources(ConditionRule::Unconscious).any(|condition| {
        !matches!(
            condition.parameters,
            ConditionParameters::Unconscious {
                waking: atlas_local_state::WakingRule::SourceControlled
            }
        )
    });
    if normal_unconscious
        && participant.current_hp == Some(0)
        && known_effective(participant, ConditionRule::Dying).ok() == Some(0)
        && participant.mortality == MortalityState::Alive
        && !has(participant, ConditionRule::Petrified)
    {
        add(Offer::StableRecovery, "Recover after sufficient rest", None);
    }
    if normal_unconscious && require_waking(participant).is_ok() {
        add(Offer::Wake, "Wake", None);
        add(
            Offer::WakeFromNoise {
                check: LifecycleCheckFactsView {
                    dc: None,
                    modifier: perception,
                },
            },
            "Check to wake from noise",
            Some("waking-noise"),
        );
    }
    for saved in pending {
        match &saved.consequence {
            PendingConsequence::ConfusedDamageRecovery {
                participant_key,
                condition_ids,
                qualifying_damage,
            } if participant_key == &participant.participant_key => {
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
            PendingConsequence::BrokenHpThreshold { participant_key }
                if participant_key == &participant.participant_key =>
            {
                add(
                    Offer::ResolveBroken {
                        consequence_key: saved.consequence_key.clone(),
                    },
                    "Apply broken HP threshold",
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
                Some("No resolved numeric Stunned loss is available at this action regain.".into());
        }
    }
    actions
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
        | (O::FrightenedEndTurn { .. }, A::FrightenedEndTurn { .. })
        | (O::TreatWounds, A::TreatWounds { .. })
        | (O::FullHpRest, A::FullHpRest { .. })
        | (O::OvernightRest, A::OvernightRest)
        | (O::StableRecovery, A::StableRecovery { .. })
        | (O::Wake, A::Wake { .. })
        | (O::WakeFromNoise { .. }, A::WakeFromNoise { .. }) => true,
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
        )
        | (
            O::ResolveBroken {
                consequence_key: offered,
            },
            A::ResolveBroken {
                consequence_key, ..
            },
        ) => offered == consequence_key,
        _ => false,
    }
}
