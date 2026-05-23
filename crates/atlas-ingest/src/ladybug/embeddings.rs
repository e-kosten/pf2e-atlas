use std::collections::BTreeSet;
use std::path::Path;

use atlas_artifact::storage::decode_f32_vector_blob;
use rusqlite::OpenFlags;
use tracing::info;

use crate::error::IngestError;
use crate::source::SourceLoad;

#[derive(Debug, Clone)]
pub(crate) struct LadybugEmbedding {
    pub(crate) embedding_unit_key: String,
    pub(crate) record_key: String,
    pub(crate) unit_kind: String,
    pub(crate) label: Option<String>,
    pub(crate) ordinal: i64,
    pub(crate) semantic_input_hash: String,
    pub(crate) dimensions: usize,
    pub(crate) vector: Vec<f32>,
}

pub(crate) fn ladybug_embeddings(
    source: &SourceLoad,
) -> Result<Vec<LadybugEmbedding>, IngestError> {
    if !source.document_embeddings.is_empty() {
        return Ok(source
            .document_embeddings
            .iter()
            .map(|embedding| LadybugEmbedding {
                embedding_unit_key: embedding.embedding_unit_key.clone(),
                record_key: embedding.record_key.clone(),
                unit_kind: embedding.unit_kind.as_str().to_string(),
                label: embedding.label.clone(),
                ordinal: embedding.ordinal as i64,
                semantic_input_hash: embedding.input_hash.clone(),
                dimensions: embedding.dimensions,
                vector: embedding.vector.clone(),
            })
            .collect());
    }

    let Some(path) = std::env::var_os("ATLAS_LADYBUG_LEGACY_EMBEDDINGS_SQLITE") else {
        return Ok(Vec::new());
    };
    load_legacy_sqlite_embeddings(Path::new(&path), source)
}

fn load_legacy_sqlite_embeddings(
    path: &Path,
    source: &SourceLoad,
) -> Result<Vec<LadybugEmbedding>, IngestError> {
    let source_record_keys = source
        .records
        .iter()
        .map(|loaded| loaded.record.key.to_string())
        .collect::<BTreeSet<_>>();
    let connection = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut select = connection
        .prepare(
            "SELECT record_key, dimensions, semantic_input_hash, vector_blob
             FROM embeddings
             ORDER BY record_key",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let rows = select
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut embeddings = Vec::new();
    for row in rows {
        let (record_key, dimensions, semantic_input_hash, vector_blob) =
            row.map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        if !source_record_keys.contains(&record_key) {
            continue;
        }
        let dimensions = usize::try_from(dimensions).map_err(|_| {
            IngestError::ArtifactWriteFailed(format!(
                "legacy embedding `{record_key}` has invalid dimensions"
            ))
        })?;
        let vector = decode_f32_vector_blob(&vector_blob)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        if vector.len() != dimensions {
            return Err(IngestError::ArtifactWriteFailed(format!(
                "legacy embedding `{record_key}` vector has {} dimensions; expected {dimensions}",
                vector.len()
            )));
        }
        embeddings.push(LadybugEmbedding {
            embedding_unit_key: format!("{record_key}#legacy-parent"),
            record_key,
            unit_kind: "legacy_parent".to_string(),
            label: None,
            ordinal: 0,
            semantic_input_hash,
            dimensions,
            vector,
        });
    }
    info!(
        legacy_sqlite_embeddings = embeddings.len(),
        source = %path.display(),
        "loaded legacy SQLite embeddings for Ladybug spike"
    );
    Ok(embeddings)
}
