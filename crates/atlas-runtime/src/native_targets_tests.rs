//! Migration29 preserves populated saved domains and exact native bytes.
use super::*;
use atlas_index::test_support::{reference_preservation_rows, stage_pending_native_fixture};
use rusqlite::types::Value;
use std::collections::BTreeMap;

fn rows(workspace: &Workspace, schema: u32) -> BTreeMap<String, Vec<Vec<Value>>> {
    reference_preservation_rows(workspace.read(schema).unwrap().connection()).unwrap()
}
fn populated_twenty_eight() -> (tempfile::TempDir, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    register_extensions().unwrap();
    let workspace = Workspace::at(dir.path().join("migration29.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..22], |_| Ok(())).unwrap();
    super::saved_state_integrity_tests::populate(&workspace, 22);
    workspace.migrate(&MIGRATIONS[..26], |_| Ok(())).unwrap();
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "native-probe",
            "Item",
            serde_json::json!({"_id":"nativeprobe00001","type":"action","name":"Native preservation probe","system":{"description":{"value":"<h2>Kept section</h2><p>Exact retained publication.</p>"}}}),
        ),
    ]);
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(26, "authentic pre-section fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    workspace.migrate(&MIGRATIONS[..28], |_| Ok(())).unwrap();
    // Complete the real supported offline metadata refresh before capturing
    // the current schema28 publication; do not spoof its stored recipe.
    assert!(atlas_index::ensure_reference_projections(&workspace, 28).unwrap());
    {
        let write = workspace.write(28).unwrap();
        let job = atlas_index::start_reference_job(&write, "legacy unknown target").unwrap();
        for model in &input.context.search_profiles {
            let unit = input
                .semantic_units
                .iter()
                .find(|unit| unit.profile_id == model.profile_id)
                .unwrap();
            let mut vector = vec![0.; model.dimensions];
            vector[71] = 1.;
            let descriptor = atlas_embedding::NativeInputDescriptor {
                input_sha256: unit.input_hash.clone(),
                token_count: unit.input_token_count,
            };
            stage_pending_native_fixture(&write, &job, model, &descriptor, &vector).unwrap();
        }
        write.commit().unwrap();
    }
    (dir, workspace)
}
#[test]
fn native_target_migration29_preserves_every_saved_row_both_profiles_and_unknown_checkpoints() {
    let (_dir, workspace) = populated_twenty_eight();
    let before = rows(&workspace, 28);
    let read = workspace.read(28).unwrap();
    let key = read.workspace_key().unwrap();
    let revision = read.reference_revision().unwrap();
    let ledger = read
        .connection()
        .prepare("SELECT version,name,checksum FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    drop(read);
    initialize_workspace(workspace.path()).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.workspace_key().unwrap(), key);
    assert_eq!(read.reference_revision().unwrap(), revision);
    let mut after = reference_preservation_rows(read.connection()).unwrap();
    assert!(after.remove("native_update_targets").unwrap().is_empty());
    assert!(after.remove("native_target_inputs").unwrap().is_empty());
    assert_eq!(after, before);
    let actual=read.connection().prepare("SELECT version,name,checksum FROM schema_migrations WHERE version<=28 ORDER BY version").unwrap().query_map([],|r|Ok((r.get::<_,u32>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(actual, ledger);
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
                r.get::<_, usize>(0)
            })
            .unwrap(),
        2
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM search_profiles", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        2
    );
    assert!(
        !read
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    atlas_index::validate_reference(read.connection()).unwrap();
}
#[test]
fn failed_batch_after_native_target_migration_rolls_back_ledger_tables_and_all_retained_values() {
    let (_dir, workspace) = populated_twenty_eight();
    let before = rows(&workspace, 28);
    fn fail(_: &rusqlite::Connection) -> atlas_storage::StorageResult<()> {
        Err(atlas_storage::StorageError::Migration(
            "injected failure after native targets".into(),
        ))
    }
    let mut migrations = MIGRATIONS.to_vec();
    migrations.push(Migration {
        version: 30,
        name: "native-target-rollback-probe",
        sql: "CREATE TABLE after_native_probe(id INTEGER PRIMARY KEY);",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "native-target-failure-test/v1",
            apply: fail,
        }),
    });
    assert!(
        workspace
            .migrate(&migrations, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("injected failure")
    );
    assert_eq!(rows(&workspace, 28), before);
    let read = workspace.read(28).unwrap();
    assert!(!read.connection().query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name IN ('native_update_targets','native_target_inputs','after_native_probe'))",[],|r|r.get::<_,bool>(0)).unwrap());
    drop(read);
    assert!(initialize_workspace(workspace.path()).is_ok());
}
