use std::fs;

mod support;
use support::command::atlas_command;
use support::json::{parse_json, parse_ok_data};
use support::path::temp_source_root;
use support::source::write_single_action_source;

#[test]
fn source_values_reports_missing_values_counts_and_refs_without_an_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("source-values");
    write_single_action_source(&root)?;
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{"_id":"a","type":"action","values":["usual","usual",null,"rare"]}"#,
    )?;
    fs::write(
        root.join("packs/actions/second.json"),
        r#"{"_id":"b","type":"action"}"#,
    )?;
    let command = || {
        let mut cmd = atlas_command();
        cmd.args(["index", "source-values", "--source"])
            .arg(&root)
            .args(["--record-type", "action", "--path", "$.values[]"]);
        cmd
    };
    let output = command().arg("--json").output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = parse_ok_data(&output)?;
    assert_eq!(data["schema_version"], "pf2e-source-values/v1");
    assert_eq!(data["record_count"], 2);
    let field = &data["fields"][0];
    assert_eq!(field["missing_record_count"], 1);
    assert_eq!(field["occurrence_count"], 4);
    assert_eq!(field["distinct_value_count"], 3);
    assert_eq!(field["values"][0]["value_json"], "\"usual\"");
    assert_eq!(field["values"][0]["occurrence_count"], 2);
    assert_eq!(field["values"][0]["record_count"], 1);
    assert_eq!(
        field["values"][0]["examples"][0]["source_pointer"],
        "/values/0"
    );
    assert_eq!(field["values"][0]["examples"][0]["record_key"], "actions:a");
    assert_eq!(data["complete"], true);
    assert!(!root.join("artifact.sqlite").exists());
    let limited = command()
        .args(["--limit", "1", "--sample-limit", "0", "--json"])
        .output()?;
    let data = parse_ok_data(&limited)?;
    assert_eq!(data["complete"], false);
    assert_eq!(data["fields"][0]["distinct_value_count"], 3);
    assert_eq!(data["fields"][0]["values"].as_array().unwrap().len(), 1);
    assert_eq!(
        data["fields"][0]["values"][0]["examples"],
        serde_json::json!([])
    );
    let terminal = command().args(["--limit", "1"]).output()?;
    assert!(terminal.status.success());
    let text = String::from_utf8(terminal.stdout)?;
    assert!(text.contains("1 present, 1 missing"));
    assert!(text.contains("Showing 1 of 3 values"));
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn source_values_cli_rejects_missing_paths_and_malformed_source()
-> Result<(), Box<dyn std::error::Error>> {
    let missing_path = atlas_command()
        .args(["index", "source-values", "--json"])
        .output()?;
    assert_eq!(missing_path.status.code(), Some(2));
    assert_eq!(parse_json(&missing_path)?["error"]["code"], "invalid_input");
    let root = temp_source_root("source-values-invalid");
    write_single_action_source(&root)?;
    fs::write(root.join("packs/actions/treat-wounds.json"), "{")?;
    let output = atlas_command()
        .args(["index", "source-values", "--source"])
        .arg(&root)
        .args(["--path", "$.system", "--json"])
        .output()?;
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        parse_json(&output)?["error"]["code"],
        "source_discovery_failed"
    );
    let invalid = atlas_command()
        .args(["index", "source-values", "--source"])
        .arg(&root)
        .args(["--path", "system", "--json"])
        .output()?;
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(parse_json(&invalid)?["error"]["code"], "invalid_input");
    fs::remove_dir_all(root)?;
    Ok(())
}
