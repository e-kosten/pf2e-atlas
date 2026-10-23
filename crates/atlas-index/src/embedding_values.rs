//! Exact native input identity and values. Published passage vectors live only in
//! vec0; trait-only native values and unfinished computations have bounded stores.
use crate::{
    IndexError, SourceSearchProfileIdentity,
    read_units::{decode_vector, vector_bytes},
    validation::check_vector,
};
use atlas_embedding::PreparedEmbeddingInput;
use atlas_storage::{ReadSession, WriteSession};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;

pub(crate) fn digest(value: &str) -> Result<Vec<u8>, IndexError> {
    if !crate::codec::checked_hash(value) {
        return Err(IndexError::Invalid("invalid SHA-256 digest".into()));
    }
    (0..value.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&value[i..i + 2], 16).map_err(|e| IndexError::Invalid(e.to_string()))
        })
        .collect()
}
pub(crate) fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).to_vec()
}
pub(crate) fn model_id(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
) -> Result<i64, IndexError> {
    let identity = serde_json::to_string(model)?;
    c.execute(
        "INSERT INTO embedding_models(identity_json) VALUES(?) ON CONFLICT DO NOTHING",
        [&identity],
    )?;
    Ok(c.query_row(
        "SELECT model_id FROM embedding_models WHERE identity_json=?",
        [identity],
        |r| r.get(0),
    )?)
}
pub(crate) fn input_id(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    hash: &str,
    tokens: usize,
) -> Result<i64, IndexError> {
    if tokens == 0 || tokens > model.token_limit {
        return Err(IndexError::Invalid("invalid native token count".into()));
    }
    let model = model_id(c, model)?;
    let hash = digest(hash)?;
    c.execute("INSERT INTO embedding_inputs(model_id,input_hash,token_count) VALUES(?,?,?) ON CONFLICT DO NOTHING",params![model,hash,tokens])?;
    let (id, stored): (i64, usize) = c.query_row(
        "SELECT input_id,token_count FROM embedding_inputs WHERE model_id=? AND input_hash=?",
        params![model, hash],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if stored != tokens {
        return Err(IndexError::Invalid(
            "native input token count mismatch".into(),
        ));
    }
    Ok(id)
}
pub(crate) fn publish_input(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    hash: &str,
    tokens: usize,
    vector: &[f32],
) -> Result<i64, IndexError> {
    check_vector(vector, model.dimensions)?;
    let id = input_id(c, model, hash, tokens)?;
    let checksum = hash_bytes(&vector_bytes(vector));
    c.execute("UPDATE embedding_inputs SET published_hash=?1 WHERE input_id=?2 AND published_hash IS NOT ?1",params![checksum,id])?;
    Ok(id)
}
/// Pending computations take precedence over the still-readable publication.
/// One batch SQL lookup per value store; no per-input vector SQL queries.
pub fn read_reusable_embeddings(
    session: &ReadSession,
    model: &SourceSearchProfileIdentity,
    inputs: &[PreparedEmbeddingInput],
) -> Result<BTreeMap<String, Vec<f32>>, IndexError> {
    let c = session.connection();
    let identity = serde_json::to_string(model)?;
    c.execute_batch("DROP TABLE IF EXISTS temp.requested_embedding_inputs; CREATE TEMP TABLE requested_embedding_inputs(input_hash BLOB PRIMARY KEY,token_count INTEGER NOT NULL) WITHOUT ROWID;")?;
    let result = (|| {
        let mut requested = BTreeMap::new();
        let mut insert = c.prepare(
            "INSERT INTO temp.requested_embedding_inputs VALUES(?,?) ON CONFLICT DO NOTHING",
        )?;
        for input in inputs {
            let hash = digest(&input.input_sha256)?;
            if input.token_count == 0
                || input.token_count > model.token_limit
                || requested
                    .get(&hash)
                    .is_some_and(|previous: &&PreparedEmbeddingInput| {
                        previous.token_count != input.token_count
                    })
            {
                return Err(IndexError::Invalid(
                    "native input token count mismatch".into(),
                ));
            }
            insert.execute(params![hash, input.token_count])?;
            requested.entry(hash).or_insert(input);
        }
        drop(insert);
        let mut reusable = BTreeMap::new();
        let table = crate::schema::vector_table(model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?)",
            [&table],
            |r| r.get(0),
        )?;
        let mut queries=vec![
            "SELECT i.input_hash,i.token_count,p.vector,p.vector_hash FROM embedding_models m JOIN embedding_inputs i USING(model_id) JOIN temp.requested_embedding_inputs q USING(input_hash) JOIN pending_embedding_values p USING(input_id) WHERE m.identity_json=?1".to_owned(),
            "SELECT i.input_hash,i.token_count,t.vector,i.published_hash FROM embedding_models m JOIN embedding_inputs i USING(model_id) JOIN temp.requested_embedding_inputs q USING(input_hash) JOIN trait_native_values t USING(input_id) WHERE m.identity_json=?1 AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=i.input_id)".to_owned(),
        ];
        if exists {
            // Scan unit IDs in storage order instead of sorting vector blobs or
            // making randomly ordered vec0 lookups through the input-hash index.
            queries.push(format!("SELECT i.input_hash,i.token_count,v.embedding,i.published_hash FROM semantic_units u NOT INDEXED JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) JOIN temp.requested_embedding_inputs q USING(input_hash) LEFT JOIN {table} v USING(unit_id) WHERE m.identity_json=?1 AND u.unit_id=(SELECT min(unit_id) FROM semantic_units s WHERE s.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=i.input_id) ORDER BY u.unit_id"));
        }
        for sql in queries {
            let mut statement = c.prepare(&sql)?;
            let mut rows = statement.query([&identity])?;
            while let Some(row) = rows.next()? {
                let hash: Vec<u8> = row.get(0)?;
                let input = requested
                    .get(&hash)
                    .ok_or_else(|| IndexError::Invalid("unrequested embedding".into()))?;
                let tokens: usize = row.get(1)?;
                let bytes: Vec<u8> = row.get(2)?;
                let checksum: Vec<u8> = row.get(3)?;
                if tokens != input.token_count || hash_bytes(&bytes) != checksum {
                    return Err(IndexError::Invalid(
                        "reusable embedding identity or checksum mismatch".into(),
                    ));
                }
                let vector = decode_vector(&bytes)?;
                check_vector(&vector, model.dimensions)?;
                reusable.insert(input.reuse_key.clone(), vector);
            }
        }
        Ok(reusable)
    })();
    c.execute_batch("DROP TABLE temp.requested_embedding_inputs")?;
    result
}
/// Commit only after inference releases its work; never change published hashes.
pub fn stage_embedding_values(
    session: &WriteSession,
    job: &str,
    model: &SourceSearchProfileIdentity,
    batch: &[(&PreparedEmbeddingInput, &[f32])],
) -> Result<(), IndexError> {
    let c = session.connection();
    for (input, vector) in batch {
        check_vector(vector, model.dimensions)?;
        let id = input_id(c, model, &input.input_sha256, input.token_count)?;
        let bytes = vector_bytes(vector);
        c.execute("INSERT INTO pending_embedding_values VALUES(?,?,?,?) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",params![id,bytes,hash_bytes(&bytes),job])?;
    }
    Ok(())
}
/// Publication is the sole garbage collection boundary. Retain native values
/// needed by traits whose last document occurrence disappeared in this update.
pub(crate) fn prune(c: &Connection) -> Result<(), IndexError> {
    c.execute_batch("DELETE FROM trait_native_values WHERE EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=trait_native_values.input_id) OR NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=trait_native_values.input_id);
DELETE FROM pending_embedding_values;
DELETE FROM embedding_inputs WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=embedding_inputs.input_id);
DELETE FROM embedding_models WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id);
DELETE FROM semantic_locations WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.location_id=semantic_locations.location_id);")?;
    Ok(())
}
pub(crate) fn native_value(
    c: &Connection,
    input: i64,
    model: &SourceSearchProfileIdentity,
) -> Result<Vec<f32>, IndexError> {
    let table = crate::schema::vector_table(model)?;
    let row:Option<(Vec<u8>,Vec<u8>)>=c.query_row(&format!("SELECT v.embedding,i.published_hash FROM embedding_inputs i JOIN {table} v ON v.unit_id=(SELECT min(unit_id) FROM semantic_units u WHERE u.input_id=i.input_id) WHERE i.input_id=? UNION ALL SELECT t.vector,i.published_hash FROM trait_native_values t JOIN embedding_inputs i USING(input_id) WHERE input_id=?1 LIMIT 1"),[input],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    let (bytes, hash) =
        row.ok_or_else(|| IndexError::Invalid("published native embedding missing".into()))?;
    if hash_bytes(&bytes) != hash {
        return Err(IndexError::Invalid(
            "native embedding checksum mismatch".into(),
        ));
    }
    let vector = decode_vector(&bytes)?;
    check_vector(&vector, model.dimensions)?;
    Ok(vector)
}
