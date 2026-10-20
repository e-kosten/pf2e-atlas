//! Explicit observer-specific detection transitions; no inferred map geometry.
use super::conditions::ConditionRule;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    ApplyEncounterDetectionEventRequest, DetectionCheckOutcomeView, DetectionEventView,
    EncounterMutationView,
};
use atlas_local_state::*;

impl AtlasAppService {
    pub fn apply_encounter_detection_event(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterDetectionEventRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"detection_event","request":request});
        self.with_write(|session| {
            let store = session.local_state().encounters();
            let snapshot = store
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            if store
                .condition_event_receipt(
                    &snapshot.encounter.encounter_key,
                    &request.request_id,
                    &intent,
                )?
                .is_some()
            {
                return Ok(EncounterMutationView {
                    encounter_key: snapshot.encounter.encounter_key,
                });
            }
            let subject = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == request.subject_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Subject is no longer in this encounter")
                })?;
            if request.subject_key == request.observer_key
                || !snapshot
                    .participants
                    .iter()
                    .any(|p| p.participant_key == request.observer_key)
            {
                return Err(AppServiceError::invalid_request("Choose another observer"));
            }
            let rows = store.relationships(&snapshot.encounter.encounter_key)?;
            let prior = super::rules::relationships::relationship_knowledge(
                subject,
                &request.observer_key,
                RelationshipProperty::Detection,
                &rows,
            );
            let changes = plan_detection_event(&snapshot, subject, &request, &prior)?;
            store.apply_condition_event(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                changes,
            )?;
            Ok(EncounterMutationView {
                encounter_key: snapshot.encounter.encounter_key,
            })
        })
    }
}
fn plan_detection_event(
    snapshot: &EncounterWithParticipants,
    subject: &EncounterParticipant,
    request: &ApplyEncounterDetectionEventRequest,
    prior: &RelationshipKnowledge,
) -> AppServiceResult<PreparedConditionEvent> {
    let causes = super::rules::causes::condition_causes(subject, Some(&request.observer_key));
    let mut invisible_ids: Vec<_> = causes
        .iter()
        .filter(|cause| cause.rule == ConditionRule::Invisible)
        .map(|cause| cause.source.condition_id)
        .collect();
    invisible_ids.sort_unstable();
    let detection = match request.event {
        DetectionEventView::BecameInvisible {
            observed_before,
            can_observe,
        } => {
            if invisible_ids.is_empty() {
                return Err(AppServiceError::invalid_request("Apply Invisible first"));
            }
            Some(if can_observe {
                DetectionState::Observed
            } else if observed_before {
                DetectionState::Hidden
            } else {
                DetectionState::Undetected
            })
        }
        DetectionEventView::BecameAware => None,
        DetectionEventView::Seek {
            outcome,
            prior: input,
            can_observe,
        } => {
            let input = super::relationship_views::detection_input(input);
            if let RelationshipKnowledge::Known {
                value: RelationshipValue::Detection(known),
                ..
            } = prior
                && *known != input
            {
                return Err(AppServiceError::invalid_request(
                    "Detection changed; review the check",
                ));
            }
            match outcome {
                DetectionCheckOutcomeView::Failure | DetectionCheckOutcomeView::CriticalFailure => {
                    return Ok(Default::default());
                }
                DetectionCheckOutcomeView::CriticalSuccess => Some(if can_observe {
                    DetectionState::Observed
                } else {
                    DetectionState::Hidden
                }),
                DetectionCheckOutcomeView::Success => {
                    Some(if input == DetectionState::Undetected || !can_observe {
                        DetectionState::Hidden
                    } else {
                        DetectionState::Observed
                    })
                }
            }
        }
    };
    let mut changes = PreparedConditionEvent::default();
    let rules: &[ConditionRule] = if detection.is_some() {
        &[
            ConditionRule::Hidden,
            ConditionRule::Undetected,
            ConditionRule::Unnoticed,
        ]
    } else {
        &[ConditionRule::Unnoticed]
    };
    changes.condition_changes = super::rules::relationships::end_condition_scopes(
        snapshot,
        subject,
        &request.observer_key,
        request.expected_encounter_revision,
        rules,
    );
    let mut after = subject.clone();
    for change in &changes.condition_changes {
        match change {
            ConditionEventChange::Remove { condition_id, .. } => after
                .conditions
                .retain(|source| source.condition_id != *condition_id),
            ConditionEventChange::Replace(update) => {
                if let Some(source) = after
                    .conditions
                    .iter_mut()
                    .find(|source| source.condition_id == update.condition_id)
                {
                    source.scope = update.scope.clone();
                }
            }
            ConditionEventChange::Add(_) => {}
        }
    }
    let mut set = |property, value| {
        let sources = super::rules::relationships::relationship_source_snapshot(
            &after,
            &request.observer_key,
            property,
        );
        let (layer, provenance) = if sources.is_empty() {
            (RelationshipLayer::Baseline, RelationshipProvenance::Table)
        } else {
            (
                RelationshipLayer::Adjudication,
                RelationshipProvenance::Adjudication { sources },
            )
        };
        changes
            .relationship_changes
            .push(RelationshipChange::Set(DirectedRelationship {
                layer,
                subject_key: subject.participant_key.clone(),
                other_key: request.observer_key.clone(),
                property,
                knowledge: RelationshipKnowledge::Known { value, provenance },
            }));
    };
    if let Some(detection) = detection {
        set(
            RelationshipProperty::Detection,
            RelationshipValue::Detection(detection),
        );
    } else if matches!(
        prior,
        RelationshipKnowledge::Known {
            value: RelationshipValue::Detection(DetectionState::Undetected),
            ..
        }
    ) {
        set(
            RelationshipProperty::Detection,
            RelationshipValue::Detection(DetectionState::Undetected),
        );
    }
    let aware = match request.event {
        DetectionEventView::BecameInvisible {
            observed_before,
            can_observe,
        } => observed_before || can_observe,
        _ => true,
    };
    // Becoming invisible does not erase an observer's prior awareness.
    if aware {
        set(
            RelationshipProperty::Awareness,
            RelationshipValue::Awareness(true),
        );
    }
    Ok(changes)
}
