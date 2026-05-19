use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

#[test]
fn metric_query_and_short_label_filters_use_catalog_resolution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-metric-resolution");
    write_metric_fixture_source(&root)?;
    let index_path = build_index(&root)?;

    let metric_query = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--family",
        "creature",
        "--metric-query",
        "defense ac",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_query.status.success());
    let metric_query_json: Value = serde_json::from_slice(&metric_query.stdout)?;
    let metric_query_data = ok_data(&metric_query_json);
    let armor_class = metric_query_data["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metric| metric["metric_key"] == "ac.value")
        .expect("armor class metric should be discoverable by query");
    assert_eq!(armor_class["label"], "Armor Class");
    assert_eq!(armor_class["short_label"], "AC");
    assert_eq!(armor_class["group"], "defense");

    let metric_short_label_values = atlas(&[
        "filters",
        "values",
        "--field",
        "metric",
        "--family",
        "creature",
        "--metric",
        "AC",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_short_label_values.status.success());
    let metric_short_label_json: Value = serde_json::from_slice(&metric_short_label_values.stdout)?;
    let metric_short_label_data = ok_data(&metric_short_label_json);
    assert_eq!(metric_short_label_data["metric"]["metric_key"], "ac.value");
    assert_eq!(
        metric_short_label_data["values"]["stats"]["p50"],
        json!(17.0)
    );

    let metric_filter_search = atlas(&[
        "search",
        "--family",
        "creature",
        "--metric",
        "AC>=17",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert!(metric_filter_search.status.success());
    let metric_filter_json: Value = serde_json::from_slice(&metric_filter_search.stdout)?;
    let metric_filter_data = ok_data(&metric_filter_json);
    assert_eq!(metric_filter_data["pagination"]["total"], 1);
    assert_eq!(
        metric_filter_data["results"][0]["record"]["key"],
        "bestiary:testActor0001"
    );

    let unknown_metric_filter = atlas(&[
        "search",
        "--family",
        "creature",
        "--metric",
        "armor>=17",
        "--index",
        index_path.to_str().unwrap(),
    ])?;
    assert_eq!(unknown_metric_filter.status.code(), Some(3));
    let unknown_metric_json: Value = serde_json::from_slice(&unknown_metric_filter.stdout)?;
    assert_eq!(unknown_metric_json["error"]["code"], "invalid_filter");
    let unknown_metric_message = unknown_metric_json["error"]["message"].as_str().unwrap();
    assert!(unknown_metric_message.contains("unknown metric `armor`"));
    assert!(unknown_metric_message.contains("ac.value"));
    assert!(unknown_metric_message.contains("--metric-query armor"));

    fs::remove_dir_all(root)?;
    Ok(())
}

fn atlas(args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut args = args.to_vec();
    if !args.contains(&"--json") {
        args.push("--json");
    }
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

fn write_metric_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" }
          ]
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
    Ok(())
}
