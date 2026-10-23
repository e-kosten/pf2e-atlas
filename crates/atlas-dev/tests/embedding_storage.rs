//! Explicit full-corpus regression/size probe; never modifies its source workspace.
use atlas_index::{SourceSearchProfileIdentity, read_stored_reference_context};
use atlas_storage::{ReadSession, Workspace};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

fn table(model: &SourceSearchProfileIdentity) -> String {
    format!(
        "semantic_vectors_{:x}",
        Sha256::digest(serde_json::to_vec(model).unwrap())
    )
}
fn vector_digest(read: &ReadSession, models: &[SourceSearchProfileIdentity]) -> String {
    let mut digest = Sha256::new();
    for model in models {
        let mut s = read
            .connection()
            .prepare(&format!(
                "SELECT unit_id,embedding FROM {} ORDER BY unit_id",
                table(model)
            ))
            .unwrap();
        let mut rows = s.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            digest.update(row.get::<_, i64>(0).unwrap().to_le_bytes());
            digest.update(row.get::<_, Vec<u8>>(1).unwrap());
        }
    }
    format!("{:x}", digest.finalize())
}
fn saved_digest(read: &ReadSession) -> String {
    let c = read.connection();
    let mut digest = Sha256::new();
    let tables=c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND (name LIKE 'encounter%' OR name LIKE 'saved_%' OR name LIKE 'part%') ORDER BY name").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    for table in tables {
        digest.update(table.as_bytes());
        let mut s = c
            .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY 1"))
            .unwrap();
        let columns = s.column_count();
        let mut rows = s.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                let value = format!("{:?}", row.get_ref(i).unwrap());
                digest.update(value.len().to_le_bytes());
                digest.update(value.as_bytes());
            }
        }
    }
    format!("{:x}", digest.finalize())
}
fn attribution_digest(read: &ReadSession, old: bool) -> String {
    let sql = if old {
        "SELECT unit_id,record_id,profile_id,owners_json,field_path,section_json,chunk_ordinal,input_token_count,input_hash,unit_kind FROM semantic_units ORDER BY unit_id"
    } else {
        "SELECT u.unit_id,u.record_id,u.profile_id,l.owners_json,l.field_path,l.section_json,l.chunk_ordinal,i.token_count,lower(hex(i.input_hash)),u.unit_kind FROM semantic_units u JOIN semantic_locations l USING(location_id) JOIN embedding_inputs i USING(input_id) ORDER BY u.unit_id"
    };
    let mut digest = Sha256::new();
    let mut s = read.connection().prepare(sql).unwrap();
    let mut rows = s.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        for i in 0..10 {
            let value = format!("{:?}", row.get_ref(i).unwrap());
            digest.update(value.len().to_le_bytes());
            digest.update(value.as_bytes());
        }
    }
    format!("{:x}", digest.finalize())
}
fn knn(read: &ReadSession, models: &[SourceSearchProfileIdentity]) -> Vec<Vec<f64>> {
    let c = read.connection();
    let mut results = Vec::new();
    for model in models {
        let table = table(model);
        let vector: Vec<u8> = c
            .query_row(
                &format!("SELECT embedding FROM {table} ORDER BY unit_id LIMIT 1"),
                [],
                |r| r.get(0),
            )
            .unwrap();
        for k in [1, 25, 4096] {
            let start = Instant::now();
            let mut s=c.prepare(&format!("WITH knn AS MATERIALIZED(SELECT unit_id,distance FROM {table} WHERE embedding MATCH ?1 AND k=?2 AND unit_id IN (SELECT unit_id FROM semantic_units WHERE profile_id=?3 AND record_id%2=0)) SELECT distance FROM knn ORDER BY distance")).unwrap();
            let distances = s
                .query_map((&vector, k, model.profile_id.as_str()), |r| {
                    r.get::<_, f64>(0)
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            eprintln!(
                "knn profile={} k={k} hits={} ms={}",
                model.profile_id,
                distances.len(),
                start.elapsed().as_millis()
            );
            results.push(distances);
        }
    }
    results
}
#[test]
#[ignore = "requires ATLAS_STORAGE_PROBE_SOURCE pointing at a schema-9 repository development corpus; creates one disposable copy"]
fn full_corpus_storage_upgrade_preserves_values_and_reuse() {
    let source = PathBuf::from(
        std::env::var_os("ATLAS_STORAGE_PROBE_SOURCE").expect("set source corpus path"),
    );
    let directory = tempfile::tempdir().unwrap();
    // Register the normal runtime extension without upgrading the source.
    drop(
        atlas_runtime::initialize_workspace(directory.path().join("registration.sqlite")).unwrap(),
    );
    let path = directory.path().join("probe.sqlite");
    Workspace::at(source).unwrap().snapshot_to(&path).unwrap();
    let workspace = Workspace::at(&path).unwrap();
    let before = workspace.read(9).unwrap();
    let context = read_stored_reference_context(before.connection())
        .unwrap()
        .unwrap();
    let key = before.workspace_key().unwrap();
    let revision = before.reference_revision().unwrap();
    let vectors = vector_digest(&before, &context.search_profiles);
    let saved = saved_digest(&before);
    let attribution = attribution_digest(&before, true);
    let expected_knn = knn(&before, &context.search_profiles);
    let before_size = std::fs::metadata(&path).unwrap().len();
    drop(before);
    let start = Instant::now();
    let workspace = atlas_runtime::initialize_workspace(&path).unwrap();
    eprintln!("migration seconds={:.2}", start.elapsed().as_secs_f64());
    let owner = workspace.update_owner().unwrap();
    owner
        .reclaim_space(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    drop(owner);
    let read = workspace
        .read(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    assert_eq!(read.workspace_key().unwrap(), key);
    assert_eq!(read.reference_revision().unwrap(), revision);
    assert_eq!(saved_digest(&read), saved);
    assert_eq!(attribution_digest(&read, false), attribution);
    assert_eq!(vector_digest(&read, &context.search_profiles), vectors);
    assert_eq!(knn(&read, &context.search_profiles), expected_knn);
    let c = read.connection();
    for model in &context.search_profiles {
        let mut s=c.prepare("SELECT lower(hex(i.input_hash)),i.token_count FROM embedding_inputs i JOIN embedding_models m USING(model_id) WHERE m.identity_json=?").unwrap();
        let inputs = s
            .query_map([serde_json::to_string(model).unwrap()], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, usize>(1)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .map(|(hash, tokens)| atlas_embedding::PreparedEmbeddingInput {
                address: atlas_domain::SourcePassageAddress::Identity {},
                input: String::new(),
                input_sha256: hash.clone(),
                reuse_key: hash,
                token_count: tokens,
                body_token_count: tokens,
                overlap_token_count: 0,
                context_shortened: false,
            })
            .collect::<Vec<_>>();
        let start = Instant::now();
        let reused = atlas_index::read_reusable_embeddings(&read, model, &inputs).unwrap();
        assert_eq!(reused.len(), inputs.len());
        eprintln!(
            "reuse profile={} inputs={} seconds={:.2}",
            model.profile_id,
            inputs.len(),
            start.elapsed().as_secs_f64()
        );
        let mut blocks = BTreeMap::new();
        let mut s = c
            .prepare(&format!(
                "SELECT count(*),sum(size) FROM {}_chunks",
                table(model)
            ))
            .unwrap();
        let (count, slots) = s
            .query_row([], |r| Ok((r.get::<_, u64>(0)?, r.get::<_, u64>(1)?)))
            .unwrap();
        blocks.insert("blocks", count);
        blocks.insert("slots", slots);
        eprintln!("allocation profile={} {blocks:?}", model.profile_id);
    }
    assert_eq!(
        c.query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
            r.get::<_, usize>(0)
        })
        .unwrap(),
        0
    );
    drop(read);
    let start = Instant::now();
    let read = workspace
        .read(atlas_runtime::WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    atlas_index::validate_reference(read.connection()).unwrap();
    drop(read);
    eprintln!(
        "deep_validation seconds={:.2}",
        start.elapsed().as_secs_f64()
    );
    eprintln!(
        "storage before_bytes={before_size} after_bytes={} vectors_sha256={vectors} saved_sha256={saved}",
        std::fs::metadata(&path).unwrap().len()
    );
    // TempDir removes the corpus, migration backup and all sidecars on return.
}
