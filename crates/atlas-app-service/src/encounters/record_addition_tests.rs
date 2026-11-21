use crate::test_support::encounter_fixture_worker;
use atlas_app_model::*;

#[test]
fn record_addition_replays_one_batch_through_concurrency_removal_and_exchange() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(2, 16);
    let service = crate::test_support::fixture_worker_with_executor(executor.clone()).worker;
    let encounter = |name: &str| {
        service
            .create_encounter(CreateEncounterRequest {
                name: name.into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
    };
    let first = encounter("First roster");
    let second = encounter("Second roster");
    let request = AddEncounterRecordParticipantRequest {
        request_id: "same-request".into(),
        record_ref: "Test Creature 1".into(),
        creature_scenario: None,
        participant_variant: None,
        initiative: Some(12),
        quantity: 3,
    };
    let key = &first.encounter_key;
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|threads| {
        for _ in 0..2 {
            threads.spawn(|| {
                barrier.wait();
                assert_eq!(
                    service
                        .add_encounter_record_participant(key, request.clone())
                        .unwrap()
                        .encounter_key,
                    *key
                );
            });
        }
    });
    assert_eq!(service.encounter(key).unwrap().participants.len(), 3);
    let changed = AddEncounterRecordParticipantRequest {
        quantity: 1,
        ..request.clone()
    };
    assert_eq!(
        service
            .add_encounter_record_participant(key, changed)
            .unwrap_err()
            .app_error()
            .code,
        AppErrorCode::EncounterAdditionConflict
    );
    let fresh = AddEncounterRecordParticipantRequest {
        request_id: "intentional-second-batch".into(),
        ..request.clone()
    };
    service
        .add_encounter_record_participant(key, fresh)
        .unwrap();
    service
        .add_encounter_record_participant(&second.encounter_key, request.clone())
        .unwrap();
    let all = service.encounter(key).unwrap().participants;
    assert_eq!(all.len(), 6);
    assert_eq!(
        service
            .encounter(&second.encounter_key)
            .unwrap()
            .participants
            .len(),
        3
    );
    for p in all {
        service
            .remove_encounter_participant(key, &p.participant_key)
            .unwrap();
    }
    let encoded = service.export_encounter(key).unwrap();
    let copy = service
        .import_encounter(encoded.as_slice(), Some("portable-copy".into()))
        .unwrap();
    let mut archive = Vec::new();
    service
        .runtime
        .capture_workspace_archive()
        .unwrap()
        .write(&mut archive)
        .unwrap();
    let destination = tempfile::tempdir().unwrap();
    let target = destination.path().join("restored.sqlite");
    atlas_runtime::initialize_workspace(&target).unwrap();
    let restored = crate::AtlasAppService::start(crate::AtlasAppServiceOptions {
        index_path: Some(target),
        retrieval_mode: crate::AppServiceRetrievalMode::OnDemandNoQueryInference,
        ..Default::default()
    })
    .unwrap();
    let read_archive = atlas_runtime::WorkspaceArchive::read(archive.as_slice()).unwrap();
    let preflight = restored.runtime.preflight_workspace_restore().unwrap();
    restored
        .runtime
        .restore_workspace_archive(&read_archive, &preflight, false)
        .unwrap();
    // No reference data exists in the restored workspace. The fixture executor also rejects
    // any later reference preparation, to catch accidental hydration on a replay.
    executor.fail_after_submissions(0);
    for dest in [key, &copy.encounter_key] {
        for svc in [&service, &restored] {
            assert_eq!(
                svc.add_encounter_record_participant(dest, request.clone())
                    .unwrap()
                    .encounter_key,
                *dest
            );
            assert_eq!(
                svc.encounters()
                    .unwrap()
                    .encounters
                    .iter()
                    .find(|row| row.encounter_key == *dest)
                    .unwrap()
                    .participant_count,
                0
            );
        }
    }
}

#[test]
fn failed_record_addition_saves_neither_participants_nor_a_receipt() {
    let service = encounter_fixture_worker().worker;
    let encounter = service
        .create_encounter(CreateEncounterRequest {
            name: "Batch rollback".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let key = &encounter.encounter_key;
    let request = AddEncounterRecordParticipantRequest {
        request_id: "retry-after-rollback".into(),
        record_ref: "Test Hazard 1".into(),
        quantity: 3,
        participant_variant: None,
        initiative: None,
        creature_scenario: None,
    };
    let workspace = atlas_runtime::initialize_workspace(service.workspace_path()).unwrap();
    let trigger = |sql: &str| {
        let write = workspace
            .write(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
            .unwrap();
        write.connection().execute_batch(sql).unwrap();
        write.commit().unwrap();
    };
    let invalid = AddEncounterRecordParticipantRequest {
        quantity: 0,
        ..request.clone()
    };
    assert!(
        service
            .add_encounter_record_participant(key, invalid)
            .is_err()
    );
    trigger(
        "CREATE TRIGGER stop_addition BEFORE INSERT ON encounter_command_receipts WHEN NEW.family='record_addition' BEGIN SELECT RAISE(ABORT,'test rollback'); END;",
    );
    assert!(
        service
            .add_encounter_record_participant(key, request.clone())
            .is_err()
    );
    assert!(service.encounter(key).unwrap().participants.is_empty());
    trigger("DROP TRIGGER stop_addition");
    service
        .add_encounter_record_participant(key, request)
        .unwrap();
    assert_eq!(service.encounter(key).unwrap().participants.len(), 3);
}
