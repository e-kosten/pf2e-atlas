use std::fs;
use std::process::Command;

use rusqlite::Connection;
use serde_json::Value;

mod support;

use support::db::{create_contract_database, ok_data, temp_db_path};
use support::graph::insert_reference_edge;
use support::vector::insert_vector_embeddings;

#[test]
fn similar_json_returns_semantic_and_reference_evidence() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-similar");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "actions:testAction1", "--index"])
        .arg(&path)
        .args(["--limit", "2", "--json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["detail"], "summary");
    assert_eq!(data["seed"]["key"], "actions:testAction1");
    assert_json_key_absent(data, "seed_embedding_unit_key");
    assert_json_key_absent(data, "embedding_unit_key");
    let results = data["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 2);
    assert_ne!(results[0]["record"]["key"], "actions:testAction1");
    assert!(results[0]["similarity"]["score"].is_number());
    assert_eq!(results[0]["similarity"]["semantic"]["unit_kind"], "parent");
    assert!(results.iter().any(|result| {
        !result["similarity"]["graph"]["shared_references"]
            .as_array()
            .expect("shared references should be an array")
            .is_empty()
    }));

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_json_reports_vector_readiness_errors() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-similar-no-vectors");
    create_contract_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "actions:testAction1", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "index_unavailable");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_accepts_strict_seed_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-similar-name");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "Test Action 1", "--index"])
        .arg(&path)
        .args(["--limit", "1", "--json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["key"], "actions:testAction1");

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_json_reports_missing_seed_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-similar-missing");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "Missing Action", "--index"])
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

#[test]
fn similar_json_reports_ambiguous_seed_name() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-similar-ambiguous");
    create_similar_database(&path)?;
    let connection = Connection::open(&path)?;
    for record_key in ["actions:testAction1", "actions:testAction2"] {
        connection.execute(
            "UPDATE records
             SET name = 'Shared Action', normalized_name = 'shared action'
             WHERE record_key = ?1",
            [record_key],
        )?;
    }
    drop(connection);

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "Shared Action", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "record_resolution_ambiguous");
    let alternatives = json["error"]["data"]["result"]["alternatives"]
        .as_array()
        .expect("ambiguous similar seed should include alternatives");
    assert_eq!(alternatives.len(), 2);
    assert!(
        alternatives
            .iter()
            .any(|alternative| alternative["record"]["key"] == "actions:testAction1")
    );
    assert!(
        alternatives
            .iter()
            .all(|alternative| alternative["resolution"]["query"] == "Shared Action")
    );

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_rejects_reference_dominant_weights_before_opening_index()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "similar",
            "actions:testAction1",
            "--semantic-weight",
            "0",
            "--reference-weight",
            "1",
            "--trait-weight",
            "0",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_option");

    Ok(())
}

#[test]
fn similar_rejects_non_finite_weights_before_opening_index()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "similar",
            "actions:testAction1",
            "--semantic-weight",
            "1",
            "--reference-weight",
            "NaN",
            "--trait-weight",
            "0",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_option");

    Ok(())
}

#[test]
fn similar_resolves_seed_name_before_applying_candidate_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-similar-filtered-name");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "Test Action 1", "--index"])
        .arg(&path)
        .args(["--family", "spell", "--json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["seed"]["key"], "actions:testAction1");
    assert_eq!(data["filter"]["kind"], "record_family");
    assert_eq!(
        data["results"]
            .as_array()
            .expect("results should be an array")
            .len(),
        0
    );

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_explain_human_output_includes_evidence_counts() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-similar-explain");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "actions:testAction1", "--index"])
        .arg(&path)
        .args(["--limit", "1", "--explain"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Seed: actions:testAction1"));
    assert!(stdout.contains("shared_references=1"));
    assert!(stdout.contains("shared_traits=0"));

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn similar_explain_human_output_survives_detailed_records() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-similar-explain-preview");
    create_similar_database(&path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["similar", "actions:testAction1", "--index"])
        .arg(&path)
        .args(["--limit", "1", "--detail", "preview", "--explain"])
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Seed: actions:testAction1"));
    assert!(stdout.contains("score="));
    assert!(stdout.contains("distance="));
    assert!(stdout.contains("shared_references=1"));
    assert!(stdout.contains("shared_traits=0"));

    fs::remove_file(path)?;
    Ok(())
}

fn assert_json_key_absent(value: &Value, key: &str) {
    match value {
        Value::Object(map) => {
            assert!(
                !map.contains_key(key),
                "JSON should not contain key `{key}`"
            );
            for child in map.values() {
                assert_json_key_absent(child, key);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_json_key_absent(child, key);
            }
        }
        _ => {}
    }
}

fn create_similar_database(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    create_contract_database(path)?;
    insert_vector_embeddings(path)?;
    let connection = Connection::open(path)?;
    connection.execute(
        "UPDATE records
         SET traits_json = '[\"auditory\"]'
         WHERE record_key IN ('actions:testAction1', 'actions:testAction3')",
        [],
    )?;
    insert_reference_edge(
        &connection,
        "actions:testAction1",
        "actions:testAction3",
        Some("Test Action 3"),
        "@UUID[Compendium.pf2e.actions.Item.testAction3]{Test Action 3}",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &connection,
        "actions:testAction2",
        "actions:testAction3",
        Some("Test Action 3"),
        "@UUID[Compendium.pf2e.actions.Item.testAction3]{Test Action 3}",
        "description",
        "public",
    )?;
    Ok(())
}
