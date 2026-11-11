//! Exact native value checksums and fixture registry inputs.
#[cfg(any(test, feature = "test-support"))]
use crate::IndexError;
#[cfg(test)]
use crate::SourceSearchProfileIdentity;
#[cfg(test)]
use rusqlite::{Connection, params};

#[cfg(any(test, feature = "test-support"))]
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
#[cfg(test)]
pub(crate) fn model_id(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
) -> Result<i64, IndexError> {
    let identity = serde_json::to_string(model)?;
    c.prepare_cached(
        "INSERT INTO embedding_models(identity_json) VALUES(?) ON CONFLICT DO NOTHING",
    )?
    .execute([&identity])?;
    Ok(
        c.prepare_cached("SELECT model_id FROM embedding_models WHERE identity_json=?")?
            .query_row([identity], |r| r.get(0))?,
    )
}
#[cfg(test)]
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
    c.prepare_cached("INSERT INTO embedding_inputs(model_id,input_hash,token_count) VALUES(?,?,?) ON CONFLICT DO NOTHING")?.execute(params![model,hash,tokens])?;
    let (id, stored): (i64, usize) = c
        .prepare_cached(
            "SELECT input_id,token_count FROM embedding_inputs WHERE model_id=? AND input_hash=?",
        )?
        .query_row(params![model, hash], |r| Ok((r.get(0)?, r.get(1)?)))?;
    if stored != tokens {
        return Err(IndexError::Invalid(
            "native input token count mismatch".into(),
        ));
    }
    Ok(id)
}
