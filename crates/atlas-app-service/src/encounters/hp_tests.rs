use atlas_app_model::*;
use atlas_local_state::EncounterParticipant;

use crate::AtlasAppService;
use crate::test_support::{FixtureWorker, encounter_fixture_worker};

fn manual(max: Option<i64>, current: Option<i64>) -> (FixtureWorker, String, String) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP actions".into(),
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
            max_hp: max,
            current_hp: current,
            initiative: Some(12),
        })
        .unwrap();
    let participant = f
        .worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&key)?)
        })
        .unwrap()
        .unwrap()
        .participants
        .remove(0)
        .participant_key;
    (f, key, participant)
}

fn disable_dying_tracking(worker: &AtlasAppService, key: &str, participant: &str) {
    let revision = worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get(key)?
                .unwrap()
                .revision)
        })
        .unwrap();
    worker
        .set_encounter_life_settings(
            key,
            participant,
            SetEncounterLifeSettingsRequest {
                knockout_context: None,
                mortality: None,
                request_id: crate::test_support::next_request_id(),
                expected_encounter_revision: revision,
                track_dying: false,
                level_override: None,
                base_death_threshold: None,
            },
        )
        .unwrap();
}

fn saved(worker: &AtlasAppService, key: &str) -> EncounterParticipant {
    worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(key)?)
        })
        .unwrap()
        .unwrap()
        .participants
        .remove(0)
}

fn apply(
    worker: &AtlasAppService,
    key: &str,
    participant: &str,
    action: EncounterHpAction,
) -> EncounterParticipant {
    let revision = saved(worker, key).revision;
    let receipt = worker
        .apply_encounter_hp_action(
            key,
            ApplyEncounterHpActionRequest {
                request_id: crate::test_support::next_request_id(),
                context: None,
                participant_key: participant.into(),
                expected_revision: revision,
                action,
            },
        )
        .unwrap();
    assert_eq!(receipt.encounter_key, key);
    let result = saved(worker, key);
    assert_eq!(result.revision, revision + 1);
    result
}

#[test]
fn damage_absorbs_temporary_hp_then_floors_and_healing_preserves_defeated() {
    let (f, key, p) = manual(Some(30), Some(25));
    disable_dying_tracking(&f.worker, &key, &p);
    apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetTemporary {
            input: "5+3".into(),
        },
    );
    let first = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Damage {
            amount: "3+1".into(),
        },
    );
    assert_eq!(
        (first.current_hp, first.temporary_hp, first.defeated),
        (Some(25), 4, false)
    );
    let second = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Damage { amount: "9".into() },
    );
    assert_eq!((second.current_hp, second.temporary_hp), (Some(20), 0));
    let zero = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Damage {
            amount: "100".into(),
        },
    );
    assert_eq!((zero.current_hp, zero.defeated), (Some(0), true));
    let healed = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Heal {
            amount: "50-2".into(),
        },
    );
    assert_eq!(
        (healed.current_hp, healed.defeated, healed.initiative),
        (Some(30), true, Some(12))
    );
}

#[test]
fn signed_amounts_reverse_operations_through_actual_hp_policy() {
    for (action, current_hp, temporary_hp) in [
        (
            EncounterHpAction::Heal {
                amount: "10".into(),
            },
            30,
            8,
        ),
        (
            EncounterHpAction::Heal {
                amount: "-10".into(),
            },
            23,
            0,
        ),
        (
            EncounterHpAction::Damage {
                amount: "+10".into(),
            },
            23,
            0,
        ),
        (
            EncounterHpAction::Damage {
                amount: "-10".into(),
            },
            30,
            8,
        ),
        (
            EncounterHpAction::Heal {
                amount: "+2-12".into(),
            },
            23,
            0,
        ),
        (
            EncounterHpAction::Damage {
                amount: "-12+2".into(),
            },
            30,
            8,
        ),
        (
            EncounterHpAction::Damage {
                amount: "-(10+5-2)".into(),
            },
            30,
            8,
        ),
        (
            EncounterHpAction::Heal {
                amount: "-(10+5-2)".into(),
            },
            20,
            0,
        ),
    ] {
        let (f, key, p) = manual(Some(30), Some(25));
        apply(
            &f.worker,
            &key,
            &p,
            EncounterHpAction::SetTemporary { input: "8".into() },
        );
        let after = apply(&f.worker, &key, &p, action);
        assert_eq!(
            (after.current_hp, after.temporary_hp, after.defeated),
            (Some(current_hp), temporary_hp, false)
        );
    }
}

#[test]
fn negative_heal_floors_and_marks_defeated_while_negative_damage_caps_without_clearing_it() {
    let (f, key, p) = manual(Some(30), Some(5));
    disable_dying_tracking(&f.worker, &key, &p);
    apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetTemporary { input: "3".into() },
    );
    let zero = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Heal {
            amount: "-100".into(),
        },
    );
    assert_eq!(
        (zero.current_hp, zero.temporary_hp, zero.defeated),
        (Some(0), 0, true)
    );
    let healed = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Damage {
            amount: "-40".into(),
        },
    );
    assert_eq!(
        (healed.current_hp, healed.temporary_hp, healed.defeated),
        (Some(30), 0, true)
    );
    apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetMaximum { input: None },
    );
    let uncapped = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Damage {
            amount: "-2".into(),
        },
    );
    assert_eq!((uncapped.current_hp, uncapped.defeated), (Some(32), true));
}

#[test]
fn exact_corrections_preserve_outliers_availability_and_maximum_intent() {
    let (f, key, p) = manual(Some(30), Some(25));
    let corrected = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetCurrent {
            input: Some("90+10".into()),
        },
    );
    assert_eq!(corrected.current_hp, Some(100));
    let reduced_max = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetMaximum {
            input: Some("10+5".into()),
        },
    );
    assert_eq!(
        (
            reduced_max.current_hp,
            reduced_max.max_hp,
            reduced_max.hp_origin
        ),
        (
            Some(100),
            Some(15),
            atlas_local_state::ParticipantHpOrigin::Explicit
        )
    );
    let unknown_max = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetMaximum { input: None },
    );
    assert_eq!(
        (unknown_max.current_hp, unknown_max.max_hp),
        (Some(100), None)
    );
    let uncapped = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::Heal { amount: "7".into() },
    );
    assert_eq!(uncapped.current_hp, Some(107));
    let floored = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetCurrent {
            input: Some("3-8".into()),
        },
    );
    assert_eq!((floored.current_hp, floored.defeated), (Some(0), false));
    let unknown = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetCurrent { input: None },
    );
    assert_eq!(unknown.current_hp, None);
    for action in [
        EncounterHpAction::Damage { amount: "1".into() },
        EncounterHpAction::Heal { amount: "1".into() },
        EncounterHpAction::Heal {
            amount: "-1".into(),
        },
        EncounterHpAction::Damage {
            amount: "-1".into(),
        },
    ] {
        let before = saved(&f.worker, &key);
        assert_eq!(
            f.worker
                .apply_encounter_hp_action(
                    &key,
                    ApplyEncounterHpActionRequest {
                        request_id: crate::test_support::next_request_id(),
                        context: None,
                        participant_key: p.clone(),
                        expected_revision: before.revision,
                        action
                    }
                )
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::InvalidRequest
        );
        assert_eq!(saved(&f.worker, &key), before);
    }
    let temp = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetTemporary {
            input: "2-3".into(),
        },
    );
    assert_eq!(temp.temporary_hp, 0);
}

#[test]
fn invalid_or_stale_actions_leave_every_field_unchanged() {
    let (f, key, p) = manual(Some(30), Some(25));
    let before = saved(&f.worker, &key);
    for action in [
        EncounterHpAction::Damage {
            amount: "2--3".into(),
        },
        EncounterHpAction::Heal {
            amount: "--3".into(),
        },
        EncounterHpAction::SetTemporary {
            input: "1d6".into(),
        },
        EncounterHpAction::Damage {
            amount: "-9007199254740992".into(),
        },
        EncounterHpAction::Heal {
            amount: "-9007199254740991-1".into(),
        },
        EncounterHpAction::SetCurrent {
            input: Some("9007199254740992".into()),
        },
        EncounterHpAction::SetMaximum {
            input: Some("170141183460469231731687303715884105728".into()),
        },
    ] {
        assert_eq!(
            f.worker
                .apply_encounter_hp_action(
                    &key,
                    ApplyEncounterHpActionRequest {
                        request_id: crate::test_support::next_request_id(),
                        context: None,
                        participant_key: p.clone(),
                        expected_revision: before.revision,
                        action
                    }
                )
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::InvalidRequest
        );
        assert_eq!(saved(&f.worker, &key), before);
    }
    assert_eq!(
        f.worker
            .apply_encounter_hp_action(
                &key,
                ApplyEncounterHpActionRequest {
                    request_id: crate::test_support::next_request_id(),
                    context: None,
                    participant_key: p,
                    expected_revision: before.revision - 1,
                    action: EncounterHpAction::Damage {
                        amount: "invalid".into()
                    }
                }
            )
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterParticipantConflict
    );
    assert_eq!(saved(&f.worker, &key), before);
}

#[test]
fn concurrent_amount_actions_cannot_overwrite_from_the_same_revision() {
    let (f, key, p) = manual(Some(30), Some(25));
    let request = ApplyEncounterHpActionRequest {
        request_id: crate::test_support::next_request_id(),
        context: None,
        participant_key: p,
        expected_revision: saved(&f.worker, &key).revision,
        action: EncounterHpAction::Heal {
            amount: "-5".into(),
        },
    };
    let mut other_request = request.clone();
    other_request.request_id = crate::test_support::next_request_id();
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| f.worker.apply_encounter_hp_action(&key, request.clone()));
        let second = scope.spawn(|| f.worker.apply_encounter_hp_action(&key, other_request));
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
    let error = outcomes
        .into_iter()
        .find_map(Result::err)
        .unwrap()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::EncounterParticipantConflict);
    let after = saved(&f.worker, &key);
    assert_eq!(
        (after.current_hp, after.revision),
        (Some(20), request.expected_revision + 1)
    );
}

#[test]
fn hp_receipt_does_not_depend_on_postcommit_reference_hydration() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(1, 16);
    let f = crate::test_support::fixture_worker_with_executor(executor.clone());
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP receipt".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let before = saved(&f.worker, &key);
    executor.fail_after_submissions(0);
    let after = apply(
        &f.worker,
        &key,
        &before.participant_key,
        EncounterHpAction::Heal {
            amount: "-5".into(),
        },
    );
    assert_eq!(after.current_hp, Some(55));
    assert_eq!(
        after.hp_origin,
        atlas_local_state::ParticipantHpOrigin::DerivedEdited
    );
    assert!(f.worker.encounter(&key).is_err());
}

#[test]
fn derived_maximum_reuses_existing_deficit_policy() {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP provenance".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let p = saved(&f.worker, &key).participant_key;
    apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetMaximum {
            input: Some("100".into()),
        },
    );
    apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetCurrent {
            input: Some("90".into()),
        },
    );
    let restored = apply(&f.worker, &key, &p, EncounterHpAction::UseDerivedMaximum);
    assert_eq!(
        (restored.max_hp, restored.current_hp, restored.hp_origin),
        (
            Some(60),
            Some(50),
            atlas_local_state::ParticipantHpOrigin::DerivedEdited
        )
    );
}

#[test]
fn healing_overflow_and_wrong_encounter_owner_do_not_write() {
    let (f, key, p) = manual(None, Some(i64::MAX));
    let before = saved(&f.worker, &key);
    let action = ApplyEncounterHpActionRequest {
        request_id: crate::test_support::next_request_id(),
        context: None,
        participant_key: p.clone(),
        expected_revision: before.revision,
        action: EncounterHpAction::Heal { amount: "1".into() },
    };
    assert_eq!(
        f.worker
            .apply_encounter_hp_action(&key, action.clone())
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    let other = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Other".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    assert_eq!(
        f.worker
            .apply_encounter_hp_action(&other, action)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterParticipantNotFound
    );
    assert_eq!(saved(&f.worker, &key), before);
}

#[test]
fn same_value_exact_current_correction_marks_derived_hp_edited() {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Same value".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let before = saved(&f.worker, &key);
    assert_eq!(
        before.hp_origin,
        atlas_local_state::ParticipantHpOrigin::DerivedPristine
    );
    let after = apply(
        &f.worker,
        &key,
        &before.participant_key,
        EncounterHpAction::SetCurrent {
            input: Some("60".into()),
        },
    );
    assert_eq!(after.current_hp, before.current_hp);
    assert_eq!(
        after.hp_origin,
        atlas_local_state::ParticipantHpOrigin::DerivedEdited
    );
}

fn exact(input: Option<&str>) -> Option<EncounterHpValueInput> {
    Some(EncounterHpValueInput {
        input: input.map(Into::into),
    })
}

#[test]
fn atomic_corrections_save_three_fields_once_and_keep_exact_manual_outliers() {
    let (f, key, p) = manual(Some(30), Some(25));
    let before = saved(&f.worker, &key);
    let after = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: exact(Some("90+10")),
            maximum: exact(Some("10+5")),
            temporary: Some("2+3".into()),
        },
    );
    assert_eq!(
        (after.current_hp, after.max_hp, after.temporary_hp),
        (Some(100), Some(15), 5)
    );
    assert_eq!(
        after.hp_origin,
        atlas_local_state::ParticipantHpOrigin::Explicit
    );
    assert_eq!(
        (after.defeated, after.initiative, after.note),
        (before.defeated, before.initiative, before.note)
    );
    let cleared = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: exact(None),
            maximum: exact(None),
            temporary: Some("0".into()),
        },
    );
    assert_eq!(
        (cleared.current_hp, cleared.max_hp, cleared.temporary_hp),
        (None, None, 0)
    );
    let floor = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: exact(Some("-2")),
            maximum: exact(Some("2-3")),
            temporary: Some("-10".into()),
        },
    );
    assert_eq!(
        (
            floor.current_hp,
            floor.max_hp,
            floor.temporary_hp,
            floor.defeated
        ),
        (Some(0), Some(0), 0, false)
    );
}

#[test]
fn atomic_corrections_reject_any_invalid_or_stale_field_without_partial_write() {
    let (f, key, p) = manual(Some(30), Some(25));
    let before = saved(&f.worker, &key);
    for action in [
        EncounterHpAction::SetValues {
            current: exact(Some("8")),
            maximum: exact(Some("12")),
            temporary: Some("bad".into()),
        },
        EncounterHpAction::SetValues {
            current: exact(Some("8")),
            maximum: exact(Some("1d6")),
            temporary: Some("0".into()),
        },
        EncounterHpAction::SetValues {
            current: exact(Some("9007199254740992")),
            maximum: exact(None),
            temporary: Some("0".into()),
        },
        EncounterHpAction::SetValues {
            current: None,
            maximum: None,
            temporary: None,
        },
    ] {
        assert_eq!(
            f.worker
                .apply_encounter_hp_action(
                    &key,
                    ApplyEncounterHpActionRequest {
                        request_id: crate::test_support::next_request_id(),
                        context: None,
                        participant_key: p.clone(),
                        expected_revision: before.revision,
                        action
                    }
                )
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::InvalidRequest
        );
        assert_eq!(saved(&f.worker, &key), before);
    }
    assert_eq!(
        f.worker
            .apply_encounter_hp_action(
                &key,
                ApplyEncounterHpActionRequest {
                    request_id: crate::test_support::next_request_id(),
                    context: None,
                    participant_key: p,
                    expected_revision: before.revision - 1,
                    action: EncounterHpAction::SetValues {
                        current: exact(Some("8")),
                        maximum: exact(None),
                        temporary: Some("0".into())
                    }
                }
            )
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterParticipantConflict
    );
    assert_eq!(saved(&f.worker, &key), before);
}

#[test]
fn omitted_atomic_fields_preserve_derived_provenance_and_supplied_same_values_mark_intent() {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Atomic provenance".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let p = saved(&f.worker, &key).participant_key;
    let temp = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: None,
            maximum: None,
            temporary: Some("4".into()),
        },
    );
    assert_eq!(
        (
            temp.current_hp,
            temp.max_hp,
            temp.temporary_hp,
            temp.hp_origin
        ),
        (
            Some(60),
            Some(60),
            4,
            atlas_local_state::ParticipantHpOrigin::DerivedPristine
        )
    );
    let current = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: exact(Some("60")),
            maximum: None,
            temporary: None,
        },
    );
    assert_eq!(
        (
            current.current_hp,
            current.max_hp,
            current.temporary_hp,
            current.hp_origin
        ),
        (
            Some(60),
            Some(60),
            4,
            atlas_local_state::ParticipantHpOrigin::DerivedEdited
        )
    );
    let maximum = apply(
        &f.worker,
        &key,
        &p,
        EncounterHpAction::SetValues {
            current: None,
            maximum: exact(Some("60")),
            temporary: None,
        },
    );
    assert_eq!(
        (
            maximum.current_hp,
            maximum.max_hp,
            maximum.temporary_hp,
            maximum.hp_origin
        ),
        (
            Some(60),
            Some(60),
            4,
            atlas_local_state::ParticipantHpOrigin::Explicit
        )
    );
}
