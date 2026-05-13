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
    assert_eq!(inspect_json["records"]["default_visible_records"], 1);
    assert_eq!(inspect_json["records"]["by_record_family"]["rule"], 1);
    assert_eq!(
        inspect_json["records"]["by_publication_family"]["unknown"],
        1
    );
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
    create_minimal_contract_schema(&connection)?;
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

fn create_minimal_contract_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute_batch(
        "
        CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        CREATE TABLE packs (
          name TEXT PRIMARY KEY, label TEXT, document_type TEXT, declared_path TEXT,
          resolved_path TEXT, record_count INTEGER
        );
        CREATE TABLE records (
          record_key TEXT PRIMARY KEY, id TEXT, name TEXT, normalized_name TEXT,
          record_family TEXT, pack_name TEXT, pack_label TEXT, foundry_document_type TEXT,
          foundry_record_type TEXT, level INTEGER, rarity TEXT, traits_json TEXT,
          system_category TEXT, system_group TEXT, system_base_item TEXT, system_usage TEXT,
          system_price_json TEXT, system_actions_value INTEGER, system_time_value TEXT,
          system_duration_value TEXT, price_cp INTEGER, activation_time_kind TEXT,
          activation_time_actions INTEGER, activation_time_duration_value INTEGER,
          activation_time_duration_unit TEXT, activation_time_text TEXT, duration_kind TEXT,
          duration_value INTEGER, duration_unit TEXT, duration_text TEXT,
          publication_title TEXT, publication_remaster INTEGER, description_text TEXT,
          blurb_text TEXT, description_snippet TEXT, publication_family TEXT, folder_id TEXT,
          taxonomy_families_json TEXT, variant_group_key TEXT, variant_base_name TEXT,
          variant_label TEXT, variant_axes_json TEXT, variant_confidence REAL,
          variant_source TEXT, source_path TEXT, is_default_visible INTEGER,
          search_text_projection TEXT, raw_json TEXT
        );
        CREATE TABLE record_traits (record_key TEXT, trait TEXT);
        CREATE TABLE reference_edges (
          from_record_key TEXT, to_record_key TEXT, display_text TEXT, reference_text TEXT
        );
        CREATE TABLE record_aliases (
          canonical_record_key TEXT, alias_text TEXT, normalized_alias TEXT,
          source_kind TEXT, source_ref TEXT
        );
        CREATE TABLE remaster_links (
          remaster_record_key TEXT, legacy_record_key TEXT, source_kind TEXT, source_ref TEXT
        );
        CREATE TABLE record_metrics (
          record_key TEXT, metric_domain TEXT, metric_key TEXT, value_type TEXT,
          number_value REAL, text_value TEXT, bool_value INTEGER
        );
        CREATE TABLE metric_key_catalog (
          metric_domain TEXT, record_family TEXT, namespace_prefix TEXT, metric_key TEXT,
          value_type TEXT, catalog_count INTEGER, numeric_min REAL, numeric_max REAL
        );
        CREATE TABLE metric_value_catalog (
          metric_domain TEXT, record_family TEXT, metric_key TEXT, value TEXT,
          catalog_count INTEGER
        );
        CREATE TABLE actor_records (
          record_key TEXT PRIMARY KEY, size TEXT, languages_json TEXT, speed_types_json TEXT,
          senses_json TEXT, immunities_json TEXT, resistances_json TEXT, weaknesses_json TEXT,
          disable_text TEXT, disable_skills_json TEXT, is_complex INTEGER
        );
        CREATE TABLE item_records (
          record_key TEXT PRIMARY KEY, system_category TEXT, system_base_item TEXT,
          system_group TEXT, system_usage TEXT, price_cp INTEGER, bulk_value REAL,
          hands_requirement TEXT, damage_types_json TEXT
        );
        CREATE TABLE spell_records (
          record_key TEXT PRIMARY KEY, traditions_json TEXT, spell_kinds_json TEXT,
          range_text TEXT, range_value REAL, target_text TEXT, area_type TEXT,
          area_value REAL, save_type TEXT, sustained INTEGER, basic_save INTEGER,
          damage_types_json TEXT
        );
        CREATE VIRTUAL TABLE records_fts USING fts5(
          record_key UNINDEXED, name, search_text_projection
        );
        ",
    )?;
    Ok(())
}

fn insert_minimal_contract_rows(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
         VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 3)",
        [],
    )?;
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let record_id = format!("testAction{index}");
        let name = format!("Test Action {index}");
        let normalized_name = name.to_lowercase();
        let source_path = format!("packs/actions/test-action-{index}.json");
        connection.execute(
            "INSERT INTO records (
              record_key, id, name, normalized_name, record_family, pack_name, pack_label,
              foundry_document_type, foundry_record_type, traits_json, publication_remaster,
              publication_family, taxonomy_families_json, variant_axes_json, variant_source,
              source_path, is_default_visible, search_text_projection, raw_json
            ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
              '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, ?3, '{}')",
            [
                record_key.as_str(),
                record_id.as_str(),
                name.as_str(),
                normalized_name.as_str(),
                source_path.as_str(),
            ],
        )?;
        connection.execute(
            "INSERT INTO records_fts (record_key, name, search_text_projection)
             VALUES (?1, ?2, ?2)",
            [record_key.as_str(), name.as_str()],
        )?;
    }
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
