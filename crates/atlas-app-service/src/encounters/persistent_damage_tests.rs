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
        recoveries: vec![],
        boundary_key: None,
        request_id: "damage-once".into(),
        expected_encounter_revision: snapshot(f, key).encounter.revision,
        participant_key: participant.into(),
        artifact_generation: None,
        groups: vec![
            PersistentDamageGroupInput {
                episode_id: "episode-fire".into(),
                damage: PersistentDamageInput::TableResult { amount: 4 },
            },
            PersistentDamageGroupInput {
                episode_id: "episode-acid".into(),
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
    let fire = result
        .groups
        .iter()
        .find(|group| group.damage_type == "fire")
        .unwrap();
    let after = snapshot(&f, &key);
    let selected = after.participants[0]
        .conditions
        .iter()
        .find(|condition| condition.condition_id == fire.source_condition_id)
        .unwrap();
    assert!(
        matches!(&selected.parameters, ConditionParameters::PersistentDamage { formula, .. } if formula == "4")
    );
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
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "recovery-gate".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: Some(100),
    };
    assert!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .is_err()
    );
    request.mode = PersistentRecoveryMode::Explicit;
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
}

#[test]
fn typed_protections_preserve_missing_hardness_and_source_exceptions() {
    let (f, key, _) = fixture();
    let participant = &snapshot(&f, &key).participants[0];
    let record = |kind: &str, attributes: serde_json::Value| {
        atlas_search::test_support::record("actors","Actor",serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":kind,"name":"Protection","system":{"traits":{"value":[]},"attributes":attributes}})).record
    };
    let npc = record(
        "npc",
        serde_json::json!({"weaknesses":[{"type":"fire","value":5}],"resistances":[{"type":"fire","value":3}]}),
    );
    let protections =
        super::rules::damage::persistent_protection(Some(&npc), participant, "fire").unwrap();
    assert_eq!(protections.apply(10), 12);
    let sleep = record(
        "npc",
        serde_json::json!({"immunities":[{"type":"sleep"}],"weaknesses":[],"resistances":[]}),
    );
    assert_eq!(
        super::rules::damage::persistent_protection(Some(&sleep), participant, "fire")
            .unwrap()
            .apply(10),
        10
    );
    let construct=atlas_search::test_support::record("actors","Actor",serde_json::json!({"_id":"constructImmune1","type":"npc","name":"Construct","system":{"traits":{"value":["construct","mindless"]},"attributes":{"immunities":[],"weaknesses":[],"resistances":[]}}})).record;
    assert!(
        super::rules::damage::persistent_protection(Some(&construct), participant, "bleed")
            .unwrap()
            .immune
    );
    assert_eq!(
        super::rules::damage::persistent_protection(Some(&construct), participant, "fire")
            .unwrap()
            .apply(10),
        10
    );

    let mut bleeding = participant.clone();
    if let ConditionParameters::PersistentDamage { damage_type, .. } =
        &mut bleeding.conditions[0].parameters
    {
        *damage_type = "bleed".into();
    }
    let groups = super::persistent_damage::group_views(&bleeding, Some(&construct));
    assert_eq!(
        groups
            .iter()
            .find(|group| group.damage_type == "bleed")
            .unwrap()
            .immunity_note
            .as_deref(),
        Some("Immune to bleed")
    );
    assert!(
        groups
            .iter()
            .filter(|group| group.damage_type != "bleed")
            .all(|group| group.immunity_note.is_none())
    );
    assert!(
        super::persistent_damage::group_views(&bleeding, None)
            .iter()
            .all(|group| group.immunity_note.is_none())
    );

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
        recoveries: vec![],
        boundary_key: None,
        request_id: "statue-damage".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: statue.participant_key.clone(),
        artifact_generation: f.worker.encounter(&key).unwrap().artifact_generation,
        groups: ["fire-statue", "acid-statue"]
            .into_iter()
            .map(|episode| PersistentDamageGroupInput {
                episode_id: episode.into(),

                damage: PersistentDamageInput::Roll,
            })
            .collect(),
        context: Default::default(),
    };
    let result = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert!(result.groups.iter().all(|group| {
        group
            .protection
            .as_ref()
            .is_some_and(|p| !p.immune && p.weakness == 0 && p.resistance == 0)
    }));
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
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::Explicit,
        request_id: "recover-fire".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant,
        episode_id: "episode-fire".into(),
        dc_override: Some(0),
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
fn late_recovery_never_clears_a_new_same_type_episode() {
    let (f, key, participant) = fixture();
    f.worker
        .apply_persistent_damage(&key, batch(&f, &key, &participant))
        .unwrap();
    let hp_after_damage = snapshot(&f, &key).participants[0].current_hp;
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
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: Some(boundary),
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "late-success".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: Some(0),
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(result.success);
    assert!(result.roll.is_none());
    assert_eq!(
        f.worker
            .recover_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    let state = snapshot(&f, &key);
    assert!(state.participants[0].conditions.iter().any(|c|matches!(&c.parameters,ConditionParameters::PersistentDamage{episode_id,..} if episode_id=="new-fire")));
    assert_eq!(state.participants[0].current_hp, hp_after_damage);
    assert!(
        f.worker
            .recover_persistent_damage(
                &key,
                RecoverPersistentDamageRequest {
                    resolution: PersistentRecoveryResolution::Roll,
                    request_id: "duplicate-claim".into(),
                    expected_encounter_revision: state.encounter.revision,
                    ..request
                }
            )
            .is_err()
    );
    let manual = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Roll,
        boundary_key: None,
        mode: PersistentRecoveryMode::Explicit,
        request_id: "unknown-dc-success".into(),
        expected_encounter_revision: state.encounter.revision,
        participant_key: participant.clone(),
        episode_id: "new-fire".into(),
        dc_override: Some(0),
    };
    let result = f
        .worker
        .recover_persistent_damage(&key, manual.clone())
        .unwrap();
    assert!(result.success);
    assert_eq!(result.dc, Some(0));
    assert!(result.roll.is_none());
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
                parameters: ConditionParametersInputView::PersistentDamage {
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
                parameters: ConditionParametersInputView::PersistentDamage {
                    formula: Some("1d6".into()),
                    damage_type: Some("fire".into()),
                    multiplier: DamageMultiplierView::Normal,
                    recovery: PersistentRecoveryView::Standard,
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

#[test]
fn persistent_edits_preserve_or_admit_episode_identity_without_browser_authority() {
    let (f, key, participant) = fixture();
    let saved = snapshot(&f, &key);
    let source = &saved.participants[0].conditions[0];
    let edit = |id: &str, kind: &str, revision| UpdateEncounterParticipantConditionRequest {
        request_id: id.into(),
        expected_encounter_revision: revision,
        condition_id: source.condition_id,
        condition_ref: source.condition_key.clone(),
        name: source.name.clone(),
        value: None,
        source_participant_key: None,
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        origin: None,
        duration: None,
        note: Some("new note".into()),
        event_intent: Default::default(),
        decisions: Default::default(),
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some("4".into()),
            damage_type: Some(kind.into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            edit("note", "fire", saved.encounter.revision),
        )
        .unwrap();
    let after_note = snapshot(&f, &key);
    let edited = after_note.participants[0]
        .conditions
        .iter()
        .find(|row| row.condition_id == source.condition_id)
        .unwrap();
    assert!(
        matches!(&edited.parameters, ConditionParameters::PersistentDamage { episode_id, recovery: PersistentRecovery::Standard, .. } if episode_id == "episode-fire")
    );
    f.worker
        .update_encounter_participant_condition(
            &key,
            &participant,
            edit("new-type", "cold", after_note.encounter.revision),
        )
        .unwrap();
    let after_type = snapshot(&f, &key);
    let edited = after_type.participants[0]
        .conditions
        .iter()
        .find(|row| row.condition_id == source.condition_id)
        .unwrap();
    assert!(
        matches!(&edited.parameters, ConditionParameters::PersistentDamage { episode_id, damage_type, .. } if episode_id == "new-type:cold" && damage_type == "cold")
    );
    assert!(after_type.participants[0].conditions.iter().any(|row| matches!(&row.parameters, ConditionParameters::PersistentDamage { episode_id, damage_type, .. } if episode_id == "episode-fire" && damage_type == "fire")));
}

#[test]
fn table_clear_needs_no_dc_source_choice_or_damage_and_cannot_repeat_damage() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key).participants[0].current_hp;
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Clear,
        boundary_key: None,
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "clear-fire-at-table".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: None,
    };
    let receipt = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    assert!(receipt.success);
    assert!(receipt.dc.is_none());
    assert!(receipt.roll.is_none());
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, before);
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.damage_groups.len(), 1);
    assert_eq!(review.damage_groups[0].damage_type, "acid");
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        receipt
    );
    let mut request = batch(&f, &key, &participant);
    request
        .groups
        .retain(|group| group.episode_id == "episode-acid");
    let damage = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!(damage.total, 3);
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(2));
}

#[test]
fn outgoing_clear_preserves_later_fire_sources_and_excludes_ended_fire_from_damage() {
    let (f, key, participant) = fixture();
    let boundary = outgoing_boundary(&f, &key, &participant);
    f.worker.with_write(|session| {
        let mut later = snapshot(&f, &key).participants[0].conditions.iter().find(|source|
            matches!(&source.parameters,ConditionParameters::PersistentDamage {damage_type,..} if damage_type == "fire")).unwrap().clone();
        later.condition_id = 0;
        if let ConditionParameters::PersistentDamage {formula, ..} = &mut later.parameters { *formula = "99".into(); }
        session.local_state().encounters().add_condition(atlas_local_state::AddEncounterParticipantCondition {
            participant_key: participant.clone(), condition_key: later.condition_key, name: later.name, value: None,
            source_participant_key: None, scope: Default::default(), extra_action: Default::default(), immunity_override:false,
            parameters:later.parameters, origin:None, duration:None, note:Some("After turn".into()),
        })?;
        Ok(())
    }).unwrap();
    let request = RecoverPersistentDamageRequest {
        resolution: PersistentRecoveryResolution::Clear,
        boundary_key: Some(boundary.clone()),
        mode: PersistentRecoveryMode::EndTurn,
        request_id: "clear-outgoing-fire".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        episode_id: "episode-fire".into(),
        dc_override: None,
    };
    let receipt = f
        .worker
        .recover_persistent_damage(&key, request.clone())
        .unwrap();
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].current_hp, Some(5));
    assert_eq!(after.participants[0].conditions.len(), 2);
    assert!(
        after.participants[0]
            .conditions
            .iter()
            .any(|source| source.note.as_deref() == Some("After turn"))
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, Some(&boundary))
        .unwrap();
    assert_eq!(review.damage_groups.len(), 1);
    assert_eq!(review.damage_groups[0].damage_type, "acid");
    assert_eq!(
        f.worker.recover_persistent_damage(&key, request).unwrap(),
        receipt
    );
    assert_eq!(snapshot(&f, &key), after);
    let mut request = batch(&f, &key, &participant);
    request.boundary_key = Some(boundary);
    request
        .groups
        .retain(|group| group.episode_id == "episode-acid");
    let damage = f.worker.apply_persistent_damage(&key, request).unwrap();
    assert_eq!(damage.total, 3);
    assert!(
        snapshot(&f, &key).participants[0]
            .conditions
            .iter()
            .any(|source| source.note.as_deref() == Some("After turn"))
    );
}

#[test]
fn persistent_precedence_is_atomic_and_retains_both_ambiguous_fallback_pairs() {
    use PersistentDamageSourceRefView as Choice;
    let (f, key, participant) = fixture();
    for source in snapshot(&f, &key).participants[0].conditions.clone() {
        if matches!(&source.parameters, ConditionParameters::PersistentDamage {damage_type, ..} if damage_type == "fire")
        {
            f.worker
                .remove_encounter_participant_condition(
                    &key,
                    &participant,
                    source.condition_id,
                    RemoveEncounterParticipantConditionRequest {
                        request_id: format!("remove-{}", source.condition_id),
                        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                        intent: Default::default(),
                    },
                )
                .unwrap();
        }
    }
    let request = |id: &str, formula: &str| AddEncounterParticipantConditionRequest {
        request_id: id.into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        participant_key: participant.clone(),
        condition_ref: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
        name: None,
        value: None,
        source_participant_key: None,
        duration: None,
        note: None,
        event_intent: Default::default(),
        decisions: Default::default(),
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        origin: None,
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some(formula.into()),
            damage_type: Some("fire".into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    let first = request("d12", "1d12");
    assert!(
        f.worker
            .preview_encounter_condition_add(&key, first.clone())
            .unwrap()
            .persistent_precedence
            .is_none()
    );
    f.worker
        .add_encounter_participant_condition(&key, first)
        .unwrap();
    let d12 = f.worker.encounter(&key).unwrap().participants[0].conditions.iter().find(|c| matches!(&c.parameters, ConditionParametersView::PersistentDamage{formula,..} if formula == "1d12")).unwrap().condition_id;
    let mut second = request("2d6", "2d6");
    let before = snapshot(&f, &key);
    let choice = f
        .worker
        .preview_encounter_condition_add(&key, second.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(choice.sources[0].source, Choice::Edited);
    assert_eq!(choice.sources.len(), 2);
    assert_eq!(
        snapshot(&f, &key).encounter.revision,
        before.encounter.revision
    );
    let acid = before.participants[0].conditions.iter().find(|c| matches!(&c.parameters,ConditionParameters::PersistentDamage{damage_type,..} if damage_type == "acid")).unwrap().condition_id;
    second.decisions.persistent_precedence = Some(vec![
        Choice::Existing { condition_id: acid },
        Choice::Edited,
    ]);
    assert!(
        f.worker
            .add_encounter_participant_condition(&key, second.clone())
            .is_err()
    );
    assert_eq!(
        snapshot(&f, &key).encounter.revision,
        before.encounter.revision
    );
    second.decisions.persistent_precedence =
        Some(vec![Choice::Existing { condition_id: d12 }, Choice::Edited]);
    let receipt = f
        .worker
        .add_encounter_participant_condition(&key, second.clone())
        .unwrap();
    assert_eq!(
        f.worker
            .add_encounter_participant_condition(&key, second)
            .unwrap(),
        receipt
    );
    let detail = f.worker.encounter(&key).unwrap();
    let fire = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap();
    assert_eq!(fire.selected_source_id, d12);
    let d6 = fire
        .sources
        .iter()
        .find(|s| s.formula == "2d6")
        .unwrap()
        .condition_id;
    let source = detail.participants[0]
        .conditions
        .iter()
        .find(|c| c.condition_id == d6)
        .unwrap();
    let edit = UpdateEncounterParticipantConditionRequest {
        request_id: "edit-choice".into(),
        expected_encounter_revision: detail.revision,
        condition_id: d6,
        condition_ref: source.condition_key.clone(),
        name: source.name.clone(),
        value: source.value,
        source_participant_key: None,
        scope: source.scope.clone(),
        extra_action: source.extra_action.clone(),
        immunity_override: false,
        origin: source.origin.clone(),
        duration: None,
        note: None,
        event_intent: Default::default(),
        decisions: Default::default(),
        parameters: ConditionParametersInputView::PersistentDamage {
            formula: Some("2d8".into()),
            damage_type: Some("fire".into()),
            multiplier: DamageMultiplierView::Normal,
            recovery: PersistentRecoveryView::Standard,
        },
    };
    assert_eq!(
        f.worker
            .preview_encounter_condition_update(&key, &participant, edit.clone())
            .unwrap()
            .persistent_precedence
            .unwrap()
            .sources[0]
            .source,
        Choice::Edited
    );
    f.worker
        .update_encounter_participant_condition(&key, &participant, edit)
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    assert_eq!(
        detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .selected_source_id,
        d6
    );
    f.worker
        .remove_encounter_participant_condition(
            &key,
            &participant,
            d6,
            RemoveEncounterParticipantConditionRequest {
                request_id: "remove-selected".into(),
                expected_encounter_revision: detail.revision,
                intent: Default::default(),
            },
        )
        .unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .selected_source_id,
        d12
    );
    // Rebuild the lower ambiguous pair with the GM's preferred 1d12 first.
    let mut lower = request("lower-pair", "2d6");
    lower.decisions.persistent_precedence =
        Some(vec![Choice::Existing { condition_id: d12 }, Choice::Edited]);
    f.worker
        .add_encounter_participant_condition(&key, lower)
        .unwrap();
    f.worker
        .add_encounter_participant_condition(&key, request("higher-d12", "2d12"))
        .unwrap();
    let detail = f.worker.encounter(&key).unwrap();
    let higher_d12 = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap()
        .sources
        .iter()
        .find(|s| s.formula == "2d12")
        .unwrap()
        .condition_id;
    let lower_d6 = detail.participants[0]
        .persistent_damage_groups
        .iter()
        .find(|g| g.damage_type == "fire")
        .unwrap()
        .sources
        .iter()
        .find(|s| s.formula == "2d6")
        .unwrap()
        .condition_id;
    let mut higher = request("higher-pair", "4d6");
    // Reject duplicate/missing members without changing the recorded state.
    let revision = higher.expected_encounter_revision;
    higher.decisions.persistent_precedence = Some(vec![Choice::Edited, Choice::Edited]);
    assert!(
        f.worker
            .add_encounter_participant_condition(&key, higher.clone())
            .is_err()
    );
    assert_eq!(snapshot(&f, &key).encounter.revision, revision);
    higher.decisions.persistent_precedence = Some(vec![
        Choice::Existing {
            condition_id: higher_d12,
        },
        Choice::Edited,
        Choice::Existing { condition_id: d12 },
        Choice::Existing {
            condition_id: lower_d6,
        },
    ]);
    f.worker
        .add_encounter_participant_condition(&key, higher)
        .unwrap();
    let assert_order = |expected: &[&str]| {
        let detail = f.worker.encounter(&key).unwrap();
        let fire = detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap();
        assert_eq!(
            fire.sources
                .iter()
                .map(|s| s.formula.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(fire.selected_source_id, fire.sources[0].condition_id);
    };
    assert_order(&["2d12", "4d6", "1d12", "2d6"]);
    // A uniquely strongest source still exposes precedence for ambiguous fallbacks.
    let strongest = request("uniquely-strongest", "6d12");
    let preview = f
        .worker
        .preview_encounter_condition_add(&key, strongest.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(
        preview
            .sources
            .iter()
            .map(|s| s.formula.as_str())
            .collect::<Vec<_>>(),
        ["6d12", "2d12", "4d6", "1d12", "2d6"]
    );
    f.worker
        .add_encounter_participant_condition(&key, strongest)
        .unwrap();
    for formula in ["6d12", "2d12", "4d6"] {
        let detail = f.worker.encounter(&key).unwrap();
        let source = detail.participants[0]
            .persistent_damage_groups
            .iter()
            .find(|g| g.damage_type == "fire")
            .unwrap()
            .sources
            .iter()
            .find(|s| s.formula == formula)
            .unwrap()
            .condition_id;
        f.worker
            .remove_encounter_participant_condition(
                &key,
                &participant,
                source,
                RemoveEncounterParticipantConditionRequest {
                    request_id: format!("expire-{source}"),
                    expected_encounter_revision: detail.revision,
                    intent: Default::default(),
                },
            )
            .unwrap();
    }
    // The higher-average 2d6 must not replace the GM's retained lower-pair preference.
    assert_order(&["1d12", "2d6"]);
    let next = request("new-stronger-source", "2d8");
    let preview = f
        .worker
        .preview_encounter_condition_add(&key, next.clone())
        .unwrap()
        .persistent_precedence
        .unwrap();
    assert_eq!(
        preview
            .sources
            .iter()
            .map(|s| s.formula.as_str())
            .collect::<Vec<_>>(),
        ["2d8", "1d12", "2d6"]
    );
    f.worker
        .add_encounter_participant_condition(&key, next)
        .unwrap();
    assert_order(&["2d8", "1d12", "2d6"]);
}

#[test]
fn stale_recovery_cannot_clear_an_episode() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    assert!(
        f.worker
            .recover_persistent_damage(
                &key,
                RecoverPersistentDamageRequest {
                    resolution: PersistentRecoveryResolution::Roll,
                    boundary_key: None,
                    mode: PersistentRecoveryMode::Explicit,
                    request_id: "stale-recovery".into(),
                    expected_encounter_revision: before.encounter.revision - 1,
                    participant_key: participant,
                    episode_id: "episode-fire".into(),
                    dc_override: Some(1),
                }
            )
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before);
}

#[test]
fn prepared_recovery_keeps_damage_due_and_commits_all_changes_once() {
    let (f, key, participant) = fixture();
    let before = snapshot(&f, &key);
    let mut damage = batch(&f, &key, &participant);
    damage.artifact_generation = f.worker.encounter(&key).unwrap().artifact_generation;
    let prepare = PreparePersistentDamageRequest {
        request_id: "prepare-recovery".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: participant.clone(),
        boundary_key: None,
        artifact_generation: damage.artifact_generation.clone(),
        groups: vec![damage.groups[0].clone()],
        recovery: Some(PreparedPersistentRecoveryInput {
            episode_id: "episode-fire".into(),
            dc_override: Some(0),
        }),
    };
    let result = f
        .worker
        .prepare_persistent_damage(&key, prepare.clone())
        .unwrap();
    assert!(!result.complete);
    assert_eq!(result.damage.total, 4);
    assert!(result.recovery.as_ref().unwrap().success);
    assert_eq!(
        snapshot(&f, &key),
        before,
        "Preparation must preserve HP, conditions and revisions"
    );
    assert_eq!(
        f.worker
            .prepare_persistent_damage(&key, prepare.clone())
            .unwrap(),
        result
    );
    let mut duplicate = prepare;
    duplicate.request_id = "another-recovery-roll".into();
    assert!(f.worker.prepare_persistent_damage(&key, duplicate).is_err());
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.damage_groups.len(), 2);
    assert_eq!(review.prepared_recovery.len(), 1);
    damage.recoveries = vec![
        PersistentRecoveryInput::Prepared {
            episode_id: "episode-fire".into(),
            request_id: "prepare-recovery".into(),
        },
        PersistentRecoveryInput::Clear {
            episode_id: "episode-acid".into(),
        },
    ];
    let applied = f
        .worker
        .apply_persistent_damage(&key, damage.clone())
        .unwrap();
    assert_eq!(applied.total, 7);
    let after = snapshot(&f, &key);
    assert_eq!(after.participants[0].current_hp, Some(0));
    assert_eq!(
        after.participants[0]
            .conditions
            .iter()
            .find(|c| c.name == "Dying")
            .unwrap()
            .value,
        Some(1)
    );
    assert!(
        !after.participants[0]
            .conditions
            .iter()
            .any(|c| matches!(c.parameters, ConditionParameters::PersistentDamage { .. }))
    );
    let review = f
        .worker
        .persistent_damage_review(&key, &participant, None)
        .unwrap();
    assert_eq!(review.recovery_results.len(), 2);
    assert_eq!(
        f.worker.apply_persistent_damage(&key, damage).unwrap(),
        applied
    );
    assert_eq!(snapshot(&f, &key), after);
}

#[test]
fn prepared_rolls_keep_shared_hardness_and_reject_changed_subjects() {
    let (f, key, _) = fixture();
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "actors:testCreature1000".into(),
            quantity: 1,
            initiative: Some(8),
        })
        .unwrap();
    let actor = snapshot(&f, &key)
        .participants
        .into_iter()
        .find(|p| p.record_key.is_some())
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
                    participant_key: actor.participant_key.clone(),
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
    let before = snapshot(&f, &key);
    let generation = f.worker.encounter(&key).unwrap().artifact_generation;
    let mut groups = Vec::new();
    for (episode, subtotal, complete) in [("fire-statue", 2, false), ("acid-statue", 12, true)] {
        let mut inputs = groups.clone();
        inputs.push(PersistentDamageGroupInput {
            episode_id: episode.into(),
            damage: PersistentDamageInput::Roll,
        });
        let request = PreparePersistentDamageRequest {
            request_id: format!("prepare-{episode}"),
            expected_encounter_revision: before.encounter.revision,
            participant_key: actor.participant_key.clone(),
            boundary_key: None,
            artifact_generation: generation.clone(),
            groups: inputs,
            recovery: None,
        };
        let result = f
            .worker
            .prepare_persistent_damage(&key, request.clone())
            .unwrap();
        assert_eq!((result.damage.total, result.complete), (subtotal, complete));
        assert_eq!(
            f.worker.prepare_persistent_damage(&key, request).unwrap(),
            result
        );
        groups.push(PersistentDamageGroupInput {
            episode_id: episode.into(),
            damage: PersistentDamageInput::Prepared {
                request_id: format!("prepare-{episode}"),
            },
        });
    }
    assert_eq!(snapshot(&f, &key), before);
    let review = f
        .worker
        .persistent_damage_review(&key, &actor.participant_key, None)
        .unwrap();
    assert_eq!(review.prepared_damage.len(), 2);
    let request = ApplyPersistentDamageRequest {
        request_id: "apply-prepared-statue".into(),
        expected_encounter_revision: before.encounter.revision,
        participant_key: actor.participant_key.clone(),
        boundary_key: None,
        artifact_generation: generation,
        groups,
        recoveries: vec![],
        context: Default::default(),
    };
    let result = f
        .worker
        .apply_persistent_damage(&key, request.clone())
        .unwrap();
    assert_eq!((result.total, result.hardness_absorbed), (12, 8));
    assert!(
        result
            .groups
            .iter()
            .all(|g| g.roll.is_some() && !g.table_resolved)
    );
    assert_eq!(
        f.worker
            .apply_persistent_damage(&key, request.clone())
            .unwrap(),
        result
    );
    let mut stale = request;
    stale.request_id = "different-boundary-request".into();
    stale.expected_encounter_revision = snapshot(&f, &key).encounter.revision;
    assert!(f.worker.apply_persistent_damage(&key, stale).is_err());
}

#[test]
fn table_clear_retains_a_failed_prepared_check_without_repeating_damage() {
    let rolled_failure = atlas_dice::RollOutcome {
        formula: "1d20".into(),
        total: 4,
        expression: atlas_dice::RollExpression::Dice {
            dice: vec![atlas_dice::DieOutcome {
                sides: 20,
                value: 4,
                kept: true,
            }],
            total: 4,
        },
    };
    for (dc, roll) in [(100, None), (15, Some(rolled_failure))] {
        let (f, key, participant) = fixture();
        let before = snapshot(&f, &key);
        let mut damage = batch(&f, &key, &participant);
        let prepared = if roll.is_none() {
            f.worker
                .prepare_persistent_damage(
                    &key,
                    PreparePersistentDamageRequest {
                        request_id: "failed-recovery".into(),
                        expected_encounter_revision: before.encounter.revision,
                        participant_key: participant.clone(),
                        boundary_key: None,
                        artifact_generation: None,
                        groups: vec![damage.groups[0].clone()],
                        recovery: Some(PreparedPersistentRecoveryInput {
                            episode_id: "episode-fire".into(),
                            dc_override: Some(dc),
                        }),
                    },
                )
                .unwrap()
                .recovery
                .unwrap()
        } else {
            let saved_recovery = atlas_local_state::PreparedPersistentRecovery {
                episode_id: "episode-fire".into(),
                damage_type: "fire".into(),
                dc,
                roll: roll.clone(),
                success: false,
            };
            let claim = atlas_local_state::EncounterBoundaryClaim {
                participant_key: participant.clone(),
                round_number: before.encounter.round_number,
                kind: atlas_local_state::EncounterBoundaryKind::PreparedPersistentRecovery,
                scope_key: "current:episode-fire".into(),
            };
            f.worker
                .with_write(|session| {
                    session
                        .local_state()
                        .encounters()
                        .prepare_condition_event_with_claims(
                            &key,
                            "failed-recovery",
                            &serde_json::json!({"fixture":"rolled-recovery-failure"}),
                            before.encounter.revision,
                            atlas_local_state::ConditionEventBoundary {
                                claims: &[claim],
                                outgoing_boundary_key: None,
                            },
                            || {
                                Ok(vec![
                                    atlas_local_state::ConditionEventOutcome::PersistentPrepared {
                                        context: Box::new(
                                            atlas_local_state::PersistentPreparationContext {
                                                participant_key: participant.clone(),
                                                record_key: None,
                                                boundary_key: None,
                                                artifact_generation: None,
                                            },
                                        ),
                                        groups: vec![atlas_local_state::PersistentDamageOutcome {
                                            episode_id: "episode-fire".into(),
                                            damage_type: "fire".into(),
                                            source_condition_id: before.participants[0].conditions
                                                [0]
                                            .condition_id,
                                            roll: None,
                                            amount: 4,
                                            table_resolved: true,
                                            protection: None,
                                        }],
                                        hardness: 0,
                                        complete: false,
                                        recovery: Some(saved_recovery),
                                    },
                                ])
                            },
                        )?;
                    Ok(())
                })
                .unwrap();
            f.worker
                .persistent_damage_review(&key, &participant, None)
                .unwrap()
                .prepared_recovery
                .remove(0)
        };
        assert_eq!(snapshot(&f, &key), before);
        assert_eq!(
            (prepared.dc, &prepared.roll, prepared.success),
            (Some(dc), &roll, false)
        );
        for recoveries in [
            vec![],
            vec![PersistentRecoveryInput::Prepared {
                episode_id: "episode-fire".into(),
                request_id: "unrelated-preparation".into(),
            }],
            vec![
                PersistentRecoveryInput::Clear {
                    episode_id: "episode-fire".into(),
                },
                PersistentRecoveryInput::Prepared {
                    episode_id: "episode-fire".into(),
                    request_id: "failed-recovery".into(),
                },
            ],
        ] {
            let mut invalid = damage.clone();
            invalid.recoveries = recoveries;
            assert!(f.worker.apply_persistent_damage(&key, invalid).is_err());
            assert_eq!(snapshot(&f, &key), before);
        }
        damage.recoveries = vec![PersistentRecoveryInput::Clear {
            episode_id: "episode-fire".into(),
        }];
        let applied = f
            .worker
            .apply_persistent_damage(&key, damage.clone())
            .unwrap();
        assert_eq!(applied.total, 7);
        let after = snapshot(&f, &key);
        assert_eq!(after.participants[0].current_hp, Some(0));
        assert!(after.participants[0].conditions.iter().all(|condition| {
            !matches!(&condition.parameters, ConditionParameters::PersistentDamage { damage_type, .. } if damage_type == "fire")
        }));
        assert!(after.participants[0].conditions.iter().any(|condition| {
            matches!(&condition.parameters, ConditionParameters::PersistentDamage { damage_type, .. } if damage_type == "acid")
        }));
        let review = f
            .worker
            .persistent_damage_review(&key, &participant, None)
            .unwrap();
        let retained = review
            .recovery_results
            .iter()
            .find(|result| result.episode_id == "episode-fire")
            .unwrap();
        assert_eq!(
            (retained.dc, &retained.roll, retained.success),
            (prepared.dc, &prepared.roll, false)
        );
        assert_eq!(
            f.worker.apply_persistent_damage(&key, damage).unwrap(),
            applied
        );
        assert_eq!(snapshot(&f, &key), after);
        assert_eq!(
            f.worker
                .persistent_damage_review(&key, &participant, None)
                .unwrap(),
            review
        );
    }
}

#[test]
fn concurrent_preparation_cannot_be_omitted_or_lost_by_table_clear() {
    for clearing in [false, true] {
        let (f, key, participant) = fixture();
        let before = snapshot(&f, &key);
        let mut damage = batch(&f, &key, &participant);
        if clearing {
            damage.recoveries = vec![PersistentRecoveryInput::Clear {
                episode_id: "episode-fire".into(),
            }];
        }
        let prepare = PreparePersistentDamageRequest {
            request_id: "concurrent-recovery".into(),
            expected_encounter_revision: before.encounter.revision,
            participant_key: participant.clone(),
            boundary_key: None,
            artifact_generation: None,
            groups: vec![damage.groups[0].clone()],
            recovery: Some(PreparedPersistentRecoveryInput {
                episode_id: "episode-fire".into(),
                dc_override: Some(100),
            }),
        };
        let barrier = std::sync::Barrier::new(2);
        let (applied, prepared) = std::thread::scope(|scope| {
            let apply = scope.spawn(|| {
                barrier.wait();
                f.worker.apply_persistent_damage(&key, damage)
            });
            let preparation = scope.spawn(|| {
                barrier.wait();
                f.worker.prepare_persistent_damage(&key, prepare)
            });
            (apply.join().unwrap(), preparation.join().unwrap())
        });
        let review = f
            .worker
            .persistent_damage_review(&key, &participant, None)
            .unwrap();
        if clearing {
            assert!(
                applied.is_ok(),
                "Table Clear must remain usable: {applied:?}"
            );
            let result = review
                .recovery_results
                .iter()
                .find(|result| result.episode_id == "episode-fire")
                .unwrap();
            if let Ok(prepared) = prepared {
                let check = prepared.recovery.unwrap();
                assert_eq!(
                    (result.dc, &result.roll, result.success),
                    (check.dc, &check.roll, check.success)
                );
            } else {
                assert_eq!(
                    (result.dc, &result.roll, result.success),
                    (None, &None, true)
                );
            }
            assert!(snapshot(&f, &key).participants[0].conditions.iter().all(|condition| {
                !matches!(&condition.parameters, ConditionParameters::PersistentDamage { damage_type, .. } if damage_type == "fire")
            }));
        } else {
            assert_ne!(
                applied.is_ok(),
                prepared.is_ok(),
                "An omitted check and its damage cannot both commit: {applied:?}, {prepared:?}"
            );
            if prepared.is_ok() {
                assert_eq!(snapshot(&f, &key), before);
                assert!(!review.damage_applied);
            } else {
                assert!(review.damage_applied);
                assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(0));
            }
        }
    }
}
