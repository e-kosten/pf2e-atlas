#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use atlas_artifact::test_support::{
    create_minimal_contract_schema, insert_contract_metadata_entries, insert_minimal_contract_rows,
    legacy_minilm_metadata_entries,
};
use rusqlite::Connection;
use serde_json::Value;

pub fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

pub fn create_contract_database(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_entries(&connection, legacy_minilm_metadata_entries(), None)?;
    insert_minimal_contract_rows(&connection)?;
    Ok(())
}

pub fn temp_db_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}.sqlite",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_file(&path);
    path
}
