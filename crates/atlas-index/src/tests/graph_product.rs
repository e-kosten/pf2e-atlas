use std::fs;

use atlas_domain::RecordKey;
use rusqlite::Connection;

use crate::{ReferenceEdgeDirection, RemasterReadIndex, SqliteIndexReader, VariantReadIndex};

use super::{create_valid_artifact_database, insert_reference_edge, temp_db_path};

#[test]
fn reference_edges_for_seed_returns_authored_outgoing_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-outgoing");
    create_valid_artifact_database(&path)?;
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
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records
         SET record_role = 'source_instance',
             retrieval_disposition = 'direct_only',
             retrieval_rationale = 'duplicate_source_instance',
             is_default_visible = 0
         WHERE record_key = 'actions:testAction3'",
        [],
    )?;
    drop(connection);

    let index = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Outgoing,
    )?;

    assert_eq!(edges.len(), 2);
    let edges = edges
        .into_iter()
        .map(|edge| (edge.reference_text.clone(), edge))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        edges["visible-ref"].to_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(
        edges["visible-ref"].display_text.as_deref(),
        Some("Visible")
    );
    assert_eq!(
        edges["private-ref"].to_record_key,
        RecordKey::parse("actions:testAction3")?
    );
    assert_eq!(
        edges["private-ref"].display_text.as_deref(),
        Some("Private")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reference_edges_for_seed_returns_authored_backlinks() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-backlinks");
    create_valid_artifact_database(&path)?;
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

    let index = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let edges = index.reference_edges_for_seed(
        &RecordKey::parse("actions:testAction1")?,
        ReferenceEdgeDirection::Backlink,
    )?;

    assert_eq!(edges.len(), 2);
    assert_eq!(
        edges[0].from_record_key,
        RecordKey::parse("actions:testAction2")?
    );
    assert_eq!(edges[0].reference_text, "incoming-ref");
    assert_eq!(
        edges[1].from_record_key,
        RecordKey::parse("actions:testAction3")?
    );
    assert_eq!(edges[1].reference_text, "private-ref");
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn outgoing_reference_targets_for_records_batches_authored_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-outgoing-batch");
    create_valid_artifact_database(&path)?;
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
        "actions:testAction2",
        "actions:testAction3",
        Some("Also Visible"),
        "also-visible-ref",
        "description",
        "public",
    )?;
    insert_reference_edge(
        &path,
        "actions:testAction2",
        "actions:testAction1",
        Some("Embedded"),
        "embedded-ref",
        "embedded_item_description",
        "public",
    )?;

    let index = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let targets = index.outgoing_reference_targets_for_records(&[
        RecordKey::parse("actions:testAction1")?,
        RecordKey::parse("actions:testAction2")?,
        RecordKey::parse("actions:testAction3")?,
    ])?;

    assert_eq!(
        targets[&RecordKey::parse("actions:testAction1")?]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["actions:testAction2", "actions:testAction3"]
    );
    assert_eq!(
        targets[&RecordKey::parse("actions:testAction2")?]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["actions:testAction3"]
    );
    assert!(targets[&RecordKey::parse("actions:testAction3")?].is_empty());
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn variant_group_returns_ordered_siblings() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-group");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_variant_group(&connection)?;
    drop(connection);

    let reader = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let group = reader
        .variant_group_for_record(&RecordKey::parse("actions:testAction2")?)?
        .expect("variant seed should have group");

    assert_eq!(group.variant_group_key.as_deref(), Some("test-action"));
    assert_eq!(
        group
            .record_keys
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![
            "actions:testAction1",
            "actions:testAction2",
            "actions:testAction3"
        ]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn variant_group_reports_missing_and_non_variant_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-missing");
    create_valid_artifact_database(&path)?;

    let reader = SqliteIndexReader::open_unpublished_read_only(&path)?;
    assert!(
        reader
            .variant_group_for_record(&RecordKey::parse("actions:missing")?)?
            .is_none()
    );
    let group = reader
        .variant_group_for_record(&RecordKey::parse("actions:testAction1")?)?
        .expect("existing non-variant seed should be represented");
    assert_eq!(group.variant_group_key, None);
    assert!(group.record_keys.is_empty());

    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn variant_base_name_returns_ordinary_matching_groups() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-base");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_variant_group(&connection)?;
    connection.execute(
        "UPDATE records
         SET retrieval_disposition = 'direct_only',
             retrieval_rationale = 'canonical_edition_duplicate',
             is_default_visible = 0
         WHERE record_key = 'actions:testAction3'",
        [],
    )?;
    drop(connection);

    let reader = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let groups = reader.variant_groups_by_base_name("test action")?;

    assert_eq!(
        groups
            .iter()
            .map(|group| group.variant_group_key.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("test-action")]
    );
    assert_eq!(
        groups[0]
            .record_keys
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["actions:testAction1", "actions:testAction2"]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn remaster_links_are_bidirectional_and_can_be_empty() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-remaster");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO remaster_links (remaster_record_key, legacy_record_key, source_kind, source_ref)
             VALUES ('actions:testAction2', 'actions:testAction1', 'migration', 'fixture')",
        [],
    )?;
    drop(connection);

    let reader = SqliteIndexReader::open_unpublished_read_only(&path)?;
    let legacy_links = reader
        .remaster_links_for_record(&RecordKey::parse("actions:testAction1")?)?
        .expect("legacy record exists");
    assert_eq!(legacy_links.links.len(), 1);
    assert_eq!(
        legacy_links.links[0].remaster_record_key.to_string(),
        "actions:testAction2"
    );
    let remaster_links = reader
        .remaster_links_for_record(&RecordKey::parse("actions:testAction2")?)?
        .expect("remaster record exists");
    assert_eq!(remaster_links.links.len(), 1);
    assert_eq!(
        remaster_links.links[0].legacy_record_key.to_string(),
        "actions:testAction1"
    );
    let empty_links = reader
        .remaster_links_for_record(&RecordKey::parse("actions:testAction3")?)?
        .expect("unlinked record exists");
    assert!(empty_links.links.is_empty());
    assert!(
        reader
            .remaster_links_for_record(&RecordKey::parse("actions:missing")?)?
            .is_none()
    );

    fs::remove_file(path)?;
    Ok(())
}

fn insert_variant_group(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}
