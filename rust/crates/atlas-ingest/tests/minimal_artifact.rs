use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::{Category, Subcategory, ValidationStatus};
use atlas_index::validate_index;
use atlas_ingest::{BuildArtifactOptions, build_minimal_artifact, load_foundry_source};
use rusqlite::Connection;

#[test]
fn loads_tolerant_foundry_source_and_normalizes_records() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("load");
    write_fixture_source(&root)?;

    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.packs.len(), 2);
    assert_eq!(source.records.len(), 3);
    assert!(source.skipped_records.is_empty());
    assert!(source.warnings.is_empty());

    let treat_wounds = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "actions:testAction0001")
        .expect("action record should load");
    assert_eq!(treat_wounds.name, "Treat Wounds");
    assert_eq!(treat_wounds.normalized_name, "treat wounds");
    assert_eq!(treat_wounds.category, Category::Rule);
    assert_eq!(treat_wounds.subcategory, Some(Subcategory::Action));
    assert_eq!(treat_wounds.traits, vec!["exploration", "healing"]);
    assert_eq!(
        treat_wounds.description_text.as_deref(),
        Some("You spend 10 minutes treating one injured living creature.")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn reports_all_skipped_records_without_aborting_source_load()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("skips");
    write_fixture_source(&root)?;
    fs::write(root.join("packs/actions/broken-json.json"), "{")?;
    fs::write(
        root.join("packs/actions/missing-id.json"),
        r#"{
          "name": "Missing Id",
          "type": "action",
          "system": {
            "description": { "value": "<p>This record should be skipped.</p>" }
          }
        }"#,
    )?;

    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.records.len(), 3);
    assert_eq!(source.skipped_records.len(), 2);
    assert!(
        source
            .skipped_records
            .iter()
            .any(|record| record.path.ends_with("broken-json.json")
                && record.reason.contains("source record failed to parse"))
    );
    assert!(
        source
            .skipped_records
            .iter()
            .any(|record| record.path.ends_with("missing-id.json")
                && record.reason.contains("missing _id"))
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn resolves_namespaced_pf2e_pack_paths_from_manifest_declarations()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("namespaced");
    fs::create_dir_all(root.join("packs/pf2e/actions"))?;
    fs::write(
        root.join("system.pf2e.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/pf2e/actions/treat-wounds.json"),
        r#"{
          "_id": "testAction0001",
          "name": "Treat Wounds",
          "type": "action",
          "system": {
            "description": { "value": "<p>Use Medicine to help a wounded creature recover.</p>" }
          }
        }"#,
    )?;

    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.packs.len(), 1);
    assert_eq!(source.records.len(), 1);
    assert_eq!(
        source.packs[0].resolved_path,
        root.join("packs/pf2e/actions")
    );
    assert!(source.warnings.is_empty());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn writes_minimal_artifact_that_validate_index_accepts() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("build");
    write_fixture_source(&root)?;
    let output_path = root.join("artifact.sqlite");

    let report = build_minimal_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
    })?;

    assert_eq!(report.pack_count, 2);
    assert_eq!(report.record_count, 3);
    assert!(report.skipped_records.is_empty());

    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(validation.source_record_count.as_deref(), Some("3"));

    let connection = Connection::open(&output_path)?;
    let pack_count: usize =
        connection.query_row("SELECT COUNT(*) FROM packs", [], |row| row.get(0))?;
    let record_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
    let fts_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records_fts", [], |row| row.get(0))?;
    let spell_category: String = connection.query_row(
        "SELECT category FROM records WHERE record_key = 'spells:testSpell0001'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(pack_count, 2);
    assert_eq!(record_count, 3);
    assert_eq!(fts_count, 3);
    assert_eq!(spell_category, "spell");

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::create_dir_all(root.join("packs/spells"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "spells", "label": "Spells", "type": "Item", "path": "packs/spells" }
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
            "publication": { "title": "Player Core", "remaster": true },
            "description": { "value": "<p>You spend 10 minutes treating one injured living creature.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/demoralize.json"),
        r#"{
          "_id": "testAction0002",
          "name": "Demoralize",
          "type": "action",
          "system": {
            "traits": { "value": ["auditory", "concentrate", "emotion", "fear", "mental"] },
            "description": { "value": "<p>Use Intimidation to frighten a creature.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/spells/heal.json"),
        r#"{
          "_id": "testSpell0001",
          "name": "Heal",
          "type": "spell",
          "system": {
            "traits": { "value": ["healing", "vitality"] },
            "publication": { "title": "Player Core" },
            "description": { "value": "<p>You channel vital energy.</p>" }
          }
        }"#,
    )?;
    Ok(())
}

fn fixture_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-ingest-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}
