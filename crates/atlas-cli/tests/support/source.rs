#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

pub fn write_single_action_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("single-action", root)
}

pub fn write_changed_single_action_record(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        root.join("packs/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "description": { "value": "<p>You treat changed wounds.</p>" },
            "traits": { "value": ["healing"] }
          }
        }"#,
    )?;
    Ok(())
}

pub fn write_record_search_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("record-search", root)
}

pub fn write_rule_discovery_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("rule-discovery", root)
}

pub fn write_filter_discovery_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("filter-discovery", root)
}

pub fn write_metric_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("metric-creature", root)
}

pub fn write_price_sort_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("price-sort", root)
}

pub fn write_creature_preview_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("creature-preview", root)
}

pub fn write_hazard_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pack = root.join("packs/hazards");
    let actions = root.join("packs/actionspf2e");
    fs::create_dir_all(&pack)?;
    fs::create_dir_all(&actions)?;
    fs::write(
        root.join("module.json"),
        r#"{"packs":[{"name":"hazards","label":"Hazards","type":"Actor","path":"packs/hazards"},{"name":"actionspf2e","label":"Actions","type":"Item","path":"packs/actionspf2e"}]}"#,
    )?;
    let hazard_path = pack.join("hidden-pit.json");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../atlas-ingest/tests/fixtures/hazards/pinned/packs/hazards/hidden-pit.json"),
        &hazard_path,
    )?;
    let mut hazard: serde_json::Value = serde_json::from_slice(&fs::read(&hazard_path)?)?;
    let description = hazard
        .pointer_mut("/system/details/description")
        .and_then(|value| value.as_str())
        .ok_or("Hidden Pit description fixture")?
        .to_string();
    *hazard
        .pointer_mut("/system/details/description")
        .ok_or("Hidden Pit description fixture")? = serde_json::Value::String(format!(
        "{description}<p>Reference fixture: @UUID[Compendium.pf2e.actionspf2e.Item.Grab an Edge].</p>"
    ));
    fs::write(&hazard_path, serde_json::to_vec(&hazard)?)?;
    fs::write(
        actions.join("grab-an-edge.json"),
        r#"{"_id":"grabEdgeTest0001","name":"Grab an Edge","type":"action","system":{"description":{"value":"<p>A referenced reaction.</p>"},"traits":{"value":[]}}}"#,
    )?;
    Ok(())
}

pub fn write_consumable_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pack = root.join("packs/equipment");
    fs::create_dir_all(&pack)?;
    fs::write(
        root.join("module.json"),
        r#"{"packs":[{"name":"equipment","label":"Equipment","type":"Item","path":"packs/equipment"}]}"#,
    )?;
    fs::write(
        pack.join("test-elixir.json"),
        r#"{
          "_id": "cliConsumable0001",
          "name": "Test Elixir",
          "type": "consumable",
          "system": {
            "bulk": { "value": "L" },
            "category": "elixir",
            "containerId": null,
            "description": { "value": "<p>A typed consumable fixture.</p>" },
            "hardness": 0,
            "hp": { "max": 1, "value": 1 },
            "level": { "value": 2 },
            "price": { "per": 1, "value": { "gp": 3 } },
            "quantity": 2,
            "traits": { "otherTags": [], "rarity": "common", "value": ["alchemical", "consumable"] },
            "usage": { "value": "held-in-one-hand" },
            "uses": { "autoDestroy": false, "max": 1, "value": 1 }
          }
        }"#,
    )?;
    Ok(())
}

pub fn write_ambiguous_action_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("ambiguous-actions", root)
}

pub fn write_tooling_collision_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    copy_fixture_source("tooling-collision", root)
}

fn copy_fixture_source(name: &str, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fixture_source_dir(name);
    copy_dir_recursive(&source, root)
}

fn fixture_source_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundry-source")
        .join(name)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_source = entry.path();
        let entry_target = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry_source, &entry_target)?;
        } else {
            fs::copy(&entry_source, &entry_target)?;
        }
    }
    Ok(())
}
