use atlas_runtime::{WORKSPACE_SCHEMA_VERSION, Workspace, initialize_workspace};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn workspace_command(db: &Path, action: &str, file: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["workspace", action, "--index"])
        .arg(db)
        .arg(file)
        .args(extra)
        .output()
        .unwrap()
}

fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn failure(output: Output, message: &str) {
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "{output:?}"
    );
}

fn write_archive(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn read_archive(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/workspace-archive-1.0.json")).unwrap()
}

#[test]
fn frozen_archive_restores_edits_exports_and_restores_offline() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.sqlite");
    let dest = temp.path().join("destination.sqlite");
    let original = temp.path().join("original.json");
    let unstaged = temp.path().join("absent").join("archive.json");
    let exported = temp.path().join("exported.json");
    let reexported = temp.path().join("reexported.json");
    write_archive(&original, &fixture());
    failure(
        workspace_command(&source, "restore", &original, &[]),
        "atlas setup",
    );
    assert!(!source.exists());
    let workspace = initialize_workspace(&source).unwrap();
    initialize_workspace(&dest).unwrap();

    success(workspace_command(&source, "restore", &original, &[]));
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write.connection().execute(
        "UPDATE saved_lists SET name='Edited fixture', revision=5 WHERE list_key='recovery-fixture'", [],
    ).unwrap();
    write.commit().unwrap();
    failure(
        workspace_command(&source, "export", &unstaged, &[]),
        "Cannot stage",
    );
    assert!(!unstaged.exists());
    success(workspace_command(&source, "export", &exported, &[]));
    let actual = read_archive(&exported);
    assert_eq!(actual["version"], "1.0");
    assert_eq!(
        actual["saved"]["owners"][0]["lists"][0]["name"],
        "Edited fixture"
    );
    assert_eq!(actual["saved"]["owners"][0]["lists"][0]["revision"], 5);
    assert_eq!(actual["saved"]["condition_floor"], 100);
    failure(
        workspace_command(&source, "export", &exported, &[]),
        "Cannot publish",
    );
    assert_eq!(read_archive(&exported), actual);
    success(workspace_command(&dest, "restore", &exported, &[]));
    success(workspace_command(&dest, "export", &reexported, &[]));
    assert_eq!(read_archive(&reexported), actual);
    failure(
        workspace_command(&dest, "restore", &exported, &[]),
        "already contains saved state",
    );
}

#[test]
fn major_minor_admission_and_conflicting_configuration_leave_no_partial_restore() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("destination.sqlite");
    let input = temp.path().join("archive.json");
    let workspace = initialize_workspace(&db).unwrap();
    let mut archive = fixture();
    write_archive(
        &input,
        &serde_json::json!({
            "format": "pf2e-atlas-workspace",
            "version": "2.0",
            "future_saved_state": {}
        }),
    );
    failure(
        workspace_command(&db, "restore", &input, &[]),
        "Unsupported workspace archive version 2.0",
    );
    archive["version"] = "1.10".into();
    archive["saved"]["owners"][0]["lists"][0]["future_display_hint"] = "ignored".into();
    write_archive(&input, &archive);
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    atlas_index::write_requested_reference_config(
        &write,
        &atlas_index::RequestedReferenceConfig {
            source_root: "/fixture/source".into(),
            manifest_path: None,
            indexing_locale: "en".into(),
            search_profiles: vec![],
        },
    )
    .unwrap();
    write.commit().unwrap();
    failure(
        workspace_command(&db, "restore", &input, &[]),
        "--replace-reference-config",
    );
    failure(
        workspace_command(&db, "restore", &input, &["--replace-reference-config"]),
        "requires interactive confirmation",
    );
    let read = Workspace::existing(&db)
        .unwrap()
        .read(WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let count: i64 = read
        .connection()
        .query_row("SELECT count(*) FROM saved_lists", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
    assert!(
        atlas_index::read_requested_reference_config(&read)
            .unwrap()
            .is_some()
    );
    drop(read);
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    atlas_index::clear_requested_reference_config(&write).unwrap();
    write.commit().unwrap();
    success(workspace_command(&db, "restore", &input, &[]));
}

#[test]
fn all_owner_recovery_preserves_live_and_archived_encounters_and_rolls_back_late_failure() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.sqlite");
    let dest = temp.path().join("dest.sqlite");
    let exported = temp.path().join("exported.json");
    let recovered = temp.path().join("recovered.json");
    let from = initialize_workspace(&source).unwrap();
    let to = initialize_workspace(&dest).unwrap();
    let write = from.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write.connection().execute_batch(r#"
        INSERT INTO principals(principal_key,name) VALUES('guest','Guest');
        INSERT INTO parties(party_key,owner_id,name,revision)
            SELECT 'guest-party',id,'Guests',7 FROM principals WHERE principal_key='guest';
        INSERT INTO party_members(member_key,party_key,name,level,position)
            VALUES('pc-two','guest-party','Second',5,9),('pc-one','guest-party','First',5,3);
        INSERT INTO record_identity(key,last_known_name,availability)
            VALUES('bestiary:aaaaaaaaaaaaaaaa',NULL,'unavailable');
        INSERT INTO saved_lists(owner_id,list_key,slug,name,revision,created_at,updated_at)
            SELECT id,principal_key || '-list','shared','Shared',3,'created','updated' FROM principals;
        INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at)
            SELECT id,'bestiary:aaaaaaaaaaaaaaaa',13,'Keep this note','Saved title','added','edited'
            FROM saved_lists WHERE list_key='guest-list';
        INSERT INTO encounters(owner_id,encounter_key,slug,name,status,round_number,created_at,updated_at,revision)
            SELECT id,'enc-running','live','Live','running',4,'created','updated',8
            FROM principals WHERE principal_key='guest';
        INSERT INTO encounters(owner_id,encounter_key,slug,name,status,round_number,created_at,updated_at)
            SELECT id,'enc-archived','enc-running','Archived','archived',6,'created','updated'
            FROM principals WHERE principal_key='guest';
        INSERT INTO encounter_participants(encounter_id,participant_key,record_key,participant_kind,
                    position,display_name,initiative,initiative_order,created_at,updated_at)
            SELECT id,'saved-participant','bestiary:aaaaaaaaaaaaaaaa','creature',3,'Saved creature',15,1,'created','updated'
            FROM encounters WHERE encounter_key='enc-running';
        UPDATE encounters SET current_turn_participant_key='saved-participant' WHERE encounter_key='enc-running';
        INSERT INTO encounter_pending_consequences(encounter_id,consequence_key,participant_key,payload_json)
            SELECT id,'pending','saved-participant',
                '{"version":1,"payload":{"kind":"broken_hp_threshold","participant_key":"saved-participant"}}'
            FROM encounters WHERE encounter_key='enc-running';
        INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,receipt_json)
            SELECT id,'event','request-1','{"example":"unchanged"}',
                '{"version":1,"payload":{"encounter_key":"enc-running","request_id":"request-1","added_condition_ids":[45],"outcomes":[],"pending":[]}}'
            FROM encounters WHERE encounter_key='enc-running';
        INSERT INTO encounter_manual_trackers(participant_key,tracker_key,payload_json,position)
            VALUES('saved-participant','tracker-1',
                '{"version":1,"payload":{"tracker_key":"tracker-1","name":"Countdown","duration":{"amount":2,"unit":"rounds","timing":null},"progress":{"unit":"rounds","remaining":2}}}',12);
        INSERT INTO sqlite_sequence(name,seq) VALUES('encounter_participant_conditions',73);
    "#).unwrap();
    atlas_index::write_requested_reference_config(
        &write,
        &atlas_index::RequestedReferenceConfig {
            source_root: "/fixture/source".into(),
            manifest_path: None,
            indexing_locale: "en".into(),
            search_profiles: vec![],
        },
    )
    .unwrap();
    write.commit().unwrap();
    success(workspace_command(&source, "export", &exported, &[]));
    let original = read_archive(&exported);
    assert_eq!(
        original["saved"]["owners"][0]["encounters"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(original["saved"]["condition_floor"], 73);

    let mut incompatible = original.clone();
    let encounters = incompatible["saved"]["owners"][0]["encounters"]
        .as_array_mut()
        .unwrap();
    encounters[0]["snapshot"]["encounter"]["status"] = "future_status".into();
    write_archive(&recovered, &incompatible);
    failure(
        workspace_command(&dest, "restore", &recovered, &[]),
        "future_status",
    );
    fs::remove_file(&recovered).unwrap();

    let write = to.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute_batch(
            "CREATE TABLE injected_late_failure(list_id INTEGER REFERENCES saved_lists(id)
             DEFERRABLE INITIALLY DEFERRED);
         CREATE TRIGGER reject_config AFTER INSERT ON requested_reference_config
         BEGIN INSERT INTO injected_late_failure VALUES(-1); END;",
        )
        .unwrap();
    write.commit().unwrap();
    failure(
        workspace_command(&dest, "restore", &exported, &[]),
        "failed foreign key validation",
    );
    success(workspace_command(&dest, "export", &recovered, &[]));
    let after_failure = read_archive(&recovered);
    assert_eq!(
        after_failure["saved"]["owners"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        after_failure["saved"]["owners"][0]["lists"],
        serde_json::json!([])
    );
    assert_eq!(after_failure["saved"]["references"], serde_json::json!([]));
    assert_eq!(after_failure["saved"]["condition_floor"], 0);
    assert!(after_failure["reference_config"].is_null());

    let write = to.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute_batch("DROP TRIGGER reject_config; DROP TABLE injected_late_failure;")
        .unwrap();
    write.commit().unwrap();
    success(workspace_command(&dest, "restore", &exported, &[]));
    fs::remove_file(&recovered).unwrap();
    success(workspace_command(&dest, "export", &recovered, &[]));
    assert_eq!(read_archive(&recovered), original);
    let write = to.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    let next: i64 = write.connection().query_row(
        "INSERT INTO encounter_participant_conditions(participant_id,name,created_at,updated_at)
         SELECT id,'New condition','created','updated' FROM encounter_participants
         WHERE participant_key='saved-participant' RETURNING id",
        [], |row| row.get(0),
    ).unwrap();
    assert_eq!(next, 74);
}

#[test]
fn rich_sample_round_trips_and_retries_through_the_app_offline() {
    use atlas_app_model::{AdvanceEncounterTurnRequest, UpdateEncounterManualTrackerRequest};

    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("workspace.sqlite");
    let sample = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workspace-archive-rich-1.0.json");
    let reexported = temp.path().join("reexported.json");
    initialize_workspace(&db).unwrap();

    success(workspace_command(&db, "restore", &sample, &[]));
    let app = sample_app(&db);
    let encounter_key = "enc-running";
    let detail = app.encounter(encounter_key).unwrap();
    assert_eq!(detail.encounter.name, "Ambush at the old sluice");
    assert_eq!(app.parties().unwrap().parties[0].name, "Lantern Company");
    assert!(
        app.encounter_turn_review(encounter_key)
            .unwrap()
            .outgoing
            .is_some()
    );
    let participant = &detail.participants[0];
    let tracker = &participant.manual_trackers[0];
    assert_eq!(tracker.name, "Sluice opens");
    assert!(tracker.timing_available);
    assert_eq!(tracker.remaining_rounds, 2);

    let archive = read_archive(&sample);
    let receipts = archive["saved"]["owners"][1]["encounters"][0]["receipts"]
        .as_array()
        .unwrap();
    let intent = |request_id: &str| {
        receipts
            .iter()
            .find(|row| row["request_id"] == request_id)
            .unwrap()["intent"]
            .clone()
    };
    let mut tracker_request = intent("add-sluice-countdown")["request"].clone();
    let target = tracker_request
        .as_object_mut()
        .unwrap()
        .remove("participant_key")
        .unwrap();
    assert_eq!(target, participant.participant_key);
    app.update_encounter_manual_tracker(
        encounter_key,
        &participant.participant_key,
        serde_json::from_value::<UpdateEncounterManualTrackerRequest>(tracker_request).unwrap(),
    )
    .unwrap();
    app.advance_encounter_turn(
        encounter_key,
        serde_json::from_value::<AdvanceEncounterTurnRequest>(intent("advance-round-four"))
            .unwrap(),
    )
    .unwrap();

    success(workspace_command(&db, "export", &reexported, &[]));
    assert_eq!(read_archive(&sample), read_archive(&reexported));
}

fn sample_app(db: &Path) -> atlas_app_service::AtlasAppService {
    use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
    AtlasAppService::start(AtlasAppServiceOptions {
        default_search_profile: None,
        path_mode: atlas_runtime::AtlasPathMode::Global,
        source_root: None,
        embedding_cache_root: None,
        index_path: Some(db.to_owned()),
        retrieval_mode: AppServiceRetrievalMode::OnDemandSavedWorkspace,
    })
    .unwrap()
}
