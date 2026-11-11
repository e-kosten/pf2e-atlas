//! Attest the published input registry independently of vector storage.
use crate::{IndexError, SourceSearchProfileIdentity, native_values::NativeModelKey};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

pub const NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION: u32 = 38;

const FORMAT: u8 = 1;
// Changing native counting semantics with unchanged assets must invalidate this recipe.
const DOMAIN: &[u8] = b"atlas/published-native-token-counts/v1\0";
const PROOF_BYTES: usize = 33;
// A power-of-two capacity keeps vector growth within roughly 9 MiB.
const MAX_LOOKUP_ROWS: usize = 262_144;
type Descriptor = ([u8; 32], u32);

/// One exact profile's authenticated published counts, in input hash order.
#[derive(Default)]
pub struct VerifiedNativeTokenCounts(Vec<Descriptor>);

impl VerifiedNativeTokenCounts {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn count_for(&self, input_sha256: &str) -> Option<usize> {
        let hash = crate::native_values::NativeInput::decode_hash(input_sha256).ok()?;
        let index = self.0.binary_search_by_key(&hash, |row| row.0).ok()?;
        Some(self.0[index].1 as usize)
    }
}

struct ModelRow {
    id: i64,
    proof: Option<Vec<u8>>,
}

fn model_row(c: &Connection, key: &NativeModelKey) -> Result<Option<ModelRow>, IndexError> {
    Ok(c.query_row(
        "SELECT model_id, CASE
            WHEN published_token_count_digest IS NULL THEN NULL
            WHEN typeof(published_token_count_digest)='blob'
             AND length(published_token_count_digest)=?2 THEN published_token_count_digest
            ELSE zeroblob(0) END
         FROM embedding_models WHERE identity_json=?1",
        params![key.identity_json(), PROOF_BYTES],
        |row| {
            Ok(ModelRow {
                id: row.get(0)?,
                proof: row.get(1)?,
            })
        },
    )
    .optional()?)
}

fn descriptor(row: &rusqlite::Row<'_>, token_limit: usize) -> Result<Descriptor, IndexError> {
    let hash: [u8; 32] = row
        .get_ref(0)?
        .as_blob()
        .map_err(|_| invalid("invalid published input hash"))?
        .try_into()
        .map_err(|_| invalid("invalid published input hash length"))?;
    let count = row
        .get::<_, u32>(1)
        .map_err(|_| invalid("invalid published token count"))?;
    if count == 0 || count as usize > token_limit {
        return Err(invalid("invalid published token count"));
    }
    Ok((hash, count))
}

fn scan(
    c: &Connection,
    key: &NativeModelKey,
    model_id: i64,
    mut accept: impl FnMut(Descriptor) -> Result<(), IndexError>,
) -> Result<[u8; PROOF_BYTES], IndexError> {
    let mut statement = c.prepare(
        "SELECT input_hash, token_count FROM embedding_inputs
         WHERE model_id=? AND published_hash IS NOT NULL ORDER BY input_hash",
    )?;
    let mut rows = statement.query([model_id])?;
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    digest.update((key.identity_json().len() as u64).to_be_bytes());
    digest.update(key.identity_json().as_bytes());
    let mut count = 0_u64;
    while let Some(row) = rows.next()? {
        let pair = descriptor(row, key.model().token_limit)?;
        digest.update(pair.0);
        digest.update(pair.1.to_be_bytes());
        accept(pair)?;
        count += 1;
    }
    digest.update(count.to_be_bytes());
    let mut proof = [FORMAT; PROOF_BYTES];
    proof[1..].copy_from_slice(&digest.finalize());
    Ok(proof)
}

pub(crate) fn verify(
    c: &Connection,
    key: &NativeModelKey,
    retain_lookup: bool,
) -> Result<VerifiedNativeTokenCounts, IndexError> {
    let Some(ModelRow {
        id,
        proof: Some(proof),
    }) = model_row(c, key)?
    else {
        return Ok(VerifiedNativeTokenCounts::default());
    };
    if proof.len() != PROOF_BYTES || proof[0] != FORMAT {
        return Err(invalid("invalid published token-count digest format"));
    }
    let mut lookup = Vec::new();
    let mut retain = retain_lookup;
    let actual = scan(c, key, id, |pair| {
        if retain && lookup.len() == MAX_LOOKUP_ROWS {
            retain = false;
            lookup = Vec::new();
        }
        if retain {
            lookup.push(pair);
        }
        Ok(())
    })?;
    if proof != actual {
        return Err(invalid("published token-count digest mismatch"));
    }
    Ok(VerifiedNativeTokenCounts(lookup))
}

/// Seal only tuples accepted through the verified ingest path and retained after pruning.
pub(crate) fn publish(
    c: &Connection,
    selected: &Connection,
    ordinal: usize,
    key: &NativeModelKey,
    verified: bool,
) -> Result<(), IndexError> {
    let ModelRow { id, proof } = model_row(c, key)?
        .ok_or_else(|| invalid("published model is missing from the token-count registry"))?;
    if !verified {
        if proof.is_some() {
            c.execute(
                "UPDATE embedding_models SET published_token_count_digest=NULL WHERE model_id=?",
                [id],
            )?;
        }
        return Ok(());
    }
    let mut statement = selected
        .prepare("SELECT input_hash,token_count FROM selected WHERE model=? ORDER BY input_hash")?;
    let mut rows = statement.query([ordinal])?;
    let mut next = rows
        .next()?
        .map(|row| descriptor(row, key.model().token_limit))
        .transpose()?;
    let actual = scan(c, key, id, |pair| {
        while next.is_some_and(|selected| selected.0 < pair.0) {
            next = rows
                .next()?
                .map(|row| descriptor(row, key.model().token_limit))
                .transpose()?;
        }
        if next != Some(pair) {
            return Err(invalid(
                "published token count lacks matching verified input",
            ));
        }
        Ok(())
    })?;
    if proof.as_deref() != Some(actual.as_slice()) {
        c.execute(
            "UPDATE embedding_models SET published_token_count_digest=?1 WHERE model_id=?2",
            params![actual.as_slice(), id],
        )?;
    }
    Ok(())
}

pub(crate) fn audit(
    c: &Connection,
    models: &[SourceSearchProfileIdentity],
) -> Result<(), IndexError> {
    for key in NativeModelKey::all(models)? {
        verify(c, &key, false)?;
    }
    Ok(())
}

fn invalid(message: &str) -> IndexError {
    IndexError::ReferenceRepairRequired(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_binds_model_and_published_descriptors_but_ignores_pending_inputs() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY, identity_json TEXT,
                published_token_count_digest BLOB);
             CREATE TABLE embedding_inputs(model_id INTEGER, input_hash BLOB, token_count INTEGER,
                published_hash BLOB);
             INSERT INTO embedding_inputs VALUES (1,zeroblob(32),7,zeroblob(32));
             INSERT INTO embedding_inputs VALUES (1,randomblob(32),8,NULL);",
        )
        .unwrap();
        let model = SourceSearchProfileIdentity::from_spec(atlas_embedding::profile_for_model(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        ));
        let key = NativeModelKey::new(0, &model).unwrap();
        let proof = scan(&c, &key, 1, |_| Ok(())).unwrap();
        c.execute(
            "INSERT INTO embedding_models VALUES (1,?1,?2)",
            params![key.identity_json(), proof.as_slice()],
        )
        .unwrap();
        assert_eq!(
            verify(&c, &key, true).unwrap().count_for(&"00".repeat(32)),
            Some(7)
        );
        c.execute(
            "UPDATE embedding_inputs SET token_count=9 WHERE published_hash IS NULL",
            [],
        )
        .unwrap();
        assert!(verify(&c, &key, false).is_ok());
        for sql in [
            "UPDATE embedding_inputs SET published_hash=NULL",
            "UPDATE embedding_inputs SET published_hash=zeroblob(32)",
            "UPDATE embedding_inputs SET token_count=8 WHERE published_hash IS NOT NULL",
            "UPDATE embedding_models SET published_token_count_digest=zeroblob(1)",
        ] {
            c.execute_batch("SAVEPOINT corrupted").unwrap();
            c.execute(sql, []).unwrap();
            assert!(verify(&c, &key, false).is_err(), "{sql}");
            c.execute_batch("ROLLBACK TO corrupted; RELEASE corrupted")
                .unwrap();
        }
        let mut other = model;
        other.input_recipe.push_str("-other");
        let other = NativeModelKey::new(0, &other).unwrap();
        c.execute(
            "UPDATE embedding_models SET identity_json=?",
            [other.identity_json()],
        )
        .unwrap();
        assert!(verify(&c, &other, false).is_err());
    }
}
