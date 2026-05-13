use atlas_artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION,
    EXPECTED_CONTENT_HASH_ALGORITHM, EXPECTED_EMBEDDING_DIMENSIONS,
    EXPECTED_EMBEDDING_DISTANCE_METRIC, EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
    EXPECTED_EMBEDDING_DTYPE, EXPECTED_EMBEDDING_MODEL_ID, EXPECTED_EMBEDDING_MODEL_REVISION,
    EXPECTED_EMBEDDING_NORMALIZATION, EXPECTED_EMBEDDING_POOLING,
    EXPECTED_EMBEDDING_PROVIDER_FAMILY, EXPECTED_EMBEDDING_QUERY_PREFIX,
    EXPECTED_EMBEDDING_TOKENIZER_ID, EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND,
    artifact_metadata_keys,
};
use rusqlite::Connection;

use crate::IngestError;

pub(super) fn write_artifact_metadata(
    connection: &Connection,
    source_record_count: usize,
    artifact_record_count: usize,
    generated_record_count: usize,
    source_signature: &str,
) -> Result<(), IngestError> {
    let metadata = [
        (
            artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
            ARTIFACT_CONTRACT_VERSION.to_string(),
        ),
        (
            artifact_metadata_keys::SCHEMA_VERSION,
            ARTIFACT_SCHEMA_VERSION.to_string(),
        ),
        (
            artifact_metadata_keys::SOURCE_KIND,
            EXPECTED_SOURCE_KIND.to_string(),
        ),
        (
            artifact_metadata_keys::SOURCE_SIGNATURE,
            source_signature.to_string(),
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
            EXPECTED_CONTENT_HASH_ALGORITHM.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            EXPECTED_EMBEDDING_PROVIDER_FAMILY.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            EXPECTED_EMBEDDING_MODEL_ID.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            EXPECTED_EMBEDDING_MODEL_REVISION.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            EXPECTED_EMBEDDING_TOKENIZER_ID.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            EXPECTED_EMBEDDING_POOLING.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            EXPECTED_EMBEDDING_NORMALIZATION.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DIMENSIONS,
            EXPECTED_EMBEDDING_DIMENSIONS.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            EXPECTED_EMBEDDING_DTYPE.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            EXPECTED_EMBEDDING_DISTANCE_METRIC.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            EXPECTED_EMBEDDING_DOCUMENT_PREFIX.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            EXPECTED_EMBEDDING_QUERY_PREFIX.to_string(),
        ),
        (
            artifact_metadata_keys::FTS_TOKENIZER,
            EXPECTED_FTS_TOKENIZER.to_string(),
        ),
        (
            artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
            "manifest.json".to_string(),
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
