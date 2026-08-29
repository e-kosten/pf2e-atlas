use std::fs;

use crate::artifact::metadata::artifact_metadata_keys;
use atlas_embedding::EmbeddingModelId;
use rusqlite::Connection;

use super::{
    create_minimal_artifact_schema, create_valid_artifact_database,
    create_valid_artifact_database_with_override, create_valid_artifact_database_without,
    insert_artifact_metadata, insert_artifact_metadata_entries, insert_minimal_artifact_rows,
    temp_db_path, valid_metadata_entries_for_embedding,
};
use crate::{ArtifactValidationFamily, SqliteIndexReader, ValidationCode, ValidationStatus};

#[test]
fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("valid");
    create_valid_artifact_database(&path)?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Ok);
    assert_eq!(report.code, ValidationCode::Ok);
    assert_eq!(
        report.artifact_contract_version.as_deref(),
        Some(crate::artifact::metadata::ARTIFACT_CONTRACT_VERSION)
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

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::MissingArtifactMetadata);
    assert_eq!(report.legacy_schema_version.as_deref(), Some("25"));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_metadata_key() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-key");
    create_valid_artifact_database_without(&path, artifact_metadata_keys::EMBEDDING_DTYPE)?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

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
    create_valid_artifact_database_with_override(
        &path,
        artifact_metadata_keys::SOURCE_SIGNATURE,
        "stale:fixture",
    )?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::StaleSourceSignature);
    assert_eq!(report.diagnostics.len(), 1);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("embedding-mismatch");
    create_valid_artifact_database_with_override(
        &path,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        "unknown/model",
    )?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

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
fn reports_embedding_unit_policy_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("embedding-unit-policy-mismatch");
    create_valid_artifact_database_with_override(
        &path,
        artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
        "coverage-driven-rich-content/v1",
    )?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::EmbeddingMismatch);
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some(artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION)
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn accepts_known_non_default_embedding_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("known-non-default-embedding");
    let connection = Connection::open(&path)?;
    create_minimal_artifact_schema(&connection)?;
    let embedding_spec = atlas_embedding::embedding_model_spec(EmbeddingModelId::BgeSmallEnV15);
    insert_artifact_metadata_entries(
        &connection,
        valid_metadata_entries_for_embedding(embedding_spec),
        None,
    )?;
    insert_minimal_artifact_rows(&connection)?;
    drop(connection);

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Ok);
    assert_eq!(report.code, ValidationCode::Ok);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("unsupported-schema");
    create_valid_artifact_database_with_override(
        &path,
        artifact_metadata_keys::SCHEMA_VERSION,
        "999",
    )?;

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::UnsupportedSchemaVersion);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn reports_missing_required_artifact_table() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("missing-contract-table");
    let connection = Connection::open(&path)?;
    create_minimal_artifact_schema(&connection)?;
    insert_artifact_metadata(&connection, None)?;
    connection.execute("DROP TABLE item_records", [])?;
    drop(connection);

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::UnsupportedSchemaVersion);
    assert!(report.message.contains("`atlas setup`"));
    assert!(report.message.contains("`atlas index build`"));
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].family,
        ArtifactValidationFamily::Schema
    );
    assert_eq!(
        report.diagnostics[0].key.as_deref(),
        Some("table:item_records")
    );
    assert_eq!(
        report.diagnostics[0].code,
        ValidationCode::UnsupportedSchemaVersion
    );
    assert!(
        report.diagnostics[0]
            .message
            .contains("cannot be upgraded in place")
    );
    assert!(report.diagnostics[0].message.contains("`atlas setup`"));
    assert!(
        report.diagnostics[0]
            .message
            .contains("`atlas index build`")
    );
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn old_v2_layouts_fail_with_actionable_rebuild_guidance() -> Result<(), Box<dyn std::error::Error>>
{
    let cases = [
        (
            "missing-activities-table",
            "DROP TABLE record_activities",
            "table:record_activities",
            "required artifact table `record_activities` is missing",
        ),
        (
            "missing-spellcasting-table",
            "DROP TABLE record_spellcasting_entries",
            "table:record_spellcasting_entries",
            "required artifact table `record_spellcasting_entries` is missing",
        ),
        (
            "missing-metric-ordinal",
            "ALTER TABLE record_metrics RENAME COLUMN ordinal TO legacy_ordinal",
            "column:record_metrics.ordinal",
            "required artifact column `record_metrics.ordinal` is missing",
        ),
        (
            "missing-visibility-state",
            "ALTER TABLE records RENAME COLUMN visibility_state TO legacy_visibility_state",
            "column:records.visibility_state",
            "required artifact column `records.visibility_state` is missing",
        ),
        (
            "missing-visibility-reason",
            "ALTER TABLE records RENAME COLUMN visibility_reason TO legacy_visibility_reason",
            "column:records.visibility_reason",
            "required artifact column `records.visibility_reason` is missing",
        ),
    ];

    for (name, mutation, key, fact) in cases {
        let path = temp_db_path(name);
        create_valid_artifact_database(&path)?;
        let connection = Connection::open(&path)?;
        connection.execute_batch(mutation)?;
        drop(connection);

        let reader = SqliteIndexReader::open_unpublished_read_only(&path)?;
        let report = reader.validate()?;
        assert_eq!(report.status, ValidationStatus::Error, "{name}: {report:?}");
        assert_eq!(
            report.code,
            ValidationCode::UnsupportedSchemaVersion,
            "{name}: {report:?}"
        );
        assert!(
            report.message.contains("`atlas setup`"),
            "{name}: {report:?}"
        );
        assert!(
            report.message.contains("`atlas index build`"),
            "{name}: {report:?}"
        );
        assert_eq!(report.diagnostics.len(), 1, "{name}: {report:?}");
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.code, ValidationCode::UnsupportedSchemaVersion);
        assert_eq!(diagnostic.family, ArtifactValidationFamily::Schema);
        assert_eq!(diagnostic.key.as_deref(), Some(key));
        assert!(diagnostic.message.contains(fact), "{name}: {diagnostic:?}");
        assert!(diagnostic.message.contains("cannot be upgraded in place"));
        assert!(diagnostic.message.contains("`atlas setup`"));
        assert!(diagnostic.message.contains("`atlas index build`"));

        let fast_report = reader.check()?;
        if key.starts_with("table:") {
            assert_eq!(
                fast_report.code,
                ValidationCode::UnsupportedSchemaVersion,
                "{name}: {fast_report:?}"
            );
            assert_eq!(fast_report.diagnostics[0].key.as_deref(), Some(key));
        }
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn reports_fts_rows_for_hidden_records() -> Result<(), Box<dyn std::error::Error>> {
    let path = temp_db_path("hidden-fts");
    create_valid_artifact_database(&path)?;
    let connection = Connection::open(&path)?;
    connection.execute(
        "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction1'",
        [],
    )?;
    drop(connection);

    let report = SqliteIndexReader::open_unpublished_read_only(&path)?.validate()?;

    assert_eq!(report.status, ValidationStatus::Error);
    assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.family == ArtifactValidationFamily::Fts
            && diagnostic.key.as_deref() == Some("records_fts:hidden_rows")
    }));
    fs::remove_file(path)?;
    Ok(())
}
