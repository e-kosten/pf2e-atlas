use atlas_artifact::schema::document_embedding_cache_insert_sql;
use rusqlite::{Connection, params};

use crate::{GeneratedDocumentEmbedding, IngestError};

pub(super) fn write_document_embedding_cache(
    connection: &Connection,
    embeddings: &[GeneratedDocumentEmbedding],
) -> Result<(), IngestError> {
    let insert_sql = document_embedding_cache_insert_sql();
    let mut insert = connection
        .prepare(&insert_sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for embedding in embeddings {
        insert
            .execute(params![
                embedding.embedding_unit_key.as_str(),
                embedding.record_key.as_str(),
                embedding.unit_kind.as_str(),
                embedding.label.as_deref(),
                embedding.ordinal as i64,
                embedding.input_hash.as_str(),
                embedding.dimensions as i64,
                encode_vector_blob(&embedding.vector),
            ])
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }

    Ok(())
}

fn encode_vector_blob(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(vector));
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use atlas_artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;

    use super::*;

    #[test]
    fn encodes_vectors_as_little_endian_f32_blobs() {
        assert_eq!(
            encode_vector_blob(&[1.0, -2.5]),
            [1.0f32.to_le_bytes(), (-2.5f32).to_le_bytes()].concat()
        );
    }

    #[test]
    fn writes_document_embedding_cache_rows() {
        let connection = Connection::open_in_memory().expect("in-memory database should open");
        connection
            .execute_batch(CREATE_ARTIFACT_SCHEMA_SQL)
            .expect("schema should create");
        connection
            .execute(
                "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
                 VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 1)",
                [],
            )
            .expect("pack row should insert");
        connection
            .execute(
                "INSERT INTO records (
                  record_key, id, name, normalized_name, record_family, pack_name, pack_label,
                  foundry_document_type, foundry_record_type, traits_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, search_text_projection, raw_json
                ) VALUES ('actions:testAction1', 'testAction1', 'Test Action', 'test action',
                  'rule', 'actions', 'Actions', 'Item', 'action', '[]', 0, 'unknown', '[]',
                  '[]', 'none', 'packs/actions/test-action.json', 1, 'Test Action', '{}')",
                [],
            )
            .expect("record row should insert");
        let embeddings = vec![GeneratedDocumentEmbedding {
            embedding_unit_key: "actions:testAction1#parent".to_string(),
            record_key: "actions:testAction1".to_string(),
            unit_kind: atlas_embedding::EmbeddingUnitKind::Parent,
            label: None,
            ordinal: 0,
            input_hash: "fixture-hash".to_string(),
            dimensions: 2,
            vector: vec![1.0, -2.5],
        }];

        write_document_embedding_cache(&connection, &embeddings)
            .expect("document embedding row should write");

        let row: (String, i64, Vec<u8>) = connection
            .query_row(
                "SELECT semantic_input_hash, dimensions, vector_blob
                 FROM document_embedding_cache
                 WHERE embedding_unit_key = 'actions:testAction1#parent'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("document embedding row should be readable");
        assert_eq!(row.0, "fixture-hash");
        assert_eq!(row.1, 2);
        assert_eq!(row.2, encode_vector_blob(&[1.0, -2.5]));
    }
}
