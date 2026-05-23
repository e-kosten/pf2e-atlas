use std::fs;

use atlas_domain::RecordKey;

use super::{create_contract_database, insert_reference_edge, temp_db_path};
use crate::{ReferenceEdgeDirection, SqliteIndexReader};

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
