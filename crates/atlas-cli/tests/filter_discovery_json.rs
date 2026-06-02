use std::fs;
use std::path::Path;
use std::process::Command;

use rusqlite::Connection;
use serde_json::{Value, json};

mod support;

use support::command::build_index;
use support::json::ok_data;
use support::path::temp_source_root;
use support::source::{write_filter_discovery_source, write_rule_discovery_source};

#[test]
fn reports_fields_values_and_dynamic_refinements() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery");
    write_rule_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let fields_output = atlas(&[
        "filters",
        "fields",
        "--kind",
        "rule",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(fields_output.status.success());
    let fields_json: Value = serde_json::from_slice(&fields_output.stdout)?;
    let fields_data = ok_data(&fields_json);
    assert_eq!(fields_data["execution"], "catalog");
    assert!(
        fields_data["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "traits"
                && field["value_policy"] == "enumerable"
                && field["cli_flags"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("--trait")))
    );
    assert!(
        !fields_data["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field"] == "target_text")
    );

    let catalog_values = atlas(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "rule",
        "--sort",
        "alpha",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(catalog_values.status.success());
    let catalog_json: Value = serde_json::from_slice(&catalog_values.stdout)?;
    let catalog_data = ok_data(&catalog_json);
    assert_eq!(catalog_data["execution"], "catalog");
    assert_eq!(catalog_data["null_count"], 0);
    assert_eq!(
        catalog_data["values"],
        json!([
            { "value": "exploration", "count": 1 },
            { "value": "healing", "count": 1 }
        ])
    );

    let dynamic_values = atlas(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "rule",
        "--trait",
        "healing",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(dynamic_values.status.success());
    let dynamic_json: Value = serde_json::from_slice(&dynamic_values.stdout)?;
    let dynamic_data = ok_data(&dynamic_json);
    assert_eq!(dynamic_data["execution"], "dynamic");
    assert_eq!(dynamic_data["matching_record_count"], 1);

    let invalid_sample_limit = atlas(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--sample-limit",
        "1",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert_eq!(invalid_sample_limit.status.code(), Some(2));
    let invalid_sample_json: Value = serde_json::from_slice(&invalid_sample_limit.stdout)?;
    assert_eq!(invalid_sample_json["status"], "error");
    assert_eq!(invalid_sample_json["error"]["code"], "invalid_option");

    let invalid_field = atlas(&[
        "filters",
        "values",
        "--field",
        "packs",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert_eq!(invalid_field.status.code(), Some(2));
    let invalid_field_json: Value = serde_json::from_slice(&invalid_field.stdout)?;
    assert_eq!(invalid_field_json["error"]["code"], "invalid_field");
    assert!(
        invalid_field_json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("pack_name")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn filter_discovery_defaults_to_human_readable_output() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-text");
    write_rule_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let fields_output = atlas_raw(&[
        "filters",
        "fields",
        "--kind",
        "rule",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(fields_output.status.success());
    assert!(serde_json::from_slice::<Value>(&fields_output.stdout).is_err());
    let fields_text = String::from_utf8(fields_output.stdout)?;
    assert!(fields_text.contains("Filter fields"));
    assert!(fields_text.contains("Filter: kind = rule"));
    assert!(fields_text.contains("Matching records:"));
    assert!(fields_text.contains("record"));
    assert!(fields_text.contains("traits"));
    assert!(fields_text.contains("flags: --trait, --any-trait"));

    let values_output = atlas_raw(&[
        "filters",
        "values",
        "--field",
        "traits",
        "--kind",
        "rule",
        "--sort",
        "alpha",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(values_output.status.success());
    assert!(serde_json::from_slice::<Value>(&values_output.stdout).is_err());
    let values_text = String::from_utf8(values_output.stdout)?;
    assert!(values_text.contains("Values for field: traits"));
    assert!(values_text.contains("Filter: kind = rule"));
    assert!(values_text.contains("exploration"));
    assert!(values_text.contains("healing"));
    assert!(values_text.contains("Null values: 0"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn reports_metric_numeric_sample_and_boolean_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-rich");
    write_filter_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let metric_keys = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--kind",
        "creature",
        "--metric-prefix",
        "save.",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_keys.status.success());
    let metric_keys_json: Value = serde_json::from_slice(&metric_keys.stdout)?;
    let metric_keys_data = ok_data(&metric_keys_json);
    assert_eq!(metric_keys_data["execution"], "catalog");
    let fortitude = metric_keys_data["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metric| metric["metric_key"] == "save.fort.mod")
        .expect("fortitude save metric should be discoverable");
    assert_eq!(fortitude["label"], "Fortitude modifier");
    assert_eq!(fortitude["numeric_stats"]["count"], 1);
    assert_eq!(fortitude["numeric_stats"]["null_count"], 0);
    assert_eq!(fortitude["numeric_stats"]["p50"], 5.0);

    let metric_label_values = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--kind",
        "creature",
        "--metric",
        "Best save",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_label_values.status.success());
    let metric_label_json: Value = serde_json::from_slice(&metric_label_values.stdout)?;
    let metric_label_data = ok_data(&metric_label_json);
    assert_eq!(metric_label_data["execution"], "catalog");
    assert_eq!(metric_label_data["value_policy"], "metric_values");
    assert_eq!(metric_label_data["metric"]["metric_key"], "save.best");
    assert_eq!(
        metric_label_data["values"],
        json!({ "kind": "text_values", "values": [{ "value": "ref", "count": 1 }] })
    );

    let metric_numeric_values = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--kind",
        "creature",
        "--metric",
        "ac.value",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_numeric_values.status.success());
    let metric_numeric_json: Value = serde_json::from_slice(&metric_numeric_values.stdout)?;
    let metric_numeric_data = ok_data(&metric_numeric_json);
    assert_eq!(metric_numeric_data["values"]["kind"], "numeric_stats");
    assert_eq!(metric_numeric_data["values"]["stats"]["count"], 1);
    assert_eq!(metric_numeric_data["values"]["stats"]["null_count"], 0);
    assert_eq!(metric_numeric_data["values"]["stats"]["p50"], 17.0);

    let sample_values = atlas(&[
        "filters",
        "values",
        "--field",
        "target_text",
        "--kind",
        "spell",
        "--sample-limit",
        "50",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(sample_values.status.success());
    let sample_json: Value = serde_json::from_slice(&sample_values.stdout)?;
    let sample_data = ok_data(&sample_json);
    assert_eq!(sample_data["sample"]["sample_limit"], 50);
    assert_eq!(
        sample_data["sample"]["examples"][0]["text"],
        "1 willing creature"
    );
    assert_eq!(sample_data["field_stats"]["null_count"], 0);

    let sample_values_with_limit_alias = atlas(&[
        "filters",
        "values",
        "--field",
        "target_text",
        "--kind",
        "spell",
        "--limit",
        "25",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(sample_values_with_limit_alias.status.success());
    let sample_alias_json: Value = serde_json::from_slice(&sample_values_with_limit_alias.stdout)?;
    let sample_alias_data = ok_data(&sample_alias_json);
    assert_eq!(sample_alias_data["sample"]["sample_limit"], 25);

    let boolean_values = atlas(&[
        "filters",
        "values",
        "--field",
        "basic_save",
        "--kind",
        "spell",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(boolean_values.status.success());
    let boolean_json: Value = serde_json::from_slice(&boolean_values.stdout)?;
    let boolean_data = ok_data(&boolean_json);
    assert_eq!(boolean_data["value_policy"], "boolean_counts");
    assert_eq!(boolean_data["counts"]["true"], 1);
    assert_eq!(boolean_data["counts"]["false"], 0);
    assert_eq!(boolean_data["counts"]["null"], 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn catalog_validation_rejects_missing_rows() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-validation");
    write_rule_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "DELETE FROM filter_field_catalog WHERE field = 'traits' AND record_kind = 'rule'",
        [],
    )?;
    drop(connection);

    let validate_data = validate_contract_violation(&index_path)?;
    assert_diagnostic(&validate_data, "filter_field_catalog.missing_rows");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn catalog_validation_rejects_missing_payload_rows() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-payload-validation");
    write_filter_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "DELETE FROM filter_value_catalog WHERE field = 'traits' AND value = 'healing'",
        [],
    )?;
    connection.execute(
        "DELETE FROM filter_sample_catalog WHERE field = 'target_text' AND record_kind = 'spell'",
        [],
    )?;
    connection.execute(
        "DELETE FROM filter_numeric_catalog
         WHERE field = 'metric' AND metric_key = 'ac.value' AND record_kind = 'creature'",
        [],
    )?;
    drop(connection);

    let validate_data = validate_contract_violation(&index_path)?;
    for key in [
        "filter_value_catalog.missing_rows",
        "filter_sample_catalog.missing_rows",
        "filter_numeric_catalog.missing_rows",
    ] {
        assert_diagnostic(&validate_data, key);
    }

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn catalog_validation_rejects_stale_payload_rows() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-stale-validation");
    write_filter_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "UPDATE filter_field_catalog
         SET value_policy = 'sample'
         WHERE field = 'traits' AND record_kind = 'spell'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_value_catalog
         SET catalog_count = catalog_count + 1
         WHERE field = 'traits' AND record_kind = 'spell' AND value = 'healing'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_sample_catalog
         SET sample_rank = sample_rank + 1
         WHERE field = 'target_text' AND record_kind = 'spell'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_numeric_catalog
         SET p50 = p50 + 1
         WHERE field = 'metric' AND metric_key = 'ac.value' AND record_kind = 'creature'",
        [],
    )?;
    drop(connection);

    let validate_data = validate_contract_violation(&index_path)?;
    for key in [
        "filter_field_catalog.mismatched_rows",
        "filter_value_catalog.stale_rows",
        "filter_sample_catalog.stale_rows",
        "filter_numeric_catalog.stale_rows",
    ] {
        assert_diagnostic(&validate_data, key);
    }

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn catalog_validation_rejects_duplicate_global_rows() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-duplicate-validation");
    write_filter_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    for sql in [
        "INSERT INTO filter_field_catalog
         SELECT * FROM filter_field_catalog
         WHERE field = 'traits' AND record_kind IS NULL",
        "INSERT INTO filter_value_catalog
         SELECT * FROM filter_value_catalog
         WHERE field = 'traits' AND record_kind IS NULL AND value = 'healing'",
        "INSERT INTO filter_sample_catalog
         SELECT * FROM filter_sample_catalog
         WHERE field = 'target_text' AND record_kind IS NULL
         LIMIT 1",
        "INSERT INTO filter_numeric_catalog
         SELECT * FROM filter_numeric_catalog
         WHERE field = 'metric' AND record_kind IS NULL AND metric_key = 'ac.value'",
        "INSERT INTO metric_key_catalog
         SELECT * FROM metric_key_catalog
         WHERE record_kind IS NULL AND metric_key = 'ac.value'",
        "INSERT INTO metric_value_catalog
         SELECT * FROM metric_value_catalog
         WHERE record_kind IS NULL AND metric_key = 'save.best'",
    ] {
        connection.execute(sql, [])?;
    }
    drop(connection);

    let validate_data = validate_contract_violation(&index_path)?;
    for key in [
        "filter_field_catalog.duplicate_rows",
        "filter_value_catalog.duplicate_rows",
        "filter_sample_catalog.duplicate_rows",
        "filter_numeric_catalog.duplicate_rows",
        "metric_key_catalog.duplicate_rows",
        "metric_value_catalog.duplicate_rows",
    ] {
        assert_diagnostic(&validate_data, key);
    }

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn metric_discovery_uses_global_catalog_and_rejects_conflicting_options()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-global-metric");
    write_filter_discovery_source(&root)?;
    let index_path = build_index(&root)?;

    let global_keys = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--metric-prefix",
        "save.",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(global_keys.status.success());
    let global_keys_json: Value = serde_json::from_slice(&global_keys.stdout)?;
    let global_keys_data = ok_data(&global_keys_json);
    assert_eq!(global_keys_data["execution"], "catalog");
    assert!(
        global_keys_data["metrics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|metric| metric["metric_key"] == "save.best" && metric["kind"] == "all")
    );

    let global_numeric = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--metric",
        "ac.value",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(global_numeric.status.success());
    let global_numeric_json: Value = serde_json::from_slice(&global_numeric.stdout)?;
    let global_numeric_data = ok_data(&global_numeric_json);
    assert_eq!(global_numeric_data["execution"], "catalog");
    assert_eq!(global_numeric_data["values"]["kind"], "numeric_stats");
    assert_eq!(global_numeric_data["values"]["stats"]["count"], 2);
    assert_eq!(global_numeric_data["values"]["stats"]["p50"], 17.0);

    let conflicting_options = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--metric",
        "ac.value",
        "--metric-label",
        "AC",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert_eq!(conflicting_options.status.code(), Some(2));
    let conflicting_json: Value = serde_json::from_slice(&conflicting_options.stdout)?;
    assert_eq!(conflicting_json["status"], "error");
    assert_eq!(conflicting_json["error"]["code"], "invalid_option");

    fs::remove_dir_all(root)?;
    Ok(())
}

fn atlas(args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut args = args.to_vec();
    if !args.contains(&"--json") {
        args.push("--json");
    }
    atlas_raw(&args)
}

fn atlas_raw(args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(args)
        .output()?)
}

fn validate_contract_violation(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let output = atlas(&[
        "index",
        "validate",
        "--no-embeddings",
        "--index",
        path.to_str().unwrap(),
        "--json",
    ])?;
    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&json);
    assert_eq!(data["code"], "artifact_contract_violation");
    Ok(data.clone())
}

fn assert_diagnostic(data: &Value, key: &str) {
    assert!(
        data["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["key"] == key),
        "missing diagnostic {key}"
    );
}
