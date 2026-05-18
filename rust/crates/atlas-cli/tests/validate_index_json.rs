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
    write_fixture_source(&source)?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--path-mode", "user", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--no-embeddings")
        .arg("--json")
        .output()?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], true);
    assert_eq!(actual["target"], "records");
    assert_eq!(actual["path_mode"], "user");
    assert_eq!(actual["paths"]["source"], source.display().to_string());
    assert_eq!(actual["embedding"]["model"], "bge-small-en-v1.5");
    assert_eq!(
        actual["embedding"]["cache_root"],
        cache.display().to_string()
    );
    assert_eq!(actual["embedding"]["ready"], false);
    assert_eq!(actual["paths"]["index"], index.display().to_string());
    assert_eq!(actual["readiness"]["records"]["status"], "ready");
    assert_eq!(actual["readiness"]["semantic_search"]["status"], "skipped");
    assert!(index.is_file());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_records_second_run_skips_rebuild() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-idempotent");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let first = setup_records_offline(&source, &cache, &index)?;
    assert!(first.status.success());

    let second = setup_records_offline(&source, &cache, &index)?;
    assert!(second.status.success());
    let actual: Value = serde_json::from_slice(&second.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], true);
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "skipped" })
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn help_text_includes_setup_validate_and_record_examples() -> Result<(), Box<dyn std::error::Error>>
{
    let root_help = help_output(&[])?;
    assert!(root_help.contains("atlas setup"));
    assert!(root_help.contains("atlas record get actionspf2e:1kGNdIIhuglAjIp9"));

    let setup_help = help_output(&["setup"])?;
    assert!(setup_help.contains("atlas setup --no-embeddings"));
    assert!(setup_help.contains("--offline"));

    let validate_help = help_output(&["index", "validate"])?;
    assert!(validate_help.contains("atlas index validate --embeddings-only"));
    assert!(validate_help.contains("--no-embeddings"));

    let build_help = help_output(&["index", "build"])?;
    assert!(build_help.contains("atlas index build --no-embeddings"));
    assert!(build_help.contains("Standard users should run `atlas setup` instead."));

    let record_get_help = help_output(&["record", "get"])?;
    assert!(record_get_help.contains("equipment-srd:s1vB3HdXjMigYAnY"));
    assert!(record_get_help.contains("Canonical record keys"));

    let record_resolve_help = help_output(&["record", "resolve"])?;
    assert!(record_resolve_help.contains("atlas record resolve \"Treat Wounds\""));
    assert!(record_resolve_help.contains("--filter-json"));

    let search_help = help_output(&["search"])?;
    assert!(search_help.contains("atlas search \"low level healing spell\""));
    assert!(search_help.contains("--retrieval selects fts, vector, or hybrid retrieval"));

    Ok(())
}

#[test]
fn setup_check_force_rebuild_reports_not_ready() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-check-force");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--offline",
            "--check",
            "--force-rebuild",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert!(actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "build_index"
            && action["status"] == "planned"
            && action["reason"] == "force rebuild requested"
    }));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_full_check_reports_record_ready_when_vectors_missing()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-base-artifact-full-check");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--offline",
            "--check",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["records"]["status"], "ready");
    assert_eq!(
        actual["readiness"]["semantic_search"]["status"],
        "not_ready"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_failed_source_update_is_runtime_failure() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-fetch-fail");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;

    let build_output = setup_records_offline(&source, &cache, &index)?;
    assert!(build_output.status.success());
    fs::create_dir(source.join(".git"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "fetch_source" && action["status"] == "failed" })
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_failed_source_update_blocks_missing_index_build() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_source_root("cli-setup-fetch-fail-missing-index");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");
    write_fixture_source(&source)?;
    fs::create_dir(source.join(".git"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--no-embeddings",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(3));
    assert!(!index.exists());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "blocked" })
    );
    assert!(
        !actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "done" })
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn setup_check_offline_missing_source_uses_json_status_vocabulary()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-missing-source");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--offline",
            "--check",
            "--source",
        ])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert_eq!(actual["ready"], false);
    assert_eq!(actual["readiness"]["source"]["status"], "not_ready");
    assert_eq!(
        actual["readiness"]["embedding_model"]["status"],
        "not_ready"
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn setup_check_online_missing_assets_plans_dependent_build()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-setup-online-check-missing");
    let source = root.join("source");
    let cache = root.join("hf-models");
    let index = root.join("index.sqlite");

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["setup", "--path-mode", "user", "--check", "--source"])
        .arg(&source)
        .args(["--embedding-cache-path"])
        .arg(&cache)
        .args(["--index"])
        .arg(&index)
        .arg("--json")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "fetch_source" && action["status"] == "planned" })
    );
    assert!(actual["actions"].as_array().unwrap().iter().any(|action| {
        action["kind"] == "prepare_embedding_model" && action["status"] == "planned"
    }));
    assert!(
        actual["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| { action["kind"] == "build_index" && action["status"] == "planned" })
    );

    let _ = fs::remove_dir_all(root);
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
    assert_eq!(build_json["status"], "ok");
    let build_json = build_json
        .get_mut("data")
        .expect("build output should have data");
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
        build_json.clone(),
        json!({
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

    let full_validate_output = run_atlas(&index_path)?;
    assert_eq!(full_validate_output.status.code(), Some(3));
    let full_validate_json: Value = serde_json::from_slice(&full_validate_output.stdout)?;
    let full_validate_json = ok_data(&full_validate_json);
    assert_eq!(full_validate_json["code"], "artifact_contract_violation");
    assert_eq!(
        full_validate_json["diagnostics"][0]["key"],
        "table:record_vector_index"
    );

    let validate_output = run_atlas_base(&index_path)?;

    assert!(validate_output.status.success());
    let validate_json: Value = serde_json::from_slice(&validate_output.stdout)?;
    assert_eq!(validate_json["status"], "ok");
    let validate_json = ok_data(&validate_json);
    assert_eq!(validate_json["source_signature"], source_signature);
    assert_eq!(validate_json["source_record_count"], "1");
    assert_eq!(validate_json["artifact_record_count"], "1");
    assert_eq!(validate_json["generated_record_count"], "0");

    let validate_vectors_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate", "--embeddings-only", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;

    assert_eq!(validate_vectors_output.status.code(), Some(3));
    let validate_vectors_json: Value = serde_json::from_slice(&validate_vectors_output.stdout)?;
    let validate_vectors_json = ok_data(&validate_vectors_json);
    assert_eq!(validate_vectors_json["code"], "artifact_contract_violation");
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
    let inspect_json = ok_data(&inspect_json);
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
fn validate_rejects_conflicting_embedding_flags() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate", "--no-embeddings", "--embeddings-only"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("cannot be used with"));
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
    let analyze_json = ok_data(&analyze_json);
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
fn record_get_resolve_and_filter_search_use_shared_record_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-search");
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

    let get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "actions:testAction0001", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(get_output.status.success());
    let get_json: Value = serde_json::from_slice(&get_output.stdout)?;
    let get_data = ok_data(&get_json);
    assert_eq!(get_data["detail"], "standard");
    assert_eq!(get_data["record"]["key"], "actions:testAction0001");
    assert_eq!(get_data["record"]["name"], "Treat Wounds");
    assert_eq!(get_data["record"]["record_family"], "rule");
    assert!(record_sections(&get_data["record"]).contains(&"description"));
    assert!(!record_sections(&get_data["record"]).contains(&"description_preview"));
    assert!(get_data["record"].get("source_json").is_none());

    let full_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "--detail",
            "full",
            "--include-raw",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(full_get_output.status.success());
    let full_get_json: Value = serde_json::from_slice(&full_get_output.stdout)?;
    let full_get_data = ok_data(&full_get_json);
    assert_eq!(full_get_data["detail"], "full");
    assert!(full_get_data["record"]["source_json"].as_str().is_some());
    assert_eq!(
        full_get_data["record"]["source"]["foundry"]["document_type"],
        "Item"
    );
    assert!(record_sections(&full_get_data["record"]).contains(&"description"));
    assert!(!record_sections(&full_get_data["record"]).contains(&"description_preview"));

    let batch_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "get",
            "actions:testAction0001",
            "actions:missingAction999",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(batch_get_output.status.code(), Some(1));
    let batch_get_json: Value = serde_json::from_slice(&batch_get_output.stdout)?;
    let batch_get_data = ok_data(&batch_get_json);
    assert_eq!(batch_get_data["counts"]["requested"], 2);
    assert_eq!(batch_get_data["counts"]["matched"], 1);
    assert_eq!(batch_get_data["counts"]["failed"], 1);
    assert_eq!(
        batch_get_data["results"][1]["error"]["code"],
        "record_not_found"
    );

    let resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "Treat Wounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(resolve_output.status.success());
    let resolve_json: Value = serde_json::from_slice(&resolve_output.stdout)?;
    let resolve_data = ok_data(&resolve_json);
    assert_eq!(
        resolve_data["result"]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(resolve_data["detail"], "standard");
    assert!(record_sections(&resolve_data["result"]["record"]).contains(&"description"));
    assert!(!record_sections(&resolve_data["result"]["record"]).contains(&"description_preview"));
    assert_eq!(resolve_data["result"]["resolution"]["match_kind"], "name");

    let search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(search_output.status.success());
    let search_json: Value = serde_json::from_slice(&search_output.stdout)?;
    let search_data = ok_data(&search_json);
    assert_eq!(search_data["sort"]["kind"], "alphabetical");
    assert_eq!(search_data["pagination"]["total"], 1);
    assert_eq!(
        search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(search_data["results"][0]["match"]["kind"], "filter");

    let fts_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(fts_search_output.status.success());
    assert_eq!(String::from_utf8(fts_search_output.stderr)?, "");
    let fts_search_json: Value = serde_json::from_slice(&fts_search_output.stdout)?;
    let fts_search_data = ok_data(&fts_search_json);
    assert_eq!(fts_search_data["retrieval"], "fts");
    assert_eq!(
        fts_search_data["query_analysis"]["fts_tokens"][0],
        "healing"
    );
    assert_eq!(fts_search_data["fusion"]["method"], "weighted-rrf");
    assert_eq!(fts_search_data["candidate_windows"]["fts_top_k"], 200);
    assert_eq!(fts_search_data["candidate_windows"]["vector_top_k"], 200);
    assert_eq!(fts_search_data["sort"]["kind"], "ranked");
    assert_eq!(fts_search_data["pagination"]["total"], 1);
    assert_eq!(
        fts_search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );
    assert_eq!(fts_search_data["results"][0]["match"]["kind"], "ranked");
    assert_eq!(fts_search_data["results"][0]["match"]["retrieval"], "fts");
    assert_eq!(
        fts_search_data["results"][0]["match"]["explain"]["fts_rank"],
        1
    );

    let excluded_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--exclude",
            "treating",
            "--retrieval",
            "fts",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(excluded_search_output.status.success());
    let excluded_search_json: Value = serde_json::from_slice(&excluded_search_output.stdout)?;
    let excluded_search_data = ok_data(&excluded_search_json);
    assert_eq!(excluded_search_data["pagination"]["total"], 0);
    assert_eq!(
        excluded_search_data["query_analysis"]["exclude_tokens"][0],
        "treating"
    );

    let offset_window_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--limit",
            "1",
            "--offset",
            "2",
            "--fts-top-k",
            "1",
            "--explain",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(offset_window_output.status.success());
    let offset_window_json: Value = serde_json::from_slice(&offset_window_output.stdout)?;
    let offset_window_data = ok_data(&offset_window_json);
    assert_eq!(offset_window_data["candidate_windows"]["fts_top_k"], 3);

    let default_hybrid_without_embeddings = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "healing", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(default_hybrid_without_embeddings.status.code(), Some(3));
    let hybrid_error: Value = serde_json::from_slice(&default_hybrid_without_embeddings.stdout)?;
    assert_eq!(hybrid_error["status"], "error");
    assert_eq!(hybrid_error["error"]["code"], "vector_readiness_required");

    let text_get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "actions:testAction0001", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(text_get_output.status.success());
    let text_get_stdout = String::from_utf8(text_get_output.stdout)?;
    assert!(text_get_stdout.contains("actions:testAction0001\tTreat Wounds\trule"));
    assert!(!text_get_stdout.contains("\"status\""));

    let text_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .output()?;
    assert!(text_search_output.status.success());
    assert_eq!(String::from_utf8(text_search_output.stderr)?, "");
    let text_search_stdout = String::from_utf8(text_search_output.stdout)?;
    assert!(text_search_stdout.contains("showing 1 of 1 records"));
    assert!(text_search_stdout.contains("key"));
    assert!(text_search_stdout.contains("type"));
    assert!(text_search_stdout.contains("name"));
    assert!(text_search_stdout.contains("actions:testAction0001  rule  Treat Wounds"));
    assert!(!text_search_stdout.contains("\"status\""));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn record_commands_return_standard_error_codes() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-errors");
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

    let invalid_key = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "not-a-key", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(invalid_key.status.code(), Some(2));
    let invalid_key_json: Value = serde_json::from_slice(&invalid_key.stdout)?;
    assert_eq!(invalid_key_json["status"], "error");
    assert_eq!(invalid_key_json["error"]["code"], "invalid_record_key");

    let miss = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "No Such Record", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(miss.status.code(), Some(1));
    let miss_json: Value = serde_json::from_slice(&miss.stdout)?;
    assert_eq!(miss_json["status"], "error");
    assert_eq!(miss_json["error"]["code"], "record_resolution_miss");

    let invalid_filter = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "--filter-json",
            "{",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(invalid_filter.status.code(), Some(2));
    let invalid_filter_json: Value = serde_json::from_slice(&invalid_filter.stdout)?;
    assert_eq!(invalid_filter_json["status"], "error");
    assert_eq!(invalid_filter_json["error"]["code"], "invalid_filter_json");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn record_resolve_reports_ambiguity() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-ambiguity");
    write_ambiguous_fixture_source(&root)?;
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

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Duplicate Action",
            "--alternatives",
            "2",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(
        data["result"]["error"]["code"],
        "record_resolution_ambiguous"
    );
    assert_eq!(data["result"]["alternatives"].as_array().unwrap().len(), 2);
    assert_eq!(
        data["result"]["alternatives"][0]["record"]["key"],
        "actions:duplicateAction1"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn filter_search_reports_pagination_and_random_seed() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-pagination-random");
    write_ambiguous_fixture_source(&root)?;
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

    let paged_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--limit", "1", "--offset", "1", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(paged_output.status.success());
    let paged_json: Value = serde_json::from_slice(&paged_output.stdout)?;
    let paged_data = ok_data(&paged_json);
    assert_eq!(paged_data["pagination"]["offset"], 1);
    assert_eq!(paged_data["pagination"]["limit"], 1);
    assert_eq!(paged_data["pagination"]["count"], 1);
    assert_eq!(paged_data["pagination"]["total"], 2);
    assert_eq!(paged_data["pagination"]["has_more"], false);
    assert!(paged_data["pagination"].get("next_offset").is_none());

    let seeded_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--sort", "random", "--seed", "1234", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(seeded_output.status.success());
    let seeded_json: Value = serde_json::from_slice(&seeded_output.stdout)?;
    let seeded_data = ok_data(&seeded_json);
    assert_eq!(seeded_data["sort"]["kind"], "random");
    assert_eq!(seeded_data["sort"]["seed"], 1234);

    let generated_seed_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--sort", "random", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(generated_seed_output.status.success());
    let generated_seed_json: Value = serde_json::from_slice(&generated_seed_output.stdout)?;
    let generated_seed_data = ok_data(&generated_seed_json);
    assert_eq!(generated_seed_data["sort"]["kind"], "random");
    assert!(generated_seed_data["sort"]["seed"].as_u64().is_some());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn tooling_records_are_hidden_from_default_resolution_and_search_but_gettable_by_key()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-record-tooling-collision");
    write_tooling_collision_fixture_source(&root)?;
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

    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "resolve", "Treat Wounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["result"]["record"]["key"], "actions:testAction0001");
    assert_eq!(data["result"]["record"]["record_family"], "rule");

    let search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(search_output.status.success());
    let search_json: Value = serde_json::from_slice(&search_output.stdout)?;
    let search_data = ok_data(&search_json);
    assert_eq!(search_data["pagination"]["total"], 1);
    assert_eq!(
        search_data["results"][0]["record"]["key"],
        "actions:testAction0001"
    );

    let filtered_search_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["search", "--family", "rule", "--pack", "actions", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(filtered_search_output.status.success());
    let filtered_search_json: Value = serde_json::from_slice(&filtered_search_output.stdout)?;
    let filtered_search_data = ok_data(&filtered_search_json);
    assert_eq!(filtered_search_data["pagination"]["total"], 1);
    assert_eq!(filtered_search_data["filter"]["kind"], "all_of");
    assert_eq!(
        filtered_search_data["filter"]["children"][1]["predicate"]["field"],
        "pack_name"
    );

    let filtered_resolve_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "record",
            "resolve",
            "Treat Wounds",
            "--pack",
            "actions",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(filtered_resolve_output.status.success());
    let filtered_resolve_json: Value = serde_json::from_slice(&filtered_resolve_output.stdout)?;
    let filtered_resolve_data = ok_data(&filtered_resolve_json);
    assert_eq!(
        filtered_resolve_data["result"]["record"]["key"],
        "actions:testAction0001"
    );

    let get_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["record", "get", "pf2e-macros:macroTreatWounds", "--index"])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(get_output.status.success());
    let get_json: Value = serde_json::from_slice(&get_output.stdout)?;
    let get_data = ok_data(&get_json);
    assert_eq!(get_data["record"]["key"], "pf2e-macros:macroTreatWounds");
    assert_eq!(get_data["record"]["record_family"], "tooling");

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
fn search_rejects_invalid_filter_json_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--index",
            "missing.sqlite",
            "--filter-json",
            "{",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_filter_json");
    Ok(())
}

#[test]
fn search_rejects_filter_json_with_convenience_flags_before_runtime_loading()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--index",
            "missing.sqlite",
            "--filter-json",
            r#"{"kind":"record_family","value":"rule"}"#,
            "--family",
            "rule",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_filter");
    Ok(())
}

#[test]
fn search_rejects_unweighted_rrf_with_lane_weights() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "healing",
            "--retrieval",
            "fts",
            "--fusion",
            "rrf",
            "--fts-weight",
            "2",
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
fn validate_index_json_reports_valid_minimal_contract() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-valid");
    create_contract_database(&path, None)?;

    let output = run_atlas_base(&path)?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": true,
                "code": "ok",
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
                "embedding_unit_policy_version": "explicit-heading-sections/v1",
                "fts_tokenizer": "unicode61 remove_diacritics 2",
                "adjacent_manifest_path": "manifest.json"
            }
        })
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn validate_index_json_reports_unavailable_index() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("cli-unavailable");

    let output = run_atlas(&path)?;

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "index_unavailable",
                "index": path.display().to_string(),
                "message": format!("index is unavailable: unable to open database file: {}", path.display())
            }
        })
    );
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
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "index_unavailable",
                "index": path.display().to_string(),
                "message": format!("index is unavailable: unable to open database file: {}", path.display())
            }
        })
    );
    Ok(())
}

#[test]
fn validate_vectors_subcommand_is_removed() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate-vectors"])
        .output()?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unrecognized subcommand 'validate-vectors'"));
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

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "missing_artifact_metadata",
                "index": path.display().to_string(),
                "message": "index opened, but the Rust artifact contract metadata table is missing",
                "legacy_schema_version": "25"
            }
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

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "embedding_mismatch",
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
                "embedding_unit_policy_version": "explicit-heading-sections/v1",
                "fts_tokenizer": "unicode61 remove_diacritics 2",
                "adjacent_manifest_path": "manifest.json",
                "diagnostics": [
                    {
                        "code": "embedding_mismatch",
                        "family": "embedding",
                        "message": "metadata key `embedding_dimensions` has an unsupported value",
                        "key": "embedding_dimensions",
                        "expected": "384",
                        "actual": "768"
                    }
                ]
            }
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

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "missing_required_metadata",
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
                "embedding_unit_policy_version": "explicit-heading-sections/v1",
                "fts_tokenizer": "unicode61 remove_diacritics 2",
                "adjacent_manifest_path": "manifest.json",
                "missing_keys": ["embedding_dtype"]
            }
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

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "stale_source_signature",
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
                "embedding_unit_policy_version": "explicit-heading-sections/v1",
                "fts_tokenizer": "unicode61 remove_diacritics 2",
                "adjacent_manifest_path": "manifest.json",
                "diagnostics": [
                    {
                        "code": "stale_source_signature",
                        "family": "source",
                        "message": "source signature marks this artifact as stale",
                        "key": "source_signature",
                        "expected": "current source signature",
                        "actual": "stale:fixture"
                    }
                ]
            }
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

    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        actual,
        json!({
            "status": "ok",
            "data": {
                "valid": false,
                "code": "unsupported_schema_version",
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
                "embedding_unit_policy_version": "explicit-heading-sections/v1",
                "fts_tokenizer": "unicode61 remove_diacritics 2",
                "adjacent_manifest_path": "manifest.json",
                "diagnostics": [
                    {
                        "code": "unsupported_schema_version",
                        "family": "schema",
                        "message": "metadata key `schema_version` has an unsupported value",
                        "key": "schema_version",
                        "expected": "1",
                        "actual": "2"
                    }
                ]
            }
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

fn run_atlas_base(path: &PathBuf) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "validate", "--no-embeddings", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

fn help_output(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(args)
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?)
}

fn setup_records_offline(
    source: &Path,
    cache: &Path,
    index: &Path,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "setup",
            "--path-mode",
            "user",
            "--offline",
            "--no-embeddings",
            "--source",
        ])
        .arg(source)
        .args(["--embedding-cache-path"])
        .arg(cache)
        .args(["--index"])
        .arg(index)
        .arg("--json")
        .output()?)
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
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

fn record_sections(record: &Value) -> Vec<&str> {
    record["sections"]
        .as_array()
        .expect("sections")
        .iter()
        .map(|section| section["kind"].as_str().expect("section kind"))
        .collect()
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

fn write_ambiguous_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" }
          ]
        }"#,
    )?;
    for id in ["duplicateAction1", "duplicateAction2"] {
        fs::write(
            root.join(format!("packs/actions/{id}.json")),
            format!(
                r#"{{
                  "_id": "{id}",
                  "name": "Duplicate Action",
                  "type": "action",
                  "system": {{
                    "traits": {{ "value": ["exploration"] }},
                    "description": {{ "value": "<p>Duplicate action fixture.</p>" }}
                  }}
                }}"#
            ),
        )?;
    }
    Ok(())
}

fn write_tooling_collision_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    write_fixture_source(root)?;
    fs::create_dir_all(root.join("packs/pf2e-macros"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "pf2e-macros", "label": "Macros", "type": "Macro", "path": "packs/pf2e-macros" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/pf2e-macros/treat-wounds-macro.json"),
        r#"{
          "_id": "macroTreatWounds",
          "name": "Treat Wounds",
          "type": "script",
          "command": "console.log('Treat Wounds')"
        }"#,
    )?;
    Ok(())
}
