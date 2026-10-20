use super::conditions::{ConditionRule, modeled_condition_by_rule};
use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;
use atlas_local_state::{AddEncounterParticipantCondition, EncounterWithParticipants};

fn fixture() -> (FixtureWorker, String, Vec<String>) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Relationships".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    for name in ["Subject", "Observer A", "Observer B"] {
        f.worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: key.clone(),
                display_name: name.into(),
                max_hp: Some(20),
                current_hp: Some(20),
                initiative: None,
            })
            .unwrap();
    }
    let participants = snapshot(&f, &key)
        .participants
        .into_iter()
        .map(|p| p.participant_key)
        .collect();
    (f, key, participants)
}
fn snapshot(f: &FixtureWorker, key: &str) -> EncounterWithParticipants {
    f.worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(key)?)
        })
        .unwrap()
        .unwrap()
}
fn add(f: &FixtureWorker, subject: &str, rule: ConditionRule) {
    let definition = modeled_condition_by_rule(rule).unwrap();
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .add_condition(AddEncounterParticipantCondition {
                    participant_key: subject.into(),
                    condition_key: Some(definition.condition_ref.into()),
                    name: definition.name.into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: if matches!(
                        rule,
                        ConditionRule::Friendly | ConditionRule::Helpful | ConditionRule::Hostile
                    ) {
                        atlas_local_state::ConditionParameters::Attitude { supernatural: true }
                    } else {
                        Default::default()
                    },
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            Ok(())
        })
        .unwrap();
}
fn knowledge(
    f: &FixtureWorker,
    key: &str,
    subject: &str,
    observer: &str,
    property: RelationshipPropertyView,
) -> RelationshipKnowledgeView {
    f.worker
        .encounter_relationships(key, subject, observer)
        .unwrap()
        .relationships
        .into_iter()
        .find(|r| r.property == property)
        .unwrap()
        .knowledge
}
#[test]
fn awareness_ends_only_one_observers_unnoticed_without_revealing_location_and_retries_once() {
    let (f, key, p) = fixture();
    add(&f, &p[0], ConditionRule::Unnoticed);
    let request = ApplyEncounterDetectionEventRequest {
        request_id: "heard-subject".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        subject_key: p[0].clone(),
        observer_key: p[1].clone(),
        event: DetectionEventView::BecameAware,
    };
    f.worker
        .apply_encounter_detection_event(&key, request.clone())
        .unwrap();
    let revision = snapshot(&f, &key).encounter.revision;
    f.worker
        .apply_encounter_detection_event(&key, request)
        .unwrap();
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Awareness),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Awareness(true),
            ..
        }
    ));
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Undetected),
            ..
        }
    ));
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[2], RelationshipPropertyView::Awareness),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Awareness(false),
            ..
        }
    ));
}
#[test]
fn seek_preserves_imprecise_limit_and_stale_retry_does_not_rewrite_another_observer() {
    let (f, key, p) = fixture();
    add(&f, &p[0], ConditionRule::Undetected);
    let request = ApplyEncounterDetectionEventRequest {
        request_id: "seek-success".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        subject_key: p[0].clone(),
        observer_key: p[1].clone(),
        event: DetectionEventView::Seek {
            outcome: DetectionCheckOutcomeView::CriticalSuccess,
            prior: DetectionStateView::Undetected,
            can_observe: false,
        },
    };
    f.worker
        .apply_encounter_detection_event(&key, request)
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Hidden),
            ..
        }
    ));
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[2], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Undetected),
            ..
        }
    ));
    let bad = ApplyEncounterDetectionEventRequest {
        request_id: "stale-prior".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        subject_key: p[0].clone(),
        observer_key: p[1].clone(),
        event: DetectionEventView::Seek {
            outcome: DetectionCheckOutcomeView::Success,
            prior: DetectionStateView::Undetected,
            can_observe: true,
        },
    };
    assert!(f.worker.apply_encounter_detection_event(&key, bad).is_err());
}
#[test]
fn newly_invisible_requires_fresh_observer_assessment_and_keeps_independent_source() {
    let (f, key, p) = fixture();
    f.worker
        .set_encounter_relationship(
            &key,
            SetEncounterRelationshipRequest {
                layer: atlas_app_model::RelationshipLayerView::Baseline,
                request_id: "observed".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                subject_key: p[0].clone(),
                other_key: p[1].clone(),
                property: RelationshipPropertyView::Detection,
                knowledge: Some(RelationshipKnowledgeView::Known {
                    value: RelationshipValueView::Detection(DetectionStateView::Observed),
                    provenance: RelationshipProvenanceView::Table,
                }),
            },
        )
        .unwrap();
    add(&f, &p[0], ConditionRule::Invisible);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Unknown { .. }
    ));
    f.worker
        .apply_encounter_detection_event(
            &key,
            ApplyEncounterDetectionEventRequest {
                request_id: "vanished".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                subject_key: p[0].clone(),
                observer_key: p[1].clone(),
                event: DetectionEventView::BecameInvisible {
                    observed_before: true,
                    can_observe: false,
                },
            },
        )
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Hidden),
            ..
        }
    ));
    assert_eq!(snapshot(&f, &key).participants[0].conditions.len(), 1);
}

#[test]
fn hostile_act_worsens_only_its_pair_and_rejects_improvement() {
    let (f, key, p) = fixture();
    add(&f, &p[0], ConditionRule::Friendly);
    let request = WorsenEncounterAttitudeRequest {
        request_id: "hostile-act".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        subject_key: p[0].clone(),
        other_key: p[1].clone(),
        previous: AttitudeStateView::Friendly,
        attitude: AttitudeStateView::Unfriendly,
    };
    f.worker
        .worsen_encounter_attitude(&key, request.clone())
        .unwrap();
    let revision = snapshot(&f, &key).encounter.revision;
    f.worker.worsen_encounter_attitude(&key, request).unwrap();
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(AttitudeStateView::Unfriendly),
            ..
        }
    ));
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[2], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(AttitudeStateView::Friendly),
            ..
        }
    ));
    assert!(
        f.worker
            .worsen_encounter_attitude(
                &key,
                WorsenEncounterAttitudeRequest {
                    request_id: "not-worse".into(),
                    expected_encounter_revision: revision,
                    subject_key: p[0].clone(),
                    other_key: p[1].clone(),
                    previous: AttitudeStateView::Unfriendly,
                    attitude: AttitudeStateView::Helpful
                }
            )
            .is_err()
    );
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
}

#[test]
fn adjudication_preserves_baseline_sources_and_other_observers_then_invalidates_on_source_change() {
    let (f, key, p) = fixture();
    let set = |layer, attitude, request_id: &str| SetEncounterRelationshipRequest {
        layer,
        request_id: request_id.into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        subject_key: p[0].clone(),
        other_key: p[1].clone(),
        property: RelationshipPropertyView::Attitude,
        knowledge: Some(RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(attitude),
            provenance: RelationshipProvenanceView::Table,
        }),
    };
    f.worker
        .set_encounter_relationship(
            &key,
            set(
                RelationshipLayerView::Baseline,
                AttitudeStateView::Indifferent,
                "baseline",
            ),
        )
        .unwrap();
    add(&f, &p[0], ConditionRule::Friendly);
    add(&f, &p[0], ConditionRule::Hostile);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Conflicting { .. }
    ));
    let request = set(
        RelationshipLayerView::Adjudication,
        AttitudeStateView::Friendly,
        "adjudicate",
    );
    f.worker
        .set_encounter_relationship(&key, request.clone())
        .unwrap();
    let revision = snapshot(&f, &key).encounter.revision;
    f.worker.set_encounter_relationship(&key, request).unwrap();
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
    assert_eq!(snapshot(&f, &key).participants[0].conditions.len(), 2);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(AttitudeStateView::Friendly),
            ..
        }
    ));
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[2], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Conflicting { .. }
    ));
    let rows = f
        .worker
        .encounter_relationships(&key, &p[0], &p[1])
        .unwrap();
    let row = rows
        .relationships
        .iter()
        .find(|row| row.property == RelationshipPropertyView::Attitude)
        .unwrap();
    assert!(matches!(
        row.baseline,
        Some(RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(AttitudeStateView::Indifferent),
            ..
        })
    ));
    let ids: Vec<_> = snapshot(&f, &key).participants[0]
        .conditions
        .iter()
        .map(|source| source.condition_id)
        .collect();
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&p[0], ids[0])?;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Invalidated { .. }
    ));
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&p[0], ids[1])?;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Attitude),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Attitude(AttitudeStateView::Indifferent),
            ..
        }
    ));
}

#[test]
fn invisible_assessment_restores_existing_baseline_when_source_ends() {
    let (f, key, p) = fixture();
    f.worker
        .set_encounter_relationship(
            &key,
            SetEncounterRelationshipRequest {
                layer: RelationshipLayerView::Baseline,
                request_id: "usual-detection".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                subject_key: p[0].clone(),
                other_key: p[1].clone(),
                property: RelationshipPropertyView::Detection,
                knowledge: Some(RelationshipKnowledgeView::Known {
                    value: RelationshipValueView::Detection(DetectionStateView::Observed),
                    provenance: RelationshipProvenanceView::Table,
                }),
            },
        )
        .unwrap();
    add(&f, &p[0], ConditionRule::Invisible);
    f.worker
        .apply_encounter_detection_event(
            &key,
            ApplyEncounterDetectionEventRequest {
                request_id: "now-invisible".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                subject_key: p[0].clone(),
                observer_key: p[1].clone(),
                event: DetectionEventView::BecameInvisible {
                    observed_before: true,
                    can_observe: false,
                },
            },
        )
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Hidden),
            ..
        }
    ));
    let id = snapshot(&f, &key).participants[0].conditions[0].condition_id;
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&p[0], id)?;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Detection),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::Detection(DetectionStateView::Observed),
            ..
        }
    ));
}

#[test]
fn controller_adjudication_keeps_competing_sources_and_invalidates_when_instructions_change() {
    let (f, key, p) = fixture();
    let definition = modeled_condition_by_rule(ConditionRule::Controlled).unwrap();
    for controller in [&p[1], &p[2]] {
        f.worker
            .with_write(|session| {
                session.local_state().encounters().add_condition(
                    AddEncounterParticipantCondition {
                        participant_key: p[0].clone(),
                        condition_key: Some(definition.condition_ref.into()),
                        name: definition.name.into(),
                        value: None,
                        source_participant_key: Some(controller.clone()),
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: Some("Stay here".into()),
                    },
                )?;
                Ok(())
            })
            .unwrap();
    }
    f.worker
        .set_encounter_relationship(
            &key,
            SetEncounterRelationshipRequest {
                layer: RelationshipLayerView::Adjudication,
                request_id: "choose-controller".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                subject_key: p[0].clone(),
                other_key: p[1].clone(),
                property: RelationshipPropertyView::Control,
                knowledge: Some(RelationshipKnowledgeView::Known {
                    value: RelationshipValueView::Control {
                        controller_key: p[1].clone(),
                        instructions: Some("Stay here".into()),
                    },
                    provenance: RelationshipProvenanceView::Table,
                }),
            },
        )
        .unwrap();
    assert_eq!(snapshot(&f, &key).participants[0].conditions.len(), 2);
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[2], RelationshipPropertyView::Control),
        RelationshipKnowledgeView::Known {
            value: RelationshipValueView::NoControl,
            ..
        }
    ));
    let current = snapshot(&f, &key);
    let source = &current.participants[0].conditions[0];
    f.worker
        .with_write(|session| {
            session.local_state().encounters().update_condition(
                atlas_local_state::UpdateEncounterParticipantCondition {
                    expected_encounter_revision: current.encounter.revision,
                    participant_key: p[0].clone(),
                    condition_id: source.condition_id,
                    condition_key: source.condition_key.clone(),
                    name: source.name.clone(),
                    value: source.value,
                    source_participant_key: source.source_participant_key.clone(),
                    scope: source.scope.clone(),
                    extra_action: source.extra_action.clone(),
                    immunity_override: source.immunity_override,
                    parameters: source.parameters.clone(),
                    origin: source.origin.clone(),
                    duration: source.duration.clone(),
                    note: Some("Run away".into()),
                },
            )?;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        knowledge(&f, &key, &p[0], &p[1], RelationshipPropertyView::Control),
        RelationshipKnowledgeView::Invalidated { .. }
    ));
}
