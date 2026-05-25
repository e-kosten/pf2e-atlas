#![deny(unsafe_code)]

use atlas_artifact::metadata::{
    ARTIFACT_METADATA_TABLE, LEGACY_METADATA_TABLE, REQUIRED_ARTIFACT_METADATA_KEYS,
};
use atlas_artifact::schema::required_tables;
use rusqlite::Connection;
use thiserror::Error;

mod build_input;
mod contract;
mod discovery;
mod embedding_cache;
mod filters;
mod fts;
mod inspect;
mod ladybug;
mod metadata;
mod records;
mod relationship_edges;
mod search;
mod sql;
mod sqlite;
#[cfg(test)]
mod tests;
mod validation;
mod vector;
mod write;
mod writer_progress;
mod writer_visibility;

pub use build_input::{IndexBuildInput, IndexBuildPack};
pub use discovery::{DiscoveryError, DiscoveryValueSort, FilterValueRequest};
pub use embedding_cache::{DocumentEmbeddingCacheError, DocumentEmbeddingCacheReader};
pub use filters::FilterCompileError;
pub use inspect::{
    IndexInspectionReport, MetricCoverageReport, RecordCoverageReport, RelationshipCoverageReport,
    TaxonomyCoverageReport, TextCoverageReport, VariantCoverageReport,
};
pub use ladybug::{LadybugIndexReader, LadybugIndexReaderError, LadybugIndexWriter};
pub use records::RecordLoadError;
pub use relationship_edges::GraphReferenceEdge;
pub use search::{
    RecordLoadOptions, RecordResolutionMatchKind, RecordResolutionResult, SearchCandidateRecord,
    SearchError, SearchIndex,
};
pub use sqlite::{
    FilteredRecordKeyPage, FilteredRecordSort, FtsColumnWeights, FtsQuery, FtsSearchHit,
    FtsSearchLane, RecordIdentityMatch, RecordIdentityMatchKind, ReferenceEdgeDirection,
    SqliteIndexReader, SqliteIndexWriter,
};
pub use validation::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationDiagnostic,
    ArtifactValidationReport, ValidationCode, ValidationStatus, ValidationTarget,
};
pub use vector::{VectorQueryError, VectorSearchHit};
pub use write::{IndexArtifactWriter, IndexWriteError};

#[derive(Debug, Error)]
pub enum IndexValidationError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
    #[error("index artifact metadata is invalid: {0}")]
    InvalidArtifact(String),
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
    let diagnostics = contract::validate_artifact_contract(connection, &artifact_metadata)?;
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
        .into_iter()
        .filter_map(|table| {
            let table_name = table.name();
            match sql::table_exists(connection, table_name) {
                Ok(true) => None,
                Ok(false) => Some(Ok(contract::contract_diagnostic(
                    ArtifactContractFamily::Schema,
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

fn validation_report_from_error(
    path: &std::path::Path,
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
    path: &std::path::Path,
    error: IndexValidationError,
) -> ArtifactValidationReport {
    validation_report_from_error(path, error)
}
