#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use atlas_index::test_support::{
    create_minimal_artifact_schema, insert_artifact_metadata_entries,
    insert_artifact_metadata_omitting, insert_minimal_artifact_rows,
    legacy_minilm_metadata_entries, write_bound_test_manifest,
};
use rusqlite::Connection;
use serde_json::Value;

pub fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

pub fn create_valid_artifact_database(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    create_valid_artifact_database_with_override(path, None)
}

pub fn create_valid_artifact_database_with_override(
    path: &Path,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata_entries(
        &connection,
        legacy_minilm_metadata_entries(),
        override_entry,
    )?;
    if override_entry.is_none() {
        insert_minimal_artifact_rows(&connection)?;
    }
    drop(connection);
    write_bound_test_manifest(path)?;
    Ok(())
}

pub fn create_valid_artifact_database_omitting(
    path: &Path,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata_omitting(&connection, legacy_minilm_metadata_entries(), omitted_key)?;
    drop(connection);
    write_bound_test_manifest(path)?;
    Ok(())
}

pub fn temp_db_path(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "atlas-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("CLI fixture directory");
    root.join("pf2e-index.sqlite")
}

pub fn refresh_bound_test_manifest(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_bound_test_manifest(path)
}

pub fn remove_fixture_artifact(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_dir_all(path.parent().expect("CLI fixture parent"))?;
    Ok(())
}
