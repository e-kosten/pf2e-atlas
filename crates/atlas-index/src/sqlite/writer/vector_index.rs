use diesel::prelude::*;
use diesel::sql_types::{BigInt, Binary};
use diesel::{QueryableByName, SqliteConnection, sql_query};

use crate::IndexWriteError;

const TABLE_DOCUMENT_EMBEDDING_CACHE: &str = "document_embedding_cache";
const TABLE_RECORD_VECTOR_INDEX: &str = "record_vector_index";

pub(super) fn write_record_vector_index(
    connection: &mut SqliteConnection,
) -> Result<(), IndexWriteError> {
    let dimensions = embedding_dimensions(connection)?;
    sql_query(format!("DROP TABLE IF EXISTS {TABLE_RECORD_VECTOR_INDEX}"))
        .execute(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    sql_query(format!(
        "CREATE VIRTUAL TABLE {TABLE_RECORD_VECTOR_INDEX} USING vec0(embedding FLOAT[{dimensions}])"
    ))
    .execute(connection)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    let rows = sql_query(format!(
        "SELECT rowid, vector_blob
         FROM {TABLE_DOCUMENT_EMBEDDING_CACHE}
         ORDER BY embedding_unit_key"
    ))
    .load::<VectorSourceRow>(connection)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    for row in rows {
        diesel::sql_query(format!(
            "INSERT INTO {TABLE_RECORD_VECTOR_INDEX} (rowid, embedding) VALUES (?, ?)"
        ))
        .bind::<BigInt, _>(row.rowid)
        .bind::<Binary, _>(row.vector_blob)
        .execute(connection)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

#[derive(QueryableByName)]
struct DimensionRow {
    #[diesel(sql_type = BigInt)]
    dimensions: i64,
}

#[derive(QueryableByName)]
struct VectorSourceRow {
    #[diesel(sql_type = BigInt)]
    rowid: i64,
    #[diesel(sql_type = Binary)]
    vector_blob: Vec<u8>,
}

fn embedding_dimensions(connection: &mut SqliteConnection) -> Result<usize, IndexWriteError> {
    let dimensions = sql_query(format!(
        "SELECT dimensions
         FROM {TABLE_DOCUMENT_EMBEDDING_CACHE}
         LIMIT 1"
    ))
    .get_result::<DimensionRow>(connection)
    .map(|row| row.dimensions)
    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    usize::try_from(dimensions).map_err(|_| {
        IndexWriteError::WriteFailed(format!(
            "document embedding dimensions must be non-negative, got {dimensions}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::artifact_schema::CREATE_ARTIFACT_SCHEMA_SQL;
    use crate::artifact_storage::encode_f32_vector_blob;
    use diesel::connection::SimpleConnection;
    use diesel::sql_types::{BigInt, Binary, Text};
    use diesel::{
        Connection as DieselConnection, QueryableByName, RunQueryDsl, SqliteConnection, sql_query,
    };

    use super::write_record_vector_index;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    #[test]
    fn writes_record_vector_index_from_document_embedding_cache() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().expect("sqlite-vec should register");
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
        sql_query(
            "INSERT INTO document_embedding_cache (
                    embedding_unit_key, record_key, unit_kind, label, ordinal,
                    semantic_input_hash, dimensions, vector_blob
                 ) VALUES (?, ?, ?, NULL, 0, ?, 2, ?)",
        )
        .bind::<Text, _>("actions:testAction1#parent")
        .bind::<Text, _>("actions:testAction1")
        .bind::<Text, _>("parent")
        .bind::<Text, _>("fixture-hash")
        .bind::<Binary, _>(encode_f32_vector_blob(&[1.0, -2.5]))
        .execute(&mut connection)
        .expect("document embedding row should insert");

        write_record_vector_index(&mut connection).expect("vector index should write");

        let rows = sql_query("SELECT COUNT(*) AS count FROM record_vector_index")
            .get_result::<CountRow>(&mut connection)
            .expect("vector index should be readable");
        assert_eq!(rows.count, 1);
    }
}
