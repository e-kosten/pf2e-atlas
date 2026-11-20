//! Preview and commit the same participant-edit calculation against guarded facts.
use super::{event_execution, hydration, rules, timing};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{
    AdjustmentChange, AdjustmentTiming, EncounterAdjustment, EncounterParticipant,
    EncounterWithParticipants, ParticipantKind, ParticipantVariantOrigin, PreparedConditionEvent,
    TrackerProgress,
};

impl AtlasAppService {
    pub fn preview_encounter_adjustment(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterAdjustmentRequest,
    ) -> AppServiceResult<EncounterAdjustmentPreviewView> {
        Ok(prepare(self, encounter_ref, participant_key, &request)?.preview)
    }

    pub fn update_encounter_adjustment(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        mut request: EncounterAdjustmentRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let expected_reference_revision = request.expected_reference_revision.take();
        let intent = serde_json::json!({
            "action":"gm_adjustment",
            "request":event_execution::receipt_request(&request, "participant_key", participant_key),
        });
        if let Some(receipt) =
            event_execution::recover(self, encounter_ref, &request.request_id, &intent)?
        {
            return Ok(event_execution::mutation(receipt));
        }
        let prepared = prepare(self, encounter_ref, participant_key, &request)?;
        if expected_reference_revision != prepared.preview.reference_revision {
            return Err(AppServiceError::artifact_generation_changed(
                &format!("{expected_reference_revision:?}"),
                &format!("{:?}", prepared.preview.reference_revision),
                AppRecoverableAction::RefreshResults,
            ));
        }
        event_execution::EncounterEvent {
            condition_sources: &[],
            encounter_key: &prepared.encounter_key,
            request_id: &request.request_id,
            intent: &intent,
            expected_revision: request.expected_encounter_revision,
            participant_revision: Some((participant_key, request.expected_participant_revision)),
            reference_revision: prepared.preview.reference_revision,
        }
        .commit(
            self,
            PreparedConditionEvent {
                adjustments: vec![prepared.change],
                ..Default::default()
            },
        )
        .map(event_execution::mutation)
    }
}

struct Prepared {
    encounter_key: String,
    change: AdjustmentChange,
    preview: EncounterAdjustmentPreviewView,
}

fn prepare(
    service: &AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
    request: &EncounterAdjustmentRequest,
) -> AppServiceResult<Prepared> {
    if request.request_id.is_empty() || request.request_id.len() > 128 {
        return Err(AppServiceError::invalid_request(
            "Invalid adjustment request ID.",
        ));
    }
    let snapshot =
        event_execution::snapshot(service, encounter_ref, request.expected_encounter_revision)?;
    let participant = snapshot
        .participants
        .iter()
        .find(|p| {
            p.participant_key == participant_key
                && p.participant_kind != ParticipantKind::InitiativeEntry
        })
        .ok_or_else(|| {
            AppServiceError::invalid_request("This participant cannot receive adjustments.")
        })?;
    if participant.revision != request.expected_participant_revision {
        return Err(
            atlas_local_state::LocalStateError::StaleParticipantRevision {
                participant_key: participant_key.to_owned(),
                expected_revision: request.expected_participant_revision,
                actual_revision: participant.revision,
            }
            .into(),
        );
    }
    let existing = |key: &str| {
        participant
            .adjustments
            .iter()
            .find(|entry| entry.adjustment_key == key)
            .ok_or_else(|| {
                AppServiceError::invalid_request("This adjustment is no longer available.")
            })
    };
    let (key, old, value) = match &request.action {
        EncounterAdjustmentAction::Add { value } => {
            let key = format!("adjustment_{}", request.request_id);
            (key.clone(), None, Some(input(&snapshot, key, value, None)?))
        }
        EncounterAdjustmentAction::Edit {
            adjustment_key,
            value,
        } => {
            let old = existing(adjustment_key)?;
            (
                adjustment_key.clone(),
                Some(old),
                Some(input(
                    &snapshot,
                    adjustment_key.clone(),
                    value,
                    old.timing.as_ref(),
                )?),
            )
        }
        EncounterAdjustmentAction::End { adjustment_key } => (
            adjustment_key.clone(),
            Some(existing(adjustment_key)?),
            None,
        ),
    };

    let prepared_record = if participant.record_key.is_none() {
        None
    } else {
        let subject = participant.clone();
        let result = service.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let (_, mut records) = hydration::prepare_participant_records(
                    session.retrieval_mut(),
                    std::slice::from_ref(&subject),
                )?;
                let record = subject
                    .record_key
                    .as_ref()
                    .and_then(|key| records.remove(key));
                Ok((record, session.reference_revision()?))
            },
        );
        match result {
            Ok(record) => Some(record),
            Err(error)
                if matches!(
                    error.app_error().code,
                    AppErrorCode::IndexUnavailable | AppErrorCode::ArtifactIncompatible
                ) =>
            {
                None
            }
            Err(error) => return Err(error),
        }
    };
    let reference_revision = prepared_record.as_ref().map(|(_, revision)| *revision);
    let usable = prepared_record
        .as_ref()
        .and_then(|(record, _)| record.as_ref())
        .and_then(|record| {
            Some((
                record.arithmetic_baseline.as_ref()?,
                record.adjustment_applicable,
            ))
        });
    let unknown_baseline = usable.is_some_and(|(_, applicable)| {
        applicable && participant.variant_origin == ParticipantVariantOrigin::InheritedUnknown
    });
    let before = usable.map(|(baseline, applicable)| {
        rules::project_participant(participant, baseline, applicable, &[])
    });
    let targets = before
        .as_ref()
        .map(rules::gm_adjustments::targets)
        .unwrap_or_default();
    if let Some(value) = &value
        && old.is_none_or(|old| old.target != value.target)
        && !targets.iter().any(|choice| choice.target == value.target)
    {
        return Err(AppServiceError::invalid_request(
            "This statistic is unavailable for the participant.",
        ));
    }
    let mut after = participant.clone();
    if let Some(index) = after
        .adjustments
        .iter()
        .position(|entry| entry.adjustment_key == key)
    {
        if let Some(value) = &value {
            after.adjustments[index] = value.clone();
        } else {
            after.adjustments.remove(index);
        }
    } else if let Some(value) = &value {
        after.adjustments.push(value.clone());
    }
    let after = usable.map(|(baseline, applicable)| {
        rules::project_participant(&after, baseline, applicable, &[])
    });

    let mut rows = Vec::new();
    let previous_target = old.map(|old| &old.target);
    let next_target = value.as_ref().map(|new| &new.target);
    for target in previous_target
        .into_iter()
        .filter(|old| Some(*old) != next_target)
        .chain(next_target)
    {
        let observe = |block: Option<&super::stat_block::StatBlockView>| {
            block
                .zip(usable)
                .map(|(block, (authored, _))| {
                    rules::gm_adjustments::observe(block, authored, target, &key)
                })
                .unwrap_or_default()
        };
        let mut after_rows = observe(after.as_ref());
        let mut before_rows = observe(before.as_ref());
        if after_rows.is_empty() && before_rows.is_empty() {
            rows.push(EncounterAdjustmentPreviewRow {
                label: rules::gm_adjustments::target_label(target),
                before: None,
                after: None,
                suppressed: false,
            });
        }
        for from in before_rows.drain(..) {
            let to = after_rows
                .iter()
                .position(|row| row.key == from.key)
                .map(|index| after_rows.remove(index));
            rows.push(EncounterAdjustmentPreviewRow {
                label: from.label,
                before: if unknown_baseline { None } else { from.value },
                after: if unknown_baseline {
                    None
                } else {
                    to.as_ref().and_then(|row| row.value.clone())
                },
                suppressed: Some(target) == next_target && to.is_some_and(|row| row.suppressed),
            });
        }
        rows.extend(
            after_rows
                .into_iter()
                .map(|row| EncounterAdjustmentPreviewRow {
                    label: row.label,
                    before: None,
                    after: if unknown_baseline { None } else { row.value },
                    suppressed: Some(target) == next_target && row.suppressed,
                }),
        );
    }
    Ok(Prepared {
        encounter_key: snapshot.encounter.encounter_key,
        change: AdjustmentChange {
            participant_key: participant_key.to_owned(),
            adjustment_key: key,
            value,
        },
        preview: EncounterAdjustmentPreviewView {
            reference_revision,
            rows,
        },
    })
}

fn input(
    snapshot: &EncounterWithParticipants,
    key: String,
    input: &EncounterAdjustmentInput,
    previous: Option<&AdjustmentTiming>,
) -> AppServiceResult<EncounterAdjustment> {
    let timing = input
        .timing
        .as_ref()
        .map(|input| {
            let mut duration = timing::duration_input(
                Some(input.duration.clone()),
                previous.map(|timing| &timing.duration),
                snapshot,
            )?
            .ok_or_else(|| AppServiceError::invalid_request("Choose an adjustment duration."))?;
            let initial = timing::tracker_progress(&duration)?;
            duration.remaining_seconds = None;
            let progress = match input.remaining {
                Some(remaining) => match initial {
                    TrackerProgress::Rounds(_) => TrackerProgress::Rounds(remaining),
                    TrackerProgress::Seconds(_) => TrackerProgress::Seconds(remaining),
                },
                None => previous
                    .filter(|old| {
                        old.duration.amount == duration.amount && old.duration.unit == duration.unit
                    })
                    .map_or(initial, |old| old.progress.clone()),
            };
            Ok::<_, AppServiceError>(AdjustmentTiming { duration, progress })
        })
        .transpose()?;
    let value = EncounterAdjustment {
        adjustment_key: key,
        name: match input.name.trim() {
            "" => "GM adjustment",
            name => name,
        }
        .to_owned(),
        target: input.target.clone(),
        amount: input.amount,
        modifier_type: input.modifier_type,
        timing,
    };
    if !value.valid() {
        return Err(AppServiceError::invalid_request(
            "Choose a supported stat, a nonzero change, and a valid duration.",
        ));
    }
    Ok(value)
}

pub(super) fn view(
    participant: &EncounterParticipant,
    block: Option<&super::stat_block::StatBlockView>,
    roster: &[EncounterParticipant],
    round_seconds: i64,
) -> Option<EncounterAdjustmentsView> {
    if participant.participant_kind == ParticipantKind::InitiativeEntry {
        return None;
    }
    let targets = block
        .map(rules::gm_adjustments::targets)
        .unwrap_or_default();
    let entries = participant
        .adjustments
        .iter()
        .map(|value| EncounterAdjustmentView {
            adjustment_key: value.adjustment_key.clone(),
            target: value.target.clone(),
            label: rules::gm_adjustments::target_label(&value.target),
            target_available: targets.iter().any(|choice| choice.target == value.target),
            name: value.name.clone(),
            amount: value.amount,
            modifier_type: value.modifier_type,
            timing: value
                .timing
                .as_ref()
                .map(|timing| EncounterAdjustmentTimingView {
                    duration: timing::duration_view(timing.duration.clone(), round_seconds),
                    remaining: match timing.progress {
                        TrackerProgress::Rounds(value) | TrackerProgress::Seconds(value) => value,
                    },
                    anchor_available: timing.duration.timing.as_ref().is_some_and(|anchor| {
                        roster
                            .iter()
                            .any(|row| row.participant_key == anchor.participant_key)
                    }),
                }),
        })
        .collect();
    Some(EncounterAdjustmentsView { targets, entries })
}
