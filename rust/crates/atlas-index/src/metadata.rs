use std::collections::BTreeMap;
use std::path::Path;

use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_CONTENT_HASH_ALGORITHM,
    EXPECTED_EMBEDDING_DIMENSIONS, EXPECTED_EMBEDDING_DISTANCE_METRIC,
    EXPECTED_EMBEDDING_DOCUMENT_PREFIX, EXPECTED_EMBEDDING_DTYPE, EXPECTED_EMBEDDING_MODEL_ID,
    EXPECTED_EMBEDDING_MODEL_REVISION, EXPECTED_EMBEDDING_NORMALIZATION,
    EXPECTED_EMBEDDING_POOLING, EXPECTED_EMBEDDING_PROVIDER_FAMILY,
    EXPECTED_EMBEDDING_QUERY_PREFIX, EXPECTED_EMBEDDING_TOKENIZER_ID, EXPECTED_FTS_TOKENIZER,
    EXPECTED_SOURCE_KIND, artifact_metadata_keys,
};
use rusqlite::Connection;

use crate::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationDiagnostic,
    IndexValidationError, ValidationCode,
};

pub(crate) fn is_missing_value(key: &str, value: &str) -> bool {
    if matches!(
        key,
        artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX
            | artifact_metadata_keys::EMBEDDING_QUERY_PREFIX
    ) {
        false
    } else {
        value.trim().is_empty()
    }
}

pub(crate) fn read_metadata(
    connection: &Connection,
    table: &str,
) -> Result<BTreeMap<String, String>, IndexValidationError> {
    let sql = format!("SELECT key, value FROM {table}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;

    let mut metadata = BTreeMap::new();
    for row in rows {
        let (key, value) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        metadata.insert(key, value);
    }
    Ok(metadata)
}

pub(crate) fn summarize_metadata(metadata: &BTreeMap<String, String>) -> ArtifactMetadataSummary {
    ArtifactMetadataSummary {
        artifact_contract_version: metadata
            .get(artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION)
            .cloned(),
        schema_version: metadata
            .get(artifact_metadata_keys::SCHEMA_VERSION)
            .cloned(),
        source_kind: metadata.get(artifact_metadata_keys::SOURCE_KIND).cloned(),
        source_signature: metadata
            .get(artifact_metadata_keys::SOURCE_SIGNATURE)
            .cloned(),
        source_record_count: metadata
            .get(artifact_metadata_keys::SOURCE_RECORD_COUNT)
            .cloned(),
        content_hash_algorithm: metadata
            .get(artifact_metadata_keys::CONTENT_HASH_ALGORITHM)
            .cloned(),
        embedding_provider_family: metadata
            .get(artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY)
            .cloned(),
        embedding_model_id: metadata
            .get(artifact_metadata_keys::EMBEDDING_MODEL_ID)
            .cloned(),
        embedding_model_revision: metadata
            .get(artifact_metadata_keys::EMBEDDING_MODEL_REVISION)
            .cloned(),
        embedding_tokenizer_id: metadata
            .get(artifact_metadata_keys::EMBEDDING_TOKENIZER_ID)
            .cloned(),
        embedding_pooling: metadata
            .get(artifact_metadata_keys::EMBEDDING_POOLING)
            .cloned(),
        embedding_normalization: metadata
            .get(artifact_metadata_keys::EMBEDDING_NORMALIZATION)
            .cloned(),
        embedding_dimensions: metadata
            .get(artifact_metadata_keys::EMBEDDING_DIMENSIONS)
            .cloned(),
        embedding_dtype: metadata
            .get(artifact_metadata_keys::EMBEDDING_DTYPE)
            .cloned(),
        embedding_distance_metric: metadata
            .get(artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC)
            .cloned(),
        embedding_document_prefix: metadata
            .get(artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX)
            .cloned(),
        embedding_query_prefix: metadata
            .get(artifact_metadata_keys::EMBEDDING_QUERY_PREFIX)
            .cloned(),
        fts_tokenizer: metadata.get(artifact_metadata_keys::FTS_TOKENIZER).cloned(),
        adjacent_manifest_path: metadata
            .get(artifact_metadata_keys::ADJACENT_MANIFEST_PATH)
            .cloned(),
    }
}

pub(crate) fn validate_metadata_values(
    metadata: &BTreeMap<String, String>,
) -> Vec<ArtifactValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    require_value(
        metadata,
        artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
        ARTIFACT_CONTRACT_VERSION,
        ValidationCode::UnsupportedContractVersion,
        ArtifactContractFamily::Contract,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SCHEMA_VERSION,
        ARTIFACT_SCHEMA_VERSION,
        ValidationCode::UnsupportedSchemaVersion,
        ArtifactContractFamily::Schema,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SOURCE_KIND,
        EXPECTED_SOURCE_KIND,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
        EXPECTED_CONTENT_HASH_ALGORITHM,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_positive_usize(
        metadata,
        artifact_metadata_keys::SOURCE_RECORD_COUNT,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_source_signature(metadata, &mut diagnostics);
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
        EXPECTED_EMBEDDING_PROVIDER_FAMILY,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        EXPECTED_EMBEDDING_MODEL_ID,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
        EXPECTED_EMBEDDING_MODEL_REVISION,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
        EXPECTED_EMBEDDING_TOKENIZER_ID,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_POOLING,
        EXPECTED_EMBEDDING_POOLING,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_NORMALIZATION,
        EXPECTED_EMBEDDING_NORMALIZATION,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DIMENSIONS,
        EXPECTED_EMBEDDING_DIMENSIONS,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DTYPE,
        EXPECTED_EMBEDDING_DTYPE,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
        EXPECTED_EMBEDDING_DISTANCE_METRIC,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
        EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
        EXPECTED_EMBEDDING_QUERY_PREFIX,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::FTS_TOKENIZER,
        EXPECTED_FTS_TOKENIZER,
        ValidationCode::FtsMismatch,
        ArtifactContractFamily::Fts,
        &mut diagnostics,
    );
    require_manifest_path(metadata, &mut diagnostics);
    diagnostics
}

fn require_value(
    metadata: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
    code: ValidationCode,
    family: ArtifactContractFamily,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual != expected {
        diagnostics.push(ArtifactValidationDiagnostic {
            code,
            family,
            message: format!("metadata key `{key}` has an unsupported value"),
            key: Some(key.to_string()),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_positive_usize(
    metadata: &BTreeMap<String, String>,
    key: &str,
    code: ValidationCode,
    family: ArtifactContractFamily,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    let is_positive = actual
        .parse::<usize>()
        .map(|value| value > 0)
        .unwrap_or(false);
    if !is_positive {
        diagnostics.push(ArtifactValidationDiagnostic {
            code,
            family,
            message: format!("metadata key `{key}` must be a positive integer"),
            key: Some(key.to_string()),
            expected: Some("positive integer".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_source_signature(
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let key = artifact_metadata_keys::SOURCE_SIGNATURE;
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual == "stale" || actual.starts_with("stale:") {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::StaleSourceSignature,
            family: ArtifactContractFamily::Source,
            message: "source signature marks this artifact as stale".to_string(),
            key: Some(key.to_string()),
            expected: Some("current source signature".to_string()),
            actual: Some(actual.to_string()),
        });
    } else if !actual.starts_with("foundry-pf2e:") {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::InvalidSourceMetadata,
            family: ArtifactContractFamily::Source,
            message: "source signature must identify a Foundry PF2E source snapshot".to_string(),
            key: Some(key.to_string()),
            expected: Some("foundry-pf2e:<signature>".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_manifest_path(
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let key = artifact_metadata_keys::ADJACENT_MANIFEST_PATH;
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual.trim().is_empty() || actual.contains('\0') || Path::new(actual).is_absolute() {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::ManifestMismatch,
            family: ArtifactContractFamily::Manifest,
            message: "adjacent manifest path must be a relative artifact path".to_string(),
            key: Some(key.to_string()),
            expected: Some("relative path".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}
