use super::*;
use atlas_local_state::{LocalStateRead, LocalStateWrite};

fn populated_workspace() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("integrity.sqlite")).unwrap();
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write.connection().execute_batch(r#"
        INSERT INTO encounters(id,owner_id,encounter_key,slug,name,status,round_number,created_at,updated_at)
        VALUES (1,1,'one','one','One','running',4,'before','after'),
               (2,1,'two','two','Two','draft',1,'before','after');
        INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at)
        VALUES (11,1,'actor','pc',1,'Actor',1,'before','after'),
               (22,2,'foreign','creature',1,'Foreign',1,'before','after');
        UPDATE encounters SET current_turn_participant_key='actor' WHERE id=1;
        INSERT INTO encounter_condition_applications(encounter_id,participant_key,application_id,grant_slot,condition_id)
        VALUES (1,'actor','consumed','condition',NULL);
        INSERT INTO encounter_pending_consequences VALUES(1,'pending','actor','{"version":1,"payload":{"kept":"pending"}}');
        INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,receipt_json)
        VALUES (1,'event','event','{}','{"version":1,"payload":{"kept":"event"}}');
        INSERT INTO encounter_boundary_claims(encounter_id,participant_key,round_number,kind,scope_key,request_id)
        VALUES (1,'actor',4,'persistent_damage','','event');
    "#).unwrap();
    write.commit().unwrap();
    (directory, workspace)
}

#[test]
fn integrity_constraints_reject_invalid_writes_and_defer_current_turn_until_commit() {
    let (_directory, workspace) = populated_workspace();
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    for sql in [
        "UPDATE encounters SET status='runnning' WHERE id=1",
        "UPDATE encounter_participants SET participant_kind='hazrad' WHERE id=11",
        "UPDATE encounter_participants SET participant_variant='eliet' WHERE id=11",
        "UPDATE encounter_participants SET side='allly' WHERE id=11",
        "UPDATE encounter_pending_consequences SET participant_key='foreign'",
        "UPDATE encounter_boundary_claims SET participant_key='foreign'",
        "UPDATE encounter_condition_applications SET participant_key='foreign'",
    ] {
        assert!(write.connection().execute_batch(sql).is_err(), "{sql}");
    }
    write
        .connection()
        .execute(
            "UPDATE encounters SET current_turn_participant_key='foreign' WHERE id=1",
            [],
        )
        .unwrap();
    assert!(write.commit().is_err());
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        LocalStateRead::for_local(&read)
            .unwrap()
            .encounters()
            .get("one")
            .unwrap()
            .unwrap()
            .current_turn_participant_key
            .as_deref(),
        Some("actor")
    );
}

#[test]
fn integrity_constraints_allow_current_participant_and_encounter_deletion() {
    for remove_encounter in [false, true] {
        let (_directory, workspace) = populated_workspace();
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateWrite::for_local(&write).unwrap();
        if remove_encounter {
            assert!(state.encounters().delete("one").unwrap());
        } else {
            assert!(state.encounters().remove_participant("actor").unwrap());
        }
        write.commit().unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            !read
                .connection()
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
        assert_eq!(
            read.connection()
                .query_row(
                    "SELECT count(*) FROM encounter_pending_consequences",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            read.connection()
                .query_row(
                    "SELECT count(*) FROM encounter_condition_applications",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            read.connection()
                .query_row(
                    "SELECT count(*) FROM encounter_participants WHERE participant_key='foreign'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        if !remove_encounter {
            assert!(
                LocalStateRead::for_local(&read)
                    .unwrap()
                    .encounters()
                    .get("one")
                    .unwrap()
                    .unwrap()
                    .current_turn_participant_key
                    .is_none()
            );
        }
    }
}
