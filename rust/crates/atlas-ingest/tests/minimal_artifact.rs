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
    assert!(source.warnings.is_empty());
    assert!(source.source_signature.starts_with("foundry-pf2e:"));
    assert_eq!(source.source_signature.len(), 77);

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
    assert!(report.source_signature.starts_with("foundry-pf2e:"));

    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(validation.source_record_count.as_deref(), Some("3"));
    assert_eq!(
        validation.source_signature.as_deref(),
        Some(report.source_signature.as_str())
    );

    let connection = Connection::open(&output_path)?;
    let pack_count: usize =
        connection.query_row("SELECT COUNT(*) FROM packs", [], |row| row.get(0))?;
    let record_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
    let fts_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records_fts", [], |row| row.get(0))?;
    let trait_count: usize =
        connection.query_row("SELECT COUNT(*) FROM record_traits", [], |row| row.get(0))?;
    let spell_category: String = connection.query_row(
        "SELECT category FROM records WHERE record_key = 'spells:testSpell0001'",
        [],
        |row| row.get(0),
    )?;
    let treat_wounds_healing_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM record_traits WHERE record_key = 'actions:testAction0001' AND trait = 'healing'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(pack_count, 2);
    assert_eq!(record_count, 3);
    assert_eq!(fts_count, 3);
    assert_eq!(trait_count, 9);
    assert_eq!(spell_category, "spell");
    assert_eq!(treat_wounds_healing_count, 1);

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
