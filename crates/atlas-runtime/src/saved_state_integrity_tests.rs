use super::*;
use atlas_local_state::{LocalStateRead, LocalStateWrite};
use rusqlite::{Connection, types::Value};

const DELETED_CONDITION_ID: i64 = 4096;

const SAVED_TABLES: [&str; 23] = [
    "principals",
    "saved_lists",
    "saved_list_items",
    "saved_list_tags",
    "parties",
    "party_members",
    "encounters",
    "encounter_participants",
    "encounter_participant_conditions",
    "encounter_manual_trackers",
    "encounter_spell_resources",
    "encounter_expended_spells",
    "encounter_roll_receipts",
    "encounter_condition_applications",
    "encounter_event_receipts",
    "encounter_pending_consequences",
    "encounter_persistent_precedence",
    "encounter_boundary_claims",
    "encounter_outgoing_turns",
    "encounter_ability_states",
    "encounter_participant_baselines",
    "encounter_recovery_receipts",
    "sqlite_sequence",
];

const POPULATED: &str = r#"
INSERT OR IGNORE INTO record_identity(key,last_known_name,availability) VALUES('feats:aaaaaaaaaaaaaaaa','Kept reference','unavailable');
INSERT INTO principals VALUES(2,'second-owner','Second owner');
INSERT INTO saved_lists(id,list_key,slug,name,description,revision,created_at,updated_at) VALUES(1,'list','list','List','Keep description',17,'before','after');
INSERT INTO saved_list_items VALUES(1,'feats:aaaaaaaaaaaaaaaa',0,'Keep item note','Kept reference','feat','before','after');
INSERT INTO saved_list_tags VALUES(1,'kept tag','before');
INSERT INTO parties(party_key,name,revision) VALUES('party','Party',8);
INSERT INTO party_members VALUES('member','party',0,'Member',6);
INSERT INTO encounters(id,encounter_key,slug,name,status,round_number,revision,created_at,updated_at) VALUES(1,'one','one','One','running',4,9,'before','after');
INSERT INTO encounters(id,encounter_key,slug,name,status,round_number,revision,created_at,updated_at) VALUES(2,'two','two','Two','draft',1,3,'before','after');
INSERT INTO encounter_participants(id,encounter_id,participant_key,record_key,participant_kind,position,display_name,initiative_order,current_hp,max_hp,temporary_hp,revision,created_at,updated_at) VALUES(11,1,'actor','feats:aaaaaaaaaaaaaaaa','pc',0,'Actor',0,13,20,3,7,'before','after');
INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(22,2,'foreign','creature',0,'Foreign',0,'before','after');
UPDATE encounters SET current_turn_participant_key='actor' WHERE id=1;
INSERT INTO encounter_participant_conditions(id,participant_id,name,value,note,created_at,updated_at) VALUES(31,11,'Slowed',1,'Keep condition note','before','after');
INSERT INTO encounter_spell_resources VALUES(11,'slots',4,2,3);
INSERT INTO encounter_expended_spells VALUES(11,'spell');
INSERT INTO encounter_roll_receipts VALUES(1,'roll','{"roll":1}','{"version":1,"payload":{"kept":"roll"}}');
INSERT INTO encounter_condition_applications VALUES(1,'actor','consumed','condition',NULL);
INSERT INTO encounter_condition_applications VALUES(1,'actor','removed','condition',999);
INSERT INTO encounter_event_receipts VALUES(1,'event','{"event":1}','{"version":1,"payload":{"kept":"event"}}');
INSERT INTO encounter_pending_consequences VALUES(1,'pending','actor','{"version":1,"payload":{"kept":"pending"}}');
INSERT INTO encounter_persistent_precedence VALUES(11,'episode','[31,32]');
INSERT INTO encounter_boundary_claims VALUES(1,'actor',4,'persistent_damage','','event');
INSERT INTO encounter_outgoing_turns VALUES(41,1,'boundary','turn','{"turn":1}','{"version":1,"payload":{"kept":"outgoing"}}',1);
INSERT INTO encounter_ability_states VALUES('actor','owner',-1,'{"version":1,"payload":{"kept":"ability"}}');
INSERT INTO encounter_participant_baselines VALUES('actor','{"version":1,"payload":{"kept":"baseline"}}');
INSERT INTO encounter_recovery_receipts VALUES(1,'recovery','{"recovery":1}');
"#;

fn historical() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().unwrap();
    register_extensions().unwrap();
    let workspace = Workspace::at(directory.path().join("integrity.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    (directory, workspace)
}

pub(super) fn populate(workspace: &Workspace, schema: u32) {
    let write = workspace.write(schema).unwrap();
    write.connection().execute_batch(POPULATED).unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO encounter_participant_conditions(id,participant_id,name,value,note,created_at,updated_at) VALUES(?1,11,'Frightened',1,'Deleted before upgrade','before','after')",
            [DELETED_CONDITION_ID],
        )
        .unwrap();
    write
        .connection()
        .execute(
            "DELETE FROM encounter_participant_conditions WHERE id=?1",
            [DELETED_CONDITION_ID],
        )
        .unwrap();
    write.commit().unwrap();
}

fn rows(connection: &Connection, table: &str) -> Vec<Vec<Value>> {
    crate::test_support::sql_rows(connection, &format!("SELECT * FROM {table} ORDER BY rowid"))
}

fn saved_rows(connection: &Connection) -> Vec<Vec<Vec<Value>>> {
    SAVED_TABLES
        .iter()
        .map(|table| rows(connection, table))
        .collect()
}

fn vectors(connection: &Connection) -> Vec<Vec<(i64, Vec<u8>)>> {
    let tables = connection.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND sql LIKE '%USING vec0%' ORDER BY name")
        .unwrap().query_map([], |row| row.get::<_, String>(0)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(tables.len(), 2);
    tables
        .iter()
        .map(|table| {
            connection
                .prepare(&format!(
                    "SELECT unit_id,embedding FROM {table} ORDER BY unit_id"
                ))
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        })
        .collect()
}

#[test]
fn integrity_upgrade_preserves_populated_saved_graph_reference_snapshots_and_vector_bits() {
    let (_directory, workspace) = historical();
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "feats",
            "Item",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Kept reference","type":"feat","system":{"description":{"value":"<p>Kept reference prose.</p>"}}}),
        ),
    ]);
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner.maintenance(24, "integrity upgrade fixture").unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    populate(&workspace, 24);
    let read = workspace.read(24).unwrap();
    let before = saved_rows(read.connection());
    let reference = ["workspace", "record_identity", "records", "record_bodies"]
        .map(|table| rows(read.connection(), table));
    let vector_bits = vectors(read.connection());
    let ledger = rows(read.connection(), "schema_migrations");
    drop(read);
    workspace.migrate(&MIGRATIONS[..25], |_| Ok(())).unwrap();
    workspace.migrate(&MIGRATIONS[..25], |_| Ok(())).unwrap();
    let read = workspace.read(25).unwrap();
    assert_eq!(saved_rows(read.connection()), before);
    assert_eq!(
        ["workspace", "record_identity", "records", "record_bodies"]
            .map(|table| rows(read.connection(), table)),
        reference
    );
    assert_eq!(vectors(read.connection()), vector_bits);
    assert_eq!(&rows(read.connection(), "schema_migrations")[..24], ledger);
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='encounter_participant_adjustments'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert!(
        !read
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    assert!(workspace.upgrade_backup_path().exists());
}

#[test]
fn main24_upgrade_preserves_deleted_condition_high_water_and_allocates_above_it() {
    let (_directory, workspace) = historical();
    populate(&workspace, 24);
    let read = workspace.read(24).unwrap();
    let conditions = rows(read.connection(), "encounter_participant_conditions");
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0][0], Value::Integer(31));
    let sequences = rows(read.connection(), "sqlite_sequence");
    assert_eq!(
        sequences,
        vec![vec![
            Value::Text("encounter_participant_conditions".into()),
            Value::Integer(DELETED_CONDITION_ID),
        ]]
    );
    // The outgoing context and receipt migrations have their own tests. All
    // other saved rows stay exact through the direct upgrade.
    let saved = SAVED_TABLES
        .iter()
        .filter(|table| {
            !matches!(
                **table,
                "encounter_outgoing_turns"
                    | "encounter_roll_receipts"
                    | "encounter_recovery_receipts"
                    | "encounter_event_receipts"
            )
        })
        .map(|table| (*table, rows(read.connection(), table)))
        .collect::<Vec<_>>();
    let ledger = rows(read.connection(), "schema_migrations");
    drop(read);
    workspace.migrate(&MIGRATIONS[..38], |_| Ok(())).unwrap();
    let read = workspace.read(38).unwrap();
    assert_eq!(rows(read.connection(), "sqlite_sequence"), sequences);
    assert_eq!(
        rows(read.connection(), "encounter_participant_conditions"),
        conditions
    );
    for (table, before) in saved {
        assert_eq!(rows(read.connection(), table), before, "{table}");
    }
    assert_eq!(&rows(read.connection(), "schema_migrations")[..24], ledger);
    drop(read);
    let write = workspace.write(38).unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO encounter_participant_conditions(participant_id,name,value,created_at,updated_at) VALUES(11,'Frightened',1,'after upgrade','after upgrade')",
            [],
        )
        .unwrap();
    let automatic_id = write.connection().last_insert_rowid();
    assert!(automatic_id > DELETED_CONDITION_ID, "{automatic_id}");
    write.commit().unwrap();
}

#[test]
fn integrity_upgrade_rejects_invalid_history_without_changing_rows_or_ledger() {
    for (corrupt, diagnostic) in [
        (
            "UPDATE encounters SET status='runnning' WHERE id=1",
            "encounters.status",
        ),
        (
            "UPDATE encounter_participants SET participant_kind='hazrad' WHERE id=11",
            "participant_kind",
        ),
        (
            "UPDATE encounter_participants SET participant_variant='eliet' WHERE id=11",
            "participant_variant",
        ),
        (
            "UPDATE encounter_participants SET side='allly' WHERE id=11",
            "side",
        ),
        (
            "UPDATE encounters SET current_turn_participant_key='missing' WHERE id=1",
            "current_turn_participant_key",
        ),
        (
            "UPDATE encounters SET current_turn_participant_key='foreign' WHERE id=1",
            "current_turn_participant_key",
        ),
        (
            "UPDATE encounter_pending_consequences SET participant_key='foreign'",
            "encounter_pending_consequences",
        ),
        (
            "UPDATE encounter_boundary_claims SET participant_key='foreign'",
            "encounter_boundary_claims",
        ),
        (
            "UPDATE encounter_condition_applications SET participant_key='foreign'",
            "encounter_condition_applications",
        ),
        (
            "INSERT INTO encounter_participant_adjustments VALUES(11,'historical','kept','1','before','after')",
            "historical rows",
        ),
    ] {
        let (_directory, workspace) = historical();
        populate(&workspace, 24);
        let write = workspace.write(24).unwrap();
        write.connection().execute_batch(corrupt).unwrap();
        let before = saved_rows(write.connection());
        let adjustments = rows(write.connection(), "encounter_participant_adjustments");
        let ledger = rows(write.connection(), "schema_migrations");
        write.commit().unwrap();
        let error = initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
        let read = workspace.read(24).unwrap();
        assert_eq!(saved_rows(read.connection()), before, "{corrupt}");
        assert_eq!(
            rows(read.connection(), "encounter_participant_adjustments"),
            adjustments
        );
        assert_eq!(rows(read.connection(), "schema_migrations"), ledger);
        assert_eq!(
            read.connection()
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            read.connection()
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE name LIKE '%_rebuilt'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }
}

#[test]
fn integrity_constraints_reject_invalid_writes_and_defer_current_turn_until_commit() {
    let (_directory, workspace) = historical();
    populate(&workspace, 24);
    initialize_workspace(workspace.path()).unwrap();
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
        LocalStateRead::new(&read)
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
        let (_directory, workspace) = historical();
        populate(&workspace, 24);
        initialize_workspace(workspace.path()).unwrap();
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateWrite::new(&write);
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
                LocalStateRead::new(&read)
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

#[test]
fn saved_state_closeout_repairs_live_and_frozen_order_and_rejects_an_orphan_claim() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("saved-state-closeout.sqlite");
    register_extensions().unwrap();
    let workspace = Workspace::at(&path).unwrap();
    let old = atlas_index::NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION;
    workspace
        .migrate(&MIGRATIONS[..old as usize], |_| Ok(()))
        .unwrap();
    let write = workspace.write(old).unwrap();
    write.connection().execute_batch(
        "INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at)
         VALUES(1,'closeout','closeout','Closeout','draft','before','before');
         INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative,initiative_order,created_at,updated_at)
         VALUES(11,1,'a','pc',1,'A',NULL,-10,'before','before'),
               (12,1,'b','pc',2,'B',NULL,1,'before','before'),
               (13,1,'c','pc',3,'C',15,7,'before','before');
         INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,boundary_key)
         VALUES(1,'turn','advance','{}','boundary');
         INSERT INTO encounter_outgoing_turns(encounter_id,request_id,payload_json,requires_review)
         VALUES(1,'advance','{}',0);
         INSERT INTO encounter_boundary_claims VALUES(1,'a',1,'turn_end','','event');",
    ).unwrap();
    let payload = serde_json::json!({"version":1, "payload": {"snapshot": {"participants": [
        {"participant_key":"b","initiative":null,"initiative_order":1,"position":2,"retained":"extra"},
        {"participant_key":"a","initiative":null,"initiative_order":-10,"position":1},
        {"participant_key":"c","initiative":15,"initiative_order":7,"position":3}
    ]}}});
    write
        .connection()
        .execute(
            "UPDATE encounter_outgoing_turns SET payload_json=?1",
            [payload.to_string()],
        )
        .unwrap();
    write.commit().unwrap();

    let error = initialize_workspace(&path).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("boundary claim without its event receipt"),
        "{error}"
    );
    let write = workspace.write(old).unwrap();
    write.connection().execute_batch(
        "INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,receipt_json)
         VALUES(1,'event','event','{}','{}');
         UPDATE encounter_outgoing_turns
         SET payload_json=json_remove(payload_json,'$.payload.snapshot.participants[0].position');",
    ).unwrap();
    write.commit().unwrap();

    let error = initialize_workspace(&path).err().unwrap();
    assert!(
        error.to_string().contains("invalid roster position"),
        "{error}"
    );
    let write = workspace.write(old).unwrap();
    assert_eq!(
        write
            .connection()
            .query_row(
                "SELECT initiative_order FROM encounter_participants WHERE id=11",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        -10
    );
    assert!(
        write
            .connection()
            .prepare("SELECT 1 FROM sqlite_schema WHERE name='saved_list_items_position_idx'")
            .unwrap()
            .exists([])
            .unwrap()
    );
    write
        .connection()
        .execute(
            "UPDATE encounter_outgoing_turns SET payload_json=?",
            [payload.to_string()],
        )
        .unwrap();
    write.commit().unwrap();

    let upgraded = initialize_workspace(&path).unwrap();
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let rows = read
        .connection()
        .prepare("SELECT initiative_order FROM encounter_participants ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get::<_, i64>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows, [1, 2, 7]);
    let repaired = read
        .connection()
        .query_row(
            "SELECT payload_json FROM encounter_outgoing_turns",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap();
    let repaired: serde_json::Value = serde_json::from_str(&repaired).unwrap();
    let participants = &repaired["payload"]["snapshot"]["participants"];
    assert_eq!(participants[0]["initiative_order"], 2);
    assert_eq!(participants[0]["retained"], "extra");
    assert_eq!(participants[1]["initiative_order"], 1);
    assert_eq!(participants[2]["initiative_order"], 7);
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT receipt_family FROM encounter_boundary_claims",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "event"
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

    for (family, encounter_id) in [("turn", 1), ("event", 2)] {
        let write = upgraded.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        let connection = write.connection();
        connection
            .execute_batch(
                "INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at)
             VALUES(2,'other','other','Other','draft','before','before');",
            )
            .unwrap();
        connection.execute(
            "INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,boundary_key,receipt_json)
             VALUES(?1,?2,'wrong','{}',CASE WHEN ?2='turn' THEN 'other-boundary' END,
                CASE WHEN ?2='event' THEN '{}' END)",
            rusqlite::params![encounter_id, family],
        ).unwrap();
        connection.execute(
            "INSERT INTO encounter_boundary_claims(encounter_id,participant_key,round_number,kind,scope_key,request_id)
             VALUES(1,'a',2,'turn_end','','wrong')", [],
        ).unwrap();
        assert!(
            write.commit().is_err(),
            "{family} receipt in encounter {encounter_id}"
        );
    }
}
