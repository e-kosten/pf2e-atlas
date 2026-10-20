use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;
use atlas_local_state::{
    AddEncounterParticipantCondition, ConditionParameters, DamageMultiplier, PersistentRecovery,
};

fn fixture() -> (FixtureWorker, String, String) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Persistent damage".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "PC".into(),
            max_hp: Some(20),
            current_hp: Some(5),
            initiative: Some(10),
        })
        .unwrap();
    let participant = snapshot(&f, &key).participants[0].participant_key.clone();
    f.worker
        .with_write(|session| {
            let store = session.local_state().encounters();
            for (formula, damage_type) in [("4", "fire"), ("2", "fire"), ("3", "acid")] {
                store.add_condition(AddEncounterParticipantCondition {
                    participant_key: participant.clone(),
                    condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                    name: "Persistent Damage".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: ConditionParameters::PersistentDamage {
                        formula: formula.into(),
                        damage_type: damage_type.into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: format!("episode-{damage_type}"),
                    },
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            }
            Ok(())
        })
        .unwrap();
    (f, key, participant)
}

fn snapshot(f: &FixtureWorker, key: &str) -> atlas_local_state::EncounterWithParticipants {
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

fn batch(f: &FixtureWorker, key: &str, participant: &str) -> ApplyPersistentDamageRequest {
    ApplyPersistentDamageRequest {
        boundary_key: None,
        request_id: "damage-once".into(),
        expected_encounter_revision: snapshot(f, key).encounter.revision,
        participant_key: participant.into(),
        artifact_generation: None,
        groups: vec![
            PersistentDamageGroupInput {
                episode_id: "episode-fire".into(),
                selected_source_id: None,
                damage: PersistentDamageInput::TableResult { amount: 4 },
            },
            PersistentDamageGroupInput {
                episode_id: "episode-acid".into(),
                selected_source_id: None,
                damage: PersistentDamageInput::TableResult { amount: 3 },
            },
        ],
        context: HpEventContextView {
            keep_current_initiative: Some(true),
            lethality: Some(DamageLethalityView::Lethal),
            critical: Some(false),
            enemy_source: Some(true),
            responsible_participant_key: None,
            base_death_threshold: Some(4),
            ..Default::default()
        },
    }
}

#[test]
fn simultaneous_types_enter_dying_once_and_retry_does_not_repeat_damage() {
    let (f, key, participant) = fixture();
    let request = batch(&f, &key, &participant);
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!(
        result.groups.iter().map(|group| group.amount).sum::<i64>(),
        7
    );
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].current_hp, Some(0));
    let dying: Vec<_> = after.participants[0]
        .conditions
        .iter()
        .filter(|condition| condition.name == "Dying")
        .collect();
    assert_eq!(dying.len(), 1);
    assert_eq!(dying[0].value, Some(1));
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key), after);
}

#[test]
fn invalid_group_or_stale_revision_is_atomic() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    let mut request = batch(&f, &key, &participant);
    request.groups.pop();
    assert!(f.worker.apply_persistent_damage(&key, request).is_err());
    assert_eq!(snapshot(&f, &key), before);
    let mut request = batch(&f, &key, &participant);
    request.expected_encounter_revision -= 1;
    assert!(f.worker.apply_persistent_damage(&key, request).is_err());
    assert_eq!(snapshot(&f, &key), before);
}

#[test]
fn a_fresh_request_id_cannot_apply_the_same_boundary_twice() {
    let (f, key, participant) = fixture();
    let result = f
        .worker
        .apply_persistent_damage(&key, batch(&f, &key, &participant))
        .unwrap();
    let after = snapshot(&f, &key);
    let mut second = batch(&f, &key, &participant);
    second.request_id = "different-request-same-boundary".into();
    assert!(f.worker.apply_persistent_damage(&key, second).is_err());
    assert_eq!(snapshot(&f, &key), after);
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert!(review.damage_applied);
    assert_eq!(review.damage_result, Some(result));
    assert!(
        review
            .episodes
            .iter()
            .all(|episode| episode.end_turn_recovery_available)
    );
}

#[test]
fn end_turn_recovery_requires_damage_but_explicit_recovery_is_not_turn_limited() {
    let (f, key, participant) = fixture();
    let mut request = RecoverPersistentDamageRequest {
        boundary_key: None,
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "recovery-gate".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        selected_source_id: None,
        outcome: PersistentRecoveryInput::TableResult { result: 1 },
        dc_override: None,
    };
    assert!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .is_err()
    );
    request.mode = PersistentRecoveryMode::Assisted;
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(!result.success);
    assert_eq!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    request.request_id = "second-assistance".into();
    request.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    let second = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(!second.success);
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        second
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert!(review.recovery_results.is_empty());
    assert!(
        review
            .episodes
            .iter()
            .find(|episode| episode.episode_id == "episode-fire")
            .unwrap()
            .assisted_recovery_available
    );
}

#[test]
fn typed_protections_preserve_missing_hardness_and_source_exceptions() {
    let (f, key, _) = fixture();
    let participant = &snapshot(&f, &key).participants[0];
    let record = |kind: &str, attributes: serde_json::Value| {
        atlas_search::test_support::record("actors","Actor",serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":kind,"name":"Protection","system":{"attributes":attributes}})).record
    };
    let npc = record(
        "npc",
        serde_json::json!({"weaknesses":[{"type":"fire","value":5}],"resistances":[{"type":"fire","value":3}]}),
    );
    let protections =
        super::rules::damage::persistent_protection(Some(&npc), participant, "fire").unwrap();
    assert_eq!(protections.apply(10), 12);
    let hazard = record("hazard", serde_json::json!({}));
    assert!(
        super::rules::damage::persistent_protection(Some(&hazard), participant, "fire").is_err()
    );
    let conditional = record(
        "npc",
        serde_json::json!({"resistances":[{"type":"fire","value":3,"exceptions":["magical"]}]}),
    );
    assert!(
        super::rules::damage::persistent_protection(Some(&conditional), participant, "fire")
            .is_err()
    );
}

#[test]
fn petrified_hardness_reduces_the_simultaneous_event_once() {
    let (f, key, _) = fixture();
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "actors:testCreature1000".into(),
            quantity: 1,
            initiative: Some(8),
        })
        .unwrap();
    let before = snapshot(&f, &key);
    let statue = before
        .participants
        .iter()
        .find(|participant| participant.record_key.is_some())
        .unwrap();
    f.worker
        .with_write(|session| {
            let store = session.local_state().encounters();
            for (condition_key, parameters) in [
                ("conditionitems:dTwPJuKgBQCMxixg", ConditionParameters::None),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "fire".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "fire-statue".into(),
                    },
                ),
                (
                    "conditionitems:lDVqvLKA6eF3Df60",
                    ConditionParameters::PersistentDamage {
                        formula: "10".into(),
                        damage_type: "acid".into(),
                        multiplier: DamageMultiplier::Normal,
                        recovery: PersistentRecovery::Standard,
                        episode_id: "acid-statue".into(),
                    },
                ),
            ] {
                store.add_condition(AddEncounterParticipantCondition {
                    participant_key: statue.participant_key.clone(),
                    condition_key: Some(condition_key.into()),
                    name: "Fixture".into(),
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters,
                    origin: None,
                    duration: None,
                    note: None,
                })?;
            }
            Ok(())
        })
        .unwrap();
    let request = ApplyPersistentDamageRequest {
        boundary_key: None,
        request_id: "statue-damage".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: statue.participant_key.clone(),
        artifact_generation: f.worker.encounter(&key).unwrap().artifact_generation,
        groups: ["fire-statue", "acid-statue"]
            .into_iter()
            .map(|episode| PersistentDamageGroupInput {
                episode_id: episode.into(),
                selected_source_id: None,
                damage: PersistentDamageInput::Roll,
            })
            .collect(),
        context: Default::default(),
    };
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!(
        result.groups.iter().map(|group| group.amount).sum::<i64>(),
        20
    );
    assert_eq!((result.total, result.hardness_absorbed), (12, 8));
    assert_eq!(
        snapshot(&f, &key)
            .participants
            .iter()
            .find(|participant| participant.participant_key == statue.participant_key)
            .unwrap()
            .current_hp,
        Some(48)
    );
}

#[test]
fn recovery_clears_suppressed_sources_and_recovers_receipt_after_episode_is_gone() {
    let (f, key, participant) = fixture();
    let request = RecoverPersistentDamageRequest {
        boundary_key: None,
        mode: PersistentRecoveryMode::Assisted,
        request_id: "recover-fire".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant,
        episode_id: "episode-fire".into(),
        selected_source_id: None,
        outcome: PersistentRecoveryInput::TableResult { result: 20 },
        dc_override: None,
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(result.success);
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].conditions.len(), 1);
    assert!(
        matches!(&after.participants[0].conditions[0].parameters,ConditionParameters::PersistentDamage {damage_type,..} if damage_type=="acid")
    );
    assert_eq!(after.participants[0].current_hp, Some(5));
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        result
    );
    assert_eq!(snapshot(&f, &key), after);
}

fn outgoing_boundary(f: &FixtureWorker, key: &str, participant: &str) -> String {
    f.worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: key.into(),
            participant_key: participant.into(),
        })
        .unwrap();
    f.worker
        .advance_encounter_turn(
            key,
            AdvanceEncounterTurnRequest {
                request_id: "advance-outgoing".into(),
                expected_revision: snapshot(f, key).encounter.revision,
            },
        )
        .unwrap();
    f.worker
        .encounter_turn_review(key)
        .unwrap()
        .outgoing
        .unwrap()
        .boundary_key
        .clone()
}
#[test]
fn late_table_recovery_never_clears_a_new_same_type_episode() {
    let (f, key, participant) = fixture();
    let boundary = outgoing_boundary(&f, &key, &participant);
    let old=snapshot(&f,&key).participants[0].conditions.iter().filter(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type=="fire")).map(|c|c.condition_id).collect::<Vec<_>>();
    f.worker
        .with_write(|session| {
            let store = session.local_state().encounters();
            for id in old {
                store.remove_condition(&participant, id)?;
            }
            store.add_condition(AddEncounterParticipantCondition {
                participant_key: participant.clone(),
                condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                name: "Persistent Damage".into(),
                value: None,
                source_participant_key: None,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: ConditionParameters::PersistentDamage {
                    formula: "99".into(),
                    damage_type: "fire".into(),
                    multiplier: DamageMultiplier::Normal,
                    recovery: PersistentRecovery::SourceDependent,
                    episode_id: "new-fire".into(),
                },
                origin: None,
                duration: None,
                note: None,
            })?;
            Ok(())
        })
        .unwrap();
    let request = RecoverPersistentDamageRequest {
        boundary_key: Some(boundary),
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "late-success".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        selected_source_id: None,
        outcome: PersistentRecoveryInput::TableSuccess,
        dc_override: None,
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(result.success);
    assert!(result.roll.is_none());
    assert!(result.table_result.is_none());
    assert_eq!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    let state = snapshot(&f, &key);
    assert!(state.participants[0].conditions.iter().any(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{episode_id,..} if episode_id=="new-fire")));
    assert_eq!(state.participants[0].current_hp, Some(5));
    assert!(
        f.worker
            .recover_persistent_damage(
                &key,
                RecoverPersistentDamageRequest {
                    request_id: "duplicate-claim".into(),
                    expected_encounter_revision: state.encounter.revision,
                    ..request
                }
            )
            .is_err()
    );
    let manual = RecoverPersistentDamageRequest {
        boundary_key: None,
        mode: PersistentRecoveryMode::Assisted,
        request_id: "unknown-dc-success".into(),
        expected_encounter_revision: state.encounter.revision,
        participant_key: participant.clone(),
        episode_id: "new-fire".into(),
        selected_source_id: None,
        outcome: PersistentRecoveryInput::TableSuccess,
        dc_override: None,
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, manual.clone())
        .unwrap();
    assert!(result.success);
    assert_eq!(result.dc, None);
    assert!(result.roll.is_none());
    assert!(result.table_result.is_none());
    assert_eq!(
        f.worker.recover_persistent_damage(&key, manual).unwrap(),
        result
    );
}
#[test]
fn late_damage_uses_frozen_due_formulas_and_not_new_episode_values() {
    let (f, key, participant) = fixture();
    let boundary = outgoing_boundary(&f, &key, &participant);
    let mut request = batch(&f, &key, &participant);
    request.boundary_key = Some(boundary.clone());
    let due = f
        .worker
        .persistent_damage_review(&key, &participant, Some(&boundary))
        .unwrap();
    assert_eq!(due.round_number, 1);
    assert_eq!(snapshot(&f, &key).encounter.round_number, 2);
    let fire=snapshot(&f,&key).participants[0].conditions.iter().find(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type=="fire")).unwrap().condition_id;
    f.worker
        .with_write(|session| {
            session
                .local_state()
                .encounters()
                .remove_condition(&participant, fire)?;
            Ok(())
        })
        .unwrap();
    let frozen = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap()
        .clone();
    assert!(
        frozen
            .persistent_damage_groups
            .iter()
            .flat_map(|g| &g.sources)
            .any(|source| source.condition_id == fire && source.formula == "4")
    );
    request.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!(result.total, 7);
    assert_eq!(
        f.worker.apply_persistent_damage(&key, request).unwrap(),
        result
    );
    assert!(
        f.worker
            .persistent_damage_review(&key, &participant, Some(&boundary))
            .unwrap()
            .damage_applied
    );
}

#[test]
fn partial_persistent_reminder_round_trips_without_joining_automation_then_can_be_completed() {
    let (f, key, participant) = fixture();
    let initial = f.worker.encounter(&key).unwrap();
    f.worker
        .add_encounter_participant_condition(
            &key,
            AddEncounterParticipantConditionRequest {
                request_id: "reminder-add".into(),
                expected_encounter_revision: initial.revision,
                participant_key: participant.clone(),
                condition_ref: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
                name: None,
                value: None,
                source_participant_key: None,
                duration: None,
                note: Some("Burning until the blessing ends".into()),
                event_intent: Default::default(),
                decisions: Default::default(),
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                origin: None,
                parameters: ConditionParametersView::PersistentDamageReminder {
                    formula: None,
                    damage_type: Some("fire".into()),
                    multiplier: DamageMultiplierView::Normal,
                    recovery: PersistentRecoveryView::Standard,
                },
            },
        )
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let actor = &detail.participants[0];
    assert_eq!(
        actor.persistent_damage_groups,
        initial.participants[0].persistent_damage_groups
    );
    let reminder = actor
        .conditions
        .iter()
        .find(|c| {
            matches!(
                c.parameters,
                ConditionParametersView::PersistentDamageReminder { .. }
            )
        })
        .unwrap();
    let saved = snapshot(&f, &key);
    assert!(
        matches!(&saved.participants[0].conditions.iter().find(|c| c.condition_id == reminder.condition_id).unwrap().parameters,
        ConditionParameters::PersistentDamageReminder { formula: None, damage_type: Some(kind), .. } if kind == "fire")
    );
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            UpdateEncounterParticipantConditionRequest {
                request_id: "reminder-complete".into(),
                expected_encounter_revision: detail.revision,
                condition_id: reminder.condition_id,
                condition_ref: reminder.condition_key.clone(),
                name: reminder.name.clone(),
                value: None,
                source_participant_key: None,
                duration: None,
                note: reminder.note.clone(),
                event_intent: Default::default(),
                decisions: Default::default(),
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                origin: None,
                parameters: ConditionParametersView::PersistentDamage {
                    formula: "1d6".into(),
                    damage_type: "fire".into(),
                    multiplier: DamageMultiplierView::Normal,
                    recovery: PersistentRecoveryView::Standard,
                    episode_id: None,
                },
            },
        )
        .unwrap();
    let completed = f.worker.encounter(&key).unwrap();
    let group = completed.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap();
    assert_eq!(group.sources.len(), 3);
    assert!(
        group
            .sources
            .iter()
            .any(|s| s.condition_id == reminder.condition_id)
    );
    assert_eq!(
        completed.participants[0]
            .conditions
            .iter()
            .find(|c| c.condition_id == reminder.condition_id)
            .unwrap()
            .note,
        reminder.note
    );
}
