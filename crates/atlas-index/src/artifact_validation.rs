use std::collections::BTreeMap;

use rusqlite::Connection;

use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError, ValidationCode,
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
