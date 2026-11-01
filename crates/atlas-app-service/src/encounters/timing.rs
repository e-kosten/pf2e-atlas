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
        remaining_rounds: duration.remaining_rounds,
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

/// A round advances six seconds; preserve the authored duration separately.
pub(super) fn duration_rounds(amount: i64, unit: ConditionDurationUnit) -> AppServiceResult<i64> {
    let multiplier = match unit {
        ConditionDurationUnit::Rounds => 1,
        ConditionDurationUnit::Minutes => 10,
        ConditionDurationUnit::Hours => 600,
        ConditionDurationUnit::Days => 14400,
    };
    amount
        .checked_mul(multiplier)
        .filter(|v| (0..=9_007_199_254_740_991).contains(v))
        .ok_or_else(|| AppServiceError::invalid_request("Duration is outside the supported range."))
}
pub(super) fn remaining_rounds(duration: &ConditionDuration) -> Option<i64> {
    if duration.unit == ConditionDurationUnit::Rounds {
        Some(duration.amount)
    } else {
        duration.remaining_rounds
    }
}
pub(super) fn duration_input(
    duration: Option<ConditionDurationView>,
    existing: Option<&ConditionDuration>,
    snapshot: &EncounterWithParticipants,
) -> AppServiceResult<Option<ConditionDuration>> {
    let Some(input) = duration else {
        return Ok(None);
    };
    let mut local = ConditionDuration {
        amount: input.amount,
        unit: match input.unit {
            ViewUnit::Rounds => ConditionDurationUnit::Rounds,
            ViewUnit::Minutes => ConditionDurationUnit::Minutes,
            ViewUnit::Hours => ConditionDurationUnit::Hours,
            ViewUnit::Days => ConditionDurationUnit::Days,
        },
        timing: input.timing.map(|t| ConditionTiming {
            participant_key: t.participant_key,
            event: match t.event {
                ViewEvent::Start => ConditionTurnEvent::Start,
                ViewEvent::End => ConditionTurnEvent::End,
            },
        }),
        remaining_rounds: None,
    };
    // Metadata-only edits preserve saved progress, including unresolved legacy anchors.
    if let Some(old) = existing.filter(|old| {
        old.amount == local.amount && old.unit == local.unit && old.timing == local.timing
    }) {
        return Ok(Some(old.clone()));
    }
    let ticks = duration_rounds(local.amount, local.unit)?;
    match &local.timing {
        Some(timing)
            if !snapshot
                .participants
                .iter()
                .any(|p| p.participant_key == timing.participant_key) =>
        {
            return Err(AppServiceError::invalid_request(
                "Duration timing participant must belong to this encounter.",
            ));
        }
        None if local.unit == ConditionDurationUnit::Rounds => {
            return Err(AppServiceError::invalid_request(
                "Round duration requires an explicit turn participant and start/end timing.",
            ));
        }
        _ => {}
    }
    if local.unit != ConditionDurationUnit::Rounds && local.timing.is_some() {
        local.remaining_rounds = Some(ticks);
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
            (remaining_rounds(duration).is_some_and(|count| count > 0)
                && duration.timing.as_ref().is_some_and(|timing| {
                    matches_boundary(snapshot, turn.participant_key.as_deref(), timing)
                }))
            .then(|| {
                (
                    condition.condition_id,
                    remaining_rounds(duration).unwrap_or(0) - 1,
                )
            })
        })
        .collect();
    let manual_trackers = snapshot
        .participants
        .iter()
        .flat_map(|p| {
            p.manual_trackers.iter().filter_map(|tracker| {
                (tracker.remaining_rounds > 0
                    && tracker.duration.timing.as_ref().is_some_and(|timing| {
                        matches_boundary(snapshot, turn.participant_key.as_deref(), timing)
                    }))
                .then(|| {
                    let mut value = tracker.clone();
                    value.remaining_rounds -= 1;
                    atlas_local_state::ManualTrackerChange {
                        participant_key: p.participant_key.clone(),
                        tracker_key: tracker.tracker_key.clone(),
                        value: Some(value),
                    }
                })
            })
        })
        .collect();
    EncounterTurnUpdate {
        manual_trackers,
        turn,
        durations,
        start_event: None,
        start_claims: vec![],
    }
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
                    condition_value: super::rules::causes::condition_causes(
                        participant,
                        None,
                        super::rules::source_conditions::SourceConditions::empty(),
                    )
                    .known_value(super::conditions::ConditionRule::Frightened)
                    .ok(),
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
            let reminder = if remaining_rounds(duration) == Some(0) && owner_boundary {
                Some(EncounterTurnReminderView::DurationEnded {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                })
            } else if remaining_rounds(duration).is_none()
                && duration.unit != ConditionDurationUnit::Rounds
                && duration.timing.is_none()
                && owner_boundary
            {
                Some(EncounterTurnReminderView::DurationReminder {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                    duration: duration_view(duration.clone()),
                })
            } else if remaining_rounds(duration) == Some(1)
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
            } else if (duration.unit == ConditionDurationUnit::Rounds || duration.timing.is_some())
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
