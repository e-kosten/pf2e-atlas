//! The latest completed turn supplies frozen context while its reminder is open.
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::EncounterOutgoingTurnView;
use atlas_local_state::{EncounterParticipant, EncountersRead, SavedOutgoingTurn};

pub(super) fn boundary(
    store: &EncountersRead<'_>,
    encounter: &str,
    key: Option<&str>,
    participant: &str,
) -> AppServiceResult<Option<SavedOutgoingTurn>> {
    key.map(|key| {
        let saved = store.outgoing_turn(encounter, key)?.ok_or_else(|| {
            AppServiceError::invalid_request("This turn reminder is no longer current")
        })?;
        if saved
            .snapshot
            .encounter
            .current_turn_participant_key
            .as_deref()
            != Some(participant)
            || saved.next_participant_key.is_none()
        {
            return Err(AppServiceError::invalid_request(
                "Choose the outgoing participant for this turn",
            ));
        }
        Ok(saved)
    })
    .transpose()
}
/// A missing old generic setup may be filled, but known historical episode inputs never change.
pub(super) fn frozen_participant(
    saved: &SavedOutgoingTurn,
    current: &EncounterParticipant,
) -> AppServiceResult<EncounterParticipant> {
    let mut participant = saved
        .snapshot
        .participants
        .iter()
        .find(|p| p.participant_key == current.participant_key)
        .cloned()
        .ok_or_else(|| AppServiceError::invalid_request("Outgoing participant is unavailable"))?;
    for condition in &mut participant.conditions {
        if matches!(
            condition.parameters,
            atlas_local_state::ConditionParameters::None
        ) && let Some(current) = current
            .conditions
            .iter()
            .find(|row| row.condition_id == condition.condition_id)
            && matches!(
                current.parameters,
                atlas_local_state::ConditionParameters::PersistentDamage { .. }
            )
        {
            condition.parameters = current.parameters.clone();
        }
    }
    Ok(participant)
}
pub(super) fn latest_review(
    store: &EncountersRead<'_>,
    encounter: &str,
) -> AppServiceResult<Option<EncounterOutgoingTurnView>> {
    let Some(saved) = store.latest_outgoing_turn(encounter)? else {
        return Ok(None);
    };
    let current = store
        .get_with_participants(encounter)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter))?;
    let Some(key) = saved
        .snapshot
        .encounter
        .current_turn_participant_key
        .as_deref()
    else {
        return Ok(None);
    };
    let live = current
        .participants
        .iter()
        .find(|p| p.participant_key == key);
    let original = saved
        .snapshot
        .participants
        .iter()
        .find(|p| p.participant_key == key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Outgoing participant snapshot unavailable")
        })?;
    let frozen = frozen_participant(&saved, live.unwrap_or(original))?;
    let claims = store.boundary_claims(encounter, key, saved.snapshot.encounter.round_number)?;
    let damage_claim = super::persistent_damage::boundary_claim(
        &frozen,
        saved.snapshot.encounter.round_number,
        atlas_local_state::EncounterBoundaryKind::PersistentDamage,
        "",
    );
    let damage_applied = claims.contains(&damage_claim);
    let mut recovered_episode_ids = vec![];
    for claim in &claims {
        if matches!(
            claim.kind,
            atlas_local_state::EncounterBoundaryKind::PersistentRecovery
        ) && let Some(receipt) = store.boundary_receipt(encounter, claim)?
        {
            for outcome in receipt.outcomes {
                if let atlas_local_state::ConditionEventOutcome::PersistentRecovery {
                    episode_id,
                    success: true,
                    ..
                } = outcome
                {
                    recovered_episode_ids.push(episode_id);
                }
            }
        }
    }
    let groups = super::persistent_damage::group_views(
        &frozen,
        live.unwrap_or(original),
        None,
        &super::rules::source_conditions::SourceConditions::unavailable(),
    );
    let recovery_recorded = claims.iter().any(|claim| {
        claim.kind == atlas_local_state::EncounterBoundaryKind::TurnEnd
            && claim.scope_key == "frightened"
    });
    let mut resolved_ids = std::collections::BTreeSet::new();
    let reminders = super::timing::turn_review(&saved.snapshot)
        .reminders
        .into_iter()
        .filter_map(|reminder| {
            retain_reminder(reminder, &current.participants, &groups, recovery_recorded)
        })
        .filter(|reminder| match reminder {
            atlas_app_model::EncounterTurnReminderView::Resolved {
                participant_key,
                condition_id,
                ..
            } => resolved_ids.insert((participant_key.clone(), *condition_id)),
            _ => true,
        })
        .collect::<Vec<_>>();
    if reminders.is_empty() && groups.is_empty() {
        return Ok(None);
    }
    Ok(Some(EncounterOutgoingTurnView {
        participant_removed: live.is_none(),
        damage_applied,
        recovered_episode_ids,
        participant_key: key.into(),
        participant_name: frozen.display_name.clone(),
        round_number: saved.snapshot.encounter.round_number,
        boundary_key: saved.boundary_key,
        reminders,
        persistent_damage_groups: groups,
    }))
}

/// Preserve completed rows from this boundary, without recreating an action or
/// inferring success from a client submission. Pending rows keep their exact source IDs.
fn retain_reminder(
    reminder: atlas_app_model::EncounterTurnReminderView,
    current: &[EncounterParticipant],
    groups: &[atlas_app_model::PersistentDamageGroupView],
    recovery_recorded: bool,
) -> Option<atlas_app_model::EncounterTurnReminderView> {
    use atlas_app_model::EncounterTurnReminderView as R;
    let (participant_key, participant_name, condition_id, condition_name) = match &reminder {
        R::ConditionRecovery {
            participant_key,
            participant_name,
            condition_ids,
            condition_name,
            ..
        } => {
            let id = *condition_ids.first()?;
            if recovery_recorded {
                return Some(R::Resolved {
                    participant_key: participant_key.clone(),
                    participant_name: participant_name.clone(),
                    condition_id: id,
                    condition_name: condition_name.clone(),
                    result: "End-of-turn recovery recorded".into(),
                });
            }
            if let Some(live) = current
                .iter()
                .find(|p| p.participant_key == *participant_key)
                .filter(|p| {
                    p.conditions
                        .iter()
                        .any(|c| condition_ids.contains(&c.condition_id))
                })
            {
                let condition_value = super::rules::causes::condition_causes(
                    live,
                    None,
                    super::rules::source_conditions::SourceConditions::empty(),
                )
                .known_value(super::conditions::ConditionRule::Frightened)
                .ok();
                return Some(R::ConditionRecovery {
                    participant_key: participant_key.clone(),
                    participant_name: participant_name.clone(),
                    condition_ids: condition_ids.clone(),
                    condition_name: condition_name.clone(),
                    condition_value,
                });
            }
            (participant_key, participant_name, id, condition_name)
        }
        R::PersistentDamage {
            participant_key,
            participant_name,
            condition_id,
            condition_name,
            ..
        } => {
            // The retained type card already owns both its action and result.
            if groups
                .iter()
                .any(|g| g.sources.iter().any(|s| s.condition_id == *condition_id))
            {
                return None;
            }
            (
                participant_key,
                participant_name,
                *condition_id,
                condition_name,
            )
        }
        R::DurationEnding {
            participant_key,
            participant_name,
            condition_id,
            condition_name,
            ..
        }
        | R::DurationEnded {
            participant_key,
            participant_name,
            condition_id,
            condition_name,
            ..
        }
        | R::DurationReminder {
            participant_key,
            participant_name,
            condition_id,
            condition_name,
            ..
        }
        | R::TimingUnavailable {
            participant_key,
            participant_name,
            condition_id,
            condition_name,
            ..
        } => (
            participant_key,
            participant_name,
            *condition_id,
            condition_name,
        ),
        R::Resolved { .. } => return Some(reminder),
    };
    let participant = current
        .iter()
        .find(|p| p.participant_key == *participant_key);
    if participant.is_some_and(|p| p.conditions.iter().any(|c| c.condition_id == condition_id)) {
        return Some(reminder);
    }
    Some(R::Resolved {
        participant_key: participant_key.clone(),
        participant_name: participant_name.clone(),
        condition_id,
        condition_name: condition_name.clone(),
        result: if participant.is_some() {
            "Removed"
        } else {
            "Participant removed"
        }
        .into(),
    })
}
