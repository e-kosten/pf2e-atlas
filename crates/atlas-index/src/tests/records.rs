use std::fs;

use atlas_domain::{RecordFamily, RecordKey};
use rusqlite::Connection;

use super::{create_contract_database, temp_db_path};
use crate::SqliteIndexReader;

#[test]
fn loads_persisted_records_from_artifact_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records");
    create_contract_database(&path)?;

    let records = SqliteIndexReader::open_read_only(&path)?.load_records()?;

    assert_eq!(records.len(), 3);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].record_family.as_str(), "rule");
    assert_eq!(records[0].pack_name.as_str(), "actions");
    assert_eq!(records[0].traits, Vec::<String>::new());
    assert!(records[0].is_default_visible);
    assert_eq!(records[0].source_path, "packs/actions/test-action-1.json");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn loads_persisted_records_by_key_scopes_detail_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records-by-key-scoped");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO record_metrics (
           record_key, metric_domain, metric_key, value_type, number_value
         ) VALUES (
           'actions:testAction1', 'actor', 'level', 'number', 2.0
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_metrics (
           record_key, metric_domain, metric_key, value_type
         ) VALUES (
           'actions:testAction2', 'actor', 'level', 'number'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO actor_records (
           record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
           resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
         ) VALUES (
           'actions:testAction1', 'med', '[\"Common\"]', '[]', '[]', '[]', '[]', '[]',
           NULL, '[]', 0
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO actor_records (
           record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
           resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
         ) VALUES (
           'actions:testAction2', 'med', 'not json', '[]', '[]', '[]', '[]', '[]',
           NULL, '[]', 0
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO item_records (
           record_key, system_category, system_base_item, system_group, system_usage, price_cp,
           bulk_value, hands_requirement, damage_types_json
         ) VALUES (
           'actions:testAction1', 'weapon', NULL, 'sword', NULL, 100, 1.0, NULL, '[\"slashing\"]'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO item_records (
           record_key, system_category, system_base_item, system_group, system_usage, price_cp,
           bulk_value, hands_requirement, damage_types_json
         ) VALUES (
           'actions:testAction2', 'weapon', NULL, 'sword', NULL, 100, 1.0, NULL, 'not json'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO spell_records (
           record_key, traditions_json, spell_kinds_json, range_text, range_value, target_text,
           area_type, area_value, save_type, sustained, basic_save, damage_types_json
         ) VALUES (
           'actions:testAction1', '[\"arcane\"]', '[\"spell\"]', '30 feet', 30.0, NULL,
           NULL, NULL, 'will', 0, 1, '[\"mental\"]'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO spell_records (
           record_key, traditions_json, spell_kinds_json, range_text, range_value, target_text,
           area_type, area_value, save_type, sustained, basic_save, damage_types_json
         ) VALUES (
           'actions:testAction2', 'not json', '[\"spell\"]', '30 feet', 30.0, NULL,
           NULL, NULL, 'will', 0, 1, '[]'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, content_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction1', 'content:0', 0, 'description', 'public', 1, 1, NULL,
           '{\"blocks\":[]}'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, content_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction2', 'content:0', 0, 'description', 'public', 1, 1, NULL,
           'not json'
         )",
        [],
    )?;
    drop(connection);

    let records = SqliteIndexReader::open_read_only(&path)?
        .load_records_by_key(&[RecordKey::parse("actions:testAction1")?])?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].metrics.len(), 1);
    assert!(records[0].actor_data.is_some());
    assert!(records[0].item_data.is_some());
    assert!(records[0].spell_data.is_some());
    assert_eq!(records[0].supplemental_content.len(), 1);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn loads_search_candidate_records_without_detail_hydration()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-search-candidates");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET traits_json = '[\"attack\",\"flourish\"]',
             taxonomy_families_json = '[\"action\"]',
             system_category = 'skill',
             system_group = 'athletics'
         WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, content_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction1', 'content:0', 0, 'description', 'public', 1, 1, NULL,
           'not json'
         )",
        [],
    )?;
    drop(connection);

    let candidates = SqliteIndexReader::open_read_only(&path)?
        .load_search_candidate_records(&[RecordKey::parse("actions:testAction1")?])?;

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].key.to_string(), "actions:testAction1");
    assert_eq!(candidates[0].name, "Test Action 1");
    assert_eq!(candidates[0].record_family, RecordFamily::Rule);
    assert_eq!(candidates[0].foundry_record_type, "action");
    assert_eq!(candidates[0].traits, vec!["attack", "flourish"]);
    assert_eq!(candidates[0].taxonomy_families, vec!["action"]);
    assert_eq!(candidates[0].system_category.as_deref(), Some("skill"));
    assert_eq!(candidates[0].system_group.as_deref(), Some("athletics"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_search_candidate_records_rejects_invalid_json() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-search-candidates-invalid-json");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET traits_json = 'not json' WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let error = SqliteIndexReader::open_read_only(&path)?
        .load_search_candidate_records(&[RecordKey::parse("actions:testAction1")?])
        .expect_err("invalid candidate JSON should be rejected");

    assert!(error.to_string().contains("records.traits_json"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_search_candidate_records_rejects_invalid_family() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("load-search-candidates-invalid-family");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET record_family = 'not-a-family' WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let error = SqliteIndexReader::open_read_only(&path)?
        .load_search_candidate_records(&[RecordKey::parse("actions:testAction1")?])
        .expect_err("invalid candidate family should be rejected");

    assert!(error.to_string().contains("record_family"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn loads_persisted_record_set_relationship_tables() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-record-set");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO reference_edges (from_record_key, to_record_key, display_text, reference_text, source_kind, visibility)
             VALUES ('actions:testAction1', 'actions:testAction2', 'Test Action 2', 'Compendium.pf2e.actions.Item.testAction2', 'description', 'public')",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
             VALUES ('actions:testAction1', 'Test Alias', 'test alias', 'compendium_source', 'fixture')",
        [],
    )?;
    connection.execute(
        "INSERT INTO remaster_links (remaster_record_key, legacy_record_key, source_kind, source_ref)
             VALUES ('actions:testAction1', 'actions:testAction3', 'migration', 'fixture')",
        [],
    )?;
    drop(connection);

    let record_set = SqliteIndexReader::open_read_only(&path)?.load_record_set()?;

    assert_eq!(record_set.records.len(), 3);
    assert_eq!(record_set.reference_edges.len(), 1);
    assert_eq!(record_set.aliases.len(), 1);
    assert_eq!(record_set.remaster_links.len(), 1);
    assert_eq!(
        record_set.reference_edges[0].to_record_key.to_string(),
        "actions:testAction2"
    );
    assert_eq!(record_set.aliases[0].source.as_str(), "compendium_source");
    assert_eq!(record_set.remaster_links[0].source.as_str(), "migration");
    fs::remove_file(path)?;
    Ok(())
}
