use std::fs;
use std::path::PathBuf;

use atlas_artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
use atlas_domain::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_EMBEDDING_MODEL_ID,
    ValidationCode, ValidationStatus, artifact_metadata_keys,
};
use rusqlite::Connection;

use crate::validate_index;

#[test]
fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("valid");
    create_contract_database(&path)?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Ok);
    assert_eq!(report.code, ValidationCode::Ok);
    assert_eq!(
        report.artifact_contract_version.as_deref(),
        Some(ARTIFACT_CONTRACT_VERSION)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_legacy_metadata_without_accepting_it_as_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("legacy");
    let connection = Connection::open(&path)?;
    connection.execute(
        "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    connection.execute(
        "INSERT INTO metadata (key, value) VALUES ('schema_version', '25')",
        [],
    )?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::MissingArtifactMetadata);
    assert_eq!(report.legacy_schema_version.as_deref(), Some("25"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_metadata_key() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-key");
    create_contract_database_without(&path, artifact_metadata_keys::EMBEDDING_DTYPE)?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::MissingRequiredMetadata);
    assert_eq!(
        report.missing_keys,
        vec![artifact_metadata_keys::EMBEDDING_DTYPE.to_string()]
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_stale_source_signature() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("stale-source");
    create_contract_database_with_override(
        &path,
        artifact_metadata_keys::SOURCE_SIGNATURE,
        "stale:fixture",
    )?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::StaleSourceSignature);
    assert_eq!(report.diagnostics.len(), 1);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("embedding-mismatch");
    create_contract_database_with_override(
        &path,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        "BAAI/bge-small-en-v1.5",
    )?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::EmbeddingMismatch);
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some(artifact_metadata_keys::EMBEDDING_MODEL_ID)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("unsupported-schema");
    create_contract_database_with_override(&path, artifact_metadata_keys::SCHEMA_VERSION, "2")?;

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::UnsupportedSchemaVersion);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_artifact_table() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-contract-table");
    let connection = Connection::open(&path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, None)?;
    connection.execute("DROP TABLE item_records", [])?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].family,
        atlas_domain::ArtifactContractFamily::Schema
    );
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some("table:item_records")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_fts_rows_for_hidden_records() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("hidden-fts");
    create_contract_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let report = validate_index(&path)?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == atlas_domain::ArtifactContractFamily::Fts
            && diagnostic.key.as_deref() == Some("records_fts:hidden_rows")
    }));
    fs::remove_file(path)?;
    Ok(())
}

fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    insert_contract_metadata(&connection, None)?;
    insert_minimal_contract_rows(&connection)?;
    Ok(())
}

fn create_contract_database_without(
    path: &PathBuf,
    omitted_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(path)?;
    create_minimal_contract_schema(&connection)?;
    for (key, value) in valid_metadata_entries() {
        if key != omitted_key {
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
    }
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
    for (key, mut value) in valid_metadata_entries() {
        if let Some((override_key, override_value)) = override_entry
            && key == override_key
        {
            value = override_value;
        }
        connection.execute(
            "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
    }
    Ok(())
}

fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
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
        (artifact_metadata_keys::CONTENT_HASH_ALGORITHM, "sha256"),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            "transformers-js-minilm",
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            EXPECTED_EMBEDDING_MODEL_ID,
        ),
        (artifact_metadata_keys::EMBEDDING_MODEL_REVISION, "main"),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            EXPECTED_EMBEDDING_MODEL_ID,
        ),
        (artifact_metadata_keys::EMBEDDING_POOLING, "mean"),
        (artifact_metadata_keys::EMBEDDING_NORMALIZATION, "l2"),
        (artifact_metadata_keys::EMBEDDING_DIMENSIONS, "384"),
        (artifact_metadata_keys::EMBEDDING_DTYPE, "f32"),
        (artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC, "cosine"),
        (artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX, ""),
        (artifact_metadata_keys::EMBEDDING_QUERY_PREFIX, ""),
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

fn create_minimal_contract_schema(
    connection: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)?;
    Ok(())
}

fn insert_minimal_contract_rows(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    connection.execute(
        "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 3)",
        [],
    )?;
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let record_id = format!("testAction{index}");
        let name = format!("Test Action {index}");
        let normalized_name = name.to_lowercase();
        let source_path = format!("packs/actions/test-action-{index}.json");
        connection.execute(
            "INSERT INTO records (
                  record_key, id, name, normalized_name, record_family, pack_name, pack_label,
                  foundry_document_type, foundry_record_type, traits_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, search_text_projection, raw_json
                ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
                  '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, ?3, '{}')",
            [
                record_key.as_str(),
                record_id.as_str(),
                name.as_str(),
                normalized_name.as_str(),
                source_path.as_str(),
            ],
        )?;
        connection.execute(
            "INSERT INTO records_fts (record_key, name, search_text_projection)
                 VALUES (?1, ?2, ?2)",
            [record_key.as_str(), name.as_str()],
        )?;
    }
    Ok(())
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
