use std::fs;
use std::process::Command;

use serde_json::Value;

mod support;

use support::db::{create_valid_artifact_database, ok_data, temp_db_path};
use support::graph::{insert_second_variant_group, insert_variant_group};

#[test]
fn graph_variants_json_returns_group_siblings() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variants");
    create_valid_artifact_database(&path)?;
    insert_variant_group(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Test Action 1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert_eq!(data["variant_group_key"], "test-action");
    let variants = data["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0]["record"]["key"], "actions:testAction1");
    assert_eq!(variants[0]["is_seed"], true);
    assert_eq!(variants[0]["variant_label"], "Lesser");
    assert_eq!(variants[0]["variant_axes"][0], "grade");
    assert_eq!(variants[1]["record"]["key"], "actions:testAction2");
    assert_eq!(variants[2]["record"]["key"], "actions:testAction3");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_variants_json_resolves_variant_base_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variant-base");
    create_valid_artifact_database(&path)?;
    insert_variant_group(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Test Action", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert!(data.get("seed").is_none());
    assert_eq!(data["variant_group_key"], "test-action");
    let variants = data["variants"].as_array().unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0]["record"]["key"], "actions:testAction1");
    assert_eq!(variants[0]["is_seed"], false);

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_variants_json_reports_ambiguous_base_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variant-ambiguous");
    create_valid_artifact_database(&path)?;
    insert_variant_group(&path)?;
    insert_second_variant_group(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Test Action", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "variant_group_resolution_ambiguous");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_variants_json_returns_empty_group_for_non_variant_record()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variant-non-variant");
    create_valid_artifact_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["record"]["key"], "actions:testAction1");
    assert!(data.get("variant_group_key").is_none());
    assert_eq!(data["variants"].as_array().unwrap().len(), 0);

    let human_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "actions:testAction1", "--index"])
        .arg(&path)
        .output()?;

    assert!(human_output.status.success());
    let stdout = String::from_utf8(human_output.stdout)?;
    assert!(stdout.contains("actions:testAction1"));
    assert!(stdout.contains("Variants: none"));

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn graph_variants_json_reports_missing_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-graph-variant-missing");
    create_valid_artifact_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "actions:missing", "--index"])
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
fn graph_variants_json_reports_unresolved_base_name_miss() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-graph-variant-miss");
    create_valid_artifact_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["graph", "variants", "Not Present", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_resolution_miss");

    fs::remove_file(path)?;
    Ok(())
}
