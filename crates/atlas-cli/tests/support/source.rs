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
