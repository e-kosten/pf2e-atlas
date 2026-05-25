use std::fs;

use atlas_domain::RecordKey;
use rusqlite::Connection;

use super::{create_contract_database, temp_db_path};
use crate::{RecordLoadOptions, SqliteIndexReader};

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
        "INSERT INTO record_content (
           record_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction1', 0, 'description', 'public', 1, 1, NULL,
           '{\"blocks\":[]}'
         )",
        [],
    )?;
    connection.execute(
        "INSERT INTO record_content (
           record_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction2', 0, 'description', 'public', 1, 1, NULL,
           'not json'
         )",
        [],
    )?;
    drop(connection);

    let records = SqliteIndexReader::open_read_only(&path)?
        .load_records_by_key(&[RecordKey::parse("actions:testAction1")?])?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].supplemental_content.len(), 1);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_records_by_key_can_omit_raw_source_json() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("load-records-by-key-without-raw");
    create_contract_database(&path)?;

    let records = SqliteIndexReader::open_read_only(&path)?.load_records_by_key_with_options(
        &[RecordKey::parse("actions:testAction1")?],
        RecordLoadOptions::omit_raw_json(),
    )?;

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].key.to_string(), "actions:testAction1");
    assert_eq!(records[0].raw_json, "");
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
        "INSERT INTO record_content (
           record_key, ordinal, source_kind, visibility, contributes_to_search,
           contributes_to_references, label, content_json
         ) VALUES (
           'actions:testAction1', 0, 'description', 'public', 1, 1, NULL,
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
