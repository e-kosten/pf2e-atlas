use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn filter_search_supports_price_sorting() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_source_root("cli-search-price-sort");
    write_price_sort_fixture_source(&root)?;
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

    let ascending_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "equipment",
            "--sort",
            "price_asc",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(ascending_output.status.success());
    let ascending_json: Value = serde_json::from_slice(&ascending_output.stdout)?;
    let ascending_data = ok_data(&ascending_json);
    assert_eq!(ascending_data["sort"]["kind"], "price_asc");
    assert_eq!(
        ascending_data["results"][0]["record"]["key"],
        "equipment:cheapItem0001"
    );
    assert_eq!(
        ascending_data["results"][1]["record"]["key"],
        "equipment:expensiveItem"
    );

    let descending_output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args([
            "search",
            "--family",
            "equipment",
            "--sort",
            "price_desc",
            "--index",
        ])
        .arg(&index_path)
        .arg("--json")
        .output()?;
    assert!(descending_output.status.success());
    let descending_json: Value = serde_json::from_slice(&descending_output.stdout)?;
    let descending_data = ok_data(&descending_json);
    assert_eq!(descending_data["sort"]["kind"], "price_desc");
    assert_eq!(
        descending_data["results"][0]["record"]["key"],
        "equipment:expensiveItem"
    );
    assert_eq!(
        descending_data["results"][1]["record"]["key"],
        "equipment:cheapItem0001"
    );

    fs::remove_dir_all(root)?;
    Ok(())
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

fn write_price_sort_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/equipment"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "equipment", "label": "Equipment", "type": "Item", "path": "packs/equipment" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/cheap-tool.json"),
        r#"{
          "_id": "cheapItem0001",
          "name": "Cheap Tool",
          "type": "equipment",
          "system": {
            "price": { "value": { "cp": 5 } },
            "traits": { "value": ["tool"] },
            "description": { "value": "<p>A cheap tool.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/expensive-tool.json"),
        r#"{
          "_id": "expensiveItem",
          "name": "Expensive Tool",
          "type": "equipment",
          "system": {
            "price": { "value": { "gp": 1 } },
            "traits": { "value": ["tool"] },
            "description": { "value": "<p>An expensive tool.</p>" }
          }
        }"#,
    )?;
    Ok(())
}
