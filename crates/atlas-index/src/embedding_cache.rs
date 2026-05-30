use std::collections::BTreeMap;

use atlas_artifact::metadata::{ARTIFACT_METADATA_TABLE, artifact_metadata_keys};
use atlas_artifact::storage::{decode_f32_vector_blob, f32_vector_blob_len};
use atlas_embedding::{
    EMBEDDING_UNIT_POLICY_VERSION, EmbeddingModelSpec, ReusableDocumentEmbedding,
};
use thiserror::Error;

use crate::AtlasIndex;

pub trait DocumentEmbeddingCacheReader {
    fn load_reusable_document_embeddings(
        &self,
        spec: EmbeddingModelSpec,
    ) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, DocumentEmbeddingCacheError>;
}

impl DocumentEmbeddingCacheReader for AtlasIndex {
    fn load_reusable_document_embeddings(
        &self,
        spec: EmbeddingModelSpec,
    ) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, DocumentEmbeddingCacheError> {
        validate_embedding_identity(self, spec)?;
        let mut select = self
            .connection()
            .prepare(
                "SELECT embedding_unit_key, semantic_input_hash, dimensions, vector_blob
                 FROM document_embedding_cache",
            )
            .map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))?;
        let rows = select
            .query_map([], |row| {
                let embedding_unit_key: String = row.get(0)?;
                let input_hash: String = row.get(1)?;
                let dimensions: i64 = row.get(2)?;
                let vector_blob: Vec<u8> = row.get(3)?;
                Ok((embedding_unit_key, input_hash, dimensions, vector_blob))
            })
            .map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))?;

        let expected_dimensions = spec.dimensions;
        let mut reusable = BTreeMap::new();
        for row in rows {
            let (embedding_unit_key, input_hash, dimensions, vector_blob) =
                row.map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))?;
            let dimensions = usize::try_from(dimensions).map_err(|_| {
                DocumentEmbeddingCacheError::InvalidCache(format!(
                    "cached embedding `{embedding_unit_key}` has invalid dimensions"
                ))
            })?;
            if dimensions != expected_dimensions {
                return Err(DocumentEmbeddingCacheError::InvalidCache(format!(
                    "cached embedding `{embedding_unit_key}` has {dimensions} dimensions; expected {expected_dimensions}"
                )));
            }
            let vector = decode_vector_blob(&embedding_unit_key, &vector_blob, dimensions)?;
            reusable.insert(
                embedding_unit_key,
                ReusableDocumentEmbedding {
                    input_hash,
                    dimensions,
                    vector,
                },
            );
        }
        Ok(reusable)
    }
}

#[derive(Debug, Error)]
pub enum DocumentEmbeddingCacheError {
    #[error("embedding cache query failed: {0}")]
    Query(String),
    #[error("embedding cache metadata mismatch: {0}")]
    MetadataMismatch(String),
    #[error("embedding cache is invalid: {0}")]
    InvalidCache(String),
}

fn validate_embedding_identity(
    index: &AtlasIndex,
    spec: EmbeddingModelSpec,
) -> Result<(), DocumentEmbeddingCacheError> {
    for (key, expected) in [
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            spec.provider_family.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            spec.model_id.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            spec.model_revision.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            spec.tokenizer_id.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            spec.pooling.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            spec.normalization.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DIMENSIONS,
            spec.dimensions.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            spec.dtype.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            spec.distance_metric.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            spec.document_prefix.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            spec.query_prefix.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
            EMBEDDING_UNIT_POLICY_VERSION.to_string(),
        ),
    ] {
        let actual = metadata_value(index, key)?;
        if actual.as_deref() != Some(expected.as_str()) {
            return Err(DocumentEmbeddingCacheError::MetadataMismatch(format!(
                "embedding metadata `{key}` is {}; expected {expected}",
                actual.unwrap_or_else(|| "missing".to_string())
            )));
        }
    }
    Ok(())
}

fn metadata_value(
    index: &AtlasIndex,
    key: &str,
) -> Result<Option<String>, DocumentEmbeddingCacheError> {
    let mut statement = index
        .connection()
        .prepare(&format!(
            "SELECT value FROM {ARTIFACT_METADATA_TABLE} WHERE key = ?1"
        ))
        .map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))?;
    match statement.query_row([key], |row| row.get(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(DocumentEmbeddingCacheError::Query(error.to_string())),
    }
}

fn decode_vector_blob(
    embedding_unit_key: &str,
    blob: &[u8],
    dimensions: usize,
) -> Result<Vec<f32>, DocumentEmbeddingCacheError> {
    let expected_bytes = f32_vector_blob_len(dimensions);
    if blob.len() != expected_bytes {
        return Err(DocumentEmbeddingCacheError::InvalidCache(format!(
            "cached embedding `{embedding_unit_key}` vector blob has {} bytes; expected {expected_bytes}",
            blob.len()
        )));
    }
    decode_f32_vector_blob(blob)
        .map_err(|error| DocumentEmbeddingCacheError::InvalidCache(error.to_string()))
}
