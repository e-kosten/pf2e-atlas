use std::collections::BTreeMap;
use std::path::Path;

use atlas_artifact::metadata::{ARTIFACT_METADATA_TABLE, artifact_metadata_keys};
use atlas_embedding::{EmbeddingModelSpec, EmbeddingRuntimeConfig, ReusableDocumentEmbedding};
use rusqlite::{Connection, OpenFlags};

pub(crate) fn load_reusable_document_embeddings(
    path: &Path,
    config: &EmbeddingRuntimeConfig,
) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| error.to_string())?;
    validate_embedding_identity(&connection, config.model_spec())?;
    let mut select = connection
        .prepare(
            "SELECT embedding_unit_key, semantic_input_hash, dimensions, vector_blob
             FROM document_embedding_cache",
        )
        .map_err(|error| error.to_string())?;
    let rows = select
        .query_map([], |row| {
            let embedding_unit_key: String = row.get(0)?;
            let input_hash: String = row.get(1)?;
            let dimensions: i64 = row.get(2)?;
            let vector_blob: Vec<u8> = row.get(3)?;
            Ok((embedding_unit_key, input_hash, dimensions, vector_blob))
        })
        .map_err(|error| error.to_string())?;

    let expected_dimensions = config.model_spec().dimensions;
    let mut reusable = BTreeMap::new();
    for row in rows {
        let (embedding_unit_key, input_hash, dimensions, vector_blob) =
            row.map_err(|error| error.to_string())?;
        let dimensions = usize::try_from(dimensions).map_err(|_| {
            format!("cached embedding `{embedding_unit_key}` has invalid dimensions")
        })?;
        if dimensions != expected_dimensions {
            return Err(format!(
                "cached embedding `{embedding_unit_key}` has {dimensions} dimensions; expected {expected_dimensions}"
            ));
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

fn validate_embedding_identity(
    connection: &Connection,
    spec: EmbeddingModelSpec,
) -> Result<(), String> {
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
    ] {
        let actual = metadata_value(connection, key)?;
        if actual.as_deref() != Some(expected.as_str()) {
            return Err(format!(
                "embedding metadata `{key}` is {}; expected {expected}",
                actual.unwrap_or_else(|| "missing".to_string())
            ));
        }
    }
    Ok(())
}

fn metadata_value(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut statement = connection
        .prepare(&format!(
            "SELECT value FROM {ARTIFACT_METADATA_TABLE} WHERE key = ?1"
        ))
        .map_err(|error| error.to_string())?;
    match statement.query_row([key], |row| row.get(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn decode_vector_blob(
    record_key: &str,
    blob: &[u8],
    dimensions: usize,
) -> Result<Vec<f32>, String> {
    let expected_bytes = dimensions * std::mem::size_of::<f32>();
    if blob.len() != expected_bytes {
        return Err(format!(
            "cached embedding `{record_key}` vector blob has {} bytes; expected {expected_bytes}",
            blob.len()
        ));
    }
    let mut vector = Vec::with_capacity(dimensions);
    for chunk in blob.chunks_exact(std::mem::size_of::<f32>()) {
        vector.push(f32::from_le_bytes(
            chunk
                .try_into()
                .expect("chunks_exact produced four-byte chunks"),
        ));
    }
    Ok(vector)
}
