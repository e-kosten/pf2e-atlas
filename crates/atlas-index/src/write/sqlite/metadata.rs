use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_CONTENT_HASH_ALGORITHM,
    EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND, artifact_metadata_keys,
};
use atlas_embedding::{EMBEDDING_UNIT_POLICY_VERSION, EmbeddingModelId, embedding_model_spec};
use diesel::SqliteConnection;
use diesel::prelude::*;

use super::models::ArtifactMetadataRow;
use crate::IndexWriteError;

const ADJACENT_ARTIFACT_MANIFEST_PATH: &str = "manifest.json";

fn metadata_value(value: impl Into<String>) -> String {
    value.into()
}

pub(super) fn write_artifact_metadata(
    connection: &mut SqliteConnection,
    source_record_count: usize,
    artifact_record_count: usize,
    generated_record_count: usize,
    source_signature: &str,
    embedding_model: EmbeddingModelId,
) -> Result<(), IndexWriteError> {
    let embedding_spec = embedding_model_spec(embedding_model);
    let metadata = vec![
        ArtifactMetadataRow {
            key: artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION.to_string(),
            value: metadata_value(ARTIFACT_CONTRACT_VERSION),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::SCHEMA_VERSION.to_string(),
            value: metadata_value(ARTIFACT_SCHEMA_VERSION),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::SOURCE_KIND.to_string(),
            value: metadata_value(EXPECTED_SOURCE_KIND),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::SOURCE_SIGNATURE.to_string(),
            value: metadata_value(source_signature),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::SOURCE_RECORD_COUNT.to_string(),
            value: source_record_count.to_string(),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::ARTIFACT_RECORD_COUNT.to_string(),
            value: artifact_record_count.to_string(),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::GENERATED_RECORD_COUNT.to_string(),
            value: generated_record_count.to_string(),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::CONTENT_HASH_ALGORITHM.to_string(),
            value: metadata_value(EXPECTED_CONTENT_HASH_ALGORITHM),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY.to_string(),
            value: metadata_value(embedding_spec.provider_family),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_MODEL_ID.to_string(),
            value: metadata_value(embedding_spec.model_id),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_MODEL_REVISION.to_string(),
            value: metadata_value(embedding_spec.model_revision),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_TOKENIZER_ID.to_string(),
            value: metadata_value(embedding_spec.tokenizer_id),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_POOLING.to_string(),
            value: metadata_value(embedding_spec.pooling.as_str()),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_NORMALIZATION.to_string(),
            value: metadata_value(embedding_spec.normalization.as_str()),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_DIMENSIONS.to_string(),
            value: embedding_spec.dimensions_string(),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_DTYPE.to_string(),
            value: metadata_value(embedding_spec.dtype.as_str()),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC.to_string(),
            value: metadata_value(embedding_spec.distance_metric.as_str()),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX.to_string(),
            value: metadata_value(embedding_spec.document_prefix),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_QUERY_PREFIX.to_string(),
            value: metadata_value(embedding_spec.query_prefix),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION.to_string(),
            value: metadata_value(EMBEDDING_UNIT_POLICY_VERSION),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::FTS_TOKENIZER.to_string(),
            value: metadata_value(EXPECTED_FTS_TOKENIZER),
        },
        ArtifactMetadataRow {
            key: artifact_metadata_keys::ADJACENT_MANIFEST_PATH.to_string(),
            value: metadata_value(ADJACENT_ARTIFACT_MANIFEST_PATH),
        },
    ];

    diesel::insert_into(crate::schema::artifact_metadata::table)
        .values(&metadata)
        .execute(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    Ok(())
}
