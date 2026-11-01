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

pub(super) fn duration_view(
    duration: ConditionDuration,
    round_seconds: i64,
) -> ConditionDurationView {
    ConditionDurationView {
        remaining_rounds: duration
            .remaining_seconds
            .map(|s| rounds_for_seconds(s, round_seconds)),
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

/// Explicit elapsed time remains exact when the encounter's round length changes.
pub(super) fn duration_seconds(amount: i64, unit: ConditionDurationUnit) -> AppServiceResult<i64> {
    let multiplier = match unit {
        ConditionDurationUnit::Rounds => 1,
        ConditionDurationUnit::Minutes => 60,
        ConditionDurationUnit::Hours => 3600,
        ConditionDurationUnit::Days => 86400,
    };
    amount
        .checked_mul(multiplier)
        .filter(|v| *v >= 0 && amount <= 9_007_199_254_740_991)
        .ok_or_else(|| AppServiceError::invalid_request("Duration is outside the supported range."))
}
pub(super) fn rounds_for_seconds(seconds: i64, round_seconds: i64) -> i64 {
    seconds.div_euclid(round_seconds) + i64::from(seconds.rem_euclid(round_seconds) > 0)
}
pub(super) fn remaining_rounds(duration: &ConditionDuration, round_seconds: i64) -> Option<i64> {
    if duration.unit == ConditionDurationUnit::Rounds {
        Some(duration.amount)
    } else {
        duration
            .remaining_seconds
            .map(|s| rounds_for_seconds(s, round_seconds))
    }
}
pub(super) fn tracker_progress(
    duration: &ConditionDuration,
) -> AppServiceResult<atlas_local_state::TrackerProgress> {
    let amount = duration_seconds(duration.amount, duration.unit)?;
    Ok(if duration.unit == ConditionDurationUnit::Rounds {
        atlas_local_state::TrackerProgress::Rounds(amount)
    } else {
        atlas_local_state::TrackerProgress::Seconds(amount)
    })
}
pub(super) fn tracker_rounds(
    progress: &atlas_local_state::TrackerProgress,
    round_seconds: i64,
) -> i64 {
    match progress {
        atlas_local_state::TrackerProgress::Rounds(v) => *v,
        atlas_local_state::TrackerProgress::Seconds(v) => rounds_for_seconds(*v, round_seconds),
    }
}
pub(super) fn tick_tracker(
    progress: &atlas_local_state::TrackerProgress,
    round_seconds: i64,
) -> atlas_local_state::TrackerProgress {
    match progress {
        atlas_local_state::TrackerProgress::Rounds(v) => {
            atlas_local_state::TrackerProgress::Rounds((v - 1).max(0))
        }
        atlas_local_state::TrackerProgress::Seconds(v) => {
            atlas_local_state::TrackerProgress::Seconds(v.saturating_sub(round_seconds).max(0))
        }
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
        remaining_seconds: None,
    };
    // Metadata-only edits preserve saved progress, including unresolved legacy anchors.
    if let Some(old) = existing.filter(|old| {
        old.amount == local.amount && old.unit == local.unit && old.timing == local.timing
    }) {
        return Ok(Some(old.clone()));
    }
    let seconds = duration_seconds(local.amount, local.unit)?;
    let count = if local.unit == ConditionDurationUnit::Rounds {
        local.amount
    } else {
        rounds_for_seconds(seconds, snapshot.encounter.round_duration_seconds)
    };
    require_exact_round_count(count)?;

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
        local.remaining_seconds = Some(seconds);
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
            (remaining_rounds(duration, snapshot.encounter.round_duration_seconds)
                .is_some_and(|count| count > 0)
                && duration.timing.as_ref().is_some_and(|timing| {
                    matches_boundary(snapshot, turn.participant_key.as_deref(), timing)
                }))
            .then(|| {
                (
                    condition.condition_id,
                    if duration.unit == ConditionDurationUnit::Rounds {
                        duration.amount - 1
                    } else {
                        duration
                            .remaining_seconds
                            .unwrap_or(0)
                            .saturating_sub(snapshot.encounter.round_duration_seconds)
                            .max(0)
                    },
                )
            })
        })
        .collect();
    let manual_trackers = snapshot
        .participants
        .iter()
        .flat_map(|p| {
            p.manual_trackers.iter().filter_map(|tracker| {
                (tracker_rounds(&tracker.progress, snapshot.encounter.round_duration_seconds) > 0
                    && tracker.duration.timing.as_ref().is_some_and(|timing| {
                        matches_boundary(snapshot, turn.participant_key.as_deref(), timing)
                    }))
                .then(|| {
                    let mut value = tracker.clone();
                    value.progress =
                        tick_tracker(&value.progress, snapshot.encounter.round_duration_seconds);
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
            let reminder = if remaining_rounds(duration, snapshot.encounter.round_duration_seconds)
                == Some(0)
                && owner_boundary
            {
                Some(EncounterTurnReminderView::DurationEnded {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                })
            } else if remaining_rounds(duration, snapshot.encounter.round_duration_seconds)
                .is_none()
                && duration.unit != ConditionDurationUnit::Rounds
                && duration.timing.is_none()
                && owner_boundary
            {
                Some(EncounterTurnReminderView::DurationReminder {
                    participant_key,
                    participant_name,
                    condition_id,
                    condition_name,
                    duration: duration_view(
                        duration.clone(),
                        snapshot.encounter.round_duration_seconds,
                    ),
                })
            } else if remaining_rounds(duration, snapshot.encounter.round_duration_seconds)
                == Some(1)
                && duration
                    .timing
                    .as_ref()
                    .is_some_and(|timing| matches_boundary(snapshot, next_key, timing))
            {
                duration_view(duration.clone(), snapshot.encounter.round_duration_seconds)
                    .timing
                    .map(|timing| EncounterTurnReminderView::DurationEnding {
                        participant_key,
                        participant_name,
                        condition_id,
                        condition_name,
                        timing,
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
                    duration: duration_view(
                        duration.clone(),
                        snapshot.encounter.round_duration_seconds,
                    ),
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

fn require_exact_round_count(count: i64) -> AppServiceResult<()> {
    if count > 9_007_199_254_740_991 {
        return Err(AppServiceError::invalid_request(
            "Round duration produces an unsupported countdown size.",
        ));
    }
    Ok(())
}
pub(super) fn validate_round_clock(
    snapshot: &EncounterWithParticipants,
    seconds: i64,
) -> AppServiceResult<()> {
    if !(1..=86400).contains(&seconds) {
        return Err(AppServiceError::invalid_request(
            "Round duration must be between 1 and 86400 seconds.",
        ));
    }
    for participant in &snapshot.participants {
        for remaining in participant
            .conditions
            .iter()
            .filter_map(|c| c.duration.as_ref()?.remaining_seconds)
            .chain(
                participant
                    .manual_trackers
                    .iter()
                    .filter_map(|t| match t.progress {
                        atlas_local_state::TrackerProgress::Seconds(v) => Some(v),
                        _ => None,
                    }),
            )
        {
            require_exact_round_count(rounds_for_seconds(remaining, seconds))?;
        }
    }
    Ok(())
}
