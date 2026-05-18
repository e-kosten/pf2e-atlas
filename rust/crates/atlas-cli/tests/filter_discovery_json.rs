use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::Connection;
use serde_json::{Value, json};

#[test]
fn reports_fields_values_and_dynamic_refinements() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery");
    write_rule_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let fields_output = atlas(&[
        "filters",
        "fields",
        "--family",
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
        "--family",
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
        "--family",
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
fn reports_metric_numeric_sample_and_boolean_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-filter-discovery-rich");
    write_filter_discovery_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let metric_keys = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--family",
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
        "--family",
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
        "--family",
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
        "--family",
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

    let boolean_values = atlas(&[
        "filters",
        "values",
        "--field",
        "basic_save",
        "--family",
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
    write_rule_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "DELETE FROM filter_field_catalog WHERE field = 'traits' AND record_family = 'rule'",
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
    write_filter_discovery_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "DELETE FROM filter_value_catalog WHERE field = 'traits' AND value = 'healing'",
        [],
    )?;
    connection.execute(
        "DELETE FROM filter_sample_catalog WHERE field = 'target_text' AND record_family = 'spell'",
        [],
    )?;
    connection.execute(
        "DELETE FROM filter_numeric_catalog
         WHERE field = 'metric' AND metric_key = 'ac.value' AND record_family = 'creature'",
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
    write_filter_discovery_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    connection.execute(
        "UPDATE filter_field_catalog
         SET value_policy = 'sample'
         WHERE field = 'traits' AND record_family = 'spell'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_value_catalog
         SET catalog_count = catalog_count + 1
         WHERE field = 'traits' AND record_family = 'spell' AND value = 'healing'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_sample_catalog
         SET sample_rank = sample_rank + 1
         WHERE field = 'target_text' AND record_family = 'spell'",
        [],
    )?;
    connection.execute(
        "UPDATE filter_numeric_catalog
         SET p50 = p50 + 1
         WHERE field = 'metric' AND metric_key = 'ac.value' AND record_family = 'creature'",
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
    write_filter_discovery_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let connection = Connection::open(&index_path)?;
    for sql in [
        "INSERT INTO filter_field_catalog
         SELECT * FROM filter_field_catalog
         WHERE field = 'traits' AND record_family IS NULL",
        "INSERT INTO filter_value_catalog
         SELECT * FROM filter_value_catalog
         WHERE field = 'traits' AND record_family IS NULL AND value = 'healing'",
        "INSERT INTO filter_sample_catalog
         SELECT * FROM filter_sample_catalog
         WHERE field = 'target_text' AND record_family IS NULL
         LIMIT 1",
        "INSERT INTO filter_numeric_catalog
         SELECT * FROM filter_numeric_catalog
         WHERE field = 'metric' AND record_family IS NULL AND metric_key = 'ac.value'",
        "INSERT INTO metric_key_catalog
         SELECT * FROM metric_key_catalog
         WHERE record_family IS NULL AND metric_key = 'ac.value'",
        "INSERT INTO metric_value_catalog
         SELECT * FROM metric_value_catalog
         WHERE record_family IS NULL AND metric_key = 'save.best'",
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
    write_filter_discovery_fixture_source(&root)?;
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
            .any(|metric| metric["metric_key"] == "save.best" && metric["record_family"] == "all")
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
    Ok(Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(args)
        .output()?)
}

fn build_index(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let index_path = root.join("artifact.sqlite");
    let output = atlas(&[
        "index",
        "build",
        "--source",
        root.to_str().unwrap(),
        "--output",
        index_path.to_str().unwrap(),
        "--no-embeddings",
        "--json",
    ])?;
    assert!(output.status.success());
    Ok(index_path)
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

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
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

fn write_rule_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

fn write_filter_discovery_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/spells"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/hazards"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "spells", "label": "Spells", "type": "Item", "path": "packs/spells" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "hazards", "label": "Hazards", "type": "Actor", "path": "packs/hazards" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/spells/heal.json"),
        r#"{
          "_id": "testSpell0001",
          "name": "Heal",
          "type": "spell",
          "system": {
            "level": { "value": 1 },
            "traits": { "value": ["healing", "vitality"], "traditions": ["divine"], "rarity": "common" },
            "time": { "value": "1 minute" },
            "duration": { "value": "10 minutes" },
            "range": { "value": "30 feet" },
            "target": { "value": "<p>1 willing creature</p>" },
            "defense": { "save": { "statistic": "fortitude", "basic": true } },
            "description": { "value": "<p>You channel vital energy.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/goblin.json"),
        r#"{
          "_id": "testActor0001",
          "name": "Goblin Scout",
          "type": "npc",
          "system": {
            "traits": { "value": ["goblin", "humanoid"], "size": { "value": "small" } },
            "details": { "languages": { "value": ["goblin"] } },
            "abilities": { "dex": { "mod": 4 } },
            "perception": { "mod": 7, "senses": [{ "type": "darkvision", "range": 60 }] },
            "attributes": {
              "ac": { "value": 17 },
              "hp": { "value": 16, "max": 16 },
              "speed": { "value": 25 }
            },
            "saves": {
              "fortitude": { "mod": 5 },
              "reflex": { "mod": 8 },
              "will": { "mod": 4 }
            },
            "description": { "value": "<p>A small scout.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/hazards/spear-trap.json"),
        r#"{
          "_id": "testHazard0001",
          "name": "Spear Trap",
          "type": "hazard",
          "system": {
            "traits": { "value": ["mechanical", "trap"], "size": { "value": "medium" } },
            "attributes": {
              "ac": { "value": 18 },
              "hp": { "value": 20, "max": 20 }
            },
            "saves": {
              "fortitude": { "mod": 8 },
              "reflex": { "mod": 6 },
              "will": { "mod": 2 }
            },
            "details": {
              "disable": "<p>Thievery DC 18</p>"
            },
            "description": { "value": "<p>A hidden spear trap.</p>" }
          }
        }"#,
    )?;
    Ok(())
}
