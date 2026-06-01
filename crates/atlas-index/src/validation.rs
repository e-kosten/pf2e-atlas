use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTarget {
    Full,
    BaseOnly,
    EmbeddingsOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationCode {
    Ok,
    IndexUnavailable,
    MissingArtifactMetadata,
    MissingRequiredMetadata,
    UnsupportedContractVersion,
    UnsupportedSchemaVersion,
    ArtifactContractViolation,
    InvalidSourceMetadata,
    StaleSourceSignature,
    EmbeddingMismatch,
    FtsMismatch,
    ManifestMismatch,
    VectorExtensionUnavailable,
    QueryFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactValidationDiagnostic {
    pub code: ValidationCode,
    pub family: ArtifactValidationFamily,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactValidationFamily {
    Contract,
    Schema,
    Source,
    Data,
    Embedding,
    Fts,
    Manifest,
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
    pub source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_record_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_record_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_record_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash_algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_provider_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_tokenizer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_pooling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_normalization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dimensions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_distance_metric: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_document_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_query_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_unit_policy_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fts_tokenizer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacent_manifest_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_keys: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<ArtifactValidationDiagnostic>,
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
            artifact_contract_version: metadata.artifact_contract_version.clone(),
            schema_version: metadata.schema_version.clone(),
            source_kind: metadata.source_kind.clone(),
            source_signature: metadata.source_signature.clone(),
            source_record_count: metadata.source_record_count.clone(),
            artifact_record_count: metadata.artifact_record_count.clone(),
            generated_record_count: metadata.generated_record_count.clone(),
            content_hash_algorithm: metadata.content_hash_algorithm.clone(),
            embedding_provider_family: metadata.embedding_provider_family.clone(),
            embedding_model_id: metadata.embedding_model_id.clone(),
            embedding_model_revision: metadata.embedding_model_revision.clone(),
            embedding_tokenizer_id: metadata.embedding_tokenizer_id.clone(),
            embedding_pooling: metadata.embedding_pooling.clone(),
            embedding_normalization: metadata.embedding_normalization.clone(),
            embedding_dimensions: metadata.embedding_dimensions.clone(),
            embedding_dtype: metadata.embedding_dtype.clone(),
            embedding_distance_metric: metadata.embedding_distance_metric.clone(),
            embedding_document_prefix: metadata.embedding_document_prefix.clone(),
            embedding_query_prefix: metadata.embedding_query_prefix.clone(),
            embedding_unit_policy_version: metadata.embedding_unit_policy_version.clone(),
            fts_tokenizer: metadata.fts_tokenizer.clone(),
            adjacent_manifest_path: metadata.adjacent_manifest_path.clone(),
            missing_keys: Vec::new(),
            diagnostics: Vec::new(),
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
            artifact_contract_version: metadata.artifact_contract_version.clone(),
            schema_version: metadata.schema_version.clone(),
            source_kind: metadata.source_kind.clone(),
            source_signature: metadata.source_signature.clone(),
            source_record_count: metadata.source_record_count.clone(),
            artifact_record_count: metadata.artifact_record_count.clone(),
            generated_record_count: metadata.generated_record_count.clone(),
            content_hash_algorithm: metadata.content_hash_algorithm.clone(),
            embedding_provider_family: metadata.embedding_provider_family.clone(),
            embedding_model_id: metadata.embedding_model_id.clone(),
            embedding_model_revision: metadata.embedding_model_revision.clone(),
            embedding_tokenizer_id: metadata.embedding_tokenizer_id.clone(),
            embedding_pooling: metadata.embedding_pooling.clone(),
            embedding_normalization: metadata.embedding_normalization.clone(),
            embedding_dimensions: metadata.embedding_dimensions.clone(),
            embedding_dtype: metadata.embedding_dtype.clone(),
            embedding_distance_metric: metadata.embedding_distance_metric.clone(),
            embedding_document_prefix: metadata.embedding_document_prefix.clone(),
            embedding_query_prefix: metadata.embedding_query_prefix.clone(),
            embedding_unit_policy_version: metadata.embedding_unit_policy_version.clone(),
            fts_tokenizer: metadata.fts_tokenizer.clone(),
            adjacent_manifest_path: metadata.adjacent_manifest_path.clone(),
            missing_keys,
            diagnostics: Vec::new(),
            legacy_schema_version: None,
        }
    }

    pub fn incompatible_metadata(
        index: String,
        metadata: ArtifactMetadataSummary,
        diagnostics: Vec<ArtifactValidationDiagnostic>,
    ) -> Self {
        let code = diagnostics
            .first()
            .map(|diagnostic| diagnostic.code.clone())
            .unwrap_or(ValidationCode::QueryFailed);
        Self {
            status: ValidationStatus::Error,
            code,
            index,
            message: "artifact metadata is incompatible with this runtime".to_string(),
            artifact_contract_version: metadata.artifact_contract_version.clone(),
            schema_version: metadata.schema_version.clone(),
            source_kind: metadata.source_kind.clone(),
            source_signature: metadata.source_signature.clone(),
            source_record_count: metadata.source_record_count.clone(),
            artifact_record_count: metadata.artifact_record_count.clone(),
            generated_record_count: metadata.generated_record_count.clone(),
            content_hash_algorithm: metadata.content_hash_algorithm.clone(),
            embedding_provider_family: metadata.embedding_provider_family.clone(),
            embedding_model_id: metadata.embedding_model_id.clone(),
            embedding_model_revision: metadata.embedding_model_revision.clone(),
            embedding_tokenizer_id: metadata.embedding_tokenizer_id.clone(),
            embedding_pooling: metadata.embedding_pooling.clone(),
            embedding_normalization: metadata.embedding_normalization.clone(),
            embedding_dimensions: metadata.embedding_dimensions.clone(),
            embedding_dtype: metadata.embedding_dtype.clone(),
            embedding_distance_metric: metadata.embedding_distance_metric.clone(),
            embedding_document_prefix: metadata.embedding_document_prefix.clone(),
            embedding_query_prefix: metadata.embedding_query_prefix.clone(),
            embedding_unit_policy_version: metadata.embedding_unit_policy_version.clone(),
            fts_tokenizer: metadata.fts_tokenizer.clone(),
            adjacent_manifest_path: metadata.adjacent_manifest_path.clone(),
            missing_keys: Vec::new(),
            diagnostics,
            legacy_schema_version: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArtifactMetadataSummary {
    pub artifact_contract_version: Option<String>,
    pub schema_version: Option<String>,
    pub source_kind: Option<String>,
    pub source_signature: Option<String>,
    pub source_record_count: Option<String>,
    pub artifact_record_count: Option<String>,
    pub generated_record_count: Option<String>,
    pub content_hash_algorithm: Option<String>,
    pub embedding_provider_family: Option<String>,
    pub embedding_model_id: Option<String>,
    pub embedding_model_revision: Option<String>,
    pub embedding_tokenizer_id: Option<String>,
    pub embedding_pooling: Option<String>,
    pub embedding_normalization: Option<String>,
    pub embedding_dimensions: Option<String>,
    pub embedding_dtype: Option<String>,
    pub embedding_distance_metric: Option<String>,
    pub embedding_document_prefix: Option<String>,
    pub embedding_query_prefix: Option<String>,
    pub embedding_unit_policy_version: Option<String>,
    pub fts_tokenizer: Option<String>,
    pub adjacent_manifest_path: Option<String>,
}
