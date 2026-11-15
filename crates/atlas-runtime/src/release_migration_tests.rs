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
    migrated.remove("reference_component_digests");
    let changed: Vec<_> = reference_rows
        .keys()
        .chain(migrated.keys())
        .filter(|key| reference_rows.get(*key) != migrated.get(*key))
        .collect();
    assert!(changed.is_empty(), "changed tables: {changed:?}");
    atlas_index::validate_reference(after.connection()).unwrap();
    drop(after);

    let bytes = std::fs::read(&archive_file).unwrap();
    let archive = WorkspaceArchive::read(bytes.as_slice()).unwrap();
    let source = runtime(&workspace);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        archived_json(source.capture_workspace_archive().unwrap())
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
fn workspace_upgrade_checks_admission_and_keeps_migration_atomic() {
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
        .execute("UPDATE trait_members SET source_label='corrupt label'", [])
        .unwrap();
    write.commit().unwrap();
    assert!(
        initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string()
            .contains("trait memberships differ")
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
fn migration_rejects_corrupt_snapshot_and_retains_recognized_retired_proofs() {
    for retired in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let workspace = schema_42(&directory.path().join("workspace.sqlite"));
        let write = workspace.write(42).unwrap();
        let c = write.connection();
        let proofs = sql_rows(
            c,
            "SELECT * FROM reference_component_digests ORDER BY record_id",
        );
        if retired {
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
        } else {
            c.execute("UPDATE record_bodies SET snapshot=X'00' WHERE record_id=(SELECT min(record_id) FROM records)", []).unwrap();
        }
        write.commit().unwrap();
        let result = initialize_workspace(workspace.path());
        if retired {
            result.unwrap();
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
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Repair the reference data")
            );
            assert!(workspace.read(42).is_ok());
        }
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
