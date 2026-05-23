use std::fs;
use std::mem::size_of;

use atlas_artifact::schema::{record_vector_index_create_sql, record_vector_index_insert_sql};
use atlas_artifact::storage::encode_f32_vector_blob;
use rusqlite::Connection;

use super::{create_contract_database, temp_db_path};
use crate::{
    ArtifactContractFamily, DocumentEmbeddingCacheReader, SqliteIndexReader, ValidationCode,
    ValidationStatus,
};

#[test]
fn accepts_complete_document_embedding_cache() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("document-embedding-cache");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 384, 384 * size_of::<f32>())?;
    drop(connection);

    let report = SqliteIndexReader::open_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Ok);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn sqlite_index_loads_reusable_document_embedding_cache() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("embedding-cache-reader");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 384, 384 * size_of::<f32>())?;
    drop(connection);

    let reusable = SqliteIndexReader::open_read_only(&path)?
        .load_reusable_document_embeddings(atlas_embedding::default_embedding_model_spec())?;

    let first = reusable
        .get("actions:testAction1#parent")
        .expect("fixture cache row is loaded");
    assert_eq!(reusable.len(), 3);
    assert_eq!(first.input_hash, "fixture-hash-1");
    assert_eq!(first.dimensions, 384);
    assert_eq!(first.vector.len(), 384);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_document_embedding_cache_dimension_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("document-embedding-cache-dimensions");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 383, 384 * size_of::<f32>())?;
    drop(connection);

    let report = SqliteIndexReader::open_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("document_embedding_cache:dimensions")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_incomplete_document_embedding_cache_coverage() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("document-embedding-cache-coverage");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "INSERT INTO document_embedding_cache (
           embedding_unit_key, record_key, unit_kind, label, ordinal,
           semantic_input_hash, dimensions, vector_blob
         )
         VALUES ('actions:testAction1#parent', 'actions:testAction1', 'parent', NULL, 0,
           'fixture-hash', 384, zeroblob(1536))",
        [],
    )?;
    drop(connection);

    let report = SqliteIndexReader::open_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref() == Some("document_embedding_cache:default_visible_count")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn vector_validation_reports_missing_vector_table() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("vector-table-missing");
    create_contract_database(&path)?;

    let report = SqliteIndexReader::open_read_only_with_vectors(&path)?.validate_vector_index()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Schema
            && diagnostic.key.as_deref() == Some("table:record_vector_index")
    }));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn check_embedding_readiness_skips_deep_vector_coverage() -> Result<(), Box<dyn std::error::Error>>
{
    let path = temp_db_path("vector-check-skips-coverage");
    create_contract_database(&path)?;
    crate::vector::register_sqlite_vec_extension()?;
    let connection = Connection::open(&path)?;
    insert_document_embedding_cache_rows(&connection, 384, 384 * size_of::<f32>())?;
    connection.execute_batch(&record_vector_index_create_sql(384))?;
    connection.execute(
        &record_vector_index_insert_sql(),
        (1_i64, encode_f32_vector_blob(&vec![0.0_f32; 384])),
    )?;
    drop(connection);

    let check_report =
        SqliteIndexReader::open_read_only_with_vectors(&path)?.check_embedding_readiness_report();
    let validate_report =
        SqliteIndexReader::open_read_only_with_vectors(&path)?.validate_vector_index()?;

    assert_eq!(check_report.status, ValidationStatus::Ok);
    assert_eq!(validate_report.status, ValidationStatus::Error);
    assert!(validate_report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactContractFamily::Embedding
            && diagnostic.key.as_deref()
                == Some("record_vector_index:document_embedding_cache_count")
    }));

    fs::remove_file(path)?;
    Ok(())
}

fn insert_document_embedding_cache_rows(
    connection: &Connection,
    dimensions: usize,
    byte_len: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let embedding_unit_key = format!("{record_key}#parent");
        let semantic_input_hash = format!("fixture-hash-{index}");
        connection.execute(
            "INSERT INTO document_embedding_cache (
               embedding_unit_key, record_key, unit_kind, label, ordinal,
               semantic_input_hash, dimensions, vector_blob
             )
             VALUES (?1, ?2, 'parent', NULL, 0, ?3, ?4, zeroblob(?5))",
            rusqlite::params![
                embedding_unit_key,
                record_key,
                semantic_input_hash,
                dimensions as i64,
                byte_len as i64,
            ],
        )?;
    }
    Ok(())
}
