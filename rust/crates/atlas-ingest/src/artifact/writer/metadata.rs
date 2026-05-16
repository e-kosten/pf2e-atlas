use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION,
    EXPECTED_CONTENT_HASH_ALGORITHM, EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND,
    artifact_metadata_keys,
};
use atlas_embedding::{EmbeddingModelId, embedding_model_spec};
use rusqlite::Connection;

use crate::error::IngestError;

fn metadata_value(value: impl Into<String>) -> String {
    value.into()
}

pub(super) fn write_artifact_metadata(
    connection: &Connection,
    source_record_count: usize,
    artifact_record_count: usize,
    generated_record_count: usize,
    source_signature: &str,
    embedding_model: EmbeddingModelId,
) -> Result<(), IngestError> {
    let embedding_spec = embedding_model_spec(embedding_model);
    let metadata = [
        (
            artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
            metadata_value(ARTIFACT_CONTRACT_VERSION),
        ),
        (
            artifact_metadata_keys::SCHEMA_VERSION,
            metadata_value(ARTIFACT_SCHEMA_VERSION),
        ),
        (
            artifact_metadata_keys::SOURCE_KIND,
            metadata_value(EXPECTED_SOURCE_KIND),
        ),
        (
            artifact_metadata_keys::SOURCE_SIGNATURE,
            metadata_value(source_signature),
        ),
        (
            artifact_metadata_keys::SOURCE_RECORD_COUNT,
            source_record_count.to_string(),
        ),
        (
            artifact_metadata_keys::ARTIFACT_RECORD_COUNT,
            artifact_record_count.to_string(),
        ),
        (
            artifact_metadata_keys::GENERATED_RECORD_COUNT,
            generated_record_count.to_string(),
        ),
        (
            artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
            metadata_value(EXPECTED_CONTENT_HASH_ALGORITHM),
        ),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            metadata_value(embedding_spec.provider_family),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            metadata_value(embedding_spec.model_id),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            metadata_value(embedding_spec.model_revision),
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            metadata_value(embedding_spec.tokenizer_id),
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            metadata_value(embedding_spec.pooling.as_str()),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            metadata_value(embedding_spec.normalization.as_str()),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DIMENSIONS,
            embedding_spec.dimensions_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            metadata_value(embedding_spec.dtype.as_str()),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            metadata_value(embedding_spec.distance_metric.as_str()),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            metadata_value(embedding_spec.document_prefix),
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            metadata_value(embedding_spec.query_prefix),
        ),
        (
            artifact_metadata_keys::FTS_TOKENIZER,
            metadata_value(EXPECTED_FTS_TOKENIZER),
        ),
        (
            artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
            metadata_value("manifest.json"),
        ),
    ];

    for (key, value) in metadata {
        connection
            .execute(
                &format!("INSERT INTO {ARTIFACT_METADATA_TABLE} (key, value) VALUES (?1, ?2)"),
                (key, value),
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}
