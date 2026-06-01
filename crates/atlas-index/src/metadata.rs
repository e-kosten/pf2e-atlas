use std::collections::BTreeMap;
use std::path::Path;

use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_CONTENT_HASH_ALGORITHM,
    EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND, artifact_metadata_keys,
};
use atlas_embedding::{
    EMBEDDING_UNIT_POLICY_VERSION, embedding_model_for_model_id, embedding_model_spec,
    supported_embedding_model_ids,
};
use rusqlite::Connection;

use crate::{
    ArtifactMetadataSummary, ArtifactValidationDiagnostic, ArtifactValidationFamily,
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
    let sql = format!("SELECT key AS metadata_key, value AS metadata_value FROM {table}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("metadata_key")?,
                row.get::<_, String>("metadata_value")?,
            ))
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
        artifact_record_count: metadata
            .get(artifact_metadata_keys::ARTIFACT_RECORD_COUNT)
            .cloned(),
        generated_record_count: metadata
            .get(artifact_metadata_keys::GENERATED_RECORD_COUNT)
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
        embedding_unit_policy_version: metadata
            .get(artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION)
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
        ArtifactValidationFamily::Contract,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SCHEMA_VERSION,
        ARTIFACT_SCHEMA_VERSION,
        ValidationCode::UnsupportedSchemaVersion,
        ArtifactValidationFamily::Schema,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SOURCE_KIND,
        EXPECTED_SOURCE_KIND,
        ValidationCode::InvalidSourceMetadata,
        ArtifactValidationFamily::Source,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
        EXPECTED_CONTENT_HASH_ALGORITHM,
        ValidationCode::InvalidSourceMetadata,
        ArtifactValidationFamily::Source,
        &mut diagnostics,
    );
    require_positive_usize(
        metadata,
        artifact_metadata_keys::SOURCE_RECORD_COUNT,
        ValidationCode::InvalidSourceMetadata,
        ArtifactValidationFamily::Source,
        &mut diagnostics,
    );
    require_usize(
        metadata,
        artifact_metadata_keys::ARTIFACT_RECORD_COUNT,
        ValidationCode::InvalidSourceMetadata,
        ArtifactValidationFamily::Source,
        &mut diagnostics,
    );
    require_usize(
        metadata,
        artifact_metadata_keys::GENERATED_RECORD_COUNT,
        ValidationCode::InvalidSourceMetadata,
        ArtifactValidationFamily::Source,
        &mut diagnostics,
    );
    require_source_signature(metadata, &mut diagnostics);
    let Some(embedding_model) = metadata
        .get(artifact_metadata_keys::EMBEDDING_MODEL_ID)
        .and_then(|model_id| embedding_model_for_model_id(model_id))
    else {
        let actual = metadata
            .get(artifact_metadata_keys::EMBEDDING_MODEL_ID)
            .cloned()
            .unwrap_or_default();
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::EmbeddingMismatch,
            family: ArtifactValidationFamily::Embedding,
            message: format!(
                "metadata key `{}` has an unsupported value",
                artifact_metadata_keys::EMBEDDING_MODEL_ID
            ),
            key: Some(artifact_metadata_keys::EMBEDDING_MODEL_ID.to_string()),
            expected: Some(supported_embedding_model_ids().join(", ")),
            actual: Some(actual),
        });
        return diagnostics;
    };
    let embedding_spec = embedding_model_spec(embedding_model);
    let embedding_dimensions = embedding_spec.dimensions_string();
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
        embedding_spec.provider_family,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        embedding_spec.model_id,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
        embedding_spec.model_revision,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
        embedding_spec.tokenizer_id,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_POOLING,
        embedding_spec.pooling.as_str(),
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_NORMALIZATION,
        embedding_spec.normalization.as_str(),
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DIMENSIONS,
        &embedding_dimensions,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DTYPE,
        embedding_spec.dtype.as_str(),
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
        embedding_spec.distance_metric.as_str(),
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
        embedding_spec.document_prefix,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
        embedding_spec.query_prefix,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
        EMBEDDING_UNIT_POLICY_VERSION,
        ValidationCode::EmbeddingMismatch,
        ArtifactValidationFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::FTS_TOKENIZER,
        EXPECTED_FTS_TOKENIZER,
        ValidationCode::FtsMismatch,
        ArtifactValidationFamily::Fts,
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
    family: ArtifactValidationFamily,
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
    family: ArtifactValidationFamily,
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

fn require_usize(
    metadata: &BTreeMap<String, String>,
    key: &str,
    code: ValidationCode,
    family: ArtifactValidationFamily,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let value = metadata.get(key).map(String::as_str).unwrap_or_default();
    if value.parse::<usize>().is_err() {
        diagnostics.push(ArtifactValidationDiagnostic {
            code,
            family,
            message: format!("metadata key `{key}` must be a non-negative integer"),
            key: Some(key.to_string()),
            expected: Some("non-negative integer".to_string()),
            actual: Some(value.to_string()),
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
            family: ArtifactValidationFamily::Source,
            message: "source signature marks this artifact as stale".to_string(),
            key: Some(key.to_string()),
            expected: Some("current source signature".to_string()),
            actual: Some(actual.to_string()),
        });
    } else if !actual.starts_with("foundry-pf2e:") {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::InvalidSourceMetadata,
            family: ArtifactValidationFamily::Source,
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
            family: ArtifactValidationFamily::Manifest,
            message: "adjacent manifest path must be a relative artifact path".to_string(),
            key: Some(key.to_string()),
            expected: Some("relative path".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}
