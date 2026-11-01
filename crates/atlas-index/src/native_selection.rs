//! Bounded exact native selection and checkpoint writes for an active target.
use crate::{IndexError, NATIVE_VALUE_BATCH_LIMIT, SourceSearchProfileIdentity};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::WriteSession;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeValueSelection {
    Missing,
    Reused,
    AlreadySelected,
}

pub(crate) struct NativeValue {
    pub vector: Vec<f32>,
    pub checksum: Vec<u8>,
}

pub(crate) fn bounded(length: usize) -> Result<(), IndexError> {
    if length > NATIVE_VALUE_BATCH_LIMIT {
        return Err(IndexError::Invalid(
            "native value batch exceeds its bound".into(),
        ));
    }
    Ok(())
}

pub(crate) fn require_active(c: &Connection, job: &str) -> Result<(), IndexError> {
    let active: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM native_update_targets t JOIN reference_update_jobs j USING(job_key) WHERE job_key=? AND phase='active' AND state='preparing' AND membership_digest IS NOT NULL AND expected_revision=(SELECT reference_revision FROM workspace WHERE id=1))", [job], |r|r.get(0))?;
    if !active {
        return Err(IndexError::Invalid(
            "native target is not active for this reference revision".into(),
        ));
    }
    Ok(())
}

pub(crate) fn member(
    c: &Connection,
    job: &str,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<(i64, Option<Vec<u8>>), IndexError> {
    let row:Option<(i64,usize,Option<usize>)>=c.query_row("SELECT i.input_id,i.token_count,length(t.selected_hash) FROM native_target_inputs t JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE t.job_key=? AND m.identity_json=? AND i.input_hash=?",params![job,serde_json::to_string(model)?,crate::embedding_values::digest(&input.input_sha256)?],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    let (id, tokens, length) = row.ok_or_else(|| {
        IndexError::Invalid("native input is absent from complete target membership".into())
    })?;
    if tokens != input.token_count || tokens == 0 || tokens > model.token_limit {
        return Err(IndexError::Invalid(
            "native membership token count mismatch".into(),
        ));
    }
    let hash = if let Some(length) = length {
        if length != 32 {
            return Err(IndexError::Invalid(
                "selected native checksum length mismatch".into(),
            ));
        }
        Some(c.query_row("SELECT selected_hash FROM native_target_inputs WHERE job_key=? AND input_id=? AND length(selected_hash)=32",params![job,id],|r|r.get::<_,Vec<u8>>(0))?)
    } else {
        None
    };
    Ok((id, hash))
}

/// Resolve only one numerical buffer at a time. Pending bytes are authoritative
/// even when a different, valid published value remains available.
pub(crate) fn load_value(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<NativeValue>, IndexError> {
    let row:Option<(i64,usize,Option<usize>)>=c.query_row("SELECT i.input_id,i.token_count,length(i.published_hash) FROM embedding_inputs i JOIN embedding_models m USING(model_id) WHERE m.identity_json=? AND i.input_hash=?",params![serde_json::to_string(model)?,crate::embedding_values::digest(&input.input_sha256)?],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    let Some((id, tokens, published)) = row else {
        return Ok(None);
    };
    if tokens != input.token_count || tokens == 0 || tokens > model.token_limit {
        return Err(IndexError::Invalid(
            "native input token count mismatch".into(),
        ));
    }
    let expected = model
        .dimensions
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| IndexError::Invalid("native vector length overflow".into()))?;
    let pending:Option<(usize,usize)>=c.query_row("SELECT length(vector),length(vector_hash) FROM pending_embedding_values WHERE input_id=?",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    let bytes = if let Some((length, hash_length)) = pending {
        if length != expected || hash_length != 32 {
            return Err(IndexError::Invalid(
                "pending native value length mismatch".into(),
            ));
        }
        Some(c.query_row("SELECT vector,vector_hash FROM pending_embedding_values WHERE input_id=? AND length(vector)=? AND length(vector_hash)=32",params![id,expected],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,Vec<u8>>(1)?)))?)
    } else {
        let table = crate::schema::vector_table(model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?)",
            [&table],
            |r| r.get(0),
        )?;
        let document: Option<usize> = if exists {
            c.query_row(&format!("SELECT length(embedding) FROM {table} WHERE unit_id=(SELECT min(unit_id) FROM semantic_units WHERE input_id=?)"),[id],|r|r.get(0)).optional()?
        } else {
            None
        };
        let bytes = if let Some(length) = document {
            if length != expected {
                return Err(IndexError::Invalid(
                    "published document native value length mismatch".into(),
                ));
            }
            Some(c.query_row(&format!("SELECT embedding FROM {table} WHERE unit_id=(SELECT min(unit_id) FROM semantic_units WHERE input_id=?1) AND length(embedding)=?2"),params![id,expected],|r|r.get::<_,Vec<u8>>(0))?)
        } else {
            let length: Option<usize> = c
                .query_row(
                    "SELECT length(vector) FROM trait_native_values WHERE input_id=?",
                    [id],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(length) = length {
                if length != expected {
                    return Err(IndexError::Invalid(
                        "trait native value length mismatch".into(),
                    ));
                }
                Some(c.query_row(
                    "SELECT vector FROM trait_native_values WHERE input_id=? AND length(vector)=?",
                    params![id, expected],
                    |r| r.get::<_, Vec<u8>>(0),
                )?)
            } else {
                None
            }
        };
        match bytes {
            Some(bytes) => {
                if published != Some(32) {
                    return Err(IndexError::Invalid(
                        "published native checksum missing or length mismatch".into(),
                    ));
                }
                let checksum=c.query_row("SELECT published_hash FROM embedding_inputs WHERE input_id=? AND length(published_hash)=32",[id],|r|r.get::<_,Vec<u8>>(0))?;
                Some((bytes, checksum))
            }
            None => None,
        }
    };
    let Some((bytes, checksum)) = bytes else {
        return Ok(None);
    };
    if crate::embedding_values::hash_bytes(&bytes) != checksum {
        return Err(IndexError::Invalid("native value checksum mismatch".into()));
    }
    let vector = crate::read_units::decode_vector(&bytes)?;
    crate::validation::check_vector(&vector, model.dimensions)?;
    Ok(Some(NativeValue { vector, checksum }))
}

/// Select and pin exact checksums without retaining reusable numerical arrays.
/// AlreadySelected deduplicates across bounded batches even in forced mode.
pub fn select_native_values(
    session: &WriteSession,
    job: &str,
    model: &SourceSearchProfileIdentity,
    inputs: &[NativeInputDescriptor],
    reuse: bool,
) -> Result<Vec<NativeValueSelection>, IndexError> {
    bounded(inputs.len())?;
    let c = session.connection();
    require_active(c, job)?;
    let mut output = Vec::with_capacity(inputs.len());
    for input in inputs {
        let (id, selected) = member(c, job, model, input)?;
        if let Some(expected) = selected {
            let value = load_value(c, model, input)?
                .ok_or_else(|| IndexError::Invalid("selected native value disappeared".into()))?;
            if value.checksum != expected {
                return Err(IndexError::Invalid(
                    "selected native checksum changed".into(),
                ));
            }
            output.push(NativeValueSelection::AlreadySelected);
        } else if reuse {
            if let Some(value) = load_value(c, model, input)? {
                c.execute("UPDATE native_target_inputs SET selected_hash=? WHERE job_key=? AND input_id=?",params![value.checksum,job,id])?;
                output.push(NativeValueSelection::Reused);
            } else {
                output.push(NativeValueSelection::Missing);
            }
        } else {
            output.push(NativeValueSelection::Missing);
        }
    }
    Ok(output)
}

/// A checkpoint never changes the published checksum or any published slot.
pub fn stage_native_values(
    session: &WriteSession,
    job: &str,
    model: &SourceSearchProfileIdentity,
    batch: &[(&NativeInputDescriptor, &[f32])],
) -> Result<(), IndexError> {
    bounded(batch.len())?;
    let c = session.connection();
    require_active(c, job)?;
    for (input, vector) in batch {
        crate::validation::check_vector(vector, model.dimensions)?;
        let (id, selected) = member(c, job, model, input)?;
        let bytes = crate::read_units::vector_bytes(vector);
        let checksum = crate::embedding_values::hash_bytes(&bytes);
        if selected.as_ref().is_some_and(|old| *old != checksum) {
            return Err(IndexError::Invalid(
                "attempted to replace an already selected native value".into(),
            ));
        }
        c.execute("INSERT INTO pending_embedding_values VALUES(?,?,?,?) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",params![id,bytes,checksum,job])?;
        c.execute(
            "UPDATE native_target_inputs SET selected_hash=? WHERE job_key=? AND input_id=?",
            params![checksum, job, id],
        )?;
    }
    Ok(())
}
