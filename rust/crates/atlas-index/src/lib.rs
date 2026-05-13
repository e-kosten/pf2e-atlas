#![deny(unsafe_code)]

use std::path::Path;

use atlas_domain::{
    ARTIFACT_METADATA_TABLE, ArtifactValidationReport, LEGACY_METADATA_TABLE,
    REQUIRED_ARTIFACT_METADATA_KEYS,
};
use rusqlite::{Connection, OpenFlags};
use thiserror::Error;

mod contract;
pub mod inspect;
mod metadata;
mod sql;
#[cfg(test)]
mod tests;

pub use inspect::{
    IndexInspectionReport, MetricCoverageReport, RecordCoverageReport, RelationshipCoverageReport,
    TaxonomyCoverageReport, TextCoverageReport, VariantCoverageReport, inspect_index,
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
