use std::fs;

use atlas_domain::RecordKey;
use rusqlite::Connection;

use crate::{GraphReadIndex, SqliteIndexReader};

use super::{create_contract_database, temp_db_path};

#[test]
fn variant_group_returns_ordered_siblings() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-group");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_variant_group(&connection)?;
    drop(connection);

    let reader = SqliteIndexReader::open_read_only(&path)?;
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
    create_contract_database(&path)?;

    let reader = SqliteIndexReader::open_read_only(&path)?;
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
fn variant_base_name_returns_default_visible_matching_groups()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("graph-variant-base");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_variant_group(&connection)?;
    connection.execute(
        "UPDATE records
         SET is_default_visible = 0
         WHERE record_key = 'actions:testAction3'",
        [],
    )?;
    drop(connection);

    let reader = SqliteIndexReader::open_read_only(&path)?;
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
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO remaster_links (remaster_record_key, legacy_record_key, source_kind, source_ref)
             VALUES ('actions:testAction2', 'actions:testAction1', 'migration', 'fixture')",
        [],
    )?;
    drop(connection);

    let reader = SqliteIndexReader::open_read_only(&path)?;
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
