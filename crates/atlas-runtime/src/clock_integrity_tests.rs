//! Released countdown schemas compose with integrity, receipts and reference upgrades.
use super::*;
use atlas_local_state::{
    ConditionDuration, ConditionDurationUnit, ConditionTiming, ConditionTurnEvent, LocalStateRead,
    LocalStateWrite, ManualTracker, ParticipantGameplay, SavedOutgoingTurn, TrackerProgress,
};
use rusqlite::{Connection, params};

fn envelope(value: serde_json::Value) -> String {
    serde_json::json!({"version":1,"payload":value}).to_string()
}
type VectorRows = Vec<(i64, Vec<u8>)>;
fn vector_bits(c: &Connection) -> Vec<(String, VectorRows)> {
    let tables = c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND sql LIKE '%USING vec0%' ORDER BY name").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(tables.len(), 2);
    tables
        .into_iter()
        .map(|table| {
            let values = c
                .prepare(&format!(
                    "SELECT unit_id,embedding FROM {table} ORDER BY unit_id"
                ))
                .unwrap()
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (table, values)
        })
        .collect()
}
fn ledger(c: &Connection) -> Vec<(u32, String, String)> {
    c.prepare(
        "SELECT version,name,checksum FROM schema_migrations WHERE version<=24 ORDER BY version",
    )
    .unwrap()
    .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
    .unwrap()
    .collect::<Result<Vec<_>, _>>()
    .unwrap()
}
#[test]
fn populated_main24_upgrade_preserves_clock_trackers_baseline_receipts_and_both_vector_profiles() {
    register_extensions().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("main24.sqlite");
    let workspace = Workspace::at(&path).unwrap();
    workspace.migrate(&MIGRATIONS[..24], |_| Ok(())).unwrap();
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "feats",
            "Item",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Kept","type":"feat","system":{"description":{"value":"<h2>Kept</h2><p>Preserved native values.</p>"}}}),
        ),
    ]);
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(24, "published main24 reference fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    let write = workspace.write(24).unwrap();
    write.connection().execute_batch(r#"
      INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(77,'kept-list','kept-list','Kept list','before','before');
      INSERT INTO encounters(id,encounter_key,slug,name,status,round_number,current_turn_participant_key,revision,created_at,updated_at,round_duration_seconds) VALUES(41,'clock','clock','Clock','running',3,'actor',9,'before','after',13);
      INSERT INTO encounter_participants(id,encounter_id,participant_key,record_key,participant_kind,position,display_name,initiative_order,current_hp,max_hp,revision,created_at,updated_at) VALUES(71,41,'actor','feats:aaaaaaaaaaaaaaaa','pc',0,'Actor',0,17,20,3,'before','after');
      INSERT INTO encounter_participant_conditions(id,participant_id,name,duration_amount,duration_unit,duration_anchor_key,duration_event,duration_remaining_seconds,created_at,updated_at) VALUES(91,71,'Reminder',1,'minutes','actor','end',23,'before','after');
      INSERT INTO encounter_recovery_receipts VALUES(41,'rest','{"rest":true}');
    "#).unwrap();
    let tracker = ManualTracker {
        tracker_key: "timer".into(),
        name: "Timer".into(),
        duration: ConditionDuration {
            amount: 1,
            unit: ConditionDurationUnit::Minutes,
            timing: Some(ConditionTiming {
                participant_key: "actor".into(),
                event: ConditionTurnEvent::End,
            }),
            remaining_seconds: None,
        },
        progress: TrackerProgress::Seconds(23),
    };
    write
        .connection()
        .execute(
            "INSERT INTO encounter_manual_trackers VALUES('actor','timer',?1)",
            [envelope(serde_json::json!(tracker))],
        )
        .unwrap();
    let saved = LocalStateWrite::new(&write)
        .encounters()
        .get_with_participants("clock")
        .unwrap()
        .unwrap();
    let baseline = ParticipantGameplay::from(&saved.participants[0]);
    let baseline_json = envelope(serde_json::json!(baseline));
    write
        .connection()
        .execute(
            "INSERT INTO encounter_participant_baselines VALUES('actor',?1)",
            [&baseline_json],
        )
        .unwrap();
    let mut latest_json = String::new();
    for (id, request) in [(101, "first"), (102, "second")] {
        let outgoing = SavedOutgoingTurn {
            boundary_key: format!("boundary-{request}"),
            snapshot: saved.clone(),
            next_participant_key: Some("actor".into()),
        };
        latest_json = envelope(serde_json::json!(outgoing));
        write.connection().execute("INSERT INTO encounter_outgoing_turns(id,encounter_id,boundary_key,request_id,intent_json,payload_json,requires_review) VALUES(?1,41,?2,?3,?4,?5,1)",params![id,outgoing.boundary_key,request,serde_json::json!({"advance":request}).to_string(),latest_json]).unwrap();
    }
    let before_vectors = vector_bits(write.connection());
    let before_ledger = ledger(write.connection());
    write.commit().unwrap();
    let upgraded = initialize_workspace(&path).unwrap();
    assert!(
        atlas_index::ensure_reference_projections(&upgraded, WORKSPACE_SCHEMA_VERSION).unwrap()
    );
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let domain = LocalStateRead::new(&read);
    assert_eq!(
        domain
            .encounters()
            .get_with_participants("clock")
            .unwrap()
            .unwrap(),
        saved
    );
    assert_eq!(
        domain
            .encounters()
            .participant_baseline("actor")
            .unwrap()
            .unwrap(),
        baseline
    );
    assert_eq!(vector_bits(read.connection()), before_vectors);
    assert_eq!(ledger(read.connection()), before_ledger);
    assert_eq!(read.connection().query_row("SELECT payload_json FROM encounter_participant_baselines WHERE participant_key='actor'",[],|r|r.get::<_,String>(0)).unwrap(),baseline_json);
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT payload_json FROM encounter_outgoing_turns WHERE encounter_id=41",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        latest_json
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM encounter_turn_receipts WHERE encounter_id=41",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
        2
    );
    assert_eq!(
        domain
            .encounters()
            .advance_turn_receipt("clock", "first", &serde_json::json!({"advance":"first"}))
            .unwrap()
            .unwrap(),
        saved.encounter
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT id FROM saved_lists WHERE list_key='kept-list'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        77
    );
    assert_eq!(read.connection().query_row("SELECT intent_json FROM encounter_recovery_receipts WHERE encounter_id=41 AND request_id='rest'",[],|r|r.get::<_,String>(0)).unwrap(),r#"{"rest":true}"#);
    assert!(
        !read
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    atlas_index::validate_reference(read.connection()).unwrap();
    drop(read);
    let write = upgraded.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert!(
        write
            .connection()
            .execute(
                "UPDATE encounters SET round_duration_seconds=0 WHERE id=41",
                []
            )
            .is_err()
    );
    assert!(
        write
            .connection()
            .execute(
                "UPDATE encounters SET round_duration_seconds=86401 WHERE id=41",
                []
            )
            .is_err()
    );
}
#[test]
fn legacy22_clock_conversion_precedes_receipt_compaction_and_preserves_latest_context() {
    register_extensions().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy22.sqlite");
    let workspace = Workspace::at(&path).unwrap();
    workspace.migrate(&MIGRATIONS[..22], |_| Ok(())).unwrap();
    let write = workspace.write(22).unwrap();
    write.connection().execute_batch("INSERT INTO encounters(id,encounter_key,slug,name,status,created_at,updated_at) VALUES(1,'legacy','legacy','Legacy','draft','before','before'); INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,initiative_order,created_at,updated_at) VALUES(1,1,'actor','pc',0,'Actor',0,'before','before');").unwrap();
    let duration = serde_json::json!({"amount":1,"unit":"minutes","remaining_rounds":9});
    write
        .connection()
        .execute(
            "INSERT INTO encounter_participant_baselines VALUES('actor',?1)",
            [envelope(
                serde_json::json!({"conditions":[{"duration":duration}],"marker":"baseline"}),
            )],
        )
        .unwrap();
    for (id, marker) in [(1, "older"), (2, "latest")] {
        let payload = envelope(
            serde_json::json!({"snapshot":{"participants":[{"conditions":[{"duration":duration}]}]},"marker":marker}),
        );
        write.connection().execute("INSERT INTO encounter_outgoing_turns(id,encounter_id,boundary_key,request_id,intent_json,payload_json,requires_review) VALUES(?1,1,?2,?3,'{}',?4,1)",params![id,format!("boundary-{id}"),format!("request-{id}"),payload]).unwrap();
    }
    write.commit().unwrap();
    let upgraded = initialize_workspace(&path).unwrap();
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let baseline: String = read
        .connection()
        .query_row(
            "SELECT payload_json FROM encounter_participant_baselines",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let baseline: serde_json::Value = serde_json::from_str(&baseline).unwrap();
    assert_eq!(
        baseline["payload"]["conditions"][0]["duration"]["remaining_seconds"],
        54
    );
    assert!(
        baseline["payload"]["conditions"][0]["duration"]
            .get("remaining_rounds")
            .is_none()
    );
    let outgoing: String = read
        .connection()
        .query_row(
            "SELECT payload_json FROM encounter_outgoing_turns",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let outgoing: serde_json::Value = serde_json::from_str(&outgoing).unwrap();
    assert_eq!(outgoing["payload"]["marker"], "latest");
    assert_eq!(
        outgoing["payload"]["snapshot"]["participants"][0]["conditions"][0]["duration"]["remaining_seconds"],
        54
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM encounter_turn_receipts", [], |r| {
                r.get::<_, usize>(0)
            })
            .unwrap(),
        2
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT round_duration_seconds FROM encounters", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        6
    );
}
