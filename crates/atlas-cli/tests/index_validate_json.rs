use std::fs;
use std::process::Command;

use rusqlite::Connection;

mod support;

use support::command::{check_base_index, validate_base_index, validate_index};
use support::db::{
    create_valid_artifact_database_omitting, create_valid_artifact_database_with_override,
    temp_db_path,
};
use support::json::parse_ok_data;

#[test]
fn validate_index_json_reports_valid_minimal_contract() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-valid");
    create_valid_artifact_database_with_override(&path, None)?;

    let output = validate_base_index(&path)?;

    assert!(output.status.success());
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], true);
    assert_eq!(actual["code"], "ok");
    assert_eq!(actual["index"], path.display().to_string());
    assert_eq!(actual["message"], "artifact metadata is valid");
    assert_eq!(
        actual["artifact_contract_version"],
        "pf2e-atlas-artifact/v1"
    );
    assert_eq!(actual["schema_version"], "1");
    assert_eq!(actual["source_signature"], "foundry-pf2e:fixture");
    assert_eq!(actual["embedding_dimensions"], "384");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn index_check_json_reports_valid_minimal_contract() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-check-valid");
    create_valid_artifact_database_with_override(&path, None)?;

    let output = check_base_index(&path)?;

    assert!(output.status.success());
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], true);
    assert_eq!(actual["code"], "ok");
    assert_eq!(actual["index"], path.display().to_string());

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_unavailable_index() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-unavailable");

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_unavailable_index(&actual, &path);
    Ok(())
}

#[test]
fn validate_embeddings_only_json_reports_unavailable_index()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-vector-unavailable");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate", "--embeddings-only", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_unavailable_index(&actual, &path);
    Ok(())
}

#[test]
fn validate_index_json_reports_missing_artifact_metadata() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-missing-table");
    let connection = Connection::open(&path)?;
    connection.execute(
        "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    connection.execute(
        "INSERT INTO metadata (key, value) VALUES ('schema_version', '25')",
        [],
    )?;
    drop(connection);

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], false);
    assert_eq!(actual["code"], "missing_artifact_metadata");
    assert_eq!(actual["index"], path.display().to_string());
    assert_eq!(actual["legacy_schema_version"], "25");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-embedding-mismatch");
    create_valid_artifact_database_with_override(&path, Some(("embedding_dimensions", "768")))?;

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_metadata_failure(&actual, &path, "embedding_mismatch");
    assert_eq!(actual["embedding_dimensions"], "768");
    assert_diagnostic(
        &actual,
        "embedding_mismatch",
        "embedding_dimensions",
        "384",
        "768",
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_missing_required_key() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-missing-key");
    create_valid_artifact_database_omitting(&path, "embedding_dtype")?;

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_eq!(actual["valid"], false);
    assert_eq!(actual["code"], "missing_required_metadata");
    assert_eq!(actual["index"], path.display().to_string());
    assert_eq!(actual["missing_keys"][0], "embedding_dtype");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_stale_source_signature() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-stale-source");
    create_valid_artifact_database_with_override(
        &path,
        Some(("source_signature", "stale:fixture")),
    )?;

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_metadata_failure(&actual, &path, "stale_source_signature");
    assert_eq!(actual["source_signature"], "stale:fixture");
    assert_diagnostic(
        &actual,
        "stale_source_signature",
        "source_signature",
        "current source signature",
        "stale:fixture",
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-unsupported-schema");
    create_valid_artifact_database_with_override(&path, Some(("schema_version", "2")))?;

    let output = validate_index(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual = parse_ok_data(&output)?;
    assert_metadata_failure(&actual, &path, "unsupported_schema_version");
    assert_eq!(actual["schema_version"], "2");
    assert_diagnostic(
        &actual,
        "unsupported_schema_version",
        "schema_version",
        "1",
        "2",
    );
    fs::remove_file(path)?;
    Ok(())
}

fn assert_unavailable_index(value: &serde_json::Value, path: &std::path::Path) {
    assert_eq!(value["valid"], false);
    assert_eq!(value["code"], "index_unavailable");
    assert_eq!(value["index"], path.display().to_string());
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("unable to open database file")
    );
}

fn assert_metadata_failure(value: &serde_json::Value, path: &std::path::Path, code: &str) {
    assert_eq!(value["valid"], false);
    assert_eq!(value["code"], code);
    assert_eq!(value["index"], path.display().to_string());
    assert_eq!(
        value["message"],
        "artifact metadata is incompatible with this runtime"
    );
    assert_eq!(value["artifact_contract_version"], "pf2e-atlas-artifact/v1");
}

fn assert_diagnostic(
    value: &serde_json::Value,
    code: &str,
    key: &str,
    expected: &str,
    actual: &str,
) {
    let diagnostic = &value["diagnostics"][0];
    assert_eq!(diagnostic["code"], code);
    assert_eq!(diagnostic["key"], key);
    assert_eq!(diagnostic["expected"], expected);
    assert_eq!(diagnostic["actual"], actual);
}
