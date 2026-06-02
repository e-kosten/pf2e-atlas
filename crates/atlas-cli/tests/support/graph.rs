#![allow(dead_code)]

use std::path::Path;

use rusqlite::Connection;
use serde_json::Value;

pub fn insert_graph_edges(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    for (from, to, display, reference, source_kind, visibility) in [
        (
            "actions:testAction1",
            "actions:testAction2",
            Some("Alpha"),
            "@UUID[Compendium.pf2e.actions.Item.testAction2]{Alpha}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction2",
            Some("Beta"),
            "@UUID[Compendium.pf2e.actions.Item.testAction2]{Beta}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction3",
            Some("Gamma"),
            "@UUID[Compendium.pf2e.actions.Item.testAction3]{Gamma}",
            "description",
            "public",
        ),
        (
            "actions:testAction1",
            "actions:testAction3",
            Some("Private"),
            "@UUID[Compendium.pf2e.actions.Item.testAction3]{Private}",
            "description",
            "private",
        ),
        (
            "actions:testAction2",
            "actions:testAction1",
            Some("Incoming A"),
            "@UUID[Compendium.pf2e.actions.Item.testAction1]{Incoming A}",
            "description",
            "public",
        ),
        (
            "actions:testAction3",
            "actions:testAction1",
            Some("Incoming B"),
            "@UUID[Compendium.pf2e.actions.Item.testAction1]{Incoming B}",
            "description",
            "public",
        ),
    ] {
        insert_reference_edge(
            &connection,
            from,
            to,
            display,
            reference,
            source_kind,
            visibility,
        )?;
    }
    Ok(())
}

pub fn insert_reference_edge(
    connection: &Connection,
    from: &str,
    to: &str,
    display: Option<&str>,
    reference: &str,
    source_kind: &str,
    visibility: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO reference_edges (
           from_record_key, to_record_key, display_text, reference_text, source_kind, visibility
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (from, to, display, reference, source_kind, visibility),
    )?;
    Ok(())
}

pub fn insert_variant_group(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
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
                 variant_source = 'parenthetical',
                 level = ?2
             WHERE record_key = ?3",
            (label, level, record_key),
        )?;
    }
    Ok(())
}

pub fn insert_second_variant_group(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "UPDATE records
         SET variant_group_key = 'test-action-alt',
             variant_base_name = 'Test Action',
             variant_label = 'Alternate',
             variant_axes_json = '[\"grade\"]',
             variant_confidence = 1.0,
             variant_source = 'parenthetical',
             level = 9
         WHERE record_key = 'actions:testAction3'",
        [],
    )?;
    Ok(())
}

pub fn insert_remaster_link(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "INSERT INTO remaster_links (
           remaster_record_key, legacy_record_key, source_kind, source_ref
         ) VALUES (?1, ?2, ?3, ?4)",
        (
            "actions:testAction2",
            "actions:testAction1",
            "migration",
            "test migration",
        ),
    )?;
    Ok(())
}

pub fn assert_section_edges_point_to_returned_records(section: &Value, neighbor_field: &str) {
    let record_keys = section["records"]
        .as_array()
        .expect("records should be an array")
        .iter()
        .map(|record| {
            record["key"]
                .as_str()
                .expect("record key should be a string")
        })
        .collect::<Vec<_>>();
    for edge in section["edges"]
        .as_array()
        .expect("edges should be an array")
    {
        let neighbor = edge[neighbor_field]
            .as_str()
            .expect("edge endpoint should be a string");
        assert!(
            record_keys.contains(&neighbor),
            "edge endpoint {neighbor} should be present in returned records"
        );
    }
}

pub fn set_record_visibility(
    path: &Path,
    record_key: &str,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = ?1 WHERE record_key = ?2",
        (if visible { 1_i64 } else { 0_i64 }, record_key),
    )?;
    Ok(())
}
