use atlas_artifact::{
    schema::document_embedding_cache_insert_sql, storage::encode_f32_vector_blob,
};
use atlas_embedding::GeneratedDocumentEmbedding;
use rusqlite::{Connection, params};

use crate::IndexWriteError;

pub(super) fn write_document_embedding_cache(
    connection: &Connection,
    embeddings: &[GeneratedDocumentEmbedding],
) -> Result<(), IndexWriteError> {
    let insert_sql = document_embedding_cache_insert_sql();
    let mut insert = connection
        .prepare(&insert_sql)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

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
                encode_f32_vector_blob(&embedding.vector),
            ])
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use atlas_artifact::schema::create_artifact_schema_sql;

    use super::*;

    #[test]
    fn writes_document_embedding_cache_rows() {
        let connection = Connection::open_in_memory().expect("in-memory database should open");
        connection
            .execute_batch(&create_artifact_schema_sql())
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
                  foundry_document_type, foundry_record_type, traits_json, prerequisites_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, raw_json
                ) VALUES ('actions:testAction1', 'testAction1', 'Test Action', 'test action',
                  'rule', 'actions', 'Actions', 'Item', 'action', '[]', '[]', 0, 'unknown', '[]',
                  '[]', 'none', 'packs/actions/test-action.json', 1, '{}')",
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
        assert_eq!(row.2, encode_f32_vector_blob(&[1.0, -2.5]));
    }
}
