#![deny(unsafe_code)]

use serde::Serialize;

pub mod categories;
pub mod detail;
pub mod metadata;
pub mod record;
pub mod record_key;
pub mod rule_graph;
pub mod search_request;

pub use categories::{Category, Subcategory};
pub use detail::DetailLevel;
pub use metadata::{
    BooleanOperator, CollectionOperator, EqualityOperator, MetadataBooleanField,
    MetadataEnumStringField, MetadataNumberField, MetadataPredicate, MetadataSetField,
    MetadataTextStringField, MetricOperator, NullOperator, NumberOperator, NumericMetricOperator,
    OrderingOperator, StringOperator, TextOperator,
};
pub use record::{
    ActionCost, Level, Publication, Rarity, RecordSummary, SourceCategory, SourceProvenance,
    TextStatus,
};
pub use record_key::{PackName, RecordId, RecordKey, RecordKeyParseError};
pub use rule_graph::{
    ReferenceDirection, ReferenceEdge, ReferenceRelationship, ReferenceSource, RemasterLink,
    RemasterLinkSource, RuleContextRequest, RuleContextResult, RuleGraphCollectionResult,
    RuleGraphRequest, RuleGraphResult,
};
pub use search_request::{
    BrowseSortSpec, LookupSortKind, LookupSortPolicy, LookupSortSpec, NullableNumericMatch,
    NullableStringMatch, NumericMatch, ScalarValue, ScopeSubcategoryMatch, SearchFilterNode,
    SearchProfile, SearchRequest,
};

pub const ARTIFACT_METADATA_TABLE: &str = "artifact_metadata";
pub const LEGACY_METADATA_TABLE: &str = "metadata";
pub const ARTIFACT_CONTRACT_VERSION: &str = "pf2e-atlas-artifact/v1";
pub const ARTIFACT_SCHEMA_VERSION: &str = "1";

pub mod artifact_metadata_keys {
    pub const ADJACENT_MANIFEST_PATH: &str = "adjacent_manifest_path";
    pub const ARTIFACT_CONTRACT_VERSION: &str = "artifact_contract_version";
    pub const CONTENT_HASH_ALGORITHM: &str = "content_hash_algorithm";
    pub const EMBEDDING_DIMENSIONS: &str = "embedding_dimensions";
    pub const EMBEDDING_DISTANCE_METRIC: &str = "embedding_distance_metric";
    pub const EMBEDDING_DOCUMENT_PREFIX: &str = "embedding_document_prefix";
    pub const EMBEDDING_DTYPE: &str = "embedding_dtype";
    pub const EMBEDDING_MODEL_ID: &str = "embedding_model_id";
    pub const EMBEDDING_MODEL_REVISION: &str = "embedding_model_revision";
    pub const EMBEDDING_NORMALIZATION: &str = "embedding_normalization";
    pub const EMBEDDING_POOLING: &str = "embedding_pooling";
    pub const EMBEDDING_PROVIDER_FAMILY: &str = "embedding_provider_family";
    pub const EMBEDDING_QUERY_PREFIX: &str = "embedding_query_prefix";
    pub const EMBEDDING_TOKENIZER_ID: &str = "embedding_tokenizer_id";
    pub const FTS_TOKENIZER: &str = "fts_tokenizer";
    pub const SCHEMA_VERSION: &str = "schema_version";
    pub const SOURCE_KIND: &str = "source_kind";
    pub const SOURCE_RECORD_COUNT: &str = "source_record_count";
    pub const SOURCE_SIGNATURE: &str = "source_signature";
}

pub const REQUIRED_ARTIFACT_METADATA_KEYS: &[&str] = &[
    artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
    artifact_metadata_keys::SCHEMA_VERSION,
    artifact_metadata_keys::SOURCE_KIND,
    artifact_metadata_keys::SOURCE_SIGNATURE,
    artifact_metadata_keys::SOURCE_RECORD_COUNT,
    artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
    artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
    artifact_metadata_keys::EMBEDDING_MODEL_ID,
    artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
    artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
    artifact_metadata_keys::EMBEDDING_POOLING,
    artifact_metadata_keys::EMBEDDING_NORMALIZATION,
    artifact_metadata_keys::EMBEDDING_DIMENSIONS,
    artifact_metadata_keys::EMBEDDING_DTYPE,
    artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
    artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
    artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
    artifact_metadata_keys::FTS_TOKENIZER,
    artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
];

pub const EXPECTED_SOURCE_KIND: &str = "foundry-pf2e";
pub const EXPECTED_CONTENT_HASH_ALGORITHM: &str = "sha256";
pub const EXPECTED_EMBEDDING_PROVIDER_FAMILY: &str = "transformers-js-minilm";
pub const EXPECTED_EMBEDDING_MODEL_ID: &str = "Xenova/all-MiniLM-L12-v2";
pub const EXPECTED_EMBEDDING_MODEL_REVISION: &str = "main";
pub const EXPECTED_EMBEDDING_TOKENIZER_ID: &str = "Xenova/all-MiniLM-L12-v2";
pub const EXPECTED_EMBEDDING_POOLING: &str = "mean";
pub const EXPECTED_EMBEDDING_NORMALIZATION: &str = "l2";
pub const EXPECTED_EMBEDDING_DIMENSIONS: &str = "384";
pub const EXPECTED_EMBEDDING_DTYPE: &str = "f32";
pub const EXPECTED_EMBEDDING_DISTANCE_METRIC: &str = "cosine";
pub const EXPECTED_EMBEDDING_DOCUMENT_PREFIX: &str = "";
pub const EXPECTED_EMBEDDING_QUERY_PREFIX: &str = "";
pub const EXPECTED_FTS_TOKENIZER: &str = "unicode61 remove_diacritics 2";

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
    UnsupportedContractVersion,
    UnsupportedSchemaVersion,
    InvalidSourceMetadata,
    StaleSourceSignature,
    EmbeddingMismatch,
    FtsMismatch,
    ManifestMismatch,
    QueryFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactValidationDiagnostic {
    pub code: ValidationCode,
    pub family: ArtifactContractFamily,
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
pub enum ArtifactContractFamily {
    Contract,
    Schema,
    Source,
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
    pub fts_tokenizer: Option<String>,
    pub adjacent_manifest_path: Option<String>,
}
