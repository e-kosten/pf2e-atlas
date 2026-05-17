use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_artifact::test_support::{
    create_minimal_contract_schema, insert_contract_metadata_entries,
    insert_contract_metadata_omitting, insert_minimal_contract_rows,
    legacy_minilm_metadata_entries,
};
use rusqlite::Connection;
use serde_json::{Value, json};

#[test]
fn setup_json_reports_overridden_paths_and_default_model() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_source_root("cli-setup");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    fs::create_dir_all(&source)?;
    fs::create_dir_all(cache.join("BAAI").join("bge-small-en-v1.5").join("onnx"))?;
    fs::write(
        cache
            .join("BAAI")
            .join("bge-small-en-v1.5")
            .join("tokenizer.json"),
        "{}",
    )?;
    fs::write(
        cache
            .join("BAAI")
            .join("bge-small-en-v1.5")
            .join("onnx")
            .join("model.onnx"),
        "",
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--path-mode", "user", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["status"], "ready");
    assert_eq!(actual["path_mode"], "user");
    assert_eq!(actual["source"]["path"], source.display().to_string());
    assert_eq!(actual["source"]["exists"], true);
    assert_eq!(actual["embedding"]["model"], "bge-small-en-v1.5");
    assert_eq!(
        actual["embedding"]["cache_root"],
        cache.display().to_string()
    );
    assert_eq!(actual["embedding"]["ready"], true);
    assert_eq!(actual["embedding"]["missing_files"], json!([]));
    assert_eq!(actual["index"]["path"], index.display().to_string());
    assert_eq!(actual["index"]["exists"], false);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn build_index_json_writes_valid_minimal_artifact() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-build");
    write_fixture_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;

    assert!(build_output.status.success());
    let mut build_json: Value = serde_json::from_slice(&build_output.stdout)?;
    let source_signature = build_json["source_signature"]
        .as_str()
        .expect("index build should report source signature")
        .to_string();
    assert!(source_signature.starts_with("foundry-pf2e:sha256:"));
    assert_eq!(source_signature.len(), "foundry-pf2e:sha256:".len() + 64);
    build_json["source_signature"] = json!("<source-signature>");
    assert!(
        build_json["build_duration_ms"]
            .as_u64()
            .is_some_and(|duration| duration > 0)
    );
    build_json["build_duration_ms"] = json!("<build-duration-ms>");
    assert_eq!(
        build_json,
        json!({
            "status": "ok",
            "output": index_path.display().to_string(),
            "pack_count": 1,
            "record_count": 1,
            "source_record_count": 1,
            "artifact_record_count": 1,
            "generated_record_count": 0,
            "pending_document_embedding_count": 1,
            "document_embedding_count": 0,
            "reused_document_embedding_count": 0,
            "generated_document_embedding_count": 0,
            "document_embedding_tokenization": {
                "document_count": 0,
                "truncated_document_count": 0,
                "max_token_count": null,
                "max_observed_token_count": 0,
                "total_observed_token_count": 0,
                "total_tokens_over_limit": 0,
                "unit_kind_truncations": [],
                "record_truncation_coverage": {
                    "record_count": 0,
                    "records_with_child_units": 0,
                    "records_with_any_truncated_unit": 0,
                    "records_with_truncated_parent_unit": 0,
                    "records_with_truncated_child_unit": 0,
                    "records_with_truncated_parent_and_child_units": 0,
                    "records_with_truncated_parent_and_all_child_units_fit": 0,
                    "records_with_truncated_parent_without_child_units": 0
                },
                "section_truncations": [],
                "truncated_examples": []
            },
            "embedding_timing": {
                "tokenization_duration_ms": 0,
                "model_load_duration_ms": 0,
                "generation_duration_ms": 0,
                "batch_count": 0,
                "batch_duration_min_ms": null,
                "batch_duration_p50_ms": null,
                "batch_duration_p95_ms": null,
                "batch_duration_max_ms": null
            },
            "build_duration_ms": "<build-duration-ms>",
            "source_signature": "<source-signature>",
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
                },
                "generated_afflictions": {
                    "canonical_records": 0,
                    "instance_records": 0,
                    "reference_edges": 0
                },
                "dropped_inline_macros": []
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
    assert_eq!(validate_json["source_signature"], source_signature);
    assert_eq!(validate_json["source_record_count"], "1");
    assert_eq!(validate_json["artifact_record_count"], "1");
    assert_eq!(validate_json["generated_record_count"], "0");

    let validate_vectors_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate-vectors", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;

    assert_eq!(validate_vectors_output.status.code(), Some(1));
    let validate_vectors_json: Value = serde_json::from_slice(&validate_vectors_output.stdout)?;
    assert_eq!(validate_vectors_json["code"], "ARTIFACT_CONTRACT_VIOLATION");
    assert_eq!(
        validate_vectors_json["diagnostics"][0]["key"],
        "table:record_vector_index"
    );

    let inspect_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "inspect", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;

    assert!(inspect_output.status.success());
    let inspect_json: Value = serde_json::from_slice(&inspect_output.stdout)?;
    assert_eq!(inspect_json["status"], "ok");
    assert_eq!(inspect_json["records"]["total_records"], 1);
    assert_eq!(inspect_json["records"]["default_visible_records"], 1);
    assert_eq!(inspect_json["records"]["by_record_family"]["rule"], 1);
    assert_eq!(
        inspect_json["records"]["by_publication_family"]["unknown"],
        1
    );
    assert_eq!(inspect_json["tables"]["records"], 1);
    assert_eq!(inspect_json["tables"]["packs"], 1);
    assert_eq!(inspect_json["tables"]["document_embedding_cache"], 0);
    assert_eq!(inspect_json["text"]["records_with_description"], 1);
    assert_eq!(inspect_json["relationships"]["reference_edges"], 0);
    assert_eq!(inspect_json["metrics"]["metric_value_catalog_rows"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn analyze_index_json_reports_source_without_writing_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-analyze");
    write_fixture_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let analyze_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "analyze", "--source"])
        .arg(&root)
        .arg("--json")
        .output()?;

    assert!(analyze_output.status.success());
    assert!(!index_path.exists());
    let analyze_json: Value = serde_json::from_slice(&analyze_output.stdout)?;
    assert_eq!(analyze_json["status"], "ok");
    assert_eq!(analyze_json["source"]["root"], root.display().to_string());
    assert_eq!(
        analyze_json["source"]["manifest"],
        root.join("module.json").display().to_string()
    );
    let source_signature = analyze_json["source"]["source_signature"]
        .as_str()
        .expect("index analyze should report source signature");
    assert!(source_signature.starts_with("foundry-pf2e:sha256:"));
    assert_eq!(source_signature.len(), "foundry-pf2e:sha256:".len() + 64);
    assert_eq!(analyze_json["pack_count"], 1);
    assert_eq!(analyze_json["loaded_source_pack_count"], 1);
    assert_eq!(analyze_json["record_count"], 1);
    assert_eq!(analyze_json["loaded_source_record_count"], 1);
    assert_eq!(analyze_json["generated_record_count"], 0);
    assert_eq!(analyze_json["default_visible_record_count"], 1);
    assert_eq!(analyze_json["hidden_record_count"], 0);
    assert_eq!(analyze_json["by_record_family"]["rule"], 1);
    assert_eq!(analyze_json["by_foundry_taxonomy"]["Item|action"], 1);
    assert_eq!(analyze_json["by_publication_family"]["unknown"], 1);
    assert_eq!(analyze_json["side_data"]["item_records"], 1);
    assert_eq!(analyze_json["text"]["records_with_description"], 1);
    assert_eq!(analyze_json["embeddings"]["pending_document_embeddings"], 1);
    assert_eq!(analyze_json["relationships"]["reference_edges"], 0);
    assert_eq!(analyze_json["skipped_record_count"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn legacy_top_level_index_commands_are_not_supported() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("validate-index")
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unrecognized subcommand 'validate-index'"));
    Ok(())
}

#[test]
fn semantic_search_rejects_invalid_filter_json_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "semantic",
            "--index",
            "missing.sqlite",
            "--query",
            "healing",
            "--filter-json",
            "{\"kind\":\"unknown\"}",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("failed to parse --filter-json"));
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
            "artifact_record_count": "3",
            "generated_record_count": "0",
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
fn validate_index_json_reports_unavailable_index() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-unavailable");

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "INDEX_UNAVAILABLE",
            "index": path.display().to_string(),
            "message": format!("index is unavailable: unable to open database file: {}", path.display())
        })
    );
    Ok(())
}

#[test]
fn validate_vectors_json_reports_unavailable_index() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-vector-unavailable");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate-vectors", "--index"])
        .arg(&path)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "error",
            "code": "INDEX_UNAVAILABLE",
            "index": path.display().to_string(),
            "message": format!("index is unavailable: unable to open database file: {}", path.display())
        })
    );
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
            "artifact_record_count": "3",
            "generated_record_count": "0",
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
            "artifact_record_count": "3",
            "generated_record_count": "0",
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
            "artifact_record_count": "3",
            "generated_record_count": "0",
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
            "artifact_record_count": "3",
            "generated_record_count": "0",
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
        .args(["index", "validate", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

fn create_contract_database(
    path: &PathBuf,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_entries(
        &connection,
        legacy_minilm_metadata_entries(),
        override_entry,
    )?;
    if override_entry.is_none() {
        insert_minimal_contract_rows(&connection)?;
    }
    Ok(())
}

fn create_contract_database_omitting(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_omitting(&connection, legacy_minilm_metadata_entries(), omitted_key)?;
    Ok(())
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
