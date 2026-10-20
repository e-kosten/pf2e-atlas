//! GM-selected worsening after a hostile act, scoped to one relationship.
use super::conditions::ConditionRule;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{EncounterMutationView, WorsenEncounterAttitudeRequest};
use atlas_local_state::*;
impl AtlasAppService {
    pub fn worsen_encounter_attitude(
        &self,
        encounter_ref: &str,
        request: WorsenEncounterAttitudeRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"worsen_attitude","request":request});
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
            if request.subject_key == request.other_key {
                return Err(AppServiceError::invalid_request(
                    "Choose another participant",
                ));
            }
            let rows = store.relationships(&snapshot.encounter.encounter_key)?;
            let prior = super::rules::relationships::relationship_knowledge(
                subject,
                &request.other_key,
                RelationshipProperty::Attitude,
                &rows,
            );
            let previous = super::relationship_views::attitude_input(request.previous);
            let attitude = super::relationship_views::attitude_input(request.attitude);
            let rank = |value| match value {
                AttitudeState::Helpful => 0,
                AttitudeState::Friendly => 1,
                AttitudeState::Indifferent => 2,
                AttitudeState::Unfriendly => 3,
                AttitudeState::Hostile => 4,
            };
            if rank(attitude) <= rank(previous) {
                return Err(AppServiceError::invalid_request("Choose a worse attitude"));
            }
            match prior {
                RelationshipKnowledge::Known {
                    value: RelationshipValue::Attitude(current),
                    ..
                } if current == previous => {}
                _ => {
                    return Err(AppServiceError::invalid_request(
                        "Resolve the current attitude first",
                    ));
                }
            }
            let condition_changes = super::rules::relationships::end_condition_scopes(
                &snapshot,
                subject,
                &request.other_key,
                request.expected_encounter_revision,
                &[
                    ConditionRule::Helpful,
                    ConditionRule::Friendly,
                    ConditionRule::Indifferent,
                    ConditionRule::Unfriendly,
                    ConditionRule::Hostile,
                ],
            );
            store.apply_condition_event(
                &snapshot.encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                PreparedConditionEvent {
                    condition_changes,
                    relationship_changes: vec![RelationshipChange::Set(DirectedRelationship {
                        layer: atlas_local_state::RelationshipLayer::Baseline,
                        subject_key: request.subject_key,
                        other_key: request.other_key,
                        property: RelationshipProperty::Attitude,
                        knowledge: RelationshipKnowledge::Known {
                            value: RelationshipValue::Attitude(attitude),
                            provenance: RelationshipProvenance::Table,
                        },
                    })],
                    ..Default::default()
                },
            )?;
            Ok(EncounterMutationView {
                encounter_key: snapshot.encounter.encounter_key,
            })
        })
    }
}
