#![deny(unsafe_code)]

use std::path::Path;

use atlas_artifact::metadata::{
    ARTIFACT_METADATA_TABLE, LEGACY_METADATA_TABLE, REQUIRED_ARTIFACT_METADATA_KEYS,
};
use rusqlite::{Connection, OpenFlags};
use thiserror::Error;

mod contract;
mod database;
pub mod filters;
pub mod inspect;
mod metadata;
pub mod records;
mod sql;
#[cfg(test)]
mod tests;
pub mod validation;
pub mod vector;

pub use database::AtlasIndex;
pub use inspect::{
    IndexInspectionReport, MetricCoverageReport, RecordCoverageReport, RelationshipCoverageReport,
    TaxonomyCoverageReport, TextCoverageReport, VariantCoverageReport, inspect_index,
};
pub use validation::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationDiagnostic,
    ArtifactValidationReport, ValidationCode, ValidationStatus,
};
pub use vector::{
    VectorKnnQuery, VectorQueryError, VectorSearchHit, compile_vector_knn_query,
    query_vector_index, validate_vector_index, validate_vector_index_report,
    validate_vector_index_with_loader, write_vector_index, write_vector_index_report,
    write_vector_index_with_loader,
};

#[derive(Debug, Error)]
pub enum IndexValidationError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
    #[error("index artifact metadata is invalid: {0}")]
    InvalidArtifact(String),
}

pub fn validate_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    let index = path.display().to_string();
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if !sql::table_exists(&connection, ARTIFACT_METADATA_TABLE)? {
        let legacy_schema_version = if sql::table_exists(&connection, LEGACY_METADATA_TABLE)? {
            sql::metadata_value(&connection, LEGACY_METADATA_TABLE, "schema_version")?
        } else {
            None
        };
        return Ok(ArtifactValidationReport::missing_artifact_metadata(
            index,
            legacy_schema_version,
        ));
    }

    let artifact_metadata = metadata::read_metadata(&connection, ARTIFACT_METADATA_TABLE)?;
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
    let diagnostics = if diagnostics.is_empty() {
        contract::validate_artifact_contract(&connection, &artifact_metadata)?
    } else {
        diagnostics
    };
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

pub fn validate_index_report(path: impl AsRef<Path>) -> ArtifactValidationReport {
    let path = path.as_ref();
    match validate_index(path) {
        Ok(report) => report,
        Err(error) => validation_report_from_error(path, error),
    }
}

fn validation_report_from_error(
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
        fts_tokenizer: None,
        adjacent_manifest_path: None,
        missing_keys: Vec::new(),
        diagnostics: Vec::new(),
        legacy_schema_version: None,
    }
}
