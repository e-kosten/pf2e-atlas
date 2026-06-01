use std::collections::BTreeMap;

use crate::artifact::metadata::artifact_metadata_keys;
use crate::artifact::storage::{decode_f32_vector_blob, f32_vector_blob_len};
use atlas_embedding::{
    EMBEDDING_UNIT_POLICY_VERSION, EmbeddingModelSpec, ReusableDocumentEmbedding,
};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::{Queryable, Selectable, SelectableHelper};
use thiserror::Error;

use crate::SqliteIndexReader;
use crate::schema::{artifact_metadata, document_embedding_cache};

pub trait DocumentEmbeddingCacheReader {
    fn load_reusable_document_embeddings(
        &self,
        spec: EmbeddingModelSpec,
    ) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, DocumentEmbeddingCacheError>;
}

impl DocumentEmbeddingCacheReader for SqliteIndexReader {
    fn load_reusable_document_embeddings(
        &self,
        spec: EmbeddingModelSpec,
    ) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, DocumentEmbeddingCacheError> {
        validate_embedding_identity(self, spec)?;
        let rows = self
            .with_diesel_connection(|connection| {
                document_embedding_cache::table
                    .select(ReusableEmbeddingRow::as_select())
                    .load::<ReusableEmbeddingRow>(connection)
            })
            .map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))?;

        let expected_dimensions = spec.dimensions;
        let mut reusable = BTreeMap::new();
        for row in rows {
            let dimensions = usize::try_from(row.dimensions).map_err(|_| {
                DocumentEmbeddingCacheError::InvalidCache(format!(
                    "cached embedding `{}` has invalid dimensions",
                    row.embedding_unit_key
                ))
            })?;
            if dimensions != expected_dimensions {
                return Err(DocumentEmbeddingCacheError::InvalidCache(format!(
                    "cached embedding `{}` has {dimensions} dimensions; expected {expected_dimensions}",
                    row.embedding_unit_key
                )));
            }
            let vector = decode_vector_blob(&row.embedding_unit_key, &row.vector_blob, dimensions)?;
            reusable.insert(
                row.embedding_unit_key,
                ReusableDocumentEmbedding {
                    input_hash: row.semantic_input_hash,
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
    index: &SqliteIndexReader,
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
    index: &SqliteIndexReader,
    key: &str,
) -> Result<Option<String>, DocumentEmbeddingCacheError> {
    index
        .with_diesel_connection(|connection| {
            artifact_metadata::table
                .filter(artifact_metadata::key.eq(key))
                .select(MetadataValueRow::as_select())
                .get_result::<MetadataValueRow>(connection)
                .optional()
        })
        .map(|row| row.map(|row| row.value))
        .map_err(|error| DocumentEmbeddingCacheError::Query(error.to_string()))
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = document_embedding_cache)]
#[diesel(check_for_backend(Sqlite))]
struct ReusableEmbeddingRow {
    embedding_unit_key: String,
    semantic_input_hash: String,
    dimensions: i64,
    vector_blob: Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = artifact_metadata)]
#[diesel(check_for_backend(Sqlite))]
struct MetadataValueRow {
    value: String,
}
