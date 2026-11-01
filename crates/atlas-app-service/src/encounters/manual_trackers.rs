//! Manual countdown commands need no reference data or gameplay interpretation.
use super::{event_execution, timing};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    EncounterManualTrackerAction as Action, EncounterManualTrackerView, EncounterMutationView,
    UpdateEncounterManualTrackerRequest,
};
use atlas_local_state::{
    EncounterParticipant, EncounterWithParticipants, ManualTracker, ManualTrackerChange,
    PreparedConditionEvent,
};

impl AtlasAppService {
    pub fn update_encounter_manual_tracker(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterManualTrackerRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"manual_tracker","request":request});
        if let Some(receipt) =
            event_execution::recover(self, encounter_ref, &request.request_id, &intent)?
        {
            return Ok(event_execution::mutation(receipt));
        }
        let snapshot =
            event_execution::snapshot(self, encounter_ref, request.expected_encounter_revision)?;
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("Participant is no longer in this encounter.")
            })?;
        let existing = |key: &str| {
            participant
                .manual_trackers
                .iter()
                .find(|t| t.tracker_key == key)
                .ok_or_else(|| AppServiceError::invalid_request("Tracker is no longer available."))
        };
        let (tracker_key, value) = match &request.action {
            Action::Add { name, duration } => {
                let key = format!("tracker_{}", request.request_id);
                let value = prepare(&snapshot, &key, name, duration.clone())?;
                (key, Some(value))
            }
            Action::Edit {
                tracker_key,
                name,
                duration,
            } => {
                let old = existing(tracker_key)?;
                let mut input = duration.clone();
                input.remaining_rounds = None;
                let value = if input
                    == timing::duration_view(
                        old.duration.clone(),
                        snapshot.encounter.round_duration_seconds,
                    ) {
                    let mut value = old.clone();
                    value.name = tracker_name(name)?;
                    value
                } else {
                    prepare(&snapshot, tracker_key, name, input)?
                };
                (tracker_key.clone(), Some(value))
            }
            Action::Restart { tracker_key } => {
                let old = existing(tracker_key)?;
                let value = prepare(
                    &snapshot,
                    tracker_key,
                    &old.name,
                    timing::duration_view(
                        old.duration.clone(),
                        snapshot.encounter.round_duration_seconds,
                    ),
                )?;
                (tracker_key.clone(), Some(value))
            }
            Action::Remove { tracker_key } => {
                existing(tracker_key)?;
                (tracker_key.clone(), None)
            }
        };
        event_execution::EncounterEvent {
            condition_sources: &[],
            encounter_key: &snapshot.encounter.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            reference_revision: None,
            participant_revision: Some((
                &request.participant_key,
                request.expected_participant_revision,
            )),
        }
        .commit(
            self,
            PreparedConditionEvent {
                manual_trackers: vec![ManualTrackerChange {
                    participant_key: request.participant_key.clone(),
                    tracker_key,
                    value,
                }],
                ..Default::default()
            },
        )
        .map(event_execution::mutation)
    }
}
fn prepare(
    snapshot: &EncounterWithParticipants,
    key: &str,
    name: &str,
    mut input: atlas_app_model::ConditionDurationView,
) -> AppServiceResult<ManualTracker> {
    let name = tracker_name(name)?;
    input.remaining_rounds = None;
    let mut duration = timing::duration_input(Some(input), None, snapshot)?
        .ok_or_else(|| AppServiceError::invalid_request("Choose a tracker duration."))?;
    if duration.timing.is_none() {
        return Err(AppServiceError::invalid_request(
            "Choose whose turn counts down the tracker.",
        ));
    }
    let progress = timing::tracker_progress(&duration)?;
    duration.remaining_seconds = None;
    Ok(ManualTracker {
        tracker_key: key.into(),
        name,
        duration,
        progress,
    })
}
pub(super) fn views(
    participant: &EncounterParticipant,
    participants: &[EncounterParticipant],
    round_seconds: i64,
) -> Vec<EncounterManualTrackerView> {
    participant
        .manual_trackers
        .iter()
        .map(|tracker| {
            let anchor = tracker.duration.timing.as_ref().and_then(|timing| {
                participants
                    .iter()
                    .find(|p| p.participant_key == timing.participant_key)
                    .map(|p| (p, timing.event))
            });
            EncounterManualTrackerView {
                tracker_key: tracker.tracker_key.clone(),
                name: tracker.name.clone(),
                duration: timing::duration_view(tracker.duration.clone(), round_seconds),
                remaining_rounds: timing::tracker_rounds(&tracker.progress, round_seconds),
                timing_available: anchor.is_some(),
                timing_label: anchor
                    .map(|(p, event)| {
                        format!(
                            "{} of {}’s turn",
                            match event {
                                atlas_local_state::ConditionTurnEvent::Start => "Start",
                                atlas_local_state::ConditionTurnEvent::End => "End",
                            },
                            p.display_name
                        )
                    })
                    .unwrap_or_else(|| "Timing participant unavailable".into()),
            }
        })
        .collect()
}

fn tracker_name(name: &str) -> AppServiceResult<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(AppServiceError::invalid_request(
            "Tracker name must contain 1 to 120 characters.",
        ));
    }
    Ok(name.into())
}
