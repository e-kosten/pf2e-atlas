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
                | atlas_local_state::EncounterBoundaryKind::AssistedRecovery
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
    let reminders = super::timing::turn_review(&saved.snapshot)
        .reminders
        .into_iter()
        .filter(|reminder| {
            use atlas_app_model::EncounterTurnReminderView as R;
            match reminder {
                R::PersistentDamage { condition_id, .. } => {
                    !damage_applied
                        || live.is_some_and(|p| {
                            p.conditions.iter().any(|c| c.condition_id == *condition_id)
                        })
                }
                R::ConditionRecovery { condition_ids, .. } => {
                    !claims.iter().any(|claim| {
                        claim.kind == atlas_local_state::EncounterBoundaryKind::TurnEnd
                            && claim.scope_key == "frightened"
                    }) && live.is_some_and(|p| {
                        p.conditions
                            .iter()
                            .any(|c| condition_ids.contains(&c.condition_id))
                    })
                }
                R::DurationEnding {
                    condition_id,
                    participant_key,
                    ..
                }
                | R::DurationEnded {
                    condition_id,
                    participant_key,
                    ..
                }
                | R::DurationReminder {
                    condition_id,
                    participant_key,
                    ..
                }
                | R::TimingUnavailable {
                    condition_id,
                    participant_key,
                    ..
                } => current
                    .participants
                    .iter()
                    .find(|p| p.participant_key == *participant_key)
                    .is_some_and(|p| p.conditions.iter().any(|c| c.condition_id == *condition_id)),
            }
        })
        .collect::<Vec<_>>();
    let groups = super::persistent_damage::group_views(&frozen);
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
