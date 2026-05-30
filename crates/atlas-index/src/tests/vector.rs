use std::fs;

use atlas_artifact::storage::encode_f32_vector_blob;
use atlas_domain::RecordKey;
use rusqlite::Connection;

use super::{create_contract_database, temp_db_path};
use crate::{SqliteIndexReader, VectorQueryError};

#[test]
fn loads_record_embedding_vectors_for_similarity_seed() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("record-embedding-vectors");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES (?1, ?2, 'parent', NULL, 0, 'fixture-hash-parent', 2, ?3)",
        rusqlite::params![
            "actions:testAction1#parent",
            "actions:testAction1",
            encode_f32_vector_blob(&[0.25, 0.75]),
        ],
    )?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES (?1, ?2, 'heading_section', 'Rules', 1, 'fixture-hash-child', 2, ?3)",
        rusqlite::params![
            "actions:testAction1#heading:1",
            "actions:testAction1",
            encode_f32_vector_blob(&[0.50, 0.50]),
        ],
    )?;
    drop(connection);

    let units = SqliteIndexReader::open_read_only(&path)?
        .load_record_embedding_vectors(&RecordKey::parse("actions:testAction1")?)?;

    assert_eq!(units.len(), 2);
    assert_eq!(units[0].unit_kind, "parent");
    assert_eq!(units[0].ordinal, 0);
    assert_eq!(units[0].vector, vec![0.25, 0.75]);
    assert_eq!(units[1].unit_kind, "heading_section");
    assert_eq!(units[1].ordinal, 1);
    assert_eq!(units[1].label.as_deref(), Some("Rules"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_record_embedding_vectors_rejects_invalid_unit_kind()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("record-embedding-vectors-invalid-kind");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES (?1, ?2, 'bogus', NULL, 0, 'fixture-hash-parent', 2, ?3)",
        rusqlite::params![
            "actions:testAction1#parent",
            "actions:testAction1",
            encode_f32_vector_blob(&[0.25, 0.75]),
        ],
    )?;
    drop(connection);

    let error = SqliteIndexReader::open_read_only(&path)?
        .load_record_embedding_vectors(&RecordKey::parse("actions:testAction1")?)
        .expect_err("invalid unit kind should be rejected");

    assert_eq!(
        error,
        VectorQueryError::InvalidUnitKind("bogus".to_string())
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_record_embedding_vectors_rejects_invalid_vector_blob()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("record-embedding-vectors-invalid-blob");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES (?1, ?2, 'parent', NULL, 0, 'fixture-hash-parent', 2, zeroblob(3))",
        ["actions:testAction1#parent", "actions:testAction1"],
    )?;
    drop(connection);

    let error = SqliteIndexReader::open_read_only(&path)?
        .load_record_embedding_vectors(&RecordKey::parse("actions:testAction1")?)
        .expect_err("invalid vector blob should be rejected");

    assert!(matches!(
        error,
        VectorQueryError::InvalidStoredVector {
            record_key,
            unit_kind,
            ordinal,
            ..
        } if record_key == "actions:testAction1" && unit_kind == "parent" && ordinal == 0
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn load_record_embedding_vectors_rejects_dimension_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("record-embedding-vectors-dimension-mismatch");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES (?1, ?2, 'parent', NULL, 0, 'fixture-hash-parent', 3, ?3)",
        rusqlite::params![
            "actions:testAction1#parent",
            "actions:testAction1",
            encode_f32_vector_blob(&[0.25, 0.75]),
        ],
    )?;
    drop(connection);

    let error = SqliteIndexReader::open_read_only(&path)?
        .load_record_embedding_vectors(&RecordKey::parse("actions:testAction1")?)
        .expect_err("dimension mismatch should be rejected");

    assert_eq!(
        error,
        VectorQueryError::InvalidStoredDimensions {
            record_key: "actions:testAction1".to_string(),
            unit_kind: "parent".to_string(),
            ordinal: 0,
            declared_dimensions: 3,
            decoded_dimensions: 2,
        }
    );
    fs::remove_file(path)?;
    Ok(())
}
