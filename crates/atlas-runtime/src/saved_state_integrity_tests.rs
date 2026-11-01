use super::*;
use atlas_local_state::{LocalStateRead, LocalStateWrite};
use rusqlite::{Connection, types::Value};

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
INSERT INTO encounter_boundary_claims VALUES(1,'actor',4,'prepared_persistent_damage','fire','event');
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
    write.commit().unwrap();
}

fn rows(connection: &Connection, table: &str) -> Vec<Vec<Value>> {
    let mut statement = connection
        .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
        .unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns).map(|column| row.get(column)).collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
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
