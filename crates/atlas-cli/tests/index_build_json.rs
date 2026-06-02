use std::fs;
use std::process::Command;

mod support;

use support::command::{validate_base_index, validate_index};
use support::json::parse_ok_data;
use support::path::temp_source_root;
use support::source::write_record_search_source;
use support::timing::{assert_human_duration, timing_value};

#[test]
fn build_index_json_writes_valid_minimal_artifact() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-build");
    write_record_search_source(&root)?;
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
    let build_json = parse_ok_data(&build_output)?;
    let source_signature = build_json["source_signature"]
        .as_str()
        .expect("index build should report source signature")
        .to_string();
    assert!(source_signature.starts_with("foundry-pf2e:sha256:"));
    assert_eq!(source_signature.len(), "foundry-pf2e:sha256:".len() + 64);
    assert!(
        build_json["build_duration_ms"]
            .as_u64()
            .is_some_and(|duration| duration > 0)
    );
    assert_eq!(build_json["output"], index_path.display().to_string());
    assert_eq!(build_json["pack_count"], 1);
    assert_eq!(build_json["record_count"], 1);
    assert_eq!(build_json["source_record_count"], 1);
    assert_eq!(build_json["artifact_record_count"], 1);
    assert_eq!(build_json["generated_record_count"], 0);
    assert_eq!(build_json["pending_document_embedding_count"], 1);
    assert_eq!(build_json["document_embedding_count"], 0);
    assert_eq!(build_json["reused_document_embedding_count"], 0);
    assert_eq!(build_json["generated_document_embedding_count"], 0);
    assert_eq!(build_json["skipped_record_count"], 0);
    assert_eq!(build_json["warnings"].as_array().unwrap().len(), 0);
    assert_eq!(
        build_json["diagnostics"]["dropped_inline_macros"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let full_validate_output = validate_index(&index_path)?;
    assert_eq!(full_validate_output.status.code(), Some(3));
    let full_validate_json = parse_ok_data(&full_validate_output)?;
    assert_eq!(full_validate_json["code"], "artifact_contract_violation");
    assert_eq!(
        full_validate_json["diagnostics"][0]["key"],
        "table:record_vector_index"
    );

    let validate_output = validate_base_index(&index_path)?;

    assert!(validate_output.status.success());
    let validate_json = parse_ok_data(&validate_output)?;
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
    let validate_vectors_json = parse_ok_data(&validate_vectors_output)?;
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
    let inspect_json = parse_ok_data(&inspect_output)?;
    assert_eq!(inspect_json["records"]["total_records"], 1);
    assert_eq!(inspect_json["records"]["default_visible_records"], 1);
    assert_eq!(inspect_json["records"]["by_kind"]["rule"], 1);
    assert_eq!(
        inspect_json["records"]["by_publication_category"]["unknown"],
        1
    );
    assert_eq!(inspect_json["tables"]["records"], 1);
    assert_eq!(inspect_json["tables"]["packs"], 1);
    assert_eq!(inspect_json["tables"]["document_embedding_cache"], 0);
    assert_eq!(inspect_json["text"]["records_with_description"], 1);
    assert_eq!(inspect_json["relationships"]["reference_edges"], 1);
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
    write_record_search_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let analyze_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "analyze", "--source"])
        .arg(&root)
        .arg("--json")
        .output()?;

    assert!(analyze_output.status.success());
    assert!(!index_path.exists());
    let analyze_json = parse_ok_data(&analyze_output)?;
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
    assert_eq!(analyze_json["by_kind"]["rule"], 1);
    assert_eq!(analyze_json["by_foundry_taxonomy"]["Item|action"], 1);
    assert_eq!(analyze_json["by_publication_category"]["unknown"], 1);
    assert_eq!(analyze_json["side_data"]["item_records"], 1);
    assert_eq!(analyze_json["text"]["records_with_description"], 1);
    assert_eq!(analyze_json["embeddings"]["pending_document_embeddings"], 1);
    assert_eq!(analyze_json["relationships"]["reference_edges"], 1);
    assert_eq!(analyze_json["skipped_record_count"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn build_index_human_output_reports_timing_summary() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-build-human");
    write_record_search_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let build_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["index", "build", "--source"])
        .arg(&root)
        .args(["--output"])
        .arg(&index_path)
        .arg("--no-embeddings")
        .output()?;

    assert!(build_output.status.success());
    let stdout = String::from_utf8(build_output.stdout)?;
    assert!(stdout.contains("ok: wrote 1 records from 1 packs"));
    let stderr = String::from_utf8(build_output.stderr)?;
    assert!(stderr.contains("embeddings: pending_document=1 document=0 reused=0 generated=0"));
    assert!(!stderr.contains("build_duration_ms="));
    let build_duration = timing_value(&stderr, "build=");
    assert_human_duration(build_duration);
    assert!(stderr.contains("embedding_tokenization=0ms"));
    assert!(stderr.contains("embedding_model_load=0ms"));
    assert!(stderr.contains("embedding_generation=0ms"));

    Ok(())
}
