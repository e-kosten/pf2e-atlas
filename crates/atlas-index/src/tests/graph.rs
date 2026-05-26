use std::fs;

use atlas_domain::RecordKey;

use super::{create_contract_database, insert_reference_edge, temp_db_path};
use crate::{GraphProductIndex, ReferenceEdgeDirection, SqliteIndexReader};

#[test]
fn reference_edges_for_seed_returns_policy_visible_outgoing_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-outgoing");
    create_contract_database(&path)?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction2",
        Some("Visible"),
        "visible-ref",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction3",
        Some("Private"),
        "private-ref",
        "private_notes",
        "private",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction1",
        "actions:testAction3",
        Some("Embedded"),
        "embedded-ref",
        "embedded_item_description",
        "public",
    )?;

    let index = SqliteIndexReader::open_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Outgoing,
    )?;

    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].to_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(edges[0].display_text.as_deref(), Some("Visible"));
    assert_eq!(edges[0].reference_text, "visible-ref");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reference_edges_for_seed_returns_policy_visible_backlinks()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-backlinks");
    create_contract_database(&path)?;
    insert_reference_edge(
        &path,
        "actions:testAction2",
        "actions:testAction1",
        Some("Incoming"),
        "incoming-ref",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction3",
        "actions:testAction1",
        Some("Private"),
        "private-ref",
        "private_notes",
        "private",
    )?;

    let index = SqliteIndexReader::open_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Backlink,
    )?;

    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].from_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(edges[0].reference_text, "incoming-ref");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn variant_group_for_record_returns_ordered_siblings() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variants");
    create_contract_database(&path)?;
    let connection = rusqlite::Connection::open(&path)?;
    for (record_key, level, label) in [
        ("actions:testAction1", 1_i64, "Lesser"),
        ("actions:testAction2", 3_i64, "Moderate"),
        ("actions:testAction3", 5_i64, "Greater"),
    ] {
        connection.execute(
            "UPDATE records
             SET variant_group_key = 'test-action',
                 variant_base_name = 'Test Action',
                 variant_label = ?1,
                 variant_axes_json = '[\"grade\"]',
                 variant_confidence = 1.0,
                 variant_source = 'test',
                 level = ?2
             WHERE record_key = ?3",
            (label, level, record_key),
        )?;
    }
    drop(connection);

    let result = SqliteIndexReader::open_read_only(&path)?
        .variant_group_for_record(&RecordKey::parse("actions:testAction2")?)?
        .expect("seed should exist");

    assert_eq!(result.variant_group_key.as_deref(), Some("test-action"));
    assert_eq!(result.records.len(), 3);
    assert_eq!(result.records[0].key.to_string(), "actions:testAction1");
    assert_eq!(result.records[0].variant_label.as_deref(), Some("Lesser"));
    assert_eq!(result.records[0].variant_axes, vec!["grade"]);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn variant_groups_by_base_name_returns_matching_groups() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-base");
    create_contract_database(&path)?;
    let connection = rusqlite::Connection::open(&path)?;
    for (record_key, level, label) in [
        ("actions:testAction1", 1_i64, "Lesser"),
        ("actions:testAction2", 3_i64, "Moderate"),
        ("actions:testAction3", 5_i64, "Greater"),
    ] {
        connection.execute(
            "UPDATE records
             SET variant_group_key = 'test-action',
                 variant_base_name = 'Test Action',
                 variant_label = ?1,
                 variant_axes_json = '[\"grade\"]',
                 variant_confidence = 1.0,
                 variant_source = 'test',
                 level = ?2
             WHERE record_key = ?3",
            (label, level, record_key),
        )?;
    }
    drop(connection);

    let groups =
        SqliteIndexReader::open_read_only(&path)?.variant_groups_by_base_name("test action")?;

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].seed, None);
    assert_eq!(groups[0].variant_group_key.as_deref(), Some("test-action"));
    assert_eq!(groups[0].records.len(), 3);
    assert_eq!(groups[0].records[0].key.to_string(), "actions:testAction1");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn remaster_links_for_record_returns_direct_links() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-remaster");
    create_contract_database(&path)?;
    let connection = rusqlite::Connection::open(&path)?;
    connection.execute(
        "INSERT INTO remaster_links (
           remaster_record_key, legacy_record_key, source_kind, source_ref
         ) VALUES ('actions:testAction2', 'actions:testAction1', 'migration', 'fixture')",
        [],
    )?;
    drop(connection);

    let result = SqliteIndexReader::open_read_only(&path)?
        .remaster_links_for_record(&RecordKey::parse("actions:testAction1")?)?
        .expect("seed should exist");

    assert_eq!(result.links.len(), 1);
    assert_eq!(
        result.links[0].legacy_record.key.to_string(),
        "actions:testAction1"
    );
    assert_eq!(
        result.links[0].remaster_record.key.to_string(),
        "actions:testAction2"
    );
    assert_eq!(result.links[0].source.as_str(), "migration");
    fs::remove_file(path)?;
    Ok(())
}
