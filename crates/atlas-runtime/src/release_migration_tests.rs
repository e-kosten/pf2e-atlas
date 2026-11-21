use super::*;
use crate::test_support::{occurrence_vector_bits, sql_rows};
use crate::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, WorkspaceArchive};
use flate2::read::GzDecoder;
use std::io::Read;

const LEDGER: &str = "SELECT version,name,checksum FROM schema_migrations ORDER BY version";
const BASELINE_CHECKSUM: &str = "5e0abae6ea6364eeb0408bab544fdafb2f1e25ecc6d341908a6cedb69928009e";
const TRAITS: &str = "SELECT * FROM trait_members ORDER BY value,record_key,owners_json";
const PROOFS: &str = "SELECT record_id,recipe,json_remove(digests_json,'$.trait_associations') FROM reference_component_digests ORDER BY record_id";

fn load_fixture(path: &Path, bytes: &[u8]) -> Workspace {
    register_extensions().unwrap();
    let mut data = Vec::new();
    GzDecoder::new(bytes).read_to_end(&mut data).unwrap();
    std::fs::write(path, data).unwrap();
    Workspace::existing(path).unwrap()
}

fn schema_42(path: &Path) -> Workspace {
    let workspace = load_fixture(
        path,
        include_bytes!("../tests/fixtures/schema-42.sqlite.gz"),
    );
    assert_eq!(
        workspace.check_migration_history(RELEASE_HISTORY).unwrap(),
        42
    );
    workspace
}

fn archive_path(workspace: &Workspace) -> std::path::PathBuf {
    workspace
        .path()
        .with_file_name("workspace.sqlite.upgrade-archive.json")
}

fn runtime(workspace: &Workspace) -> AtlasRuntime {
    AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: Some(workspace.path().into()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap()
}

fn archived_json(archive: WorkspaceArchive) -> serde_json::Value {
    let mut data = Vec::new();
    archive.write(&mut data).unwrap();
    serde_json::from_slice(&data).unwrap()
}

#[test]
fn schema_42_migrates_in_place_and_archives_only_saved_work() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    let write = workspace.write(42).unwrap();
    write.connection().execute_batch(
        "INSERT INTO saved_lists(id,owner_id,list_key,slug,name,created_at,updated_at)
            VALUES(17,1,'kept-list','kept','Retained list','before','before');
         INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at)
            SELECT 17,key,0,'Kept note',last_known_name,'before','before' FROM record_identity LIMIT 1;
         INSERT INTO parties(party_key,owner_id,name) VALUES('kept-party',1,'Retained party');
         INSERT INTO party_members(member_key,party_key,position,name,level)
            VALUES('kept-member','kept-party',0,'Retained member',2);
         INSERT INTO encounters(owner_id,encounter_key,slug,name,status,created_at,updated_at)
            VALUES(1,'running','running','Running encounter','running','before','before'),
                  (1,'archived','archived','Archived encounter','archived','before','before');
         DELETE FROM reference_component_digests WHERE record_id=(SELECT min(record_id) FROM records);",
    ).unwrap();
    write.commit().unwrap();

    let before = workspace.read(42).unwrap();
    assert!(WorkspaceArchive::capture(&before).is_err());
    let key = before.workspace_key().unwrap();
    let revision = before.reference_revision().unwrap();
    let members = sql_rows(before.connection(), TRAITS);
    assert!(!members.is_empty());
    let proofs = sql_rows(before.connection(), PROOFS);
    assert!(!proofs.is_empty());
    let ledger = sql_rows(before.connection(), LEDGER);
    assert_eq!(
        ledger[0][2],
        rusqlite::types::Value::Text(BASELINE_CHECKSUM.into())
    );
    let mut reference_rows =
        atlas_index::test_support::reference_preservation_rows(before.connection()).unwrap();
    reference_rows.remove("trait_references");
    reference_rows.remove("reference_component_digests");
    for list in reference_rows.get_mut("saved_lists").unwrap() {
        list.push(rusqlite::types::Value::Integer(0));
    }
    drop(before);

    let archive_file = archive_path(&workspace);
    std::fs::write(&archive_file, "older archive").unwrap();
    initialize_workspace(workspace.path()).unwrap();
    assert!(!workspace.upgrade_backup_path().exists());
    let after = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(after.workspace_key().unwrap(), key);
    assert_eq!(after.reference_revision().unwrap(), revision);
    assert_eq!(sql_rows(after.connection(), TRAITS), members);
    assert_eq!(sql_rows(after.connection(), PROOFS), proofs);
    assert_eq!(sql_rows(after.connection(), LEDGER)[0], ledger[0]);
    let mut migrated =
        atlas_index::test_support::reference_preservation_rows(after.connection()).unwrap();
    migrated.remove("workspace_settings");
    assert_eq!(migrated.remove("encounter_adjustments"), Some(Vec::new()));
    migrated.remove("reference_component_digests");
    assert_eq!(reference_rows.remove("party_members").unwrap().len(), 1);
    assert_eq!(migrated.remove("encounter_pc_characters"), Some(vec![]));
    assert_eq!(migrated.remove("character_creation_receipts"), Some(vec![]));
    assert_eq!(
        migrated.remove("party_character_memberships"),
        Some(vec![vec![
            "kept-party".to_owned().into(),
            "kept-member".to_owned().into(),
            0.into()
        ]])
    );
    assert_eq!(
        migrated.remove("characters"),
        Some(vec![vec![
            "kept-member".to_owned().into(),
            1.into(),
            "Retained member".to_owned().into(),
            2.into(),
            0.into(),
            rusqlite::types::Value::Null,
            rusqlite::types::Value::Null
        ]])
    );
    let changed: Vec<_> = reference_rows
        .keys()
        .chain(migrated.keys())
        .filter(|key| reference_rows.get(*key) != migrated.get(*key))
        .collect();
    assert!(changed.is_empty(), "changed tables: {changed:?}");
    assert_eq!(
        atlas_index::classify_reference_upgrade(after.connection()).unwrap(),
        atlas_index::ReferenceUpgrade::SourceReplacement
    );
    drop(after);

    let bytes = std::fs::read(&archive_file).unwrap();
    let archive = WorkspaceArchive::read(bytes.as_slice()).unwrap();
    let source = runtime(&workspace);
    let preserved: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(preserved["version"], "1.1");
    assert_eq!(
        preserved["saved"]["owners"][0]["parties"][0]["members"][0]["key"],
        "kept-member"
    );
    assert_eq!(
        archived_json(source.capture_workspace_archive().unwrap())["version"],
        "4.0"
    );
    let target = initialize_workspace(directory.path().join("restore.sqlite")).unwrap();
    let target = runtime(&target);
    let preflight = target.preflight_workspace_restore().unwrap();
    target
        .restore_workspace_archive(&archive, &preflight, false)
        .unwrap();
    assert_eq!(
        archived_json(source.capture_workspace_archive().unwrap()),
        archived_json(target.capture_workspace_archive().unwrap())
    );

    initialize_workspace(workspace.path()).unwrap();
    assert_eq!(std::fs::read(archive_file).unwrap(), bytes);
}

#[test]
fn schema_43_archive_migration_preserves_list_and_reference_revision() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    workspace
        .migrate_history(
            MigrationHistory {
                migrations: &RELEASE_HISTORY.migrations[..2],
                ..RELEASE_HISTORY
            },
            |_| Ok(()),
        )
        .unwrap();
    let write = workspace.write(43).unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO saved_lists(owner_id,list_key,slug,name,revision,created_at,updated_at)
         VALUES(1,'list_earlier_arc','earlier-arc','Earlier arc',6,'created','updated')",
            [],
        )
        .unwrap();
    write.commit().unwrap();
    let before = workspace.read(43).unwrap();
    let revision = before.reference_revision().unwrap();
    let reference =
        atlas_index::test_support::reference_preservation_rows(before.connection()).unwrap();
    drop(before);

    initialize_workspace(workspace.path()).unwrap();
    let after = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(after.reference_revision().unwrap(), revision);
    let mut expected = reference;
    assert_eq!(expected.remove("party_members"), Some(vec![]));
    for table in [
        "characters",
        "party_character_memberships",
        "encounter_pc_characters",
        "character_creation_receipts",
    ] {
        expected.insert(table.into(), vec![]);
    }
    for list in expected.get_mut("saved_lists").unwrap() {
        list.push(rusqlite::types::Value::Integer(0));
    }
    expected.insert("encounter_adjustments".into(), vec![]);
    assert_eq!(
        atlas_index::test_support::reference_preservation_rows(after.connection()).unwrap(),
        expected
    );
    let backup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(archive_path(&workspace)).unwrap()).unwrap();
    assert_eq!(backup["version"], "1.1");
    let lists = backup["saved"]["owners"][0]["lists"].as_array().unwrap();
    assert!(lists.iter().any(|list| list["key"] == "list_earlier_arc"
        && list["revision"] == 6
        && list.get("archived").is_none()));
}

#[test]
fn schema_44_upgrade_preserves_active_participant_and_conditions() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
    register_extensions().unwrap();
    workspace
        .migrate_history(
            atlas_storage::MigrationHistory {
                migrations: &RELEASE_HISTORY.migrations[..3],
                baseline_provenance: RELEASE_HISTORY.baseline_provenance,
            },
            |_| Ok(()),
        )
        .unwrap();
    let write = workspace.write(44).unwrap();
    write.connection().execute_batch(
        "INSERT INTO encounters(id,owner_id,encounter_key,slug,name,status,round_number,created_at,updated_at)
            VALUES(17,1,'migration-encounter','migration-encounter','Encounter','running',2,'before','before');
         INSERT INTO encounter_participants(id,encounter_id,participant_key,participant_kind,position,display_name,
            side,initiative,initiative_order,created_at,updated_at)
            VALUES(22,17,'actor','pc',1,'Hero','pc',15,1,'before','before');
         UPDATE encounters SET current_turn_participant_key='actor' WHERE id=17;
         INSERT INTO saved_lists(owner_id,list_key,slug,name,revision,created_at,updated_at,archived)
            VALUES(1,'archived-list','archived-list','Archived list',6,'before','before',1);
         INSERT INTO encounter_participant_conditions(id,participant_id,name,created_at,updated_at)
            VALUES(31,22,'GM condition','before','before');",
    ).unwrap();
    write.commit().unwrap();

    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let backup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(archive_path(&workspace)).unwrap()).unwrap();
    assert_eq!(backup["version"], "2.0");
    assert_eq!(
        sql_rows(
            read.connection(),
            "SELECT archived FROM saved_lists WHERE list_key='archived-list'"
        ),
        vec![vec![rusqlite::types::Value::Integer(1)]]
    );
    let current = sql_rows(read.connection(),
        "SELECT current_turn_participant_key, round_number, p.id, c.id, c.name
         FROM encounters e JOIN encounter_participants p ON p.participant_key=e.current_turn_participant_key
         JOIN encounter_participant_conditions c ON c.participant_id=p.id");
    use rusqlite::types::Value::{Integer, Text};
    assert_eq!(
        current,
        vec![vec![
            Text("actor".into()),
            Integer(2),
            Integer(22),
            Integer(31),
            Text("GM condition".into())
        ]]
    );
}

#[test]
fn schema_42_vector_bytes_and_generation_survive_upgrade() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "actors",
            "Actor",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Goblin","type":"npc","system":{"traits":{"value":["goblin"]}}}),
        ),
    ]);
    atlas_index::test_support::publish_input_in(&workspace, 42, &input).unwrap();
    let read = workspace.read(42).unwrap();
    let vectors = occurrence_vector_bits(read.connection());
    assert_eq!(vectors.len(), 2);
    let revision = read.reference_revision().unwrap();
    drop(read);
    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(occurrence_vector_bits(read.connection()), vectors);
    assert_eq!(read.reference_revision().unwrap(), revision);
}

#[test]
fn workspace_upgrade_checks_source_context_and_keeps_migration_atomic() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    let original = workspace.read(42).unwrap();
    let ledger = sql_rows(original.connection(), LEDGER);
    let revision = original.reference_revision().unwrap();
    drop(original);

    let archive_file = archive_path(&workspace);
    std::fs::create_dir(&archive_file).unwrap();
    assert!(
        initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string()
            .contains("could not capture upgrade archive")
    );
    std::fs::remove_dir(&archive_file).unwrap();

    let write = workspace.write(42).unwrap();
    write
        .connection()
        .execute("UPDATE artifact_context SET context_json='{}'", [])
        .unwrap();
    write.commit().unwrap();
    assert!(
        initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string()
            .contains("stored context checksum mismatch")
    );
    WorkspaceArchive::read(std::fs::File::open(&archive_file).unwrap()).unwrap();
    let original = workspace.read(42).unwrap();
    assert_eq!(original.reference_revision().unwrap(), revision);
    assert_eq!(sql_rows(original.connection(), LEDGER), ledger);
    assert_eq!(
        sql_rows(
            original.connection(),
            "SELECT name FROM sqlite_schema WHERE name IN ('workspace_settings','trait_references')"
        ),
        vec![vec![rusqlite::types::Value::Text(
            "trait_references".into()
        )]]
    );
}

#[test]
fn schema_42_authenticates_the_ledger_before_capturing() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    let write = workspace.write(42).unwrap();
    write
        .connection()
        .execute("UPDATE schema_migrations SET checksum=?", ["0".repeat(64)])
        .unwrap();
    write.commit().unwrap();
    let archive_file = archive_path(&workspace);
    std::fs::write(&archive_file, "retained archive").unwrap();
    let error = initialize_workspace(workspace.path()).unwrap_err();
    assert!(
        error.to_string().contains("applied migration 42 differs"),
        "{error}"
    );
    assert_eq!(
        std::fs::read_to_string(archive_file).unwrap(),
        "retained archive"
    );
    assert!(workspace.read(42).is_ok());
}

#[test]
fn migration_retains_recognized_retired_proofs() {
    for legacy in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let workspace = schema_42(&directory.path().join("workspace.sqlite"));
        let write = workspace.write(42).unwrap();
        let c = write.connection();
        let proofs = sql_rows(
            c,
            "SELECT * FROM reference_component_digests ORDER BY record_id",
        );
        if legacy {
            let mut context = atlas_index::read_stored_reference_context(c)
                .unwrap()
                .unwrap();
            context.source_contract =
                "6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8".into();
            c.execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                rusqlite::params![
                    serde_json::to_string(&context).unwrap(),
                    context.sha256().unwrap()
                ],
            )
            .unwrap();
        }
        write.commit().unwrap();
        initialize_workspace(workspace.path()).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(
            atlas_index::classify_reference_upgrade(read.connection()).unwrap(),
            atlas_index::ReferenceUpgrade::SourceReplacement
        );
        assert_eq!(
            sql_rows(
                read.connection(),
                "SELECT * FROM reference_component_digests ORDER BY record_id"
            ),
            proofs
        );
        WorkspaceArchive::read(std::fs::File::open(archive_path(&workspace)).unwrap()).unwrap();
    }
}

#[test]
fn schema_42_rejects_corruption_under_the_current_source_contract() {
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "actions",
            "Item",
            serde_json::json!({
                "_id": "aaaaaaaaaaaaaaaa",
                "name": "Demoralize",
                "type": "action",
                "system": {"traits": {"value": ["auditory", "emotion", "fear", "mental"]}}
            }),
        ),
    ]);
    for (corrupt, expected_error) in [
        (
            "UPDATE trait_members SET source_label='corrupt label'",
            "trait memberships differ",
        ),
        (
            "UPDATE record_bodies SET snapshot=X'00'",
            "Repair the reference data",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let workspace = schema_42(&directory.path().join("workspace.sqlite"));
        atlas_index::test_support::publish_input_in(&workspace, 42, &input).unwrap();
        let write = workspace.write(42).unwrap();
        let ledger = sql_rows(write.connection(), LEDGER);
        let revision = write.reference_revision().unwrap();
        assert!(write.connection().execute(corrupt, []).unwrap() > 0);
        write.commit().unwrap();
        let error = initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected_error), "{error}");
        let read = workspace.read(42).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision);
        assert_eq!(sql_rows(read.connection(), LEDGER), ledger);
        WorkspaceArchive::read(std::fs::File::open(archive_path(&workspace)).unwrap()).unwrap();
    }
}

#[test]
fn schema_41_is_refused_before_capture() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = load_fixture(
        &directory.path().join("workspace.sqlite"),
        include_bytes!("../tests/fixtures/schema-41.sqlite.gz"),
    );
    let original = std::fs::read(workspace.path()).unwrap();
    let error = initialize_workspace(workspace.path()).unwrap_err();
    assert!(
        error.to_string().contains("must be upgraded to schema 42"),
        "{error}"
    );
    assert_eq!(std::fs::read(workspace.path()).unwrap(), original);
    assert!(!archive_path(&workspace).exists());
    assert!(!workspace.upgrade_backup_path().exists());
}

#[test]
fn domain_fixture_has_the_runtime_release_schema_and_ledger() {
    let directory = tempfile::tempdir().unwrap();
    let runtime = initialize_workspace(directory.path().join("runtime.sqlite")).unwrap();
    let domain =
        atlas_index::test_support::initialize_workspace(directory.path().join("domain.sqlite"))
            .unwrap();
    let runtime = runtime.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let domain = domain.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let schema = "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name";
    assert_eq!(
        sql_rows(runtime.connection(), schema),
        sql_rows(domain.connection(), schema)
    );
    assert_eq!(
        sql_rows(runtime.connection(), LEDGER),
        sql_rows(domain.connection(), LEDGER)
    );
}

fn workspace_at_schema(path: &Path, version: u32) -> Workspace {
    register_extensions().unwrap();
    let workspace = Workspace::at(path).unwrap();
    let migrations = RELEASE_HISTORY
        .migrations
        .split_at(
            RELEASE_HISTORY
                .migrations
                .partition_point(|step| step.version <= version),
        )
        .0;
    workspace
        .migrate_history(
            MigrationHistory {
                migrations,
                ..RELEASE_HISTORY
            },
            |_| Ok(()),
        )
        .unwrap();
    workspace
}

#[test]
fn schema_45_upgrade_captures_party_before_reference_tables_exist() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = workspace_at_schema(&directory.path().join("workspace.sqlite"), 45);
    let write = workspace.write(45).unwrap();
    write.connection().execute_batch(
        "INSERT INTO parties(owner_id,party_key,name,revision) VALUES(1,'stable-party','Party',2);
         INSERT INTO party_members(member_key,party_key,position,name,level)
             VALUES('stable-member','stable-party',0,'PC',7);",
    ).unwrap();
    write.commit().unwrap();

    initialize_workspace(workspace.path()).unwrap();
    let backup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(archive_path(&workspace)).unwrap()).unwrap();
    assert_eq!(backup["version"], "1.1");
    assert_eq!(
        backup["saved"]["owners"][0]["parties"][0]["members"][0]["key"],
        "stable-member"
    );
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let store = atlas_local_state::LocalStateRead::for_local(&read).unwrap();
    let party = store.parties().get("stable-party").unwrap();
    assert_eq!(party.revision, 2);
    assert_eq!(party.members[0].level, Some(7));
    assert!(
        store
            .characters()
            .get("stable-member")
            .unwrap()
            .reference
            .is_none()
    );
}

#[test]
fn schema_46_promotes_party_members_and_preserves_encounter_state() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = workspace_at_schema(&directory.path().join("workspace.sqlite"), 46);
    let write = workspace.write(46).unwrap();
    let ledger = sql_rows(write.connection(), LEDGER);
    write
        .connection()
        .execute_batch(
            "INSERT INTO parties(party_key,owner_id,name,revision) VALUES('old-party',1,'Party',2);
         INSERT INTO party_members(member_key,party_key,position,name,level)
         VALUES('old-pc','old-party',0,'Same name',7),('other-pc','old-party',1,'Same name',NULL);",
        )
        .unwrap();
    let state = atlas_local_state::LocalStateWrite::for_local(&write).unwrap();
    let encounter = state
        .encounters()
        .create(atlas_local_state::NewEncounter {
            name: "Existing fight".into(),
            slug: "existing-fight".into(),
            note: None,
            description: None,
        })
        .unwrap();
    let pc = state
        .encounters()
        .add_participant(&encounter.encounter_key, old_combatant())
        .unwrap();
    let adjustment = atlas_local_state::EncounterAdjustment {
        adjustment_key: "gm-armor".into(),
        target: atlas_domain::AdjustmentTarget::ArmorClass,
        name: "Armor loss".into(),
        amount: -2,
        modifier_type: atlas_domain::AdjustmentModifierType::Stacks,
        timing: None,
    };
    let adjustment_intent = serde_json::json!({"action": "gm_adjustment"});
    let encounters = state.encounters();
    let revision = encounters
        .get(&encounter.encounter_key)
        .unwrap()
        .unwrap()
        .revision;
    encounters
        .apply_condition_event(
            &encounter.encounter_key,
            "gm-adjustment",
            &adjustment_intent,
            revision,
            atlas_local_state::PreparedConditionEvent {
                adjustments: vec![atlas_local_state::AdjustmentChange {
                    participant_key: pc.participant_key.clone(),
                    adjustment_key: adjustment.adjustment_key.clone(),
                    value: Some(adjustment.clone()),
                }],
                ..Default::default()
            },
        )
        .unwrap();
    write.commit().unwrap();
    initialize_workspace(workspace.path()).unwrap();
    let mut backup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(archive_path(&workspace)).unwrap()).unwrap();
    assert_eq!(backup["version"], "3.0");
    let migrated = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let applied = sql_rows(migrated.connection(), LEDGER);
    assert_eq!(applied.len(), ledger.len() + 3);
    assert_eq!(applied[..ledger.len()], ledger);
    assert_eq!(
        applied[ledger.len()][0],
        rusqlite::types::Value::Integer(47)
    );
    assert_eq!(
        applied.last().unwrap()[0],
        rusqlite::types::Value::Integer(49)
    );

    let state = atlas_local_state::LocalStateRead::for_local(&migrated).unwrap();
    let party = state.parties().get("old-party").unwrap();
    assert_eq!(party.revision, 2);
    assert_eq!(
        party
            .members
            .iter()
            .map(|member| (member.character_key.as_str(), member.level))
            .collect::<Vec<_>>(),
        [("old-pc", Some(7)), ("other-pc", None)]
    );
    assert!(
        state
            .characters()
            .get("old-pc")
            .unwrap()
            .reference
            .is_none()
    );
    assert!(
        state
            .characters()
            .encounter_character(&encounter.encounter_key, &pc.participant_key)
            .unwrap()
            .is_none()
    );
    let saved = state
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap();
    assert_eq!(saved.participants[0].current_hp, Some(23));
    assert_eq!(saved.participants[0].initiative, Some(12));
    assert_eq!(saved.participants[0].display_name, "Old combat name");
    assert_eq!(
        saved.participants[0].adjustments.as_slice(),
        std::slice::from_ref(&adjustment)
    );
    assert!(
        state
            .encounters()
            .condition_event_receipt(
                &encounter.encounter_key,
                "gm-adjustment",
                &adjustment_intent
            )
            .unwrap()
            .is_some()
    );
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    let add_turn_receipt = "INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,boundary_key) SELECT id,'turn',?2,'{}','migration-test-boundary' FROM encounters WHERE encounter_key=?1";
    write
        .connection()
        .execute(
            add_turn_receipt,
            [&encounter.encounter_key, "first-turn-receipt"],
        )
        .unwrap();
    let duplicate_boundary = write
        .connection()
        .execute(
            add_turn_receipt,
            [&encounter.encounter_key, "second-turn-receipt"],
        )
        .unwrap_err();
    assert_eq!(
        duplicate_boundary.sqlite_error().unwrap().extended_code,
        rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
    );
    drop(write);
    // The release upgrade has no member-owned reference tables. A preview archive
    // can still carry the earlier member-owned links in its version 3 payload.
    backup["saved"]["owners"][0]["pc"] = serde_json::json!({
        "members": [{
            "member_key": "old-pc",
            "reference": {"abilities": [{
                "key": "a".repeat(64), "kind": "feat", "label": "Selected feat",
                "choice": "Chosen option", "automatic": null, "ambiguous": false,
                "manual": "feats:testfeat00000001"
            }]}
        }],
        "associations": [{"participant_key": pc.participant_key, "member_key": "old-pc"}]
    });
    backup["saved"]["references"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "key": "feats:testfeat00000001",
            "last_known_name": "Selected feat"
        }));
    let mut duplicate = backup.clone();
    let references = duplicate["saved"]["owners"][0]["pc"]["members"]
        .as_array_mut()
        .unwrap();
    references.push(references[0].clone());
    assert!(
        WorkspaceArchive::read(duplicate.to_string().as_bytes())
            .err()
            .is_some_and(|message| message.contains("duplicate legacy PC reference"))
    );
    let target = initialize_workspace(directory.path().join("restore-preview.sqlite")).unwrap();
    let target = runtime(&target);
    let preflight = target.preflight_workspace_restore().unwrap();
    target
        .restore_workspace_archive(
            &WorkspaceArchive::read(backup.to_string().as_bytes()).unwrap(),
            &preflight,
            false,
        )
        .unwrap();
    let read = target.read_state().unwrap();
    let restored = read
        .local_state()
        .unwrap()
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap();
    assert_eq!(restored.participants[0].adjustments, [adjustment]);
    let state = read.local_state().unwrap();
    let reference = state.characters().get("old-pc").unwrap().reference.unwrap();
    let ability = &reference.abilities[0];
    assert_eq!(ability.choice.as_deref(), Some("Chosen option"));
    assert_eq!(
        ability.manual.as_ref().unwrap().to_string(),
        "feats:testfeat00000001"
    );
    assert_eq!(
        state
            .characters()
            .encounter_character(&encounter.encounter_key, &pc.participant_key)
            .unwrap()
            .as_deref(),
        Some("old-pc")
    );
}

#[test]
fn schema_47_upgrade_keeps_legacy_archive_shape_and_character_history() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = schema_42(&directory.path().join("workspace.sqlite"));
    workspace
        .migrate_history(
            MigrationHistory {
                migrations: &RELEASE_HISTORY.migrations[..RELEASE_HISTORY
                    .migrations
                    .partition_point(|m| m.version <= 47)],
                ..RELEASE_HISTORY
            },
            |_| Ok(()),
        )
        .unwrap();
    let write = workspace.write(47).unwrap();
    write.connection().execute_batch(
        r#"INSERT INTO characters(character_key,owner_id,name,level,revision,reference_payload)
            VALUES('kept-character',1,'Kept Character',7,3,'{"abilities":[]}');
           INSERT INTO character_creation_receipts(owner_id,request_id,intent_digest,character_key)
            VALUES(1,'old-import','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa','kept-character');"#,
    ).unwrap();
    write.commit().unwrap();
    let read = workspace.read(47).unwrap();
    let revision = read.reference_revision().unwrap();
    let corpus = sql_rows(read.connection(), TRAITS);
    drop(read);

    initialize_workspace(workspace.path()).unwrap();
    assert!(workspace.read(47).is_err());
    let upgraded = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(upgraded.reference_revision().unwrap(), revision);
    assert_eq!(sql_rows(upgraded.connection(), TRAITS), corpus);
    let chars = atlas_local_state::LocalStateRead::for_local(&upgraded)
        .unwrap()
        .characters();
    assert!(
        chars
            .pathbuilder_source("kept-character")
            .unwrap()
            .is_none()
    );

    let before = std::fs::read(archive_path(&workspace)).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(saved["version"], "4.0");
    assert_eq!(
        saved["saved"]["owners"][0]["pc"],
        serde_json::json!({
            "characters": [{
                "character_key": "kept-character", "name": "Kept Character", "level": 7,
                "revision": 3, "reference": {"abilities":[]}
            }],
            "creation_receipts": [{
                "request_id": "old-import", "intent_digest": "a".repeat(64), "character_key": "kept-character"
            }]
        })
    );
    assert_eq!(
        archived_json(WorkspaceArchive::capture(&upgraded).unwrap()),
        saved
    );
    let restored = WorkspaceArchive::read(before.as_slice()).unwrap();
    let destination = initialize_workspace(directory.path().join("restored.sqlite")).unwrap();
    let destination = runtime(&destination);
    let preflight = destination.preflight_workspace_restore().unwrap();
    destination
        .restore_workspace_archive(&restored, &preflight, false)
        .unwrap();
    assert_eq!(
        archived_json(destination.capture_workspace_archive().unwrap()),
        saved
    );
}

fn old_combatant() -> atlas_local_state::AddEncounterParticipant {
    atlas_local_state::AddEncounterParticipant {
        level_override: Some(7),
        spell_resources: vec![],
        participant_variant: atlas_local_state::ParticipantVariant::Normal,
        hp_origin: atlas_local_state::ParticipantHpOrigin::Explicit,
        variant_origin: atlas_local_state::ParticipantVariantOrigin::Explicit,
        record_key: None,
        participant_kind: atlas_local_state::ParticipantKind::Pc,
        display_name: "Old combat name".into(),
        record_title_snapshot: None,
        record_kind_snapshot: None,
        side: atlas_local_state::ParticipantSide::Pc,
        initiative: Some(12),
        max_hp: Some(35),
        current_hp: Some(23),
        temporary_hp: 0,
        note: None,
    }
}

#[test]
fn schema_48_upgrade_preserves_all_receipts_and_boundary_children() {
    use atlas_local_state::{
        EncounterBoundaryClaim, EncounterBoundaryKind, EncounterTurnState, EncounterTurnUpdate,
        LocalStateWrite, NewEncounter, PcAdditionReceipt,
    };
    let directory = tempfile::tempdir().unwrap();
    let workspace = workspace_at_schema(&directory.path().join("workspace.sqlite"), 48);
    let write = workspace.write(48).unwrap();
    let ledger = sql_rows(write.connection(), LEDGER);
    let encounters = LocalStateWrite::for_local(&write).unwrap().encounters();
    let root = encounters
        .create(NewEncounter {
            name: "Migration".into(),
            slug: "migration".into(),
            note: None,
            description: None,
        })
        .unwrap();
    let key = &root.encounter_key;
    let pc = encounters
        .add_participant(key, old_combatant())
        .unwrap()
        .participant_key;
    let intent = serde_json::json!({"turn":"next"});
    encounters
        .advance_turn_once(
            key,
            "old-turn",
            &intent,
            encounters.get(key).unwrap().unwrap().revision,
            |_| {
                Ok((
                    EncounterTurnUpdate {
                        turn: EncounterTurnState {
                            participant_key: Some(pc.clone()),
                            round_number: 1,
                            mark_running: true,
                        },
                        durations: vec![],
                        manual_trackers: vec![],
                        adjustments: vec![],
                        start_event: Some(Default::default()),
                        start_claims: vec![EncounterBoundaryClaim {
                            participant_key: pc.clone(),
                            round_number: 1,
                            kind: EncounterBoundaryKind::TurnStart,
                            scope_key: "turn-start".into(),
                        }],
                    },
                    true,
                ))
            },
        )
        .unwrap();
    let digest = serde_json::json!({"digest": "a".repeat(64)});
    encounters
        .record_pc_addition(
            key,
            "add-pc",
            &digest,
            &PcAdditionReceipt {
                characters: vec!["historical-character".into()],
                participants: vec![pc],
            },
        )
        .unwrap();
    let start_id = format!(
        "start:{}",
        encounters
            .latest_outgoing_turn(key)
            .unwrap()
            .unwrap()
            .boundary_key
    );
    let tables = [
        "SELECT * FROM encounter_command_receipts ORDER BY encounter_id,family,request_id",
        "SELECT * FROM encounter_boundary_claims",
        "SELECT * FROM encounter_outgoing_turns",
        "SELECT * FROM pragma_index_list('encounter_command_receipts') ORDER BY name",
    ];
    let rows = tables.map(|query| sql_rows(write.connection(), query));
    assert_eq!((rows[0].len(), rows[1].len(), rows[2].len()), (3, 1, 1));
    write.commit().unwrap();

    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let applied = sql_rows(read.connection(), LEDGER);
    assert_eq!(applied.len(), ledger.len() + 1);
    assert_eq!(applied[..ledger.len()], ledger);
    assert_eq!(tables.map(|query| sql_rows(read.connection(), query)), rows);
    assert!(sql_rows(read.connection(), "PRAGMA foreign_key_check").is_empty());
    let backup = std::fs::read(archive_path(&workspace)).unwrap();
    assert_eq!(
        archived_json(WorkspaceArchive::capture(&read).unwrap()),
        serde_json::from_slice::<serde_json::Value>(&backup).unwrap()
    );
    drop(read);
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    let encounters = LocalStateWrite::for_local(&write).unwrap().encounters();
    assert!(
        encounters
            .condition_event_receipt(key, &start_id, &intent)
            .unwrap()
            .is_some()
    );
    assert!(
        encounters
            .pc_addition_receipt(key, "add-pc", &[Some(&digest)])
            .unwrap()
            .is_some()
    );
    assert!(
        !encounters
            .record_addition_committed(key, &start_id, &intent)
            .unwrap()
    );
    encounters
        .save_record_addition_receipt(key, &start_id, &intent)
        .unwrap();
    let doc = encounters.export_document(key).unwrap();
    let copy = encounters
        .import_document(doc, Some("copied".into()))
        .unwrap()
        .encounter_key;
    let copied_start = format!(
        "start:{}",
        encounters
            .latest_outgoing_turn(&copy)
            .unwrap()
            .unwrap()
            .boundary_key
    );
    assert_ne!(copied_start, start_id);
    assert!(
        encounters
            .condition_event_receipt(&copy, &copied_start, &intent)
            .unwrap()
            .is_some()
    );
    assert!(
        encounters
            .record_addition_committed(&copy, &start_id, &intent)
            .unwrap()
    );
    assert!(
        !encounters
            .record_addition_committed(&copy, &copied_start, &intent)
            .unwrap()
    );
}
