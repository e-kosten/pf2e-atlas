use serde_json::json;
use std::fs;
use std::process::Command;

mod support;
use support::json::parse_ok_data;
use support::path::temp_source_root;
use support::source::write_record_search_source;

#[test]
fn analyze_source_json_reports_source_without_writing_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-analyze");
    write_record_search_source(&root)?;
    let index_path = root.join("artifact.sqlite");

    let analyze_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["source", "analyze", "--source"])
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
        .expect("source analyze should report source signature");
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
    assert_eq!(analyze_json["mechanics"]["item_records"], 1);
    assert_eq!(analyze_json["text"]["records_with_description"], 1);
    assert_eq!(analyze_json["embeddings"]["pending_document_embeddings"], 1);
    assert_eq!(analyze_json["relationships"]["reference_edges"], 1);
    assert_eq!(analyze_json["skipped_record_count"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn source_schema_json_reports_schema_and_checks_a_baseline()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-source-schema");
    write_record_search_source(&root)?;

    let audit_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["source", "schema", "--source"])
        .arg(&root)
        .args(["--record-type", "action", "--json"])
        .output()?;

    assert!(audit_output.status.success());
    let audit_json = parse_ok_data(&audit_output)?;
    assert_eq!(audit_json["pack_count"], 1);
    assert_eq!(audit_json["record_count"], 1);
    assert_eq!(audit_json["filters"]["record_type"], "action");
    assert_eq!(audit_json["schema_version"], "pf2e-source-schema/v1");
    assert_eq!(audit_json["complete"], true);
    assert!(audit_json.get("enforcement").is_none());
    assert!(
        audit_json["path_count"]
            .as_u64()
            .is_some_and(|count| count >= 1)
    );
    let paths = audit_json["paths"].as_array().expect("audit paths");
    assert!(paths.iter().any(|path| {
        path["path"] == "$.system.description.value"
            && path["value_types"][0]["kind"] == "string"
            && path.get("owner").is_none()
    }));

    let baseline = root.join("schema.json");
    fs::write(&baseline, &audit_output.stdout)?;

    let strict_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["source", "schema", "--source"])
        .arg(&root)
        .args(["--record-type", "action", "--strict", "--baseline"])
        .arg(&baseline)
        .arg("--json")
        .output()?;
    assert!(strict_output.status.success());
    let strict_json = parse_ok_data(&strict_output)?;
    assert_eq!(strict_json["source_diff"]["added_paths"], json!([]));

    let mut changed_baseline = audit_json.clone();
    changed_baseline["paths"]
        .as_array_mut()
        .expect("paths")
        .retain(|path| path["path"] != "$.system.description.value");
    changed_baseline["path_count"] =
        json!(changed_baseline["paths"].as_array().expect("paths").len());
    fs::write(&baseline, serde_json::to_vec(&changed_baseline)?)?;
    let changed_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["source", "schema", "--source"])
        .arg(&root)
        .args(["--record-type", "action", "--strict", "--baseline"])
        .arg(&baseline)
        .args(["--limit", "1", "--json"])
        .output()?;
    assert_eq!(changed_output.status.code(), Some(3));
    let changed_json = parse_ok_data(&changed_output)?;
    assert_eq!(
        changed_json["source_diff"]["added_paths"][0]["path"],
        "$.system.description.value"
    );
    assert_eq!(changed_json["complete"], false);

    let repeat_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["source", "schema", "--source"])
        .arg(&root)
        .args(["--record-type", "action", "--json"])
        .output()?;
    assert!(repeat_output.status.success());
    assert_eq!(audit_output.stdout, repeat_output.stdout);

    fs::remove_dir_all(root)?;
    Ok(())
}
