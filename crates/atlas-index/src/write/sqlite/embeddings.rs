use crate::artifact::storage::encode_f32_vector_blob;
use atlas_embedding::GeneratedDocumentEmbedding;
use diesel::SqliteConnection;
use diesel::prelude::*;

use super::models::DocumentEmbeddingCacheRow;
use crate::IndexWriteError;

pub(super) fn write_document_embedding_cache(
    connection: &mut SqliteConnection,
    embeddings: &[GeneratedDocumentEmbedding],
) -> Result<(), IndexWriteError> {
    if embeddings.is_empty() {
        return Ok(());
    }
    let rows = embeddings
        .iter()
        .map(|embedding| {
            Ok::<DocumentEmbeddingCacheRow, IndexWriteError>(DocumentEmbeddingCacheRow {
                embedding_unit_key: embedding.embedding_unit_key.clone(),
                record_key: embedding.record_key.clone(),
                unit_kind: embedding.unit_kind.as_str().to_string(),
                label: embedding.label.clone(),
                ordinal: i64::try_from(embedding.ordinal).map_err(|_| {
                    IndexWriteError::WriteFailed(
                        "embedding ordinal does not fit in i64".to_string(),
                    )
                })?,
                semantic_input_hash: embedding.input_hash.clone(),
                dimensions: i64::try_from(embedding.dimensions).map_err(|_| {
                    IndexWriteError::WriteFailed(
                        "embedding dimensions do not fit in i64".to_string(),
                    )
                })?,
                vector_blob: encode_f32_vector_blob(&embedding.vector),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for rows in rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::document_embedding_cache::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::artifact::schema::CREATE_ARTIFACT_SCHEMA_SQL;
    use crate::artifact::storage::decode_f32_vector_blob;
    use diesel::connection::SimpleConnection;
    use diesel::sql_types::{BigInt, Binary, Text};
    use diesel::{Connection as DieselConnection, QueryableByName, sql_query};

    use super::*;

    #[derive(QueryableByName)]
    struct EmbeddingCacheFixtureRow {
        #[diesel(sql_type = Text)]
        semantic_input_hash: String,
        #[diesel(sql_type = BigInt)]
        dimensions: i64,
        #[diesel(sql_type = Binary)]
        vector_blob: Vec<u8>,
    }

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    #[test]
    fn writes_document_embedding_cache_rows() {
        let mut connection =
            SqliteConnection::establish(":memory:").expect("in-memory database should open");
        connection
            .batch_execute(CREATE_ARTIFACT_SCHEMA_SQL)
            .expect("schema should create");
        connection
            .batch_execute(
                "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
                 VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 1)"
            )
            .expect("pack row should insert");
        connection
            .batch_execute(
                "INSERT INTO records (
                  record_key, id, name, normalized_name, record_family, pack_name, pack_label,
                  foundry_document_type, foundry_record_type, traits_json, prerequisites_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, raw_json
                ) VALUES ('actions:testAction1', 'testAction1', 'Test Action', 'test action',
                  'rule', 'actions', 'Actions', 'Item', 'action', '[]', '[]', 0, 'unknown', '[]',
                  '[]', 'none', 'packs/actions/test-action.json', 1, '{}')"
            )
            .expect("record row should insert");
        let embeddings = (0..4_100)
            .map(|index| GeneratedDocumentEmbedding {
                embedding_unit_key: format!("actions:testAction1#child-{index}"),
                record_key: "actions:testAction1".to_string(),
                unit_kind: atlas_embedding::EmbeddingUnitKind::HeadingSection,
                label: Some(format!("Child {index}")),
                ordinal: index,
                input_hash: format!("fixture-hash-{index}"),
                dimensions: 2,
                vector: vec![1.0, -2.5],
            })
            .collect::<Vec<_>>();

        write_document_embedding_cache(&mut connection, &embeddings)
            .expect("document embedding rows should write in batches");

        let count = sql_query("SELECT COUNT(*) AS count FROM document_embedding_cache")
            .get_result::<CountRow>(&mut connection)
            .expect("document embedding rows should be countable")
            .count;
        assert_eq!(count, embeddings.len() as i64);

        let row = sql_query(
            "SELECT semantic_input_hash, dimensions, vector_blob
                 FROM document_embedding_cache
                 WHERE embedding_unit_key = 'actions:testAction1#child-4099'",
        )
        .get_result::<EmbeddingCacheFixtureRow>(&mut connection)
        .expect("document embedding row should be readable");
        assert_eq!(row.semantic_input_hash, "fixture-hash-4099");
        assert_eq!(row.dimensions, 2);
        assert_eq!(
            decode_f32_vector_blob(&row.vector_blob).expect("vector blob should decode"),
            vec![1.0, -2.5]
        );
    }
}
