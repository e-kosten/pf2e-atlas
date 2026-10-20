//! Directed relationship queries and receipt-backed baseline edits.
use super::relationship_views::{knowledge_input, knowledge_view, property_input, property_view};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    EncounterMutationView, EncounterRelationshipView, EncounterRelationshipsView,
    SetEncounterRelationshipRequest,
};
use atlas_local_state::{
    DirectedRelationship, PreparedConditionEvent, RelationshipChange, RelationshipProperty,
};

impl AtlasAppService {
    pub fn encounter_relationships(
        &self,
        encounter_ref: &str,
        subject_key: &str,
        other_key: &str,
    ) -> AppServiceResult<EncounterRelationshipsView> {
        self.with_state(|session| {
            let store = session.local_state().encounters();
            let snapshot = store
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let subject = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == subject_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request("Subject is no longer in this encounter")
                })?;
            if subject_key == other_key
                || !snapshot
                    .participants
                    .iter()
                    .any(|p| p.participant_key == other_key)
            {
                return Err(AppServiceError::invalid_request(
                    "Choose another encounter participant",
                ));
            }
            let saved = store.relationships(&snapshot.encounter.encounter_key)?;
            let relationships = [
                RelationshipProperty::Detection,
                RelationshipProperty::Awareness,
                RelationshipProperty::Attitude,
                RelationshipProperty::Control,
            ]
            .into_iter()
            .map(|property| {
                let baseline = saved.iter().find(|row| {
                    row.subject_key == subject_key
                        && row.other_key == other_key
                        && row.property == property
                        && row.layer == atlas_local_state::RelationshipLayer::Baseline
                });
                EncounterRelationshipView {
                    subject_key: subject_key.into(),
                    other_key: other_key.into(),
                    property: property_view(property),
                    baseline: baseline.map(|row| knowledge_view(row.knowledge.clone())),
                    adjudication: saved
                        .iter()
                        .find(|row| {
                            row.subject_key == subject_key
                                && row.other_key == other_key
                                && row.property == property
                                && row.layer == atlas_local_state::RelationshipLayer::Adjudication
                        })
                        .map(|row| knowledge_view(row.knowledge.clone())),
                    source_condition_ids: super::rules::relationships::relationship_source_ids(
                        subject, other_key, property,
                    ),
                    knowledge: knowledge_view(super::rules::relationships::relationship_knowledge(
                        subject, other_key, property, &saved,
                    )),
                }
            })
            .collect();
            Ok(EncounterRelationshipsView {
                encounter_key: snapshot.encounter.encounter_key,
                expected_revision: snapshot.encounter.revision,
                relationships,
            })
        })
    }
    pub fn set_encounter_relationship(
        &self,
        encounter_ref: &str,
        request: SetEncounterRelationshipRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent = serde_json::json!({"action":"set_relationship","request":request});
        self.with_write(|session| {
            let store = session.local_state().encounters();
            let encounter = store
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            if store
                .condition_event_receipt(&encounter.encounter_key, &request.request_id, &intent)?
                .is_some()
            {
                return Ok(EncounterMutationView {
                    encounter_key: encounter.encounter_key,
                });
            }
            if request.subject_key == request.other_key {
                return Err(AppServiceError::invalid_request(
                    "Choose another encounter participant",
                ));
            }
            let property = property_input(request.property);
            let layer = match request.layer {
                atlas_app_model::RelationshipLayerView::Baseline => {
                    atlas_local_state::RelationshipLayer::Baseline
                }
                atlas_app_model::RelationshipLayerView::Adjudication => {
                    atlas_local_state::RelationshipLayer::Adjudication
                }
            };
            let mut knowledge = request.knowledge.map(knowledge_input);
            if layer == atlas_local_state::RelationshipLayer::Baseline
                && matches!(
                    &knowledge,
                    Some(atlas_local_state::RelationshipKnowledge::Known {
                        value: atlas_local_state::RelationshipValue::Control { .. },
                        ..
                    })
                )
            {
                return Err(AppServiceError::invalid_request(
                    "Apply Controlled before choosing a controller",
                ));
            }
            if layer == atlas_local_state::RelationshipLayer::Adjudication && knowledge.is_some() {
                let snapshot = store
                    .get_with_participants(&encounter.encounter_key)?
                    .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
                let subject = snapshot
                    .participants
                    .iter()
                    .find(|p| p.participant_key == request.subject_key)
                    .ok_or_else(|| {
                        AppServiceError::invalid_request("Subject is no longer in this encounter")
                    })?;
                let sources = super::rules::relationships::relationship_source_snapshot(
                    subject,
                    &request.other_key,
                    property,
                );
                if sources.is_empty() {
                    return Err(AppServiceError::invalid_request(
                        "No active sources to adjudicate",
                    ));
                }
                if property == RelationshipProperty::Attitude
                    && subject.participant_kind == atlas_local_state::ParticipantKind::Pc
                    && sources.iter().any(|source| {
                        !matches!(
                            source.parameters,
                            atlas_local_state::ConditionParameters::Attitude { supernatural: true }
                        )
                    })
                {
                    return Err(AppServiceError::invalid_request(
                        "Confirm the supernatural attitude sources first",
                    ));
                }
                match &mut knowledge {
                    Some(atlas_local_state::RelationshipKnowledge::Known {
                        provenance, ..
                    }) => {
                        *provenance =
                            atlas_local_state::RelationshipProvenance::Adjudication { sources }
                    }
                    _ => {
                        return Err(AppServiceError::invalid_request(
                            "Choose the adjudicated relationship",
                        ));
                    }
                }
            }
            let change = match knowledge {
                Some(knowledge) => RelationshipChange::Set(DirectedRelationship {
                    layer,
                    subject_key: request.subject_key,
                    other_key: request.other_key,
                    property,
                    knowledge,
                }),
                None => RelationshipChange::Remove {
                    layer,
                    subject_key: request.subject_key,
                    other_key: request.other_key,
                    property,
                },
            };
            let mut relationship_changes = vec![change];
            if let Some(RelationshipChange::Set(row)) = relationship_changes.first()
                && row.layer == atlas_local_state::RelationshipLayer::Adjudication
                && let atlas_local_state::RelationshipKnowledge::Known {
                    value: atlas_local_state::RelationshipValue::Control { controller_key, .. },
                    provenance: atlas_local_state::RelationshipProvenance::Adjudication { sources },
                } = &row.knowledge
            {
                let others: std::collections::BTreeSet<_> = sources
                    .iter()
                    .filter_map(|source| source.source_participant_key.as_ref())
                    .filter(|key| *key != controller_key)
                    .cloned()
                    .collect();
                let additional: Vec<_> = others
                    .into_iter()
                    .map(|other_key| {
                        RelationshipChange::Set(DirectedRelationship {
                            layer: row.layer,
                            subject_key: row.subject_key.clone(),
                            other_key,
                            property: row.property,
                            knowledge: atlas_local_state::RelationshipKnowledge::Known {
                                value: atlas_local_state::RelationshipValue::NoControl,
                                provenance:
                                    atlas_local_state::RelationshipProvenance::Adjudication {
                                        sources: sources.clone(),
                                    },
                            },
                        })
                    })
                    .collect();
                relationship_changes.extend(additional);
            }
            store.apply_condition_event(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                PreparedConditionEvent {
                    relationship_changes,
                    ..Default::default()
                },
            )?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}
