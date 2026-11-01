//! Exact native input identity and values. Published passage vectors live only in
//! vec0; trait-only native values and unfinished computations have bounded stores.
use crate::{
    IndexError, SourceSearchProfileIdentity,
    read_units::{decode_vector, vector_bytes},
    validation::check_vector,
};
use rusqlite::{Connection, OptionalExtension, params};

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
/// Final activation releases pending values and unreferenced published metadata.
/// Complete-target replacement owns bounded preparation cleanup separately.
/// Retain native values needed by traits whose last document disappeared.
pub(crate) fn prune(c: &Connection) -> Result<(), IndexError> {
    c.execute_batch("DELETE FROM trait_native_values WHERE EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=trait_native_values.input_id) OR NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=trait_native_values.input_id);
DELETE FROM pending_embedding_values;
DELETE FROM embedding_inputs WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM native_target_inputs n WHERE n.input_id=embedding_inputs.input_id);
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
    let expected = model
        .dimensions
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| IndexError::Invalid("native vector length overflow".into()))?;
    let shape:Option<(usize,usize)>=c.query_row(&format!("SELECT length(v.embedding),length(i.published_hash) FROM embedding_inputs i JOIN {table} v ON v.unit_id=(SELECT min(unit_id) FROM semantic_units u WHERE u.input_id=i.input_id) WHERE i.input_id=? UNION ALL SELECT length(t.vector),length(i.published_hash) FROM trait_native_values t JOIN embedding_inputs i USING(input_id) WHERE input_id=?1 LIMIT 1"),[input],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if shape != Some((expected, 32)) {
        return Err(IndexError::Invalid(
            "published native embedding missing or length mismatch".into(),
        ));
    }
    let (bytes,hash):(Vec<u8>,Vec<u8>)=c.query_row(&format!("SELECT v.embedding,i.published_hash FROM embedding_inputs i JOIN {table} v ON v.unit_id=(SELECT min(unit_id) FROM semantic_units u WHERE u.input_id=i.input_id) WHERE i.input_id=?1 AND length(v.embedding)=?2 AND length(i.published_hash)=32 UNION ALL SELECT t.vector,i.published_hash FROM trait_native_values t JOIN embedding_inputs i USING(input_id) WHERE input_id=?1 AND length(t.vector)=?2 AND length(i.published_hash)=32 LIMIT 1"),params![input,expected],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if hash_bytes(&bytes) != hash {
        return Err(IndexError::Invalid(
            "native embedding checksum mismatch".into(),
        ));
    }
    let vector = decode_vector(&bytes)?;
    check_vector(&vector, model.dimensions)?;
    Ok(vector)
}
