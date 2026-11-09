use super::*;
use atlas_local_state::{LocalStateRead, SavedOutgoingTurn};
use rusqlite::{Connection, params};

fn rows(connection: &Connection, table: &str) -> Vec<Vec<rusqlite::types::Value>> {
    crate::test_support::sql_rows(connection, &format!("SELECT * FROM {table} ORDER BY rowid"))
}

#[test]
fn turn_receipt_upgrade_preserves_all_requests_and_only_the_latest_snapshot() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("turns.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    let write = workspace.write(24).unwrap();
    write.connection().execute_batch(r#"
INSERT INTO encounters(id,encounter_key,slug,name,status,current_turn_participant_key,revision,round_number,created_at,updated_at) VALUES(1,'one','one','One','running','actor',5,3,'before','after');
INSERT INTO encounters(id,encounter_key,slug,name,status,current_turn_participant_key,revision,round_number,created_at,updated_at) VALUES(2,'two','two','Two','running','other',7,4,'before','after');
INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(3,'empty','empty','Empty','draft','before','after');
INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(11,1,'actor','pc',0,'Actor',0,'before','after');
INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(22,2,'other','creature',0,'Other',0,'before','after');
INSERT INTO encounter_participant_conditions(id,participant_id,name,value,created_at,updated_at) VALUES(31,11,'Frightened',2,'before','after');
INSERT INTO encounter_condition_applications VALUES(1,'actor','consumed','condition',NULL);
INSERT INTO encounter_event_receipts VALUES(1,'event','{"event":1}','{"version":1,"payload":{"retained":true}}');
INSERT INTO encounter_boundary_claims VALUES(1,'actor',3,'prepared_persistent_damage','fire','event');
"#).unwrap();
    project_legacy_tracker_order(write.connection());
    // The writer owns the transaction; use its saved-domain reader on the same snapshot.
    let state = atlas_local_state::LocalStateWrite::for_local(&write).unwrap();
    let mut latest = Vec::new();
    for id in 1..=4i64 {
        let encounter = if id % 2 == 0 { 2 } else { 1 };
        let key = if encounter == 1 { "one" } else { "two" };
        let participant = if encounter == 1 { "actor" } else { "other" };
        let saved = SavedOutgoingTurn {
            boundary_key: format!("boundary-{id}"),
            snapshot: state
                .encounters()
                .get_with_participants(key)
                .unwrap()
                .unwrap(),
            next_participant_key: Some(participant.into()),
        };
        let payload =
            serde_json::to_string(&serde_json::json!({"version":1,"payload":saved})).unwrap();
        let request = format!("request-{id}");
        let requires_review = i64::from(id != 3);
        write.connection().execute("INSERT INTO encounter_outgoing_turns(id,encounter_id,boundary_key,request_id,intent_json,payload_json,requires_review) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,encounter,saved.boundary_key,request,format!("{{\"turn\":{id}}}"),payload,requires_review]).unwrap();
        if id == 3 || id == 4 {
            latest.push((encounter, request, payload, requires_review));
        }
    }
    let receipts=write.connection().prepare("SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_outgoing_turns ORDER BY id").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    let tables = [
        "workspace",
        "encounters",
        "encounter_participants",
        "encounter_participant_conditions",
        "encounter_condition_applications",
        "encounter_event_receipts",
        "encounter_boundary_claims",
    ];
    let before = tables.map(|table| rows(write.connection(), table));
    write.commit().unwrap();
    workspace.migrate(&MIGRATIONS[..26], |_| Ok(())).unwrap();
    workspace.migrate(&MIGRATIONS[..26], |_| Ok(())).unwrap();
    let read = workspace.read(26).unwrap();
    assert_eq!(tables.map(|table| rows(read.connection(), table)), before);
    let actual=read.connection().prepare("SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_turn_receipts ORDER BY CAST(substr(request_id,9) AS INTEGER)").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(actual, receipts);
    let actual=read.connection().prepare("SELECT encounter_id,request_id,payload_json,requires_review FROM encounter_outgoing_turns ORDER BY encounter_id").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(actual, latest);
    drop(read);
    let workspace = upgrade_historical_fixture(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let store = LocalStateRead::for_local(&read).unwrap();
    assert!(
        store
            .encounters()
            .latest_outgoing_turn("one")
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .encounters()
            .outgoing_turn("one", "boundary-3")
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .encounters()
            .outgoing_turn("one", "boundary-1")
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .encounters()
            .latest_outgoing_turn("two")
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .encounters()
            .latest_outgoing_turn("empty")
            .unwrap()
            .is_none()
    );
    assert!(
        !read
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    drop(read);
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute(
            "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1",
            [],
        )
        .unwrap();
    let store = atlas_local_state::LocalStateWrite::for_local(&write).unwrap();
    assert_eq!(
        store
            .encounters()
            .advance_turn_receipt("one", "request-1", &serde_json::json!({"turn":1}))
            .unwrap()
            .unwrap()
            .encounter_key,
        "one"
    );
    write.commit().unwrap();
}

#[test]
fn failed_turn_receipt_upgrade_restores_old_payloads_schema_and_ledger() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("turn-rollback.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..25], |_| Ok(())).unwrap();
    let write = workspace.write(25).unwrap();
    write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'kept','kept','Kept','draft','before','before'); INSERT INTO encounter_outgoing_turns VALUES(7,1,'boundary','request','{}','opaque payload retained exactly',0);").unwrap();
    let before = rows(write.connection(), "encounter_outgoing_turns");
    let ledger = rows(write.connection(), "schema_migrations");
    write.commit().unwrap();
    let mut failing = MIGRATIONS[..26].to_vec();
    failing[25].backfill = Some(atlas_storage::DataBackfill {
        id: "injected-turn-upgrade-failure/v1",
        reconstruct_tables: false,
        apply: |_| {
            Err(atlas_storage::StorageError::Migration(
                "injected after turn reconstruction".into(),
            ))
        },
    });
    assert!(
        workspace
            .migrate(&failing, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("injected after turn reconstruction")
    );
    let read = workspace.read(25).unwrap();
    assert_eq!(rows(read.connection(), "encounter_outgoing_turns"), before);
    assert_eq!(rows(read.connection(), "schema_migrations"), ledger);
    assert!(
        read.connection()
            .prepare("SELECT * FROM encounter_turn_receipts")
            .is_err()
    );
    drop(read);
    workspace.migrate(&MIGRATIONS[..26], |_| Ok(())).unwrap();
    let read = workspace.read(26).unwrap();
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT payload_json FROM encounter_outgoing_turns",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "opaque payload retained exactly"
    );
}

const RECEIPT_FIXTURE: &str = r#"
INSERT INTO encounters(id,encounter_key,slug,name,status,revision,created_at,updated_at)
VALUES(1,'owner-key','owner','Owner','draft',8,'before','after'),
      (2,'other-key','other','Other','draft',3,'before','after');
INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,current_hp,created_at,updated_at)
VALUES(11,1,'actor','pc',0,'Actor',0,9,'before','after');
INSERT INTO encounter_turn_receipts VALUES
    (1,'shared','boundary-current','{ "turn": 1 }'),
    (1,'old-turn','boundary-old','{}'),
    (2,'shared','boundary-other','{}');
INSERT INTO encounter_outgoing_turns VALUES
    (1,'shared','{ "version": 1, "payload": "outgoing bytes" }',1);
INSERT INTO encounter_event_receipts VALUES
    (1,'shared','{ "damage": 7 }','{ "version":1, "payload":{"encounter_key":"owner-key","request_id":"shared","added_condition_ids":[],"outcomes":[],"pending":[]} }'),
    (1,'by-intent','{"action":"prepare_persistent_damage"}','{}'),
    (1,'by-outcome','{}','{"version":1,"payload":{"outcomes":[{"kind":"persistent_prepared"}]}}'),
    (1,'by-claim','legacy intent','legacy result'),
    (1,'kept-mixed','{"action":"prepare_persistent_damage"}','{"version":2,"payload":{}}'),
    (1,'opaque','unknown intent','unknown result');
INSERT INTO encounter_boundary_claims VALUES
    (1,'actor',1,'persistent_damage','','shared'),
    (1,'actor',1,'prepared_persistent_recovery','fire','by-claim'),
    (1,'actor',1,'prepared_persistent_damage','fire','kept-mixed'),
    (1,'actor',1,'persistent_recovery','fire','kept-mixed');
INSERT INTO encounter_roll_receipts VALUES(1,'initiative','{}','{}');
INSERT INTO encounter_recovery_receipts VALUES(1,'rest','{}');
"#;

const LEGACY_RECEIPT_TABLES: &[&str] = &[
    "encounter_roll_receipts",
    "encounter_recovery_receipts",
    "encounter_turn_receipts",
    "encounter_event_receipts",
];

#[test]
fn command_receipt_upgrade_preserves_committed_bytes_and_drops_preparation() {
    register_extensions().unwrap();
    for from in [26, 34] {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::at(directory.path().join("commands.sqlite")).unwrap();
        workspace
            .migrate(&MIGRATIONS[..from as usize], |_| Ok(()))
            .unwrap();
        let write = workspace.write(from).unwrap();
        let db = write.connection();
        db.execute_batch(RECEIPT_FIXTURE).unwrap();
        let encounters = rows(db, "encounters");
        let participants = rows(db, "encounter_participants");
        let snapshots = rows(db, "encounter_outgoing_turns");
        let before_event = crate::test_support::sql_rows(
            db,
            "SELECT * FROM encounter_event_receipts
             WHERE request_id IN ('shared','kept-mixed','opaque') ORDER BY request_id",
        );
        let before_turn = crate::test_support::sql_rows(
            db,
            "SELECT * FROM encounter_turn_receipts ORDER BY encounter_id,request_id",
        );
        let before_claim = crate::test_support::sql_rows(
            db,
            "SELECT * FROM encounter_boundary_claims
             WHERE kind IN ('persistent_damage','persistent_recovery') ORDER BY kind",
        );
        let reference_revision = write.reference_revision().unwrap();
        write.commit().unwrap();
        workspace.migrate(&MIGRATIONS[..35], |_| Ok(())).unwrap();
        let read = workspace.read(35).unwrap();
        let db = read.connection();
        assert_eq!(rows(db, "encounters"), encounters);
        assert_eq!(rows(db, "encounter_participants"), participants);
        assert_eq!(read.reference_revision().unwrap(), reference_revision);
        assert_eq!(
            crate::test_support::sql_rows(
                db,
                "SELECT encounter_id,request_id,payload_json,requires_review
                 FROM encounter_outgoing_turns ORDER BY encounter_id",
            ),
            snapshots,
        );
        assert_eq!(
            crate::test_support::sql_rows(
                db,
                "SELECT encounter_id,request_id,intent_json,receipt_json FROM encounter_command_receipts
                 WHERE family='event' ORDER BY request_id",
            ),
            before_event,
        );
        assert_eq!(
            crate::test_support::sql_rows(
                db,
                "SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_command_receipts
                 WHERE family='turn' ORDER BY encounter_id,request_id",
            ),
            before_turn,
        );
        assert_eq!(
            crate::test_support::sql_rows(
                db,
                "SELECT * FROM encounter_boundary_claims ORDER BY kind"
            ),
            before_claim,
        );
        let store = LocalStateRead::for_local(&read).unwrap();
        let encounters = store.encounters();
        assert_eq!(
            encounters
                .advance_turn_receipt("owner", "shared", &serde_json::json!({"turn":1}))
                .unwrap()
                .unwrap()
                .encounter_key,
            "owner-key",
        );
        assert_eq!(
            encounters
                .condition_event_receipt("owner-key", "shared", &serde_json::json!({"damage":7}))
                .unwrap()
                .unwrap()
                .request_id,
            "shared",
        );
        let claim = atlas_local_state::EncounterBoundaryClaim {
            participant_key: "actor".into(),
            round_number: 1,
            kind: atlas_local_state::EncounterBoundaryKind::PersistentDamage,
            scope_key: String::new(),
        };
        assert_eq!(
            encounters
                .boundary_receipt("owner-key", &claim)
                .unwrap()
                .unwrap()
                .request_id,
            "shared",
        );
        assert!(
            encounters
                .condition_event_receipt("owner", "shared", &serde_json::json!({"damage":7}))
                .unwrap()
                .is_none()
        );
        assert!(
            db.prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .is_ok_and(|found| !found)
        );
        for table in LEGACY_RECEIPT_TABLES {
            assert!(
                db.prepare(&format!("SELECT * FROM {table}")).is_err(),
                "{table}"
            );
        }
        drop(read);
        let write = workspace.write(35).unwrap();
        let db = write.connection();
        for sql in [
            // Only a turn receipt in the same encounter is a valid snapshot parent.
            "UPDATE encounter_outgoing_turns SET request_id='opaque'",
            "UPDATE encounter_outgoing_turns SET encounter_id=2,request_id='old-turn'",
            "INSERT INTO encounter_command_receipts VALUES(2,'turn','duplicate','{}','boundary-old',NULL)",
            "INSERT INTO encounter_boundary_claims VALUES(1,'actor',2,'prepared_persistent_damage','fire','x')",
        ] {
            assert!(db.execute(sql, []).is_err(), "{sql}");
        }
        db.execute("DELETE FROM encounters WHERE id=1", []).unwrap();
        assert!(rows(db, "encounter_outgoing_turns").is_empty());
        assert!(rows(db, "encounter_boundary_claims").is_empty());
        assert_eq!(rows(db, "encounter_command_receipts").len(), 1);
    }
}

#[test]
fn failed_command_receipt_migration_restores_legacy_receipts_claims_and_ledger() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("commands-rollback.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..34], |_| Ok(())).unwrap();
    let write = workspace.write(34).unwrap();
    let db = write.connection();
    db.execute_batch(RECEIPT_FIXTURE).unwrap();
    let before = LEGACY_RECEIPT_TABLES
        .iter()
        .copied()
        .chain([
            "encounter_boundary_claims",
            "encounter_outgoing_turns",
            "schema_migrations",
        ])
        .map(|table| (table, rows(db, table)))
        .collect::<Vec<_>>();
    write.commit().unwrap();
    let mut failing = MIGRATIONS[..35].to_vec();
    failing.last_mut().unwrap().backfill = Some(atlas_storage::DataBackfill {
        id: "injected-command-receipt-failure/v1",
        reconstruct_tables: false,
        apply: |_| {
            Err(atlas_storage::StorageError::Migration(
                "after receipt consolidation".into(),
            ))
        },
    });
    assert!(
        workspace
            .migrate(&failing, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("after receipt consolidation"),
    );
    let read = workspace.read(34).unwrap();
    for (table, expected) in before {
        assert_eq!(rows(read.connection(), table), expected, "{table}");
    }
    assert!(
        read.connection()
            .prepare("SELECT * FROM encounter_command_receipts")
            .is_err()
    );
}

#[test]
fn fresh_workspace_has_fourteen_encounter_tables() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("commands-fresh.sqlite")).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name GLOB 'encounter*'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        14,
    );
}
