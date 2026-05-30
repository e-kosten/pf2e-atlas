use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::db::{create_contract_database, ok_data, temp_db_path};
use support::graph::{assert_section_edges_point_to_returned_records, insert_graph_edges};

#[test]
fn graph_uses_json_returns_backlinks_as_uses() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-uses");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "uses",
            "actions:testAction1",
            "--limit",
            "1",
            "--index",
        ])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["detail"], "summary");
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert!(data.get("outgoing").is_none());
    assert!(data.get("backlinks").is_none());
    assert_eq!(data["uses"]["total_records"], 2);
    assert_eq!(data["uses"]["total_edges"], 2);
    assert_eq!(data["uses"]["truncated"], true);
    assert_eq!(data["uses"]["records"].as_array().unwrap().len(), 1);
    assert_eq!(data["uses"]["edges"].as_array().unwrap().len(), 1);
    assert_section_edges_point_to_returned_records(&data["uses"], "from");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_uses_json_resolves_record_names() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-uses-name");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "uses", "Test Action 1", "--limit", "1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["uses"]["records"].as_array().unwrap().len(), 1);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_uses_human_output_is_backlink_oriented() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-uses-human");
    create_contract_database(&path)?;
    insert_graph_edges(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "graph",
            "uses",
            "actions:testAction1",
            "--limit",
            "1",
            "--index",
        ])
        .arg(&path)
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("actions:testAction1"));
    assert!(stdout.contains("Uses: 1 records, 1 edges"));
    assert!(stdout.contains("actions:testAction2 -> Incoming A"));
    assert!(!stdout.contains("Outgoing:"));
    assert!(!stdout.contains("Backlinks:"));

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_uses_json_reports_missing_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-uses-missing");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "uses", "actions:missing", "--index"])
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
