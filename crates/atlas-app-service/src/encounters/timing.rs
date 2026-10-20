//! Explicit encounter timing policy. Source attribution never implies a timing anchor.
use atlas_app_model::{
    ConditionDurationUnit as ViewUnit, ConditionDurationView, ConditionTimingView,
    ConditionTurnEvent as ViewEvent, EncounterTurnReminderView, EncounterTurnReviewView,
};
use atlas_local_state::{
    ConditionDuration, ConditionDurationUnit, ConditionTiming, ConditionTurnEvent,
    EncounterTurnUpdate, EncounterWithParticipants,
};

use super::turns::next_turn;
use crate::error::{AppServiceError, AppServiceResult};

// Canonical Foundry condition identity; a custom condition with the same name is not this rule.
const PERSISTENT_DAMAGE_KEY: &str = "conditionitems:lDVqvLKA6eF3Df60";

pub(super) fn duration_view(duration: ConditionDuration) -> ConditionDurationView {
    ConditionDurationView {
        amount: duration.amount,
        unit: match duration.unit {
            ConditionDurationUnit::Rounds => ViewUnit::Rounds,
            ConditionDurationUnit::Minutes => ViewUnit::Minutes,
            ConditionDurationUnit::Hours => ViewUnit::Hours,
            ConditionDurationUnit::Days => ViewUnit::Days,
        },
        timing: duration.timing.map(|timing| ConditionTimingView {
            participant_key: timing.participant_key,
            event: match timing.event {
                ConditionTurnEvent::Start => ViewEvent::Start,
                ConditionTurnEvent::End => ViewEvent::End,
            },
        }),
    }
}

pub(super) fn duration_input(
    duration: Option<ConditionDurationView>,
    existing: Option<&ConditionDuration>,
    snapshot: &EncounterWithParticipants,
) -> AppServiceResult<Option<ConditionDuration>> {
    let Some(duration) = duration else {
        return Ok(None);
    };
    let local = ConditionDuration {
        amount: duration.amount,
        unit: match duration.unit {
            ViewUnit::Rounds => ConditionDurationUnit::Rounds,
            ViewUnit::Minutes => ConditionDurationUnit::Minutes,
            ViewUnit::Hours => ConditionDurationUnit::Hours,
            ViewUnit::Days => ConditionDurationUnit::Days,
        },
        timing: duration.timing.map(|timing| ConditionTiming {
            participant_key: timing.participant_key,
            event: match timing.event {
                ViewEvent::Start => ConditionTurnEvent::Start,
                ViewEvent::End => ConditionTurnEvent::End,
            },
        }),
    };
    // Preserve unresolved migrated durations on unrelated edits, including historical outliers.
    if existing == Some(&local) {
        return Ok(Some(local));
    }
    if local.amount < 0 {
        return Err(AppServiceError::invalid_request(
            "condition duration amount must be nonnegative",
        ));
    }
    match (&local.unit, &local.timing) {
        (ConditionDurationUnit::Rounds, Some(timing)) => {
            if !snapshot
                .participants
                .iter()
                .any(|p| p.participant_key == timing.participant_key)
            {
                return Err(AppServiceError::invalid_request(
                    "round duration timing participant must belong to this encounter",
                ));
            }
        }
        (ConditionDurationUnit::Rounds, None) => {
            return Err(AppServiceError::invalid_request(
                "round duration requires an explicit turn participant and start/end timing",
            ));
        }
        (_, Some(_)) => {
            return Err(AppServiceError::invalid_request(
                "only round durations have turn timing",
            ));
        }
        (_, None) => {}
    }
    Ok(Some(local))
}

fn matches_boundary(
    snapshot: &EncounterWithParticipants,
    next_key: Option<&str>,
    timing: &ConditionTiming,
) -> bool {
    // No eligible next actor means no actual transition and no timing event.
    if next_key.is_none()
        || !snapshot
            .participants
            .iter()
            .any(|p| p.participant_key == timing.participant_key)
    {
        return false;
    }
    match timing.event {
        ConditionTurnEvent::End => {
            snapshot.encounter.current_turn_participant_key.as_deref()
                == Some(&timing.participant_key)
        }
        ConditionTurnEvent::Start => next_key == Some(timing.participant_key.as_str()),
    }
}

pub(super) fn turn_update(snapshot: &EncounterWithParticipants) -> EncounterTurnUpdate {
    let turn = next_turn(snapshot);
    let durations = snapshot
        .participants
        .iter()
        .flat_map(|p| &p.conditions)
        .filter_map(|condition| {
            let duration = condition.duration.as_ref()?;
            (duration.unit == ConditionDurationUnit::Rounds
                && duration.amount > 0
                && duration.timing.as_ref().is_some_and(|timing| {
                    matches_boundary(snapshot, turn.participant_key.as_deref(), timing)
                }))
            .then(|| (condition.condition_id, duration.amount - 1))
        })
        .collect();
    EncounterTurnUpdate { turn, durations }
}

pub(super) fn turn_review(snapshot: &EncounterWithParticipants) -> EncounterTurnReviewView {
    let update = turn_update(snapshot);
    let next_key = update.turn.participant_key.as_deref();
    let current_key = snapshot.encounter.current_turn_participant_key.as_deref();
    let mut reminders = Vec::new();
    for participant in &snapshot.participants {
        let owner_ending =
            next_key.is_some() && current_key == Some(participant.participant_key.as_str());
        let owner_starting_first =
            current_key.is_none() && next_key == Some(participant.participant_key.as_str());
        let owner_boundary = owner_ending || owner_starting_first;
        if owner_ending {
            let condition_ids = participant
                .conditions
                .iter()
                .filter(|condition| {
                    super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(super::conditions::ConditionRule::Frightened)
                })
                .map(|condition| condition.condition_id)
                .collect::<Vec<_>>();
            if !condition_ids.is_empty() {
                reminders.push(EncounterTurnReminderView::ConditionRecovery {
                    participant_key: participant.participant_key.clone(),
                    participant_name: participant.display_name.clone(),
                    condition_ids,
                    condition_name: "Frightened".into(),
                });
            }
        }
        for condition in &participant.conditions {
            let participant_key = participant.participant_key.clone();
            let participant_name = participant.display_name.clone();
            let condition_id = condition.condition_id;
            let condition_name = condition.name.clone();
            if owner_ending && condition.condition_key.as_deref() == Some(PERSISTENT_DAMAGE_KEY) {
                reminders.push(EncounterTurnReminderView::PersistentDamage {
                    participant_key: participant_key.clone(),
                    participant_name: participant_name.clone(),
                    condition_id,
                    condition_name: condition_name.clone(),
                    note: condition.note.clone(),
                });
            }
            let Some(duration) = condition.duration.as_ref() else {
                continue;
            };
            let reminder = if duration.unit == ConditionDurationUnit::Rounds
                && duration.amount == 0
                && owner_boundary
            {
                Some(EncounterTurnReminderView::DurationEnded {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                })
            } else if duration.unit != ConditionDurationUnit::Rounds && owner_boundary {
                Some(EncounterTurnReminderView::DurationReminder {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                    duration: duration_view(duration.clone()),
                })
            } else if duration.unit == ConditionDurationUnit::Rounds
                && duration.amount == 1
                && duration
                    .timing
                    .as_ref()
                    .is_some_and(|timing| matches_boundary(snapshot, next_key, timing))
            {
                duration_view(duration.clone()).timing.map(|timing| {
                    EncounterTurnReminderView::DurationEnding {
                        participant_key,
                        participant_name,
                        condition_id,
                        condition_name,
                        timing,
                    }
                })
            } else if duration.unit == ConditionDurationUnit::Rounds
                && owner_boundary
                && !duration.timing.as_ref().is_some_and(|timing| {
                    snapshot
                        .participants
                        .iter()
                        .any(|p| p.participant_key == timing.participant_key)
                })
            {
                Some(EncounterTurnReminderView::TimingUnavailable {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                    duration: duration_view(duration.clone()),
                })
            } else {
                None
            };
            reminders.extend(reminder);
        }
    }
    EncounterTurnReviewView {
        next_turn_unavailable_reason: if update.turn.participant_key.is_some() {
            None
        } else if snapshot.participants.iter().any(|p| {
            !p.defeated
                && !p.hazard_disabled
                && p.mortality != atlas_local_state::MortalityState::Dead
        }) {
            Some(atlas_app_model::EncounterNextTurnUnavailableReason::NoInitiativeParticipants)
        } else {
            Some(atlas_app_model::EncounterNextTurnUnavailableReason::NoActiveParticipants)
        },
        outgoing: None,
        encounter_key: snapshot.encounter.encounter_key.clone(),
        expected_revision: snapshot.encounter.revision,
        current_participant_key: snapshot.encounter.current_turn_participant_key.clone(),
        next_participant_key: update.turn.participant_key,
        reminders,
    }
}
