use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::{RecordFamily, ValidationStatus};
use atlas_index::validate_index;
use atlas_ingest::{BuildArtifactOptions, build_minimal_artifact, load_foundry_source};
use rusqlite::Connection;

#[test]
fn loads_tolerant_foundry_source_and_normalizes_records() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("load");
    write_fixture_source(&root)?;

    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.packs.len(), 4);
    assert_eq!(source.records.len(), 5);
    assert!(source.skipped_records.is_empty());
    assert!(source.warnings.is_empty());

    let treat_wounds = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "actions:testAction0001")
        .expect("action record should load");
    assert_eq!(treat_wounds.name, "Treat Wounds");
    assert_eq!(treat_wounds.normalized_name, "treat wounds");
    assert_eq!(treat_wounds.record_family, RecordFamily::Rule);
    assert_eq!(treat_wounds.foundry_record_type, "action");
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

    assert_eq!(source.records.len(), 5);
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

    assert_eq!(report.pack_count, 4);
    assert_eq!(report.record_count, 5);
    assert!(report.skipped_records.is_empty());

    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(validation.source_record_count.as_deref(), Some("5"));

    let connection = Connection::open(&output_path)?;
    let pack_count: usize =
        connection.query_row("SELECT COUNT(*) FROM packs", [], |row| row.get(0))?;
    let record_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
    let fts_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records_fts", [], |row| row.get(0))?;
    let spell_record_family: String = connection.query_row(
        "SELECT record_family FROM records WHERE record_key = 'spells:testSpell0001'",
        [],
        |row| row.get(0),
    )?;
    let trait_count: usize =
        connection.query_row("SELECT COUNT(*) FROM record_traits", [], |row| row.get(0))?;
    let metric_count: usize =
        connection.query_row("SELECT COUNT(*) FROM record_metrics", [], |row| row.get(0))?;
    let metric_key_catalog_count: usize =
        connection.query_row("SELECT COUNT(*) FROM metric_key_catalog", [], |row| {
            row.get(0)
        })?;
    let metric_value_catalog_count: usize =
        connection.query_row("SELECT COUNT(*) FROM metric_value_catalog", [], |row| {
            row.get(0)
        })?;
    let (
        action_count,
        action_price,
        action_usage,
        action_activation_kind,
        action_activation_actions,
    ): (i64, i64, String, String, i64) = connection.query_row(
        "SELECT system_actions_value, price_cp, system_usage, activation_time_kind, activation_time_actions
         FROM records WHERE record_key = 'actions:testAction0001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    )?;
    let (spell_time, spell_activation_kind, spell_activation_duration_value, spell_activation_duration_unit, spell_duration, spell_duration_unit): (
        String,
        String,
        i64,
        String,
        i64,
        String,
    ) = connection.query_row(
        "SELECT system_time_value, activation_time_kind, activation_time_duration_value, activation_time_duration_unit, duration_value, duration_unit
         FROM records WHERE record_key = 'spells:testSpell0001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    )?;
    let goblin_ac: f64 = connection.query_row(
        "SELECT number_value FROM record_metrics
         WHERE record_key = 'bestiary:testActor0001'
           AND metric_domain = 'actor'
           AND metric_key = 'ac.value'",
        [],
        |row| row.get(0),
    )?;
    let goblin_best_save: String = connection.query_row(
        "SELECT text_value FROM record_metrics
         WHERE record_key = 'bestiary:testActor0001'
           AND metric_domain = 'actor'
           AND metric_key = 'save.best'",
        [],
        |row| row.get(0),
    )?;
    let weapon_damage_faces: f64 = connection.query_row(
        "SELECT number_value FROM record_metrics
         WHERE record_key = 'equipment:testWeapon0001'
           AND metric_domain = 'item'
           AND metric_key = 'weapon.damage_die_faces'",
        [],
        |row| row.get(0),
    )?;
    let actor_catalog_count: usize = connection.query_row(
        "SELECT catalog_count FROM metric_key_catalog
         WHERE metric_domain = 'actor'
           AND record_family = 'creature'
           AND metric_key = 'ac.value'",
        [],
        |row| row.get(0),
    )?;
    let save_best_catalog_value: String = connection.query_row(
        "SELECT value FROM metric_value_catalog
         WHERE metric_domain = 'actor'
           AND record_family = 'creature'
           AND metric_key = 'save.best'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(pack_count, 4);
    assert_eq!(record_count, 5);
    assert_eq!(fts_count, 5);
    assert_eq!(spell_record_family, "spell");
    assert_eq!(trait_count, 12);
    assert_eq!(metric_count, 20);
    assert!(metric_key_catalog_count >= 20);
    assert!(metric_value_catalog_count >= 3);
    assert_eq!(action_count, 1);
    assert_eq!(action_price, 30);
    assert_eq!(action_usage, "held-in-one-hand");
    assert_eq!(action_activation_kind, "actions");
    assert_eq!(action_activation_actions, 1);
    assert_eq!(spell_time, "1 minute");
    assert_eq!(spell_activation_kind, "duration");
    assert_eq!(spell_activation_duration_value, 1);
    assert_eq!(spell_activation_duration_unit, "minute");
    assert_eq!(spell_duration, 10);
    assert_eq!(spell_duration_unit, "minute");
    assert_eq!(goblin_ac, 17.0);
    assert_eq!(goblin_best_save, "ref");
    assert_eq!(weapon_damage_faces, 8.0);
    assert_eq!(actor_catalog_count, 1);
    assert_eq!(save_best_catalog_value, "ref");

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

fn write_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::create_dir_all(root.join("packs/spells"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/equipment"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "spells", "label": "Spells", "type": "Item", "path": "packs/spells" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "equipment", "label": "Equipment", "type": "Item", "path": "packs/equipment" }
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
            "actions": { "value": 1 },
            "usage": { "value": "held-in-one-hand" },
            "price": { "value": { "sp": 3 } },
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
            "time": { "value": "1 minute" },
            "duration": { "value": "10 minutes" },
            "publication": { "title": "Player Core" },
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
            "traits": { "value": ["goblin", "humanoid"] },
            "abilities": { "dex": { "mod": 4 } },
            "perception": { "mod": 7, "senses": [{ "type": "darkvision", "range": 60 }] },
            "attributes": {
              "ac": { "value": 17 },
              "hp": { "value": 16, "max": 16 },
              "speed": { "value": 25, "otherSpeeds": [{ "type": "climb", "value": 10 }] }
            },
            "saves": {
              "fortitude": { "mod": 5 },
              "reflex": { "mod": 8 },
              "will": { "mod": 4 }
            },
            "skills": {
              "stealth": { "mod": 9, "rank": 1 }
            },
            "description": { "value": "<p>A small scout.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/longbow.json"),
        r#"{
          "_id": "testWeapon0001",
          "name": "Longbow",
          "type": "weapon",
          "system": {
            "traits": { "value": ["deadly-d10"] },
            "range": { "increment": 100 },
            "reload": { "value": 0 },
            "damage": { "dice": 1, "die": "d8" },
            "description": { "value": "<p>A ranged weapon.</p>" }
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
