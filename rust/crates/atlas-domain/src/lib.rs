#![deny(unsafe_code)]

use serde::Serialize;

pub const ARTIFACT_METADATA_TABLE: &str = "artifact_metadata";
pub const LEGACY_METADATA_TABLE: &str = "metadata";

pub const REQUIRED_ARTIFACT_METADATA_KEYS: &[&str] = &[
    "artifact_contract_version",
    "schema_version",
    "source_signature",
    "embedding_model_id",
    "embedding_dimensions",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationCode {
    Ok,
    IndexUnavailable,
    MissingArtifactMetadata,
    MissingRequiredMetadata,
    QueryFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactValidationReport {
    pub status: ValidationStatus,
    pub code: ValidationCode,
    pub index: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_contract_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dimensions: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_keys: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_schema_version: Option<String>,
}

impl ArtifactValidationReport {
    pub fn ok(index: String, metadata: ArtifactMetadataSummary) -> Self {
        Self {
            status: ValidationStatus::Ok,
            code: ValidationCode::Ok,
            index,
            message: "artifact metadata is valid".to_string(),
            artifact_contract_version: metadata.artifact_contract_version,
            schema_version: metadata.schema_version,
            source_signature: metadata.source_signature,
            embedding_model_id: metadata.embedding_model_id,
            embedding_dimensions: metadata.embedding_dimensions,
            missing_keys: Vec::new(),
            legacy_schema_version: None,
        }
    }

    pub fn missing_artifact_metadata(index: String, legacy_schema_version: Option<String>) -> Self {
        Self {
            status: ValidationStatus::Error,
            code: ValidationCode::MissingArtifactMetadata,
            index,
            message: "index opened, but the Rust artifact contract metadata table is missing"
                .to_string(),
            artifact_contract_version: None,
            schema_version: None,
            source_signature: None,
            embedding_model_id: None,
            embedding_dimensions: None,
            missing_keys: Vec::new(),
            legacy_schema_version,
        }
    }

    pub fn missing_required_metadata(
        index: String,
        metadata: ArtifactMetadataSummary,
        missing_keys: Vec<String>,
    ) -> Self {
        Self {
            status: ValidationStatus::Error,
            code: ValidationCode::MissingRequiredMetadata,
            index,
            message: "artifact metadata table is missing required keys".to_string(),
            artifact_contract_version: metadata.artifact_contract_version,
            schema_version: metadata.schema_version,
            source_signature: metadata.source_signature,
            embedding_model_id: metadata.embedding_model_id,
            embedding_dimensions: metadata.embedding_dimensions,
            missing_keys,
            legacy_schema_version: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArtifactMetadataSummary {
    pub artifact_contract_version: Option<String>,
    pub schema_version: Option<String>,
    pub source_signature: Option<String>,
    pub embedding_model_id: Option<String>,
    pub embedding_dimensions: Option<String>,
}
