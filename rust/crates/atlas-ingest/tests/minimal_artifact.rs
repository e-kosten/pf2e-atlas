use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::{RecordFamily, ValidationStatus};
use atlas_index::validate_index;
use atlas_ingest::{
    AliasSource, BuildArtifactOptions, build_minimal_artifact, load_foundry_source,
};
use rusqlite::Connection;

#[test]
fn loads_tolerant_foundry_source_and_normalizes_records() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("load");
    write_fixture_source(&root)?;

    let source = load_foundry_source(&root, None)?;

    assert!(source.source_signature.starts_with("foundry-pf2e:sha256:"));
    assert_eq!(
        source.source_signature.len(),
        "foundry-pf2e:sha256:".len() + 64
    );
    assert_eq!(source.packs.len(), 4);
    assert_eq!(source.records.len(), 5);
    assert_eq!(source.references.len(), 2);
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
    assert_eq!(
        source.references[0].from_record_key.to_string(),
        "actions:testAction0001"
    );
    assert_eq!(
        source.references[0].to_record_key.to_string(),
        "spells:testSpell0001"
    );
    assert_eq!(source.references[0].display_text.as_deref(), Some("Heal"));
    assert_eq!(
        source.references[0].reference_text,
        "@UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal}"
    );
    assert!(source.references.iter().any(|reference| {
        reference.from_record_key.to_string() == "actions:testAction0002"
            && reference.to_record_key.to_string() == "spells:testSpell0001"
            && reference.display_text.as_deref() == Some("Heal Spell")
            && reference.reference_text == "@UUID[Compendium.pf2e.spells.Item.Heal]{Heal Spell}"
    }));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn source_signature_is_stable_and_changes_with_source() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("source-signature");
    write_fixture_source(&root)?;

    let first = load_foundry_source(&root, None)?.source_signature;
    let second = load_foundry_source(&root, None)?.source_signature;
    assert_eq!(first, second);

    fs::write(
        root.join("packs/actions/demoralize.json"),
        r#"{
          "_id": "testAction0002",
          "name": "Demoralize",
          "type": "action",
          "system": {
            "traits": { "value": ["auditory", "concentrate", "emotion", "fear", "mental"] },
            "description": { "value": "<p>Use Intimidation to unsettle a creature.</p>" }
          }
        }"#,
    )?;
    let third = load_foundry_source(&root, None)?.source_signature;
    assert_ne!(first, third);

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
fn extracts_remaster_links_from_journals_and_migrations() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("remaster-links");
    write_remaster_fixture_source(&root)?;
    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.records.len(), 6);
    assert_eq!(source.aliases.len(), 3);
    assert_eq!(source.remaster_links.len(), 2);
    assert!(source.aliases.iter().any(|alias| {
        alias.canonical_record_key.to_string() == "actions:reactiveStrike1"
            && alias.alias_text == "Attack of Opportunity"
            && alias.normalized_alias == "attack of opportunity"
            && alias.source == AliasSource::RemasterJournal
            && alias.source_ref == "journal:Class Features"
    }));
    assert!(source.aliases.iter().any(|alias| {
        alias.canonical_record_key.to_string() == "conditionitems:offGuard1"
            && alias.alias_text == "flat-footed"
            && alias.source == AliasSource::Migration
            && alias.source_ref == "src/module/migration/migrations"
    }));
    assert!(source.aliases.iter().any(|alias| {
        alias.canonical_record_key.to_string() == "conditionitems:offGuard1"
            && alias.alias_text == "Legacy Guard"
            && alias.source == AliasSource::CompendiumSource
            && alias.source_ref == "bestiary:aliasCarrier1"
    }));
    assert!(source.remaster_links.iter().any(|link| {
        link.remaster_record_key.to_string() == "actions:reactiveStrike1"
            && link.legacy_record_key.to_string() == "actions:attackOpportunity1"
            && link.source_ref == "journal:Class Features"
    }));
    assert!(source.remaster_links.iter().any(|link| {
        link.remaster_record_key.to_string() == "conditionitems:offGuard1"
            && link.legacy_record_key.to_string() == "conditionitems:flatFooted1"
            && link.source_ref == "src/module/migration/migrations"
    }));

    let output_path = root.join("artifact.sqlite");
    build_minimal_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
    })?;
    let connection = Connection::open(&output_path)?;
    let remaster_link_count: usize =
        connection.query_row("SELECT COUNT(*) FROM remaster_links", [], |row| row.get(0))?;
    let alias_count: usize =
        connection.query_row("SELECT COUNT(*) FROM record_aliases", [], |row| row.get(0))?;
    let journal_link_source: String = connection.query_row(
        "SELECT source_kind FROM remaster_links
         WHERE remaster_record_key = 'actions:reactiveStrike1'
           AND legacy_record_key = 'actions:attackOpportunity1'",
        [],
        |row| row.get(0),
    )?;
    let compendium_alias_source: String = connection.query_row(
        "SELECT source_kind FROM record_aliases
         WHERE canonical_record_key = 'conditionitems:offGuard1'
           AND normalized_alias = 'legacy guard'",
        [],
        |row| row.get(0),
    )?;
    let attack_of_opportunity_visible: i64 = connection.query_row(
        "SELECT is_default_visible FROM records WHERE record_key = 'actions:attackOpportunity1'",
        [],
        |row| row.get(0),
    )?;
    let reactive_strike_visible: i64 = connection.query_row(
        "SELECT is_default_visible FROM records WHERE record_key = 'actions:reactiveStrike1'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(remaster_link_count, 2);
    assert_eq!(alias_count, 3);
    assert_eq!(journal_link_source, "remaster_journal");
    assert_eq!(compendium_alias_source, "compendium_source");
    assert_eq!(attack_of_opportunity_visible, 0);
    assert_eq!(reactive_strike_visible, 1);

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn populates_taxonomy_families_and_variant_groups() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("families");
    write_family_fixture_source(&root)?;

    let source = load_foundry_source(&root, None)?;
    assert!(source.skipped_records.is_empty());
    assert!(source.warnings.is_empty());
    assert_eq!(source.diagnostics.taxonomy_folder_records, 1);
    assert_eq!(source.diagnostics.taxonomy_glossary_records, 1);
    assert_eq!(source.diagnostics.variant_parenthetical_records, 4);
    assert_eq!(source.diagnostics.variant_creature_blurb_records, 1);
    assert_eq!(source.diagnostics.variant_exact_base_records, 0);
    let bosun = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "pathfinder-npc-core:bosun00000001")
        .expect("bosun should load");
    let ghost_commoner = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "bestiary:ghostCommoner1")
        .expect("ghost commoner should load");
    let storm_young = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "bestiary:stormYoung001")
        .expect("storm dragon should load");
    let venexus = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "adventure-bestiary:venexus000001")
        .expect("venexus should load");
    let figurine = source
        .records
        .iter()
        .find(|record| record.key.to_string() == "equipment:figurineBear1")
        .expect("figurine should load");

    assert_eq!(bosun.taxonomy_families, vec!["seafarer"]);
    assert_eq!(ghost_commoner.taxonomy_families, vec!["ghost"]);
    assert_eq!(
        storm_young.variant_group_key.as_deref(),
        Some("creature:family:storm-dragon")
    );
    assert_eq!(storm_young.variant_label.as_deref(), Some("Young"));
    assert_eq!(storm_young.variant_axes, vec!["dragonAge"]);
    assert_eq!(
        venexus.variant_group_key.as_deref(),
        Some("creature:family:storm-dragon")
    );
    assert_eq!(venexus.variant_label.as_deref(), Some("Venexus"));
    assert_eq!(
        figurine.variant_base_name.as_deref(),
        Some("Wondrous Figurine")
    );
    assert_eq!(figurine.variant_label.as_deref(), Some("Rubber Bear"));

    let output_path = root.join("artifact.sqlite");
    build_minimal_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
    })?;
    let connection = Connection::open(&output_path)?;
    let bosun_families: String = connection.query_row(
        "SELECT taxonomy_families_json FROM records WHERE record_key = 'pathfinder-npc-core:bosun00000001'",
        [],
        |row| row.get(0),
    )?;
    let venexus_variant: (String, String, String) = connection.query_row(
        "SELECT variant_group_key, variant_label, variant_axes_json
         FROM records WHERE record_key = 'adventure-bestiary:venexus000001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    assert_eq!(bosun_families, "[\"seafarer\"]");
    assert_eq!(venexus_variant.0, "creature:family:storm-dragon");
    assert_eq!(venexus_variant.1, "Venexus");
    assert_eq!(venexus_variant.2, "[\"dragonAge\"]");

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn generates_affliction_records_from_staged_embedded_items()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("generated-afflictions");
    write_generated_affliction_fixture_source(&root)?;

    let source = load_foundry_source(&root, None)?;

    assert_eq!(source.diagnostics.generated_affliction_canonical_records, 1);
    assert_eq!(source.diagnostics.generated_affliction_instance_records, 1);
    assert_eq!(source.diagnostics.generated_affliction_reference_edges, 3);
    assert!(
        source
            .records
            .iter()
            .any(|record| record.pack_name.as_str() == "derived-afflictions"
                && record.name == "Ghoul Fever"
                && record.foundry_record_type == "affliction"
                && record.record_family == RecordFamily::Affliction
                && record.is_default_visible)
    );
    assert!(
        !source
            .records
            .iter()
            .any(|record| record.pack_name.as_str() == "derived-afflictions"
                && record.name == "Serpent Dagger")
    );

    let output_path = root.join("artifact.sqlite");
    build_minimal_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
    })?;
    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);

    let connection = Connection::open(&output_path)?;
    let generated_record_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM records WHERE pack_name IN ('derived-afflictions', 'derived-affliction-instances')",
        [],
        |row| row.get(0),
    )?;
    let generated_fts_count: usize = connection.query_row(
        "SELECT COUNT(*)
         FROM records_fts
         WHERE record_key LIKE 'derived-afflictions:%'
            OR record_key LIKE 'derived-affliction-instances:%'",
        [],
        |row| row.get(0),
    )?;
    let generated_edge_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM reference_edges WHERE reference_text LIKE 'derived-affliction-%'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(generated_record_count, 2);
    assert_eq!(generated_fts_count, 1);
    assert_eq!(generated_edge_count, 3);

    drop(connection);
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
    assert!(report.source_signature.starts_with("foundry-pf2e:sha256:"));
    assert!(report.skipped_records.is_empty());

    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(
        validation.source_signature.as_deref(),
        Some(report.source_signature.as_str())
    );
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
    let (spell_level, spell_rarity, spell_publication_family): (i64, String, String) = connection
        .query_row(
            "SELECT level, rarity, publication_family FROM records WHERE record_key = 'spells:testSpell0001'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
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
    let actor_side_count: usize =
        connection.query_row("SELECT COUNT(*) FROM actor_records", [], |row| row.get(0))?;
    let item_side_count: usize =
        connection.query_row("SELECT COUNT(*) FROM item_records", [], |row| row.get(0))?;
    let spell_side_count: usize =
        connection.query_row("SELECT COUNT(*) FROM spell_records", [], |row| row.get(0))?;
    let reference_edge_count: usize =
        connection.query_row("SELECT COUNT(*) FROM reference_edges", [], |row| row.get(0))?;
    let (reference_to, reference_display, reference_text): (String, String, String) = connection
        .query_row(
            "SELECT to_record_key, display_text, reference_text
             FROM reference_edges WHERE from_record_key = 'actions:testAction0001'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
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
    let (actor_size, actor_languages, actor_speed_types, actor_senses): (
        String,
        String,
        String,
        String,
    ) = connection.query_row(
        "SELECT size, languages_json, speed_types_json, senses_json
         FROM actor_records WHERE record_key = 'bestiary:testActor0001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    let (item_group, item_bulk, item_hands, item_damage_types): (String, f64, String, String) =
        connection.query_row(
            "SELECT system_group, bulk_value, hands_requirement, damage_types_json
             FROM item_records WHERE record_key = 'equipment:testWeapon0001'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
    let (
        spell_traditions,
        spell_kinds,
        spell_range_text,
        spell_range_value,
        spell_target_text,
        spell_save_type,
        spell_basic_save,
        spell_damage_types,
    ): (String, String, String, f64, String, String, i64, String) = connection.query_row(
        "SELECT traditions_json, spell_kinds_json, range_text, range_value, target_text, save_type, basic_save, damage_types_json
         FROM spell_records WHERE record_key = 'spells:testSpell0001'",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        },
    )?;

    assert_eq!(pack_count, 4);
    assert_eq!(record_count, 5);
    assert_eq!(fts_count, 5);
    assert_eq!(spell_record_family, "spell");
    assert_eq!(spell_level, 1);
    assert_eq!(spell_rarity, "common");
    assert_eq!(spell_publication_family, "core");
    assert_eq!(trait_count, 13);
    assert_eq!(metric_count, 20);
    assert!(metric_key_catalog_count >= 20);
    assert!(metric_value_catalog_count >= 3);
    assert_eq!(actor_side_count, 1);
    assert_eq!(item_side_count, 4);
    assert_eq!(spell_side_count, 1);
    assert_eq!(reference_edge_count, 2);
    assert_eq!(reference_to, "spells:testSpell0001");
    assert_eq!(reference_display, "Heal");
    assert_eq!(
        reference_text,
        "@UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal}"
    );
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
    assert_eq!(actor_size, "small");
    assert_eq!(actor_languages, "[\"goblin\"]");
    assert_eq!(actor_speed_types, "[\"climb\",\"land\"]");
    assert_eq!(actor_senses, "[\"darkvision\"]");
    assert_eq!(item_group, "bow");
    assert_eq!(item_bulk, 2.0);
    assert_eq!(item_hands, "two_hands");
    assert_eq!(item_damage_types, "[\"piercing\"]");
    assert_eq!(spell_traditions, "[\"divine\",\"primal\"]");
    assert_eq!(spell_kinds, "[\"cantrip\"]");
    assert_eq!(spell_range_text, "30 feet");
    assert_eq!(spell_range_value, 30.0);
    assert_eq!(spell_target_text, "1 willing creature");
    assert_eq!(spell_save_type, "fortitude");
    assert_eq!(spell_basic_save, 1);
    assert_eq!(spell_damage_types, "[\"vitality\"]");

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

fn write_generated_affliction_fixture_source(
    root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
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
        root.join("packs/bestiary/ghoul.json"),
        r#"{
          "_id": "testGhoul0001",
          "name": "Test Ghoul",
          "type": "npc",
          "items": [
            {
              "_id": "ghoulFeverItem",
              "name": "Ghoul Fever",
              "type": "action",
              "system": {
                "category": "offensive",
                "description": {
                  "value": "<p><strong>Saving Throw</strong> @Check[fortitude|dc:18]</p><p><strong>Stage 1</strong> @UUID[Compendium.pf2e.conditionitems.Item.Sickened]{Sickened 1} (1 day)</p>"
                },
                "traits": { "value": ["disease"] }
              }
            },
            {
              "_id": "serpentDaggerItem",
              "name": "Serpent Dagger",
              "type": "weapon",
              "system": {
                "category": "simple",
                "description": {
                  "value": "<p>Dagger Venom (poison) <strong>Saving Throw</strong> @Check[fortitude|dc:21]</p><p><strong>Stage 1</strong> @Damage[1d8[poison]] damage</p>"
                },
                "traits": { "value": ["agile", "poison"] }
              }
            }
          ],
          "system": {
            "details": {
              "level": { "value": 2 },
              "publication": { "title": "Pathfinder Monster Core" }
            },
            "traits": { "rarity": "common", "value": ["undead"] }
          }
        }"#,
    )?;
    Ok(())
}

fn write_family_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/pathfinder-npc-core"))?;
    fs::create_dir_all(root.join("packs/bestiary-family-ability-glossary/ghost"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/adventure-bestiary"))?;
    fs::create_dir_all(root.join("packs/equipment"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "pathfinder-npc-core", "label": "NPC Core", "type": "Actor", "path": "packs/pathfinder-npc-core" },
            { "name": "bestiary-family-ability-glossary", "label": "Family Abilities", "type": "Item", "path": "packs/bestiary-family-ability-glossary" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "adventure-bestiary", "label": "Adventure Bestiary", "type": "Actor", "path": "packs/adventure-bestiary" },
            { "name": "equipment", "label": "Equipment", "type": "Item", "path": "packs/equipment" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/pathfinder-npc-core/_folders.json"),
        r#"[
          { "_id": "folderSeafarer", "name": "Seafarer", "folder": null }
        ]"#,
    )?;
    fs::write(
        root.join("packs/pathfinder-npc-core/bosun.json"),
        r#"{
          "_id": "bosun00000001",
          "name": "Bosun",
          "type": "npc",
          "folder": "folderSeafarer",
          "system": {
            "traits": { "value": ["human", "humanoid"], "rarity": "common" },
            "description": { "value": "<p>A ship officer.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary-family-ability-glossary/ghost/frightful-moans.json"),
        r#"{
          "_id": "ghostAbility01",
          "name": "Frightful Moans",
          "type": "action",
          "system": {
            "description": { "value": "<p>A ghost family ability.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/ghost-commoner.json"),
        r#"{
          "_id": "ghostCommoner1",
          "name": "Ghost Commoner",
          "type": "npc",
          "system": {
            "traits": { "value": ["ghost", "undead"] },
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.bestiary-family-ability-glossary.Item.ghostAbility01]{Frightful Moans}"
              }
            ],
            "description": { "value": "<p>A ghostly commoner.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/storm-dragon-young.json"),
        r#"{
          "_id": "stormYoung001",
          "name": "Storm Dragon (Young)",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"] },
            "description": { "value": "<p>A young storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/storm-dragon-adult.json"),
        r#"{
          "_id": "stormAdult01",
          "name": "Storm Dragon (Adult)",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"] },
            "description": { "value": "<p>An adult storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/adventure-bestiary/venexus.json"),
        r#"{
          "_id": "venexus000001",
          "name": "Venexus",
          "type": "npc",
          "system": {
            "traits": { "value": ["dragon", "electricity"], "rarity": "unique" },
            "details": { "blurb": "Female young storm dragon" },
            "description": { "value": "<p>A named storm dragon.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/wondrous-figurine-rubber-bear.json"),
        r#"{
          "_id": "figurineBear1",
          "name": "Wondrous Figurine (Rubber Bear)",
          "type": "equipment",
          "system": {
            "description": { "value": "<p>A rubber bear figurine.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/equipment/wondrous-figurine-golden-lions.json"),
        r#"{
          "_id": "figurineLions1",
          "name": "Wondrous Figurine (Golden Lions)",
          "type": "equipment",
          "system": {
            "description": { "value": "<p>A golden lions figurine.</p>" }
          }
        }"#,
    )?;
    Ok(())
}

fn write_remaster_fixture_source(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("packs/actions"))?;
    fs::create_dir_all(root.join("packs/conditionitems"))?;
    fs::create_dir_all(root.join("packs/bestiary"))?;
    fs::create_dir_all(root.join("packs/journals"))?;
    fs::create_dir_all(root.join("src/module/migration/migrations"))?;
    fs::write(
        root.join("module.json"),
        r#"{
          "packs": [
            { "name": "actions", "label": "Actions", "type": "Item", "path": "packs/actions" },
            { "name": "conditionitems", "label": "Conditions", "type": "Item", "path": "packs/conditionitems" },
            { "name": "bestiary", "label": "Bestiary", "type": "Actor", "path": "packs/bestiary" },
            { "name": "journals", "label": "Journals", "type": "JournalEntry", "path": "packs/journals" }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/reactive-strike.json"),
        r#"{
          "_id": "reactiveStrike1",
          "name": "Reactive Strike",
          "type": "action",
          "system": {
            "publication": { "title": "Player Core", "remaster": true },
            "description": { "value": "<p>Strike as a reaction.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/actions/attack-of-opportunity.json"),
        r#"{
          "_id": "attackOpportunity1",
          "name": "Attack of Opportunity",
          "type": "action",
          "system": {
            "publication": { "title": "Core Rulebook", "remaster": false },
            "description": { "value": "<p>Legacy reaction.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/conditionitems/off-guard.json"),
        r#"{
          "_id": "offGuard1",
          "name": "Off-Guard",
          "type": "condition",
          "system": {
            "publication": { "title": "Player Core", "remaster": true },
            "description": { "value": "<p>You are distracted.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/conditionitems/flat-footed.json"),
        r#"{
          "_id": "flatFooted1",
          "name": "flat-footed",
          "type": "condition",
          "system": {
            "publication": { "title": "Core Rulebook", "remaster": false },
            "description": { "value": "<p>You are distracted.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("packs/journals/remaster-changes.json"),
        r#"{
          "_id": "journal1",
          "name": "Remaster Changes",
          "pages": [
            {
              "_id": "page1",
              "name": "Class Features",
              "type": "text",
              "text": {
                "content": "<table><tbody><tr><td>Attack of Opportunity</td><td>Multiple</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr></tbody></table>",
                "format": 1
              }
            }
          ]
        }"#,
    )?;
    fs::write(
        root.join("packs/bestiary/alias-carrier.json"),
        r#"{
          "_id": "aliasCarrier1",
          "name": "Alias Carrier",
          "type": "npc",
          "items": [
            {
              "_id": "embeddedLegacy1",
              "name": "Legacy Guard",
              "type": "condition",
              "_stats": {
                "compendiumSource": "Compendium.pf2e.conditionitems.Item.offGuard1"
              },
              "system": {
                "publication": { "title": "Core Rulebook", "remaster": false }
              }
            }
          ],
          "system": {
            "publication": { "title": "Bestiary", "remaster": false },
            "traits": { "value": ["humanoid"] },
            "description": { "value": "<p>Fixture carrier.</p>" }
          }
        }"#,
    )?;
    fs::write(
        root.join("src/module/migration/migrations/850-flat-footed-to-off-guard.ts"),
        r#"/* Rename all uses and mentions of "flat-footed" to "Off-Guard" */"#,
    )?;
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
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal} @UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal} @UUID[Compendium.pf2e.spells.Item.missingSpell]{Missing}"
              }
            ],
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
            "rules": [
              {
                "key": "Note",
                "text": "@UUID[Compendium.pf2e.spells.Item.Heal]{Heal Spell}"
              }
            ],
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
            "level": { "value": 1 },
            "traits": { "value": ["healing", "vitality", "cantrip"], "traditions": ["divine", "primal"], "rarity": "common" },
            "time": { "value": "1 minute" },
            "duration": { "value": "10 minutes" },
            "range": { "value": "30 feet" },
            "target": { "value": "<p>1 willing creature</p>" },
            "defense": { "save": { "statistic": "fortitude", "basic": true } },
            "damage": { "main": { "type": "vitality" } },
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
            "traits": { "value": ["goblin", "humanoid"], "size": { "value": "small" } },
            "details": { "languages": { "value": ["goblin"] } },
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
            "group": "bow",
            "usage": { "value": "held-in-two-hands" },
            "bulk": { "value": 2 },
            "range": { "increment": 100 },
            "reload": { "value": 0 },
            "damage": { "dice": 1, "die": "d8", "damageType": "piercing" },
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
