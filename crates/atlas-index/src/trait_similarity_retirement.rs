//! Offline removal of trait seed consumers and their exclusively owned vectors.
use crate::IndexError;
use rusqlite::Connection;

pub fn retire_trait_similarity(c: &Connection) -> atlas_storage::StorageResult<()> {
    retire(c).map_err(|error| atlas_storage::StorageError::Migration(error.to_string()))
}

fn retire(c: &Connection) -> Result<(), IndexError> {
    let retired = c
        .prepare(
            "SELECT i.input_id,m.identity_json,i.published_hash IS NOT NULL
             FROM embedding_inputs i JOIN embedding_models m USING(model_id)
             WHERE EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id)
             AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id)
             ORDER BY i.model_id,i.input_id",
        )?
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    c.execute_batch("DROP TABLE trait_seed_chunks; DROP TABLE trait_vectors;")?;
    for (input_id, model_json, published) in retired {
        if published {
            let model = serde_json::from_str(&model_json)?;
            let table = crate::schema::native_vector_table(&model)?;
            if c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [input_id])? != 1 {
                return Err(IndexError::Invalid(
                    "retired trait input is missing its native vector".into(),
                ));
            }
        }
        c.execute(
            "UPDATE embedding_inputs SET published_hash=NULL WHERE input_id=?
             AND EXISTS(SELECT 1 FROM pending_embedding_values WHERE input_id=?1)",
            [input_id],
        )?;
        c.execute(
            "DELETE FROM embedding_inputs WHERE input_id=?
             AND NOT EXISTS(SELECT 1 FROM pending_embedding_values WHERE input_id=?1)",
            [input_id],
        )?;
    }
    c.execute(
        "UPDATE workspace SET reference_revision=reference_revision+1
         WHERE id=1 AND EXISTS(SELECT 1 FROM artifact_context)",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Connection, String, Vec<u8>) {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let c = Connection::open_in_memory().unwrap();
        let model = crate::SourceSearchProfileIdentity::from_spec(
            atlas_embedding::profile_for_model(atlas_embedding::EmbeddingModelId::BgeBaseEnV15),
        );
        let table = crate::schema::native_vector_table(&model).unwrap();
        c.execute_batch(&crate::schema::native_vector_ddl(&model).unwrap())
            .unwrap();
        c.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE workspace(id INTEGER PRIMARY KEY,reference_revision INTEGER);
             INSERT INTO workspace VALUES(1,8);
             CREATE TABLE artifact_context(singleton INTEGER PRIMARY KEY);
             INSERT INTO artifact_context VALUES(1);
             CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT);
             CREATE TABLE embedding_inputs(input_id INTEGER PRIMARY KEY,model_id INTEGER,published_hash BLOB);
             CREATE TABLE pending_embedding_values(input_id INTEGER PRIMARY KEY REFERENCES embedding_inputs ON DELETE CASCADE,vector BLOB);
             CREATE TABLE semantic_units(input_id INTEGER REFERENCES embedding_inputs);
             CREATE TABLE trait_vectors(trait_key TEXT PRIMARY KEY);
             CREATE TABLE trait_seed_chunks(trait_key TEXT REFERENCES trait_vectors,input_id INTEGER REFERENCES embedding_inputs);
             INSERT INTO embedding_inputs VALUES(1,1,zeroblob(32)),(2,1,zeroblob(32)),(3,1,zeroblob(32)),(4,1,NULL),(5,1,zeroblob(32));
             INSERT INTO semantic_units VALUES(1),(5);
             INSERT INTO trait_vectors VALUES('trait:fire');
             INSERT INTO trait_seed_chunks VALUES('trait:fire',1),('trait:fire',2),('trait:fire',3);
             INSERT INTO pending_embedding_values VALUES(3,X'AB'),(4,X'CD');",
        )
        .unwrap();
        c.execute(
            "INSERT INTO embedding_models VALUES(1,?)",
            [serde_json::to_string(&model).unwrap()],
        )
        .unwrap();
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        for id in [1, 2, 3, 5] {
            c.execute(
                &format!("INSERT INTO {table} (input_id,embedding) VALUES(?,?)"),
                rusqlite::params![id, bytes],
            )
            .unwrap();
        }
        (c, table, bytes)
    }

    #[test]
    fn retirement_keeps_document_vectors_and_pending_recovery() {
        let (mut c, table, bytes) = fixture();
        let transaction = c.transaction().unwrap();
        retire_trait_similarity(&transaction).unwrap();
        transaction.commit().unwrap();
        let vectors = c
            .prepare(&format!(
                "SELECT input_id,embedding FROM {table} ORDER BY input_id"
            ))
            .unwrap()
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(vectors, [(1, bytes.clone()), (5, bytes)]);
        let retained = c
            .prepare("SELECT i.input_id,i.published_hash IS NOT NULL,p.vector FROM embedding_inputs i LEFT JOIN pending_embedding_values p USING(input_id) ORDER BY i.input_id")
            .unwrap()
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, bool>(1)?, row.get::<_, Option<Vec<u8>>>(2)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            retained,
            [
                (1, true, None),
                (3, false, Some(vec![0xAB])),
                (4, false, Some(vec![0xCD])),
                (5, true, None)
            ]
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            9
        );
        assert!(c.prepare("SELECT * FROM trait_seed_chunks").is_err());
        assert!(c.prepare("SELECT * FROM trait_vectors").is_err());
    }

    #[test]
    fn retirement_failure_rolls_back_schema_and_values() {
        let (mut c, table, bytes) = fixture();
        c.execute(&format!("DELETE FROM {table} WHERE input_id=3"), [])
            .unwrap();
        let transaction = c.transaction().unwrap();
        assert!(retire_trait_similarity(&transaction).is_err());
        transaction.rollback().unwrap();
        assert_eq!(
            c.query_row(
                &format!("SELECT embedding FROM {table} WHERE input_id=2"),
                [],
                |row| row.get::<_, Vec<u8>>(0)
            )
            .unwrap(),
            bytes
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM trait_seed_chunks", [], |row| row
                .get::<_, usize>(0))
                .unwrap(),
            3
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            8
        );
    }
}
