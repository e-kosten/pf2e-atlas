use std::fs;
use std::path::PathBuf;

use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, artifact_metadata_keys,
};
use atlas_artifact::test_support::{
    create_minimal_contract_schema, insert_contract_metadata_omitting, insert_minimal_contract_rows,
};
use atlas_embedding::{EMBEDDING_UNIT_POLICY_VERSION, default_embedding_model_spec};
use rusqlite::Connection;

mod embedding;
mod filters;
mod fts;
mod graph_product;
mod records;
mod validation;
mod vector;
mod vector_query;

fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, None)?;
    insert_minimal_contract_rows(&connection)?;
    Ok(())
}

fn insert_reference_edge(
    path: &PathBuf,
    from_record_key: &str,
    to_record_key: &str,
    display_text: Option<&str>,
    reference_text: &str,
    source_kind: &str,
    visibility: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    connection.execute(
        "INSERT INTO reference_edges (
           from_record_key, to_record_key, display_text, reference_text, source_kind, visibility
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            from_record_key,
            to_record_key,
            display_text,
            reference_text,
            source_kind,
            visibility,
        ),
    )?;
    Ok(())
}

fn create_contract_database_without(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata_omitting(&connection, valid_metadata_entries(), omitted_key)?;
    Ok(())
}

fn create_contract_database_with_override(
    path: &PathBuf,
    override_key: &str,
    override_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, Some((override_key, override_value)))?;
    Ok(())
}

fn insert_contract_metadata(
    connection: &Connection,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    insert_contract_metadata_entries(connection, valid_metadata_entries(), override_entry)
}

fn insert_contract_metadata_entries(
    connection: &Connection,
    entries: Vec<(&'static str, &'static str)>,
    override_entry: Option<(&str, &str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    atlas_artifact::test_support::insert_contract_metadata_entries(
        connection,
        entries,
        override_entry,
    )
}

fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
    valid_metadata_entries_for_embedding(default_embedding_model_spec())
}

fn valid_metadata_entries_for_embedding(
    embedding_spec: atlas_embedding::EmbeddingModelSpec,
) -> Vec<(&'static str, &'static str)> {
    vec![
        (
            artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
            ARTIFACT_CONTRACT_VERSION,
        ),
        (
            artifact_metadata_keys::SCHEMA_VERSION,
            ARTIFACT_SCHEMA_VERSION,
        ),
        (artifact_metadata_keys::SOURCE_KIND, "foundry-pf2e"),
        (
            artifact_metadata_keys::SOURCE_SIGNATURE,
            "foundry-pf2e:fixture",
        ),
        (artifact_metadata_keys::SOURCE_RECORD_COUNT, "3"),
        (artifact_metadata_keys::ARTIFACT_RECORD_COUNT, "3"),
        (artifact_metadata_keys::GENERATED_RECORD_COUNT, "0"),
        (artifact_metadata_keys::CONTENT_HASH_ALGORITHM, "sha256"),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            embedding_spec.provider_family,
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            embedding_spec.model_id,
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            embedding_spec.model_revision,
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            embedding_spec.tokenizer_id,
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            embedding_spec.pooling.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            embedding_spec.normalization.as_str(),
        ),
        (artifact_metadata_keys::EMBEDDING_DIMENSIONS, "384"),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            embedding_spec.dtype.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            embedding_spec.distance_metric.as_str(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            embedding_spec.document_prefix,
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            embedding_spec.query_prefix,
        ),
        (
            artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
            EMBEDDING_UNIT_POLICY_VERSION,
        ),
        (
            artifact_metadata_keys::FTS_TOKENIZER,
            "unicode61 remove_diacritics 2",
        ),
        (
            artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
            "manifest.json",
        ),
    ]
}

fn temp_db_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-index-{name}-{}-{}.sqlite",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_file(&path);
    path
}
