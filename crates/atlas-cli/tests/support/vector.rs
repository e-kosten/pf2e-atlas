#![allow(dead_code)]

use std::path::Path;

use atlas_index::artifact_storage::encode_f32_vector_blob;
use atlas_index::sqlite_vector_index;
use rusqlite::{Connection, params};

pub fn insert_vector_embeddings(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()?;
    let connection = Connection::open(path)?;
    connection.execute_batch(&sqlite_vector_index::create_sql(384))?;
    let insert_vector_sql = sqlite_vector_index::insert_sql();
    for (record_key, first, second) in [
        ("actions:testAction1", 1.0_f32, 0.0_f32),
        ("actions:testAction2", 0.92_f32, 0.08_f32),
        ("actions:testAction3", 0.70_f32, 0.30_f32),
    ] {
        let vector = fixture_vector(first, second);
        let embedding_unit_key = format!("{record_key}#parent");
        connection.execute(
            "INSERT INTO document_embedding_cache (
               embedding_unit_key, record_key, unit_kind, label, ordinal,
               semantic_input_hash, dimensions, vector_blob
             )
             VALUES (?1, ?2, 'parent', NULL, 0, ?3, 384, ?4)",
            params![
                embedding_unit_key,
                record_key,
                format!("{record_key}:hash"),
                encode_f32_vector_blob(&vector),
            ],
        )?;
        let rowid = connection.last_insert_rowid();
        connection.execute(
            &insert_vector_sql,
            params![rowid, encode_f32_vector_blob(&vector)],
        )?;
    }
    Ok(())
}

fn fixture_vector(first: f32, second: f32) -> Vec<f32> {
    let mut vector = vec![0.0_f32; 384];
    vector[0] = first;
    vector[1] = second;
    vector
}
