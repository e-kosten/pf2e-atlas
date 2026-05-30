use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::graph::{create_contract_database, insert_remaster_link, ok_data, temp_db_path};

#[test]
fn graph_remaster_json_returns_legacy_and_remaster_links() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-graph-remaster");
    create_contract_database(&path)?;
    insert_remaster_link(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    let links = data["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0]["direction"], "legacy_to_remaster");
    assert_eq!(links[0]["legacy"]["key"], "actions:testAction1");
    assert_eq!(links[0]["remaster"]["key"], "actions:testAction2");
    assert_eq!(links[0]["source"]["kind"], "migration");
    assert_eq!(links[0]["source"]["reference"], "test migration");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_remaster_json_resolves_record_names() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-remaster-name");
    create_contract_database(&path)?;
    insert_remaster_link(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "Test Action 1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["links"][0]["direction"], "legacy_to_remaster");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_remaster_json_reports_missing_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-remaster-missing");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "actions:missing", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_not_found");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_remaster_json_supports_remaster_seed_direction() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-graph-remaster-reverse");
    create_contract_database(&path)?;
    insert_remaster_link(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "actions:testAction2", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction2");
    let links = data["links"].as_array().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0]["direction"], "remaster_to_legacy");
    assert_eq!(links[0]["legacy"]["key"], "actions:testAction1");
    assert_eq!(links[0]["remaster"]["key"], "actions:testAction2");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_remaster_json_returns_empty_links_for_existing_record()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-remaster-empty");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "remaster", "actions:testAction3", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction3");
    assert_eq!(data["links"].as_array().unwrap().len(), 0);

    fs::remove_file(path)?;
    Ok(())
}
