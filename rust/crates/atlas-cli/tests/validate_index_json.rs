use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::Connection;
use serde_json::{Value, json};

#[test]
fn build_index_json_writes_valid_minimal_artifact() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-build");
    write_fixture_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["build-index", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--json")
        .output()?;

    assert!(build_output.status.success());
    let build_json: Value = serde_json::from_slice(&build_output.stdout)?;
    assert_eq!(
        build_json,
        json!({
            "status": "ok",
            "output": index_path.display().to_string(),
            "pack_count": 1,
            "record_count": 1,
            "diagnostics": {
                "taxonomy": {
                    "folder_records": 0,
                    "glossary_records": 0
                },
                "variants": {
                    "parenthetical_records": 0,
                    "suffix_records": 0,
                    "creature_blurb_records": 0,
                    "creature_suffix_records": 0,
                    "exact_base_records": 0
                }
            },
            "skipped_record_count": 0,
            "skipped_records": [],
            "warnings": []
        })
    );

    let validate_output = run_atlas(&index_path)?;

    assert!(validate_output.status.success());
    let validate_json: Value = serde_json::from_slice(&validate_output.stdout)?;
    assert_eq!(validate_json["status"], "ok");
    assert_eq!(validate_json["source_record_count"], "1");

    let inspect_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["inspect-index", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;

    assert!(inspect_output.status.success());
    let inspect_json: Value = serde_json::from_slice(&inspect_output.stdout)?;
    assert_eq!(inspect_json["status"], "ok");
    assert_eq!(inspect_json["records"]["total_records"], 1);
    assert_eq!(inspect_json["records"]["by_record_family"]["rule"], 1);
    assert_eq!(inspect_json["tables"]["records"], 1);
    assert_eq!(inspect_json["tables"]["packs"], 1);
    assert_eq!(inspect_json["text"]["records_with_description"], 1);
    assert_eq!(inspect_json["relationships"]["reference_edges"], 0);
    assert_eq!(inspect_json["metrics"]["metric_value_catalog_rows"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_valid_minimal_contract() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-valid");
    create_contract_database(&path, None)?;

    let output = run_atlas(&path)?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "code": "OK",
            "index": path.display().to_string(),
            "message": "artifact metadata is valid",
            "artifact_contract_version": "pf2e-atlas-artifact/v1",
            "schema_version": "1",
            "source_kind": "foundry-pf2e",
            "source_signature": "foundry-pf2e:fixture",
            "source_record_count": "3",
            "content_hash_algorithm": "sha256",
            "embedding_provider_family": "transformers-js-minilm",
            "embedding_model_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_model_revision": "main",
            "embedding_tokenizer_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_pooling": "mean",
            "embedding_normalization": "l2",
            "embedding_dimensions": "384",
            "embedding_dtype": "f32",
            "embedding_distance_metric": "cosine",
            "embedding_document_prefix": "",
            "embedding_query_prefix": "",
            "fts_tokenizer": "unicode61 remove_diacritics 2",
            "adjacent_manifest_path": "manifest.json"
        })
    );
    fs::remove_file(path)?;
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

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "MISSING_ARTIFACT_METADATA",
            "index": path.display().to_string(),
            "message": "index opened, but the Rust artifact contract metadata table is missing",
            "legacy_schema_version": "25"
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-embedding-mismatch");
    create_contract_database(&path, Some(("embedding_dimensions", "768")))?;

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "EMBEDDING_MISMATCH",
            "index": path.display().to_string(),
            "message": "artifact metadata is incompatible with this runtime",
            "artifact_contract_version": "pf2e-atlas-artifact/v1",
            "schema_version": "1",
            "source_kind": "foundry-pf2e",
            "source_signature": "foundry-pf2e:fixture",
            "source_record_count": "3",
            "content_hash_algorithm": "sha256",
            "embedding_provider_family": "transformers-js-minilm",
            "embedding_model_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_model_revision": "main",
            "embedding_tokenizer_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_pooling": "mean",
            "embedding_normalization": "l2",
            "embedding_dimensions": "768",
            "embedding_dtype": "f32",
            "embedding_distance_metric": "cosine",
            "embedding_document_prefix": "",
            "embedding_query_prefix": "",
            "fts_tokenizer": "unicode61 remove_diacritics 2",
            "adjacent_manifest_path": "manifest.json",
            "diagnostics": [
                {
                    "code": "EMBEDDING_MISMATCH",
                    "family": "embedding",
                    "message": "metadata key `embedding_dimensions` has an unsupported value",
                    "key": "embedding_dimensions",
                    "expected": "384",
                    "actual": "768"
                }
            ]
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_missing_required_key() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-missing-key");
    create_contract_database_omitting(&path, "embedding_dtype")?;

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "MISSING_REQUIRED_METADATA",
            "index": path.display().to_string(),
            "message": "artifact metadata table is missing required keys",
            "artifact_contract_version": "pf2e-atlas-artifact/v1",
            "schema_version": "1",
            "source_kind": "foundry-pf2e",
            "source_signature": "foundry-pf2e:fixture",
            "source_record_count": "3",
            "content_hash_algorithm": "sha256",
            "embedding_provider_family": "transformers-js-minilm",
            "embedding_model_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_model_revision": "main",
            "embedding_tokenizer_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_pooling": "mean",
            "embedding_normalization": "l2",
            "embedding_dimensions": "384",
            "embedding_distance_metric": "cosine",
            "embedding_document_prefix": "",
            "embedding_query_prefix": "",
            "fts_tokenizer": "unicode61 remove_diacritics 2",
            "adjacent_manifest_path": "manifest.json",
            "missing_keys": ["embedding_dtype"]
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_stale_source_signature() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-stale-source");
    create_contract_database(&path, Some(("source_signature", "stale:fixture")))?;

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "STALE_SOURCE_SIGNATURE",
            "index": path.display().to_string(),
            "message": "artifact metadata is incompatible with this runtime",
            "artifact_contract_version": "pf2e-atlas-artifact/v1",
            "schema_version": "1",
            "source_kind": "foundry-pf2e",
            "source_signature": "stale:fixture",
            "source_record_count": "3",
            "content_hash_algorithm": "sha256",
            "embedding_provider_family": "transformers-js-minilm",
            "embedding_model_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_model_revision": "main",
            "embedding_tokenizer_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_pooling": "mean",
            "embedding_normalization": "l2",
            "embedding_dimensions": "384",
            "embedding_dtype": "f32",
            "embedding_distance_metric": "cosine",
            "embedding_document_prefix": "",
            "embedding_query_prefix": "",
            "fts_tokenizer": "unicode61 remove_diacritics 2",
            "adjacent_manifest_path": "manifest.json",
            "diagnostics": [
                {
                    "code": "STALE_SOURCE_SIGNATURE",
                    "family": "source",
                    "message": "source signature marks this artifact as stale",
                    "key": "source_signature",
                    "expected": "current source signature",
                    "actual": "stale:fixture"
                }
            ]
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("cli-unsupported-schema");
    create_contract_database(&path, Some(("schema_version", "2")))?;

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "UNSUPPORTED_SCHEMA_VERSION",
            "index": path.display().to_string(),
            "message": "artifact metadata is incompatible with this runtime",
            "artifact_contract_version": "pf2e-atlas-artifact/v1",
            "schema_version": "2",
            "source_kind": "foundry-pf2e",
            "source_signature": "foundry-pf2e:fixture",
            "source_record_count": "3",
            "content_hash_algorithm": "sha256",
            "embedding_provider_family": "transformers-js-minilm",
            "embedding_model_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_model_revision": "main",
            "embedding_tokenizer_id": "Xenova/all-MiniLM-L12-v2",
            "embedding_pooling": "mean",
            "embedding_normalization": "l2",
            "embedding_dimensions": "384",
            "embedding_dtype": "f32",
            "embedding_distance_metric": "cosine",
            "embedding_document_prefix": "",
            "embedding_query_prefix": "",
            "fts_tokenizer": "unicode61 remove_diacritics 2",
            "adjacent_manifest_path": "manifest.json",
            "diagnostics": [
                {
                    "code": "UNSUPPORTED_SCHEMA_VERSION",
                    "family": "schema",
                    "message": "metadata key `schema_version` has an unsupported value",
                    "key": "schema_version",
                    "expected": "1",
                    "actual": "2"
                }
            ]
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

fn run_atlas(path: &PathBuf) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["validate-index", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

fn create_contract_database(
    path: &PathBuf,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    for (key, mut value) in valid_metadata_entries() {
        if let Some((override_key, override_value)) = override_entry
            && key == override_key
        {
            value = override_value;
        }
        connection.execute(
            "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
    }
    Ok(())
}

fn create_contract_database_omitting(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    for (key, value) in valid_metadata_entries() {
        if key != omitted_key {
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
    }
    Ok(())
}

fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("artifact_contract_version", "pf2e-atlas-artifact/v1"),
        ("schema_version", "1"),
        ("source_kind", "foundry-pf2e"),
        ("source_signature", "foundry-pf2e:fixture"),
        ("source_record_count", "3"),
        ("content_hash_algorithm", "sha256"),
        ("embedding_provider_family", "transformers-js-minilm"),
        ("embedding_model_id", "Xenova/all-MiniLM-L12-v2"),
        ("embedding_model_revision", "main"),
        ("embedding_tokenizer_id", "Xenova/all-MiniLM-L12-v2"),
        ("embedding_pooling", "mean"),
        ("embedding_normalization", "l2"),
        ("embedding_dimensions", "384"),
        ("embedding_dtype", "f32"),
        ("embedding_distance_metric", "cosine"),
        ("embedding_document_prefix", ""),
        ("embedding_query_prefix", ""),
        ("fts_tokenizer", "unicode61 remove_diacritics 2"),
        ("adjacent_manifest_path", "manifest.json"),
    ]
}

fn temp_db_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}.sqlite",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_file(&path);
    path
}

fn temp_source_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "traits": { "value": ["healing", "exploration"] },
            "description": { "value": "<p>You spend 10 minutes treating one injured living creature.</p>" }
          }
        }"#,
    )?;
    Ok(())
}
