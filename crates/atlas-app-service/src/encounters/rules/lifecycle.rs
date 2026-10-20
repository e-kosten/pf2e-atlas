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

pub(in crate::encounters) fn plan_lifecycle(
    participant: &EncounterParticipant,
    action: &ConditionLifecycleActionView,
    degree: Option<CheckDegreeView>,
    effective_maximum: Option<i64>,
    pending: Option<&SavedPendingConsequence>,
    request_id: &str,
) -> AppServiceResult<PreparedConditionEvent> {
    let mut changes = PreparedConditionEvent::default();
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
            let delta = match required_degree(degree)? {
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
            let reduction = match required_degree(degree)? {
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
            if matches!(
                required_degree(degree)?,
                CheckDegreeView::Success | CheckDegreeView::CriticalSuccess
            ) {
                clear_normal_unconscious(participant, &mut changes);
            }
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
                if matches!(
                    required_degree(degree)?,
                    CheckDegreeView::Success | CheckDegreeView::CriticalSuccess
                ) {
                    for id in condition_ids {
                        changes
                            .condition_changes
                            .push(ConditionEventChange::Remove {
                                participant_key: participant.participant_key.clone(),
                                condition_id: *id,
                            });
                    }
                }
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
    Ok(changes)
}
fn required_degree(degree: Option<CheckDegreeView>) -> AppServiceResult<CheckDegreeView> {
    degree.ok_or_else(|| AppServiceError::invalid_request("Record or roll the required check"))
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
    use ConditionLifecycleActionKindView as Kind;
    use LifecycleInputKindView as Input;
    let mut actions = Vec::new();
    let ids = |rule| {
        participant
            .conditions
            .iter()
            .filter(|condition| {
                condition_rule_for_key(condition.condition_key.as_deref()) == Some(rule)
            })
            .map(|condition| condition.condition_id)
            .collect::<Vec<_>>()
    };
    {
        let mut add = |kind,
                       label: &str,
                       ids: Vec<i64>,
                       dc,
                       modifier,
                       inputs: Vec<Input>,
                       scope: Option<&str>| {
            let claimed =
                scope.is_some_and(|scope| claims.iter().any(|claim| claim.scope_key == scope));
            actions.push(ConditionLifecycleOptionView {
                kind,
                label: label.into(),
                condition_id: (ids.len() == 1).then(|| ids[0]),
                consequence_key: None,
                qualifying_damage: None,
                source_condition_ids: ids,
                dc,
                modifier,
                issues: vec![],
                actions_lost: None,
                required_inputs: inputs,
                available: !claimed,
                unavailable_reason: claimed.then(|| "Already resolved this round".into()),
            });
        };
        let dying = ids(ConditionRule::Dying);
        if !dying.is_empty()
            && participant.track_dying
            && participant.mortality == MortalityState::Alive
            && participant.current_hp == Some(0)
        {
            add(
                Kind::DyingRecovery,
                "Recovery check",
                dying,
                known_effective(participant, ConditionRule::Dying)
                    .ok()
                    .and_then(|value| value.checked_add(10)),
                Some(0),
                vec![Input::Check],
                Some("dying"),
            );
        }
        for condition in participant.conditions.iter().filter(|condition| {
            condition_rule_for_key(condition.condition_key.as_deref())
                == Some(ConditionRule::Sickened)
        }) {
            let dc = match condition.parameters {
                ConditionParameters::Sickened { recovery_dc } => recovery_dc,
                _ => None,
            };
            let mut inputs = vec![Input::Check, Input::ActionsSpent, Input::ActionJudgments];
            if dc.is_none() {
                inputs.push(Input::SourceDc);
            }
            if fortitude.is_none() {
                inputs.push(Input::CurrentFortitude);
            }
            add(
                Kind::Retch,
                "Retch",
                vec![condition.condition_id],
                dc,
                fortitude,
                inputs,
                None,
            );
        }
        let frightened = ids(ConditionRule::Frightened);
        if !frightened.is_empty() {
            add(
                Kind::FrightenedEndTurn,
                "End turn: reduce frightened",
                frightened,
                None,
                None,
                vec![Input::FrightenedDecisions],
                Some("frightened"),
            );
        }
        let wounded = ids(ConditionRule::Wounded);
        if !wounded.is_empty() {
            add(
                Kind::TreatWounds,
                "Recover after Treat Wounds",
                wounded.clone(),
                None,
                None,
                vec![Input::HealingReceived],
                None,
            );
            if maximum.is_some() && participant.current_hp == maximum {
                add(
                    Kind::FullHpRest,
                    "Rest at full HP",
                    wounded,
                    None,
                    None,
                    vec![Input::RestMinutes],
                    None,
                );
            }
        }
        let overnight = ids(ConditionRule::Drained)
            .into_iter()
            .chain(ids(ConditionRule::Doomed))
            .chain(ids(ConditionRule::Fatigued))
            .collect::<Vec<_>>();
        if !overnight.is_empty() {
            add(
                Kind::OvernightRest,
                "Rest overnight",
                overnight,
                None,
                None,
                vec![],
                None,
            );
        }
        for condition in participant.conditions.iter().filter(|condition| {
            condition_rule_for_key(condition.condition_key.as_deref())
                == Some(ConditionRule::Stunned)
                && condition.value.is_some()
        }) {
            add(
                Kind::StunnedActionsLost,
                "Consume actions lost to stunned",
                vec![condition.condition_id],
                None,
                None,
                vec![Input::ActionsLost],
                Some("stunned-actions"),
            );
        }
        if participant.current_hp == Some(0)
            && known_effective(participant, ConditionRule::Dying).ok() == Some(0)
            && participant.mortality == MortalityState::Alive
            && !has(participant, ConditionRule::Petrified)
            && participant.conditions.iter().any(|condition| {
                condition_rule_for_key(condition.condition_key.as_deref())
                    == Some(ConditionRule::Unconscious)
                    && !matches!(
                        condition.parameters,
                        ConditionParameters::Unconscious {
                            waking: atlas_local_state::WakingRule::SourceControlled
                        }
                    )
            })
        {
            add(
                Kind::StableRecovery,
                "Recover after sufficient rest",
                ids(ConditionRule::Unconscious),
                None,
                None,
                vec![Input::RestMinutes],
                None,
            );
        }
        if require_waking(participant).is_ok()
            && participant.conditions.iter().any(|condition| {
                condition_rule_for_key(condition.condition_key.as_deref())
                    == Some(ConditionRule::Unconscious)
                    && !matches!(
                        condition.parameters,
                        ConditionParameters::Unconscious {
                            waking: atlas_local_state::WakingRule::SourceControlled
                        }
                    )
            })
        {
            let unconscious = ids(ConditionRule::Unconscious);
            add(
                Kind::Wake,
                "Wake",
                unconscious.clone(),
                None,
                None,
                vec![Input::WakeCause],
                None,
            );
            let mut inputs = vec![Input::Check, Input::SourceDc];
            if perception.is_none() {
                inputs.push(Input::CurrentPerception);
            }
            add(
                Kind::WakeFromNoise,
                "Check to wake from noise",
                unconscious,
                None,
                perception,
                inputs,
                Some("waking-noise"),
            );
        }
    }
    for option in &mut actions {
        if option.kind == Kind::StunnedActionsLost {
            option.actions_lost = super::capabilities::numeric_stunned_actions_lost(participant);
        }
        if option.kind == Kind::StunnedActionsLost
            && super::capabilities::numeric_stunned_actions_lost(participant)
                .is_none_or(|loss| loss == 0)
        {
            option.available = false;
            option.unavailable_reason =
                Some("No resolved numeric Stunned loss is available at this action regain.".into());
        }
    }
    for saved in pending {
        let (kind, label, condition_ids, inputs) = match &saved.consequence {
            PendingConsequence::ConfusedDamageRecovery {
                participant_key,
                condition_ids,
                qualifying_damage,
            } if participant_key == &participant.participant_key => {
                let mut inputs = vec![Input::SourceSelection, Input::Check];
                if qualifying_damage.is_none() {
                    inputs.push(Input::QualifyingDamage);
                }
                (
                    Kind::ConfusedDamageRecovery,
                    "Check to recover from confused",
                    condition_ids.clone(),
                    inputs,
                )
            }
            PendingConsequence::BrokenHpThreshold { participant_key }
                if participant_key == &participant.participant_key =>
            {
                (
                    Kind::ResolveBroken,
                    "Apply broken HP threshold",
                    ids(ConditionRule::Broken),
                    vec![Input::Threshold],
                )
            }
            _ => continue,
        };
        actions.push(ConditionLifecycleOptionView {
            kind,
            label: label.into(),
            condition_id: None,
            consequence_key: Some(saved.consequence_key.clone()),
            qualifying_damage: match saved.consequence {
                PendingConsequence::ConfusedDamageRecovery {
                    qualifying_damage, ..
                } => qualifying_damage,
                _ => None,
            },
            source_condition_ids: condition_ids,
            dc: None,
            modifier: None,
            issues: vec![],
            actions_lost: None,
            required_inputs: inputs,
            available: true,
            unavailable_reason: None,
        });
    }
    actions
}
