use super::*;
use atlas_local_state::{LocalStateRead, SavedOutgoingTurn};
use rusqlite::{Connection, params, types::Value};

fn rows(connection: &Connection, table: &str) -> Vec<Vec<Value>> {
    let mut statement = connection
        .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
        .unwrap();
    let count = statement.column_count();
    statement
        .query_map([], |row| (0..count).map(|column| row.get(column)).collect())
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn turn_receipt_upgrade_preserves_all_requests_and_only_the_latest_snapshot() {
    for previous in [22, 23] {
        register_extensions().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::at(directory.path().join("turns.sqlite")).unwrap();
        workspace
            .migrate(&MIGRATIONS[..previous], |_| Ok(()))
            .unwrap();
        let write = workspace.write(previous as u32).unwrap();
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
CREATE VIRTUAL TABLE turn_vector_witness USING vec0(unit_id INTEGER PRIMARY KEY,embedding float[3]);
"#).unwrap();
        let vector_bytes = vec![0u8, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 0];
        write
            .connection()
            .execute(
                "INSERT INTO turn_vector_witness(unit_id,embedding) VALUES(1,?1)",
                [&vector_bytes],
            )
            .unwrap();
        // The writer owns the transaction; use its saved-domain reader on the same snapshot.
        let state = atlas_local_state::LocalStateWrite::new(&write);
        for id in 1..=80i64 {
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
            write.connection().execute("INSERT INTO encounter_outgoing_turns(id,encounter_id,boundary_key,request_id,intent_json,payload_json,requires_review) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,encounter,saved.boundary_key,format!("request-{id}"),format!("{{\"turn\":{id}}}"),payload,i64::from(id!=79)]).unwrap();
        }
        let receipts=write.connection().prepare("SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_outgoing_turns ORDER BY id").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let latest=write.connection().prepare("SELECT t.encounter_id,t.request_id,t.payload_json,t.requires_review FROM encounter_outgoing_turns t WHERE id=(SELECT max(id) FROM encounter_outgoing_turns WHERE encounter_id=t.encounter_id) ORDER BY encounter_id").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
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
        workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
        workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
        let read = workspace.read(24).unwrap();
        assert_eq!(tables.map(|table| rows(read.connection(), table)), before);
        let actual=read.connection().prepare("SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_turn_receipts ORDER BY CAST(substr(request_id,9) AS INTEGER)").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(actual, receipts);
        let actual=read.connection().prepare("SELECT encounter_id,request_id,payload_json,requires_review FROM encounter_outgoing_turns ORDER BY encounter_id").unwrap().query_map([],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(actual, latest);
        assert_eq!(
            read.connection()
                .query_row(
                    "SELECT embedding FROM turn_vector_witness WHERE unit_id=1",
                    [],
                    |row| row.get::<_, Vec<u8>>(0)
                )
                .unwrap(),
            vector_bytes
        );
        let store = LocalStateRead::new(&read);
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
                .outgoing_turn("one", "boundary-79")
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .encounters()
                .outgoing_turn("one", "boundary-77")
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
        let write = workspace.write(24).unwrap();
        write
            .connection()
            .execute(
                "UPDATE workspace SET reference_revision=reference_revision+1 WHERE id=1",
                [],
            )
            .unwrap();
        let store = atlas_local_state::LocalStateWrite::new(&write);
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
}

#[test]
fn failed_turn_receipt_upgrade_restores_old_payloads_schema_and_ledger() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("turn-rollback.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..23], |_| Ok(())).unwrap();
    let write = workspace.write(23).unwrap();
    write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'kept','kept','Kept','draft','before','before'); INSERT INTO encounter_outgoing_turns VALUES(7,1,'boundary','request','{}','opaque payload retained exactly',0);").unwrap();
    let before = rows(write.connection(), "encounter_outgoing_turns");
    let ledger = rows(write.connection(), "schema_migrations");
    write.commit().unwrap();
    let mut failing = MIGRATIONS[..24].to_vec();
    failing[23].backfill = Some(atlas_storage::DataBackfill {
        id: "injected-turn-upgrade-failure/v1",
        reconstruct_tables: false,
        apply: |_| {
            Err(atlas_storage::StorageError::Migration(
                "injected after turn reconstruction".into(),
            ))
        },
    });
    assert!(workspace.migrate(&failing, |_| Ok(())).is_err());
    let read = workspace.read(23).unwrap();
    assert_eq!(rows(read.connection(), "encounter_outgoing_turns"), before);
    assert_eq!(rows(read.connection(), "schema_migrations"), ledger);
    assert!(
        read.connection()
            .prepare("SELECT * FROM encounter_turn_receipts")
            .is_err()
    );
    drop(read);
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    let read = workspace.read(24).unwrap();
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
