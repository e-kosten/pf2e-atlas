use std::fs;

use atlas_index::{ValidationStatus, validate_index};
use atlas_ingest::{BuildArtifactOptions, analyze_foundry_source, build_artifact};
use rusqlite::Connection;

mod fixture_sources;

use fixture_sources::{
    fixture_root, write_family_fixture_source, write_fixture_source,
    write_generated_affliction_fixture_source, write_remaster_fixture_source,
};

#[test]
fn loads_tolerant_foundry_source_and_normalizes_records() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("load");
    write_fixture_source(&root)?;

    let report = analyze_foundry_source(&root, None)?;

    assert!(
        report
            .source
            .source_signature
            .starts_with("foundry-pf2e:sha256:")
    );
    assert_eq!(
        report.source.source_signature.len(),
        "foundry-pf2e:sha256:".len() + 64
    );
    assert_eq!(report.pack_count, 4);
    assert_eq!(report.record_count, 5);
    assert_eq!(report.relationships.reference_edges, 2);
    assert_eq!(
        report.embeddings.pending_document_embeddings,
        report.default_visible_record_count
    );
    assert_eq!(report.skipped_record_count, 0);
    assert!(report.warnings.is_empty());

    let output_path = root.join("artifact.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
    })?;
    let connection = Connection::open(&output_path)?;
    let (
        treat_wounds_name,
        treat_wounds_normalized_name,
        treat_wounds_family,
        treat_wounds_record_type,
        treat_wounds_traits,
        treat_wounds_description,
    ): (String, String, String, String, String, String) = connection.query_row(
        "SELECT name, normalized_name, record_family, foundry_record_type, traits_json, description_text
         FROM records WHERE record_key = 'actions:testAction0001'",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;
    let (reference_to, reference_display, reference_text): (String, String, String) = connection
        .query_row(
            "SELECT to_record_key, display_text, reference_text
             FROM reference_edges WHERE from_record_key = 'actions:testAction0001'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
    let demoralize_reference_count: usize = connection.query_row(
        "SELECT COUNT(*)
         FROM reference_edges
         WHERE from_record_key = 'actions:testAction0002'
           AND to_record_key = 'spells:testSpell0001'
           AND display_text = 'Heal Spell'
           AND reference_text = '@UUID[Compendium.pf2e.spells.Item.Heal]{Heal Spell}'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(treat_wounds_name, "Treat Wounds");
    assert_eq!(treat_wounds_normalized_name, "treat wounds");
    assert_eq!(treat_wounds_family, "rule");
    assert_eq!(treat_wounds_record_type, "action");
    assert_eq!(treat_wounds_traits, "[\"exploration\",\"healing\"]");
    assert_eq!(
        treat_wounds_description,
        "You spend 10 minutes treating one injured living creature."
    );
    assert_eq!(reference_to, "spells:testSpell0001");
    assert_eq!(reference_display, "Heal");
    assert_eq!(
        reference_text,
        "@UUID[Compendium.pf2e.spells.Item.testSpell0001]{Heal}"
    );
    assert_eq!(demoralize_reference_count, 1);

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn source_signature_is_stable_and_changes_with_source() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("source-signature");
    write_fixture_source(&root)?;

    let first = analyze_foundry_source(&root, None)?.source.source_signature;
    let second = analyze_foundry_source(&root, None)?.source.source_signature;
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
    let third = analyze_foundry_source(&root, None)?.source.source_signature;
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

    let report = analyze_foundry_source(&root, None)?;

    assert_eq!(report.record_count, 5);
    assert_eq!(report.skipped_record_count, 2);
    assert!(
        report
            .skipped_records
            .iter()
            .any(|record| record.path.ends_with("broken-json.json")
                && record.reason.contains("source record failed to parse"))
    );
    assert!(
        report
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

    let output_path = root.join("artifact.sqlite");
    let report = build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
    })?;

    assert_eq!(report.pack_count, 1);
    assert_eq!(report.record_count, 1);
    assert!(report.warnings.is_empty());

    let connection = Connection::open(&output_path)?;
    let resolved_path: String = connection.query_row(
        "SELECT resolved_path FROM packs WHERE name = 'actions'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(
        resolved_path,
        root.join("packs/pf2e/actions").display().to_string()
    );

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn extracts_remaster_links_from_journals_and_migrations() -> Result<(), Box<dyn std::error::Error>>
{
    let root = fixture_root("remaster-links");
    write_remaster_fixture_source(&root)?;
    let report = analyze_foundry_source(&root, None)?;

    assert_eq!(report.record_count, 6);
    assert_eq!(report.relationships.record_aliases, 3);
    assert_eq!(report.relationships.remaster_links, 2);

    let output_path = root.join("artifact.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
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

    let report = analyze_foundry_source(&root, None)?;
    assert_eq!(report.skipped_record_count, 0);
    assert!(report.warnings.is_empty());
    assert_eq!(report.diagnostics["taxonomy"]["folder_records"], 1);
    assert_eq!(report.diagnostics["taxonomy"]["glossary_records"], 1);
    assert_eq!(report.diagnostics["variants"]["parenthetical_records"], 4);
    assert_eq!(report.diagnostics["variants"]["creature_blurb_records"], 1);
    assert_eq!(report.diagnostics["variants"]["exact_base_records"], 0);

    let output_path = root.join("artifact.sqlite");
    build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
    })?;
    let connection = Connection::open(&output_path)?;
    let bosun_families: String = connection.query_row(
        "SELECT taxonomy_families_json FROM records WHERE record_key = 'pathfinder-npc-core:bosun00000001'",
        [],
        |row| row.get(0),
    )?;
    let ghost_commoner_families: String = connection.query_row(
        "SELECT taxonomy_families_json FROM records WHERE record_key = 'bestiary:ghostCommoner1'",
        [],
        |row| row.get(0),
    )?;
    let storm_young_variant: (String, String, String) = connection.query_row(
        "SELECT variant_group_key, variant_label, variant_axes_json
         FROM records WHERE record_key = 'bestiary:stormYoung001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let venexus_variant: (String, String, String) = connection.query_row(
        "SELECT variant_group_key, variant_label, variant_axes_json
         FROM records WHERE record_key = 'adventure-bestiary:venexus000001'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let figurine_variant: (String, String) = connection.query_row(
        "SELECT variant_base_name, variant_label
         FROM records WHERE record_key = 'equipment:figurineBear1'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    assert_eq!(bosun_families, "[\"seafarer\"]");
    assert_eq!(ghost_commoner_families, "[\"ghost\"]");
    assert_eq!(storm_young_variant.0, "creature:family:storm-dragon");
    assert_eq!(storm_young_variant.1, "Young");
    assert_eq!(storm_young_variant.2, "[\"dragonAge\"]");
    assert_eq!(venexus_variant.0, "creature:family:storm-dragon");
    assert_eq!(venexus_variant.1, "Venexus");
    assert_eq!(venexus_variant.2, "[\"dragonAge\"]");
    assert_eq!(figurine_variant.0, "Wondrous Figurine");
    assert_eq!(figurine_variant.1, "Rubber Bear");

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn generates_affliction_records_from_staged_embedded_items()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("generated-afflictions");
    write_generated_affliction_fixture_source(&root)?;

    let output_path = root.join("artifact.sqlite");
    let report = build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
    })?;
    assert_eq!(report.diagnostics.generated_affliction_canonical_records, 1);
    assert_eq!(report.diagnostics.generated_affliction_instance_records, 1);
    assert_eq!(report.diagnostics.generated_affliction_reference_edges, 3);
    assert_eq!(report.source_record_count, 1);
    assert_eq!(report.artifact_record_count, 3);
    assert_eq!(report.generated_record_count, 2);
    assert_eq!(report.pending_document_embedding_count, 2);
    assert_eq!(report.document_embedding_count, 0);
    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(validation.source_record_count.as_deref(), Some("1"));
    assert_eq!(validation.artifact_record_count.as_deref(), Some("3"));
    assert_eq!(validation.generated_record_count.as_deref(), Some("2"));

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
    let ghoul_fever_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM records
         WHERE pack_name = 'derived-afflictions'
           AND name = 'Ghoul Fever'
           AND foundry_record_type = 'affliction'
           AND record_family = 'affliction'
           AND is_default_visible = 1",
        [],
        |row| row.get(0),
    )?;
    let serpent_dagger_count: usize = connection.query_row(
        "SELECT COUNT(*) FROM records
         WHERE pack_name = 'derived-afflictions'
           AND name = 'Serpent Dagger'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(generated_record_count, 2);
    assert_eq!(generated_fts_count, 1);
    assert_eq!(generated_edge_count, 3);
    assert_eq!(ghoul_fever_count, 1);
    assert_eq!(serpent_dagger_count, 0);

    drop(connection);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn writes_minimal_artifact_that_validate_index_accepts() -> Result<(), Box<dyn std::error::Error>> {
    let root = fixture_root("build");
    write_fixture_source(&root)?;
    let output_path = root.join("artifact.sqlite");

    let report = build_artifact(BuildArtifactOptions {
        source_root: root.clone(),
        output_path: output_path.clone(),
        manifest_path: None,
        embedding_cache_root: None,
    })?;

    assert_eq!(report.pack_count, 4);
    assert_eq!(report.record_count, 5);
    assert_eq!(report.source_record_count, 5);
    assert_eq!(report.artifact_record_count, 5);
    assert_eq!(report.generated_record_count, 0);
    assert_eq!(report.pending_document_embedding_count, 5);
    assert_eq!(report.document_embedding_count, 0);
    assert!(report.source_signature.starts_with("foundry-pf2e:sha256:"));
    assert!(report.skipped_records.is_empty());

    let validation = validate_index(&output_path)?;
    assert_eq!(validation.status, ValidationStatus::Ok);
    assert_eq!(
        validation.source_signature.as_deref(),
        Some(report.source_signature.as_str())
    );
    assert_eq!(validation.source_record_count.as_deref(), Some("5"));
    assert_eq!(validation.artifact_record_count.as_deref(), Some("5"));
    assert_eq!(validation.generated_record_count.as_deref(), Some("0"));

    let connection = Connection::open(&output_path)?;
    let pack_count: usize =
        connection.query_row("SELECT COUNT(*) FROM packs", [], |row| row.get(0))?;
    let record_count: usize =
        connection.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
    let document_embedding_count: usize =
        connection.query_row("SELECT COUNT(*) FROM document_embedding_cache", [], |row| {
            row.get(0)
        })?;
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
    assert_eq!(document_embedding_count, 0);
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
