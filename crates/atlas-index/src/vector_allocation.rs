//! Supported vec0 allocation maintenance. Never modify extension shadow tables.
use crate::{IndexError, SourceSearchProfileIdentity};
use rusqlite::Connection;

pub(crate) fn compact(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
) -> Result<(), IndexError> {
    let table = crate::schema::vector_table(model)?;
    // vec0 exposes allocated chunks through its read-only shadow schema. A full
    // repack is justified only when at least 10% and two blocks can be reclaimed.
    let (blocks, capacity): (u64, u64) = c.query_row(
        &format!("SELECT count(*),coalesce(sum(size),0) FROM {table}_chunks"),
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let live: u64 = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?;
    if blocks == 0 {
        return Ok(());
    }
    let block_size = capacity / blocks;
    let needed = live.div_ceil(block_size);
    if blocks.saturating_sub(needed) < 2 || needed * 10 > blocks * 9 {
        return Ok(());
    }
    tracing::info!(target:"atlas_progress",phase="compact_vectors",profile=%model.profile_id,blocks,needed,"Compacting search embeddings");
    // SQLite owns bounded page buffering; a file-backed temp table avoids a
    // second full Rust vector corpus. The caller owns rollback and admission.
    // Release dirty main-database pages before reading multi-MiB vec0 blocks.
    // This flush remains inside the caller's transaction, preserving rollback.
    c.cache_flush()?;
    let units: u64 = c.query_row(
        "SELECT count(*) FROM semantic_units WHERE profile_id=?",
        [model.profile_id.as_str()],
        |r| r.get(0),
    )?;
    if units != live {
        return Err(IndexError::Invalid(
            "cannot compact incomplete vector/unit coverage".into(),
        ));
    }
    c.execute_batch(
        "CREATE TEMP TABLE vector_repack(unit_id INTEGER PRIMARY KEY,embedding BLOB NOT NULL)",
    )?;
    // LEFT JOIN keeps unit IDs as the outer scan; vec0 otherwise advertises a
    // cheap fullscan and SQLite can choose scattered blob reads plus a blob sort.
    c.execute(&format!("INSERT INTO temp.vector_repack SELECT u.unit_id,v.embedding FROM semantic_units u NOT INDEXED LEFT JOIN {table} v USING(unit_id) WHERE u.profile_id=? ORDER BY u.unit_id"),[model.profile_id.as_str()])?;
    c.execute_batch(&format!("DROP TABLE {table}"))?;
    c.execute_batch(&crate::schema::vector_ddl(model)?)?;
    c.execute_batch(&format!("INSERT INTO {table}(unit_id,embedding) SELECT unit_id,embedding FROM temp.vector_repack ORDER BY unit_id; DROP TABLE temp.vector_repack;"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    #[test]
    fn compaction_preserves_bytes_filtered_distance_order_and_transaction_rollback() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("repack.sqlite")).unwrap();
        workspace
            .migrate(
                &[atlas_storage::Migration {
                    version: 1,
                    name: "native-repack-proof",
                    sql: "CREATE TABLE repack_fixture(id INTEGER PRIMARY KEY);",
                    backfill: None,
                }],
                |_| Ok(()),
            )
            .unwrap();
        let c = Connection::open(workspace.path()).unwrap();
        let model = crate::foundation_tests::fixture(true)
            .context
            .search_profiles
            .remove(0);
        let table = crate::schema::vector_table(&model).unwrap();
        c.execute_batch(&crate::schema::vector_ddl(&model).unwrap())
            .unwrap();
        c.execute_batch("CREATE TABLE semantic_units(unit_id INTEGER PRIMARY KEY,profile_id TEXT NOT NULL); CREATE TABLE eligible(id INTEGER PRIMARY KEY)")
            .unwrap();
        let mut vectors = Vec::new();
        for axis in 0..3 {
            let mut v = vec![0f32; model.dimensions];
            v[axis] = 1.;
            vectors.push(crate::read_units::vector_bytes(&v));
        }
        c.execute_batch("BEGIN").unwrap();
        for id in 1..=8192 {
            c.execute(
                &format!("INSERT INTO {table} VALUES(?,?)"),
                params![id, &vectors[id as usize % 3]],
            )
            .unwrap();
        }
        for id in (1..=8192).step_by(3) {
            c.execute(&format!("DELETE FROM {table} WHERE unit_id=?"), [id])
                .unwrap();
        }
        for id in 8193..=12288 {
            c.execute(
                &format!("INSERT INTO {table} VALUES(?,?)"),
                params![id, &vectors[id as usize % 3]],
            )
            .unwrap();
        }
        c.execute_batch("COMMIT").unwrap();
        c.execute(
            &format!("INSERT INTO eligible SELECT unit_id FROM {table} WHERE unit_id%2=0"),
            [],
        )
        .unwrap();
        c.execute(
            &format!("INSERT INTO semantic_units SELECT unit_id,? FROM {table}"),
            [model.profile_id.as_str()],
        )
        .unwrap();
        let snapshot = |c: &Connection| {
            c.prepare(&format!(
                "SELECT unit_id,embedding FROM {table} ORDER BY unit_id"
            ))
            .unwrap()
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
        };
        let blocks = |c: &Connection| {
            c.query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |r| {
                r.get::<_, usize>(0)
            })
            .unwrap()
        };
        let distances = |c: &Connection, k: usize| {
            let hits=c.prepare(&format!("WITH knn AS MATERIALIZED(SELECT unit_id,distance FROM {table} WHERE embedding MATCH ? AND k=? AND unit_id IN (SELECT id FROM eligible)) SELECT unit_id,distance FROM knn ORDER BY distance,unit_id")).unwrap().query_map(params![&vectors[0],k],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,f64>(1)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
            assert_eq!(hits.len(), k);
            assert!(hits.iter().all(|(id, _)| id % 2 == 0));
            hits.into_iter().map(|(_, d)| d).collect::<Vec<_>>()
        };
        let before = snapshot(&c);
        let before_blocks = blocks(&c);
        let queries = [1, 5, 100, 4096].map(|k| (k, distances(&c, k)));
        c.execute_batch("BEGIN").unwrap();
        compact(&c, &model).unwrap();
        assert!(
            blocks(&c) < before_blocks,
            "before={before_blocks},after={},live={}",
            blocks(&c),
            before.len()
        );
        assert_eq!(snapshot(&c), before);
        for (k, expected) in &queries {
            assert_eq!(distances(&c, *k), *expected);
        }
        c.execute_batch("ROLLBACK").unwrap();
        assert_eq!(blocks(&c), before_blocks);
        assert_eq!(snapshot(&c), before);
        drop(c);
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner.maintenance(1, "logical native repacking").unwrap();
        maintenance.publish(|c| compact(c, &model)).unwrap();
        let c = maintenance.connection();
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |row| row
                .get::<_, u64>(0))
                .unwrap(),
            1
        );
        assert!(
            blocks(c) < before_blocks,
            "before={before_blocks},after={},live={}",
            blocks(c),
            before.len()
        );
        assert_eq!(snapshot(c), before);
    }

    #[test]
    fn sparse_native_chunks_preserve_exact_neighbors_and_hidden_rowids_across_vacuum() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let c = Connection::open(directory.path().join("vacuum.sqlite")).unwrap();
        let model = crate::foundation_tests::fixture(true)
            .context
            .search_profiles
            .remove(0);
        let table = crate::schema::vector_table(&model).unwrap();
        c.execute_batch(&crate::schema::vector_ddl(&model).unwrap())
            .unwrap();
        let mut vector = vec![0f32; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        c.execute_batch("BEGIN").unwrap();
        for id in 1..=2048 {
            c.execute(
                &format!("INSERT INTO {table} VALUES(?,?)"),
                params![id, &bytes],
            )
            .unwrap();
        }
        for id in 1..=1024 {
            c.execute(&format!("DELETE FROM {table} WHERE unit_id=?"), [id])
                .unwrap();
        }
        c.execute_batch("COMMIT").unwrap();
        let snapshot = |c: &Connection| {
            let names = c
                .prepare("SELECT name FROM sqlite_master WHERE name LIKE ? ORDER BY name")
                .unwrap()
                .query_map([format!("{table}_vector_chunks%")], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            names
                .into_iter()
                .map(|name| {
                    let rowids = c
                        .prepare(&format!("SELECT rowid,_rowid_ FROM {name} ORDER BY rowid"))
                        .unwrap()
                        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
                        .unwrap()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    (name, rowids)
                })
                .collect::<Vec<_>>()
        };
        let hits = |c: &Connection| {
            c.prepare(&format!("WITH knn AS MATERIALIZED(SELECT unit_id,distance FROM {table} WHERE embedding MATCH ? AND k=1024) SELECT unit_id,distance FROM knn ORDER BY distance,unit_id")).unwrap()
                .query_map([&bytes],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,f64>(1)?.to_bits()))).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
        };
        let before = snapshot(&c);
        assert!(!before.is_empty());
        assert!(
            before.iter().all(|(_, rowids)| rowids == &[(2, 2)]),
            "the first empty chunk must leave a noncontiguous native blob rowid"
        );
        let neighbors = hits(&c);
        assert_eq!(neighbors.len(), 1024);
        c.execute_batch("VACUUM").unwrap();
        assert_eq!(snapshot(&c), before);
        assert_eq!(hits(&c), neighbors);
    }
}
