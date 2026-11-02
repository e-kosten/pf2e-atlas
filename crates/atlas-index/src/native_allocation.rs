//! Current input-owned vec0 packing; historical migration10 has its own owner.
use crate::{IndexError, SourceArtifactBuildContext, SourceSearchProfileIdentity};
use rusqlite::Connection;

fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}

fn worthwhile(blocks: u64, capacity: u64, live: u64) -> Result<bool, IndexError> {
    if blocks == 0 {
        return if live == 0 {
            Ok(false)
        } else {
            Err(invalid("native allocation has no blocks"))
        };
    }
    let block_size = capacity / blocks;
    if block_size == 0 || !capacity.is_multiple_of(blocks) || live > capacity {
        return Err(invalid("native allocation capacity is invalid"));
    }
    let needed = live.div_ceil(block_size);
    let reclaim = blocks
        .checked_sub(needed)
        .ok_or_else(|| invalid("native allocation count overflow"))?;
    Ok(reclaim >= 2
        && reclaim
            .checked_mul(10)
            .ok_or_else(|| invalid("native allocation count overflow"))?
            >= blocks)
}

pub(crate) fn compact_profiles(
    c: &Connection,
    context: &SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    for model in &context.search_profiles {
        compact(c, model)?;
    }
    Ok(())
}

fn compact(c: &Connection, model: &SourceSearchProfileIdentity) -> Result<bool, IndexError> {
    if c.is_autocommit() {
        return Err(invalid(
            "native packing requires the publication transaction",
        ));
    }
    let table = crate::schema::native_vector_table(model)?;
    // Allocation metadata is read-only. All writes below use public vec0 SQL.
    let (blocks, capacity): (u64, u64) = c.query_row(
        &format!("SELECT count(*),coalesce(sum(size),0) FROM {table}_chunks"),
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let live: u64 = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?;
    if !worthwhile(blocks, capacity, live)? {
        return Ok(false);
    }
    let identity = serde_json::to_string(model)?;
    let model_id: i64 = c.query_row(
        "SELECT model_id FROM embedding_models WHERE identity_json=?",
        [&identity],
        |r| r.get(0),
    )?;
    let expected: u64 = c.query_row(
        "SELECT count(*) FROM embedding_inputs WHERE model_id=? AND published_hash IS NOT NULL",
        [model_id],
        |r| r.get(0),
    )?;
    if expected != live {
        return Err(invalid("cannot pack incomplete native/input coverage"));
    }
    let bytes = crate::native_values::expected_bytes(model)?;
    c.cache_flush()?;
    c.execute_batch(&format!("CREATE TEMP TABLE native_repack(input_id INTEGER PRIMARY KEY,embedding BLOB NOT NULL CHECK(length(embedding)={bytes}))"))?;
    {
        let mut source = c.prepare(&format!("SELECT i.input_id,CASE WHEN length(v.embedding)=?2 THEN v.embedding ELSE zeroblob(0) END,CASE WHEN typeof(i.published_hash)='blob' AND length(i.published_hash)=32 THEN i.published_hash ELSE zeroblob(0) END FROM embedding_inputs i NOT INDEXED LEFT JOIN {table} v USING(input_id) WHERE i.model_id=?1 AND i.published_hash IS NOT NULL ORDER BY i.input_id"))?;
        let mut rows = source.query(rusqlite::params![model_id, bytes])?;
        let mut insert =
            c.prepare("INSERT INTO temp.native_repack(input_id,embedding) VALUES(?,?)")?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let value: Vec<u8> = row.get(1)?;
            let checksum: Vec<u8> = row.get(2)?;
            crate::native_values::verify_bytes(&value, &checksum, model)?;
            insert.execute(rusqlite::params![id, value])?;
        }
    }
    let copied: u64 = c.query_row("SELECT count(*) FROM temp.native_repack", [], |r| r.get(0))?;
    if copied != expected {
        return Err(invalid("native packing copy coverage differs"));
    }
    c.execute_batch(&format!("DROP TABLE {table}"))?;
    c.execute_batch(&crate::schema::native_vector_ddl(model)?)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="native_repack_replaced",profile=%model.profile_id,"Installing compacted search embeddings");
    c.execute_batch(&format!("INSERT INTO {table}(input_id,embedding) SELECT input_id,embedding FROM temp.native_repack ORDER BY input_id; DROP TABLE temp.native_repack;"))?;
    tracing::info!(target:"atlas_progress",phase="compact_vectors",profile=%model.profile_id,blocks,live,"Compacted search embeddings");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn fixture(fragmented: bool) -> (tempfile::TempDir, Connection, SourceSearchProfileIdentity) {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let c = Connection::open(directory.path().join("current-repack.sqlite")).unwrap();
        c.execute_batch("PRAGMA temp_store=FILE; CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT UNIQUE); CREATE TABLE embedding_inputs(input_id INTEGER PRIMARY KEY,model_id INTEGER,published_hash BLOB);").unwrap();
        let model = crate::foundation_tests::fixture(true)
            .context
            .search_profiles
            .remove(0);
        let table = crate::schema::native_vector_table(&model).unwrap();
        c.execute_batch(&crate::schema::native_vector_ddl(&model).unwrap())
            .unwrap();
        c.execute(
            "INSERT INTO embedding_models VALUES(1,?)",
            [serde_json::to_string(&model).unwrap()],
        )
        .unwrap();
        let mut vectors = Vec::new();
        for axis in 0..3 {
            let mut vector = vec![0.; model.dimensions];
            vector[axis] = 1.;
            vectors.push(crate::read_units::vector_bytes(&vector));
        }
        c.execute_batch("BEGIN").unwrap();
        let rows = if fragmented { 4096 } else { 1 };
        for id in 1..=rows {
            let bytes = &vectors[id as usize % 3];
            c.execute(
                "INSERT INTO embedding_inputs VALUES(?,1,?)",
                params![id, crate::embedding_values::hash_bytes(bytes)],
            )
            .unwrap();
            c.execute(
                &format!("INSERT INTO {table}(input_id,embedding) VALUES(?,?)"),
                params![id, bytes],
            )
            .unwrap();
        }
        if fragmented {
            for id in (2..=rows).step_by(2) {
                c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [id])
                    .unwrap();
                c.execute("DELETE FROM embedding_inputs WHERE input_id=?", [id])
                    .unwrap();
            }
        }
        c.execute_batch("COMMIT").unwrap();
        (directory, c, model)
    }
    fn snapshot(c: &Connection, table: &str) -> Vec<(i64, Vec<u8>)> {
        c.prepare(&format!(
            "SELECT input_id,embedding FROM {table} ORDER BY input_id"
        ))
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }
    fn blocks(c: &Connection, table: &str) -> u64 {
        c.query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn threshold_requires_both_two_blocks_and_ten_percent() {
        assert!(!worthwhile(0, 0, 0).unwrap());
        assert!(!worthwhile(20, 20480, 19456).unwrap());
        assert!(!worthwhile(21, 21504, 19456).unwrap());
        assert!(worthwhile(20, 20480, 18432).unwrap());
        assert!(worthwhile(1, 0, 0).is_err());
    }

    #[test]
    fn current_packing_keeps_input_ids_exact_bytes_filtered_distances_and_rollback() {
        let (_directory, c, model) = fixture(true);
        let table = crate::schema::native_vector_table(&model).unwrap();
        let before = snapshot(&c, &table);
        let before_blocks = blocks(&c, &table);
        // Six eligible inputs span all three axes. The query gives each axis
        // a distinct distance; K5 cuts through the farthest tie without pinning
        // whichever equal-distance input the index chooses at that boundary.
        let mut query = vec![0.; model.dimensions];
        query[0] = 0.8;
        query[1] = 0.6;
        let query = crate::read_units::vector_bytes(&query);
        let distances = |c: &Connection| {
            c.prepare(&format!("WITH knn AS MATERIALIZED(SELECT input_id,distance FROM {table} WHERE embedding MATCH ? AND k=5 AND input_id IN (SELECT input_id FROM embedding_inputs WHERE input_id<=11 AND input_id%2=1)) SELECT distance FROM knn ORDER BY distance,input_id")).unwrap().query_map([&query],|r|r.get::<_,f64>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
        };
        let expected = distances(&c);
        assert_eq!(expected.len(), 5);
        for (actual, expected_distance) in expected.iter().zip([0.2, 0.2, 0.4, 0.4, 1.0]) {
            assert!(
                (actual - expected_distance).abs() < 1e-6,
                "{actual} != {expected_distance}"
            );
        }
        c.execute_batch("BEGIN").unwrap();
        assert!(compact(&c, &model).unwrap());
        assert!(blocks(&c, &table) < before_blocks);
        assert_eq!(snapshot(&c, &table), before);
        assert_eq!(distances(&c), expected);
        c.execute_batch("ROLLBACK").unwrap();
        assert_eq!(snapshot(&c, &table), before);
        assert_eq!(blocks(&c, &table), before_blocks);
        c.execute_batch("BEGIN").unwrap();
        assert!(compact(&c, &model).unwrap());
        c.execute_batch("COMMIT").unwrap();
        assert_eq!(snapshot(&c, &table), before);
        assert!(blocks(&c, &table) < before_blocks);
    }

    #[test]
    fn complete_table_is_not_rewritten_and_balanced_missing_orphan_is_rejected() {
        let (_directory, c, model) = fixture(false);
        c.execute_batch("BEGIN").unwrap();
        assert!(!compact(&c, &model).unwrap());
        assert!(
            !c.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_temp_schema WHERE name='native_repack')",
                [],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
        );
        c.execute_batch("ROLLBACK").unwrap();
        drop(c);
        let (_fragmented, c, model) = fixture(true);
        let table = crate::schema::native_vector_table(&model).unwrap();
        let original = snapshot(&c, &table);
        let id = original[0].0;
        c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [id])
            .unwrap();
        c.execute(
            &format!("INSERT INTO {table}(input_id,embedding) VALUES(999999,?)"),
            [&original[0].1],
        )
        .unwrap();
        let corrupt = snapshot(&c, &table);
        c.execute_batch("BEGIN").unwrap();
        assert!(compact(&c, &model).is_err());
        assert_eq!(snapshot(&c, &table), corrupt);
        c.execute_batch("ROLLBACK").unwrap();
        assert_eq!(snapshot(&c, &table), corrupt);
    }

    #[test]
    fn actual_temp_sqlite_full_preserves_native_table_and_transaction_rollback() {
        let (_directory, c, model) = fixture(true);
        let table = crate::schema::native_vector_table(&model).unwrap();
        let before = snapshot(&c, &table);
        let allocation = blocks(&c, &table);
        c.execute_batch("PRAGMA temp.max_page_count=2; BEGIN")
            .unwrap();
        let error = compact(&c, &model).unwrap_err();
        assert!(error.to_string().contains("full"), "{error}");
        assert_eq!(snapshot(&c, &table), before);
        assert_eq!(blocks(&c, &table), allocation);
        if !c.is_autocommit() {
            c.execute_batch("ROLLBACK").unwrap();
        }
        assert_eq!(snapshot(&c, &table), before);
    }
}
