use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;

use crate::artifact::inventory::required_tables;
use crate::artifact::metadata::{
    ARTIFACT_METADATA_TABLE, LEGACY_METADATA_TABLE, REQUIRED_ARTIFACT_METADATA_KEYS,
};
use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus, metadata, sql,
};

mod content;
mod discovery;
mod embeddings;
mod fts;
mod metrics;
mod relationships;
mod schema;

use content::validate_content_json;
use discovery::validate_filter_discovery_catalogs;
use embeddings::validate_document_embedding_cache;
use fts::validate_fts_coverage;
use metrics::{validate_metric_catalogs, validate_metric_values};
use relationships::validate_relationships;
use schema::{
    validate_boolean_columns, validate_foreign_keys, validate_record_counts,
    validate_required_columns, validate_required_tables,
};

pub(crate) fn validate_artifact_coherence(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
) -> Result<Vec<ArtifactValidationDiagnostic>, IndexValidationError> {
    let mut diagnostics = Vec::new();
    validate_required_tables(connection, &mut diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(diagnostics);
    }

    validate_required_columns(connection, &mut diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(diagnostics);
    }

    validate_record_counts(connection, metadata, &mut diagnostics)?;
    validate_foreign_keys(connection, &mut diagnostics)?;
    validate_boolean_columns(connection, &mut diagnostics)?;
    validate_metric_values(connection, &mut diagnostics)?;
    validate_content_json(connection, &mut diagnostics)?;
    validate_fts_coverage(connection, &mut diagnostics)?;
    validate_document_embedding_cache(connection, metadata, &mut diagnostics)?;
    validate_relationships(connection, &mut diagnostics)?;
    validate_metric_catalogs(connection, &mut diagnostics)?;
    validate_filter_discovery_catalogs(connection, &mut diagnostics)?;
    Ok(diagnostics)
}

pub(crate) fn validate_index_connection(
    index: String,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let metadata_report = validate_index_metadata_connection(index.clone(), connection)?;
    if metadata_report.status != ValidationStatus::Ok {
        return Ok(metadata_report);
    }

    let artifact_metadata = metadata::read_metadata(connection, ARTIFACT_METADATA_TABLE)?;
    let diagnostics = validate_artifact_coherence(connection, &artifact_metadata)?;
    if diagnostics.is_empty() {
        Ok(metadata_report)
    } else {
        let summary = metadata::summarize_metadata(&artifact_metadata);
        Ok(ArtifactValidationReport::incompatible_metadata(
            index,
            summary,
            diagnostics,
        ))
    }
}

pub(crate) fn check_index_connection(
    index: String,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let metadata_report = validate_index_metadata_connection(index.clone(), connection)?;
    if metadata_report.status != ValidationStatus::Ok {
        return Ok(metadata_report);
    }

    let diagnostics = required_tables()
        .iter()
        .filter_map(|table| {
            let table_name = table.name();
            match sql::table_exists(connection, table_name) {
                Ok(true) => None,
                Ok(false) => Some(Ok(artifact_validation_diagnostic(
                    ArtifactValidationFamily::Schema,
                    format!("required artifact table `{table_name}` is missing"),
                    Some(format!("table:{table_name}")),
                    Some("present".to_string()),
                    Some("missing".to_string()),
                ))),
                Err(error) => Some(Err(error)),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if diagnostics.is_empty() {
        Ok(metadata_report)
    } else {
        let artifact_metadata = metadata::read_metadata(connection, ARTIFACT_METADATA_TABLE)?;
        let summary = metadata::summarize_metadata(&artifact_metadata);
        Ok(ArtifactValidationReport::incompatible_metadata(
            index,
            summary,
            diagnostics,
        ))
    }
}

pub(crate) fn validate_index_metadata_connection(
    index: String,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    if !sql::table_exists(connection, ARTIFACT_METADATA_TABLE)? {
        let legacy_schema_version = if sql::table_exists(connection, LEGACY_METADATA_TABLE)? {
            sql::metadata_value(connection, LEGACY_METADATA_TABLE, "schema_version")?
        } else {
            None
        };
        return Ok(ArtifactValidationReport::missing_artifact_metadata(
            index,
            legacy_schema_version,
        ));
    }

    let artifact_metadata = metadata::read_metadata(connection, ARTIFACT_METADATA_TABLE)?;
    let summary = metadata::summarize_metadata(&artifact_metadata);
    let missing_keys = REQUIRED_ARTIFACT_METADATA_KEYS
        .iter()
        .filter(|key| {
            artifact_metadata
                .get(**key)
                .is_none_or(|value| metadata::is_missing_value(key, value))
        })
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();

    if !missing_keys.is_empty() {
        return Ok(ArtifactValidationReport::missing_required_metadata(
            index,
            summary,
            missing_keys,
        ));
    }

    let diagnostics = metadata::validate_metadata_values(&artifact_metadata);
    if diagnostics.is_empty() {
        Ok(ArtifactValidationReport::ok(index, summary))
    } else {
        Ok(ArtifactValidationReport::incompatible_metadata(
            index,
            summary,
            diagnostics,
        ))
    }
}

pub(crate) fn validation_report_from_error(
    path: &Path,
    error: IndexValidationError,
) -> ArtifactValidationReport {
    ArtifactValidationReport {
        status: ValidationStatus::Error,
        code: match error {
            IndexValidationError::Unavailable(_) => ValidationCode::IndexUnavailable,
            IndexValidationError::QueryFailed(_) => ValidationCode::QueryFailed,
            IndexValidationError::InvalidArtifact(_) => ValidationCode::InvalidSourceMetadata,
        },
        index: path.display().to_string(),
        message: error.to_string(),
        artifact_contract_version: None,
        schema_version: None,
        source_kind: None,
        source_signature: None,
        source_record_count: None,
        artifact_record_count: None,
        generated_record_count: None,
        content_hash_algorithm: None,
        embedding_provider_family: None,
        embedding_model_id: None,
        embedding_model_revision: None,
        embedding_tokenizer_id: None,
        embedding_pooling: None,
        embedding_normalization: None,
        embedding_dimensions: None,
        embedding_dtype: None,
        embedding_distance_metric: None,
        embedding_document_prefix: None,
        embedding_query_prefix: None,
        embedding_unit_policy_version: None,
        fts_tokenizer: None,
        adjacent_manifest_path: None,
        missing_keys: Vec::new(),
        diagnostics: Vec::new(),
        legacy_schema_version: None,
    }
}

pub fn validation_report_for_error(
    path: &Path,
    error: IndexValidationError,
) -> ArtifactValidationReport {
    validation_report_from_error(path, error)
}

pub(crate) fn artifact_validation_diagnostic(
    family: ArtifactValidationFamily,
    message: String,
    key: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
) -> ArtifactValidationDiagnostic {
    artifact_validation_diagnostic_with_code(
        family,
        message,
        key,
        expected,
        actual,
        ValidationCode::ArtifactContractViolation,
    )
}

pub(crate) fn artifact_validation_diagnostic_with_code(
    family: ArtifactValidationFamily,
    message: String,
    key: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    code: ValidationCode,
) -> ArtifactValidationDiagnostic {
    ArtifactValidationDiagnostic {
        code,
        family,
        message,
        key,
        expected,
        actual,
    }
}
