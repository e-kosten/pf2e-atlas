use atlas_artifact::schema::{
    TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX, record_vector_index_create_sql,
    record_vector_index_insert_sql,
};
use rusqlite::Connection;

use crate::error::IngestError;

pub(super) fn write_record_vector_index(connection: &Connection) -> Result<(), IngestError> {
    let dimensions = embedding_dimensions(connection)?;
    connection
        .execute_batch(&format!(
            "DROP TABLE IF EXISTS {table};
             {create_sql}",
            table = TABLE_RECORD_VECTOR_INDEX,
            create_sql = record_vector_index_create_sql(dimensions)
        ))
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    let insert_sql = record_vector_index_insert_sql();
    let mut insert = connection
        .prepare(&insert_sql)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut select = connection
        .prepare(&format!(
            "SELECT rowid, vector_blob
             FROM {TABLE_DOCUMENT_EMBEDDING_CACHE}
             ORDER BY embedding_unit_key"
        ))
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let rows = select
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    for row in rows {
        let (rowid, vector_blob) =
            row.map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        insert
            .execute((rowid, vector_blob))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn embedding_dimensions(connection: &Connection) -> Result<usize, IngestError> {
    let dimensions = connection
        .query_row(
            &format!(
                "SELECT dimensions
                 FROM {TABLE_DOCUMENT_EMBEDDING_CACHE}
                 LIMIT 1"
            ),
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    usize::try_from(dimensions).map_err(|_| {
        IngestError::ArtifactWriteFailed(format!(
            "document embedding dimensions must be non-negative, got {dimensions}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use atlas_artifact::schema::create_artifact_schema_sql;
    use atlas_artifact::storage::encode_f32_vector_blob;
    use rusqlite::Connection;

    use super::write_record_vector_index;

    #[test]
    fn writes_record_vector_index_from_document_embedding_cache() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().expect("sqlite-vec should register");
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
                  foundry_document_type, foundry_record_type, traits_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, raw_json
                ) VALUES ('actions:testAction1', 'testAction1', 'Test Action', 'test action',
                  'rule', 'actions', 'Actions', 'Item', 'action', '[]', 0, 'unknown', '[]',
                  '[]', 'none', 'packs/actions/test-action.json', 1, '{}')",
                [],
            )
            .expect("record row should insert");
        connection
            .execute(
                "INSERT INTO document_embedding_cache (
                    embedding_unit_key, record_key, unit_kind, label, ordinal,
                    semantic_input_hash, dimensions, vector_blob
                 ) VALUES (?1, ?2, ?3, NULL, 0, ?4, 2, ?5)",
                (
                    "actions:testAction1#parent",
                    "actions:testAction1",
                    "parent",
                    "fixture-hash",
                    encode_f32_vector_blob(&[1.0, -2.5]),
                ),
            )
            .expect("document embedding row should insert");

        write_record_vector_index(&connection).expect("vector index should write");

        let rows: i64 = connection
            .query_row("SELECT COUNT(*) FROM record_vector_index", [], |row| {
                row.get(0)
            })
            .expect("vector index should be readable");
        assert_eq!(rows, 1);
    }
}
