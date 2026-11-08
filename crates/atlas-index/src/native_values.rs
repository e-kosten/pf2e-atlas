//! Input-addressed numerical operations owned by the publishing transaction.
//! Every operation uses the supplied connection; no workspace or secondary session is opened.
use crate::{IndexError, SourceSearchProfileIdentity};
use atlas_embedding::NativeInputDescriptor;
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn expected_bytes(model: &SourceSearchProfileIdentity) -> Result<usize, IndexError> {
    model
        .dimensions
        .checked_mul(std::mem::size_of::<f32>())
        .filter(|n| *n > 0)
        .ok_or_else(|| invalid("native vector length overflow"))
}

pub(crate) fn check_bytes(
    bytes: &[u8],
    checksum: &[u8],
    model: &SourceSearchProfileIdentity,
) -> Result<(), IndexError> {
    decode_checked(bytes, checksum, model).map(|_| ())
}

pub(crate) fn verify_bytes(
    bytes: &[u8],
    checksum: &[u8],
    model: &SourceSearchProfileIdentity,
) -> Result<(), IndexError> {
    if bytes.len() != expected_bytes(model)?
        || checksum.len() != 32
        || crate::embedding_values::hash_bytes(bytes) != checksum
    {
        return Err(invalid("native vector length or checksum mismatch"));
    }
    Ok(())
}

fn decode_checked(
    bytes: &[u8],
    checksum: &[u8],
    model: &SourceSearchProfileIdentity,
) -> Result<Vec<f32>, IndexError> {
    verify_bytes(bytes, checksum, model)?;
    let vector = crate::read_units::decode_vector(bytes)?;
    crate::validation::check_vector(&vector, model.dimensions)?;
    Ok(vector)
}

/// Immutable execution identity; it owns no database IDs and survives cache eviction.
pub(crate) struct NativeModelKey {
    model: SourceSearchProfileIdentity,
    ordinal: usize,
    json: String,
    table: String,
}
impl NativeModelKey {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn model(&self) -> &SourceSearchProfileIdentity {
        &self.model
    }
    pub fn all(models: &[SourceSearchProfileIdentity]) -> Result<Vec<Self>, IndexError> {
        models
            .iter()
            .enumerate()
            .map(|(ordinal, model)| Self::new(ordinal, model))
            .collect()
    }
    pub fn new(ordinal: usize, model: &SourceSearchProfileIdentity) -> Result<Self, IndexError> {
        Ok(Self {
            model: model.clone(),
            ordinal,
            json: serde_json::to_string(model)?,
            table: crate::schema::native_vector_table(model)?,
        })
    }
}

/// A model ID is valid only during this admitted snapshot/transaction, before any GC.
pub(crate) struct NativeModel<'session> {
    connection: &'session Connection,
    pub key: &'session NativeModelKey,
    id: Option<i64>,
    native_exists: bool,
}
pub(crate) struct NativeInput {
    hash: [u8; 32],
    token_count: usize,
}
impl NativeInput {
    pub fn new(
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Self, IndexError> {
        if !crate::codec::checked_hash(&input.input_sha256) {
            return Err(invalid("invalid SHA-256 digest"));
        }
        let mut hash = [0; 32];
        for (byte, index) in hash.iter_mut().zip((0..64).step_by(2)) {
            *byte = u8::from_str_radix(&input.input_sha256[index..index + 2], 16)
                .map_err(|error| IndexError::Invalid(error.to_string()))?;
        }
        Self::from_hash(model, hash, input.token_count)
    }
    pub fn from_hash(
        model: &SourceSearchProfileIdentity,
        hash: [u8; 32],
        token_count: usize,
    ) -> Result<Self, IndexError> {
        if token_count == 0 || token_count > model.token_limit {
            return Err(invalid("invalid native token count"));
        }
        Ok(Self { hash, token_count })
    }
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
    pub fn token_count(&self) -> usize {
        self.token_count
    }
}
struct InputRow {
    id: i64,
    published_checksum: Option<Vec<u8>>,
}
pub(crate) struct SelectedNative {
    pub checksum: Vec<u8>,
    pub staged: Option<Vec<u8>>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CachedNative {
    pub bytes: Vec<u8>,
    pub checksum: Vec<u8>,
}
impl<'session> NativeModel<'session> {
    pub fn bind(
        connection: &'session Connection,
        key: &'session NativeModelKey,
        write: bool,
    ) -> Result<Self, IndexError> {
        if write {
            connection
                .prepare_cached(
                    "INSERT INTO embedding_models(identity_json) VALUES(?) ON CONFLICT DO NOTHING",
                )?
                .execute([&key.json])?;
        }
        let id = connection
            .prepare_cached("SELECT model_id FROM embedding_models WHERE identity_json=?")?
            .query_row([&key.json], |row| row.get(0))
            .optional()?;
        let native_exists = connection
            .prepare_cached("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)")?
            .query_row([&key.table], |row| row.get(0))?;
        Ok(Self {
            connection,
            key,
            id,
            native_exists,
        })
    }
    fn find(&self, input: &NativeInput) -> Result<Option<InputRow>, IndexError> {
        let Some(model_id) = self.id else {
            return Ok(None);
        };
        let row: Option<(i64, usize, Option<Vec<u8>>)> = self.connection.prepare_cached("SELECT input_id,token_count,CASE WHEN published_hash IS NULL THEN NULL WHEN typeof(published_hash)='blob' AND length(published_hash)=32 THEN published_hash ELSE zeroblob(0) END FROM embedding_inputs WHERE model_id=? AND input_hash=?")?
            .query_row(params![model_id,input.hash()], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
        match row {
            Some((id, tokens, published_checksum)) if tokens == input.token_count => {
                Ok(Some(InputRow {
                    id,
                    published_checksum,
                }))
            }
            Some(_) => Err(invalid("native input token count mismatch")),
            None => Ok(None),
        }
    }
    pub fn input_id(&self, input: &NativeInput) -> Result<Option<i64>, IndexError> {
        Ok(self.find(input)?.map(|row| row.id))
    }
    fn checksum(&self, row: &InputRow) -> Result<Option<Vec<u8>>, IndexError> {
        match &row.published_checksum {
            None => {
                let consumers: bool = self
                    .connection
                    .prepare_cached(
                        "SELECT EXISTS(SELECT 1 FROM semantic_units WHERE input_id=?1)",
                    )?
                    .query_row([row.id], |r| r.get(0))?;
                let native: bool = self.native_exists
                    && self
                        .connection
                        .prepare_cached(&format!(
                            "SELECT EXISTS(SELECT 1 FROM {} WHERE input_id=?)",
                            self.key.table
                        ))?
                        .query_row([row.id], |r| r.get(0))?;
                if consumers || native {
                    return Err(invalid(
                        "published native checksum missing for existing value or consumer",
                    ));
                }
                Ok(None)
            }
            Some(hash) if hash.len() == 32 => Ok(Some(hash.clone())),
            Some(_) => Err(invalid("published native checksum length mismatch")),
        }
    }
    pub fn published_checksum(&self, input: &NativeInput) -> Result<Option<Vec<u8>>, IndexError> {
        self.find(input)?
            .map(|row| self.checksum(&row))
            .transpose()
            .map(Option::flatten)
    }
    fn cache(&self, id: i64) -> Result<Option<CachedNative>, IndexError> {
        let shape: Option<(usize,usize)> = self.connection.prepare_cached("SELECT length(vector),length(vector_hash) FROM pending_embedding_values WHERE input_id=?")?
            .query_row([id], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
        let Some((length, hash_length)) = shape else {
            return Ok(None);
        };
        if length != expected_bytes(&self.key.model)? || hash_length != 32 {
            return Err(invalid("cached native length mismatch"));
        }
        let (bytes,hash): (Vec<u8>,Vec<u8>) = self.connection.prepare_cached("SELECT vector,vector_hash FROM pending_embedding_values WHERE input_id=? AND length(vector)=? AND length(vector_hash)=32")?
            .query_row(params![id,length], |r| Ok((r.get(0)?,r.get(1)?)))?;
        verify_bytes(&bytes, &hash, &self.key.model)?;
        Ok(Some(CachedNative {
            bytes,
            checksum: hash,
        }))
    }
    /// One exact input lookup, retaining pending-over-published precedence.
    pub fn select(
        &self,
        input: &NativeInput,
        reuse_published: bool,
    ) -> Result<Option<SelectedNative>, IndexError> {
        let Some(row) = self.find(input)? else {
            return Ok(None);
        };
        if let Some(cached) = self.cache(row.id)? {
            return Ok(Some(SelectedNative {
                checksum: cached.checksum,
                staged: Some(cached.bytes),
            }));
        }
        if !reuse_published {
            return Ok(None);
        }
        let Some(hash) = self.checksum(&row)? else {
            return Ok(None);
        };
        Ok(Some(SelectedNative {
            checksum: hash,
            staged: None,
        }))
    }
    /// Copy an authenticated value into a publication with a different layout.
    pub fn copy_published(
        &self,
        input: &NativeInput,
    ) -> Result<Option<SelectedNative>, IndexError> {
        let Some(row) = self.find(input)? else {
            return Ok(None);
        };
        let Some(checksum) = self.checksum(&row)? else {
            return Ok(None);
        };
        let vector = self.read(row.id)?;
        Ok(Some(SelectedNative {
            checksum,
            staged: Some(crate::read_units::vector_bytes(&vector)),
        }))
    }
    /// Shape is checked before any numerical or checksum BLOB allocation.
    pub fn read(&self, id: i64) -> Result<Vec<f32>, IndexError> {
        let model_id = self
            .id
            .ok_or_else(|| invalid("selected native model missing"))?;
        let shape: (Option<usize>,Option<usize>,usize) = self.connection.prepare_cached(&format!("SELECT length(v.embedding),length(i.published_hash),i.token_count FROM embedding_inputs i LEFT JOIN {} v USING(input_id) WHERE i.input_id=? AND i.model_id=?",self.key.table))?
            .query_row(params![id,model_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let length = expected_bytes(&self.key.model)?;
        if shape.0 != Some(length)
            || shape.1 != Some(32)
            || shape.2 == 0
            || shape.2 > self.key.model.token_limit
        {
            return Err(invalid("published native geometry or token count mismatch"));
        }
        let (bytes,hash): (Vec<u8>,Vec<u8>) = self.connection.prepare_cached(&format!("SELECT v.embedding,i.published_hash FROM embedding_inputs i JOIN {} v USING(input_id) WHERE input_id=? AND length(v.embedding)=? AND length(i.published_hash)=32",self.key.table))?
            .query_row(params![id,length], |r| Ok((r.get(0)?,r.get(1)?)))?;
        decode_checked(&bytes, &hash, &self.key.model)
    }
    pub fn ensure_input(&self, input: &NativeInput) -> Result<i64, IndexError> {
        let model_id = self
            .id
            .ok_or_else(|| invalid("native write model missing"))?;
        self.connection.prepare_cached("INSERT INTO embedding_inputs(model_id,input_hash,token_count) VALUES(?,?,?) ON CONFLICT DO NOTHING")?
            .execute(params![model_id,input.hash(),input.token_count])?;
        self.input_id(input)?
            .ok_or_else(|| invalid("native input registration missing"))
    }
    pub fn ensure_table(&mut self) -> Result<(), IndexError> {
        if !self.native_exists {
            self.connection
                .execute_batch(&crate::schema::native_vector_ddl(&self.key.model)?)?;
            self.native_exists = true;
        }
        Ok(())
    }
    pub fn install(
        &mut self,
        input: &NativeInput,
        bytes: &[u8],
        checksum: &[u8],
    ) -> Result<i64, IndexError> {
        if bytes.len() != expected_bytes(&self.key.model)? || checksum.len() != 32 {
            return Err(invalid("native vector length or checksum mismatch"));
        }
        let id = self.ensure_input(input)?;
        self.ensure_table()?;
        let previous_length: Option<usize> = self
            .connection
            .prepare_cached(&format!(
                "SELECT length(embedding) FROM {} WHERE input_id=?",
                self.key.table
            ))?
            .query_row([id], |r| r.get(0))
            .optional()?;
        let expected = expected_bytes(&self.key.model)?;
        if previous_length.is_some_and(|n| n != expected) {
            return Err(invalid("previous native vector length mismatch"));
        }
        let previous: Option<Vec<u8>> = if previous_length.is_some() {
            Some(
                self.connection
                    .prepare_cached(&format!(
                        "SELECT embedding FROM {} WHERE input_id=? AND length(embedding)=?",
                        self.key.table
                    ))?
                    .query_row(params![id, expected], |r| r.get(0))?,
            )
        } else {
            None
        };
        if previous.as_deref() != Some(bytes) {
            if previous.is_some() {
                self.connection
                    .prepare_cached(&format!(
                        "UPDATE {} SET embedding=? WHERE input_id=?",
                        self.key.table
                    ))?
                    .execute(params![bytes, id])?;
            } else {
                self.connection
                    .prepare_cached(&format!(
                        "INSERT INTO {}(input_id,embedding) VALUES(?,?)",
                        self.key.table
                    ))?
                    .execute(params![id, bytes])?;
            }
        }
        self.connection.prepare_cached("UPDATE embedding_inputs SET published_hash=? WHERE input_id=? AND published_hash IS NOT ?1")?.execute(params![checksum,id])?;
        Ok(id)
    }
    pub fn selected_input(&self, input: &NativeInput, expected: &[u8]) -> Result<i64, IndexError> {
        let row = self
            .find(input)?
            .ok_or_else(|| invalid("selected native input missing"))?;
        if expected.len() != 32 || self.checksum(&row)?.as_deref() != Some(expected) {
            return Err(invalid(
                "selected native checksum differs from installed input",
            ));
        }
        Ok(row.id)
    }
}

pub(crate) struct NativeBindings<'session> {
    models: Vec<NativeModel<'session>>,
}
impl<'session> NativeBindings<'session> {
    pub fn new(
        c: &'session Connection,
        keys: &'session [NativeModelKey],
        write: bool,
    ) -> Result<Self, IndexError> {
        Ok(Self {
            models: keys
                .iter()
                .map(|key| NativeModel::bind(c, key, write))
                .collect::<Result<_, _>>()?,
        })
    }
    pub fn profile(
        &self,
        profile: &atlas_domain::SearchProfileId,
    ) -> Result<&NativeModel<'session>, IndexError> {
        self.models
            .iter()
            .find(|model| &model.key.model.profile_id == profile)
            .ok_or_else(|| invalid("native model absent from publication"))
    }
    pub fn ordinal_mut(
        &mut self,
        ordinal: usize,
    ) -> Result<&mut NativeModel<'session>, IndexError> {
        self.models
            .get_mut(ordinal)
            .ok_or_else(|| invalid("selected model ordinal invalid"))
    }
    pub fn ensure_tables(&mut self) -> Result<(), IndexError> {
        for model in &mut self.models {
            model.ensure_table()?;
        }
        Ok(())
    }
}

// One-operation conveniences for genuine fixture/reader operations; production
// preparation and publishing use a single scoped binding for the complete batch.
#[cfg(test)]
pub(crate) fn lookup_input(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<i64>, IndexError> {
    let keys = NativeModelKey::all(std::slice::from_ref(model))?;
    NativeModel::bind(c, &keys[0], false)?.input_id(&NativeInput::new(model, input)?)
}
#[cfg(test)]
pub(crate) fn published_checksum(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<Vec<u8>>, IndexError> {
    let keys = NativeModelKey::all(std::slice::from_ref(model))?;
    NativeModel::bind(c, &keys[0], false)?.published_checksum(&NativeInput::new(model, input)?)
}
#[cfg(test)]
pub(crate) fn read(
    c: &Connection,
    id: i64,
    model: &SourceSearchProfileIdentity,
) -> Result<Vec<f32>, IndexError> {
    let keys = NativeModelKey::all(std::slice::from_ref(model))?;
    NativeModel::bind(c, &keys[0], false)?.read(id)
}
#[cfg(test)]
pub(crate) fn install(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
    bytes: &[u8],
    hash: &[u8],
) -> Result<i64, IndexError> {
    check_bytes(bytes, hash, model)?;
    let keys = NativeModelKey::all(std::slice::from_ref(model))?;
    NativeModel::bind(c, &keys[0], true)?.install(&NativeInput::new(model, input)?, bytes, hash)
}
#[cfg(test)]
pub(crate) fn cached(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<CachedNative>, IndexError> {
    let keys = NativeModelKey::all(std::slice::from_ref(model))?;
    let binding = NativeModel::bind(c, &keys[0], false)?;
    binding
        .find(&NativeInput::new(model, input)?)?
        .map(|row| binding.cache(row.id))
        .transpose()
        .map(Option::flatten)
}
fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}

/// Corrupt NULL publication checksums never authorize deleting live associations.
pub(crate) fn prune_registry(c: &Connection) -> Result<(), IndexError> {
    c.execute_batch("DELETE FROM embedding_inputs WHERE published_hash IS NULL
        AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id)
        AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=embedding_inputs.input_id);
        DELETE FROM embedding_models WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id)
        AND NOT EXISTS(SELECT 1 FROM search_profiles p WHERE p.identity_json=embedding_models.identity_json);")?;
    Ok(())
}

/// Run after every document occurrence, before final validation.
pub(crate) fn prune(c: &Connection) -> Result<(), IndexError> {
    let profiles = c
        .prepare("SELECT identity_json FROM search_profiles")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let models = c
        .prepare("SELECT model_id,identity_json FROM embedding_models ORDER BY model_id")?
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, json) in models {
        let model: SourceSearchProfileIdentity = serde_json::from_str(&json)?;
        let table = crate::schema::native_vector_table(&model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)",
            [&table],
            |r| r.get(0),
        )?;
        let mut obsolete = c.prepare("SELECT input_id FROM embedding_inputs i WHERE model_id=? AND published_hash IS NOT NULL AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id)")?;
        let mut rows = obsolete.query([id])?;
        while let Some(row) = rows.next()? {
            let input_id: i64 = row.get(0)?;
            if exists {
                c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [input_id])?;
            }
            c.execute(
                "UPDATE embedding_inputs SET published_hash=NULL WHERE input_id=?",
                [input_id],
            )?;
        }
        drop(rows);
        drop(obsolete);
        if exists && !profiles.contains(&json) {
            let referenced: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM embedding_inputs i WHERE model_id=? AND EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id))",[id],|r|r.get(0))?;
            if referenced {
                return Err(invalid("retired numerical model still has consumers"));
            }
            c.execute_batch(&format!("DROP TABLE {table}"))?;
        }
    }
    c.execute("DELETE FROM semantic_locations WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.location_id=semantic_locations.location_id)",[])?;
    prune_registry(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_model_binding_keeps_full_identity_and_rebinds_after_collection() {
        let (c, model, input) = fixture();
        let keys = NativeModelKey::all(std::slice::from_ref(&model)).unwrap();
        {
            let binding = NativeModel::bind(&c, &keys[0], false).unwrap();
            assert_eq!(
                binding
                    .input_id(&NativeInput::new(&model, &input).unwrap())
                    .unwrap(),
                Some(7)
            );
            let mut wrong = input.clone();
            wrong.token_count += 1;
            assert!(
                binding
                    .input_id(&NativeInput::new(&model, &wrong).unwrap())
                    .is_err()
            );
            let mut missing = input.clone();
            missing.input_sha256 = "b".repeat(64);
            assert_eq!(
                binding
                    .input_id(&NativeInput::new(&model, &missing).unwrap())
                    .unwrap(),
                None
            );
        }
        let mut changed = model.clone();
        changed.revision.push_str("-other");
        c.execute(
            "UPDATE embedding_models SET identity_json=? WHERE model_id=1",
            [serde_json::to_string(&changed).unwrap()],
        )
        .unwrap();
        assert_eq!(
            NativeModel::bind(&c, &keys[0], false)
                .unwrap()
                .input_id(&NativeInput::new(&model, &input).unwrap())
                .unwrap(),
            None
        );
        // The same immutable key can safely register again after eviction removes
        // an unpublished model. No database ID survives in that immutable key.
        c.execute_batch("DELETE FROM embedding_inputs; DELETE FROM embedding_models; INSERT INTO embedding_models VALUES(9,'occupied');").unwrap();
        let binding = NativeModel::bind(&c, &keys[0], true).unwrap();
        let id = binding
            .ensure_input(&NativeInput::new(&model, &input).unwrap())
            .unwrap();
        assert_eq!(
            c.query_row(
                "SELECT model_id FROM embedding_inputs WHERE input_id=?",
                [id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            10
        );
        assert_eq!(
            binding
                .input_id(&NativeInput::new(&model, &input).unwrap())
                .unwrap(),
            Some(id)
        );
    }

    fn model() -> SourceSearchProfileIdentity {
        SourceSearchProfileIdentity::from_spec(atlas_embedding::profile_for_model(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        ))
    }

    fn fixture() -> (
        Connection,
        SourceSearchProfileIdentity,
        NativeInputDescriptor,
    ) {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE search_profiles(identity_json TEXT);
            CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT UNIQUE);
            CREATE TABLE embedding_inputs(input_id INTEGER PRIMARY KEY,model_id INTEGER,input_hash BLOB,token_count INTEGER,published_hash BLOB,UNIQUE(model_id,input_hash));
            CREATE TABLE semantic_units(input_id INTEGER);") .unwrap();
        let model = model();
        let input = NativeInputDescriptor {
            input_sha256: "a".repeat(64),
            token_count: 3,
        };
        c.execute(
            "INSERT INTO embedding_models VALUES(1,?)",
            [serde_json::to_string(&model).unwrap()],
        )
        .unwrap();
        c.execute(
            "INSERT INTO embedding_inputs VALUES(7,1,?,3,NULL)",
            [crate::embedding_values::digest(&input.input_sha256).unwrap()],
        )
        .unwrap();
        (c, model, input)
    }

    #[test]
    fn numerical_shape_checksum_finiteness_and_norm_are_checked() {
        let model = model();
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        let hash = crate::embedding_values::hash_bytes(&bytes);
        check_bytes(&bytes, &hash, &model).unwrap();
        assert!(check_bytes(&bytes[..bytes.len() - 4], &hash, &model).is_err());
        assert!(check_bytes(&bytes, &[0; 32], &model).is_err());
        vector[0] = f32::NAN;
        let bytes = crate::read_units::vector_bytes(&vector);
        assert!(check_bytes(&bytes, &crate::embedding_values::hash_bytes(&bytes), &model).is_err());
        vector[0] = 0.5;
        let bytes = crate::read_units::vector_bytes(&vector);
        assert!(check_bytes(&bytes, &crate::embedding_values::hash_bytes(&bytes), &model).is_err());
    }

    #[test]
    fn cache_reuse_precedence_and_corruption_are_distinct() {
        let (c, model, input) = fixture();
        c.execute_batch("CREATE TABLE pending_embedding_values(input_id INTEGER PRIMARY KEY,vector BLOB,vector_hash BLOB,cached_at INTEGER);") .unwrap();
        assert!(cached(&c, &model, &input).unwrap().is_none());
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        let hash = crate::embedding_values::hash_bytes(&bytes);
        c.execute(
            "INSERT INTO pending_embedding_values VALUES(7,?,?,0)",
            params![bytes, hash],
        )
        .unwrap();
        c.execute(
            "UPDATE embedding_inputs SET published_hash=zeroblob(8388608)",
            [],
        )
        .unwrap();
        let keys = NativeModelKey::all(std::slice::from_ref(&model)).unwrap();
        let binding = NativeModel::bind(&c, &keys[0], false).unwrap();
        let checked = NativeInput::new(&model, &input).unwrap();
        let selected = binding.select(&checked, true).unwrap().unwrap();
        assert_eq!(selected.staged.as_deref(), Some(bytes.as_slice()));
        assert_eq!(selected.checksum, hash);
        assert_eq!(
            cached(&c, &model, &input).unwrap(),
            Some(CachedNative {
                bytes,
                checksum: hash
            })
        );
        c.execute(
            "UPDATE pending_embedding_values SET vector_hash=zeroblob(32)",
            [],
        )
        .unwrap();
        assert!(cached(&c, &model, &input).is_err());
        c.execute(
            "UPDATE pending_embedding_values SET vector=zeroblob(8388608)",
            [],
        )
        .unwrap();
        assert!(
            cached(&c, &model, &input)
                .unwrap_err()
                .to_string()
                .contains("length mismatch")
        );
        c.execute("DELETE FROM pending_embedding_values", [])
            .unwrap();
        assert!(
            binding
                .select(&checked, true)
                .err()
                .unwrap()
                .to_string()
                .contains("published native checksum length mismatch")
        );
    }

    #[test]
    fn registry_gc_protects_document_and_pending_even_with_null_checksum() {
        let (c, _, _) = fixture();
        c.execute_batch("CREATE TABLE pending_embedding_values(input_id INTEGER);
            INSERT INTO embedding_inputs VALUES(8,1,randomblob(32),3,NULL),(9,1,randomblob(32),3,NULL),(10,1,randomblob(32),3,NULL);
            INSERT INTO semantic_units VALUES(7);
            INSERT INTO pending_embedding_values VALUES(9);").unwrap();
        prune_registry(&c).unwrap();
        let remaining = c
            .prepare("SELECT input_id FROM embedding_inputs ORDER BY input_id")
            .unwrap()
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(remaining, [7, 9]);
        assert_eq!(
            c.query_row("SELECT count(*) FROM embedding_models", [], |r| r
                .get::<_, usize>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn published_native_read_requires_exact_bytes_and_an_existing_input_row() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let (c, model, input) = fixture();
        let table = crate::schema::native_vector_table(&model).unwrap();
        c.execute_batch(&crate::schema::native_vector_ddl(&model).unwrap())
            .unwrap();
        assert!(published_checksum(&c, &model, &input).unwrap().is_none());
        assert!(read(&c, 7, &model).is_err());
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        let hash = crate::embedding_values::hash_bytes(&bytes);
        c.execute(
            &format!("INSERT INTO {table}(input_id,embedding) VALUES(7,?)"),
            [bytes],
        )
        .unwrap();
        c.execute(
            "UPDATE embedding_inputs SET published_hash=? WHERE input_id=7",
            [&hash],
        )
        .unwrap();
        assert_eq!(published_checksum(&c, &model, &input).unwrap(), Some(hash));
        assert_eq!(read(&c, 7, &model).unwrap(), vector);
        c.execute(
            "UPDATE embedding_inputs SET published_hash=zeroblob(32)",
            [],
        )
        .unwrap();
        assert!(read(&c, 7, &model).is_err());
        c.execute(&format!("DELETE FROM {table} WHERE input_id=7"), [])
            .unwrap();
        assert!(read(&c, 7, &model).is_err());
    }

    #[test]
    fn forced_same_input_replacement_rolls_back_with_its_checksum_and_keeps_cache() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let (c, model, input) = fixture();
        c.execute_batch("CREATE TABLE pending_embedding_values(input_id INTEGER PRIMARY KEY,vector BLOB,vector_hash BLOB,cached_at INTEGER);") .unwrap();
        let mut old = vec![0.; model.dimensions];
        old[0] = 1.;
        let mut forced = vec![0.; model.dimensions];
        forced[1] = 1.;
        let old_bytes = crate::read_units::vector_bytes(&old);
        let old_hash = crate::embedding_values::hash_bytes(&old_bytes);
        let new_bytes = crate::read_units::vector_bytes(&forced);
        let new_hash = crate::embedding_values::hash_bytes(&new_bytes);
        assert_eq!(
            install(&c, &model, &input, &old_bytes, &old_hash).unwrap(),
            7
        );
        c.execute(
            "INSERT INTO pending_embedding_values VALUES(7,?,?,1)",
            params![new_bytes, new_hash],
        )
        .unwrap();
        c.execute_batch("BEGIN IMMEDIATE").unwrap();
        assert_eq!(
            install(&c, &model, &input, &new_bytes, &new_hash).unwrap(),
            7
        );
        assert_eq!(read(&c, 7, &model).unwrap(), forced);
        assert_eq!(
            published_checksum(&c, &model, &input).unwrap(),
            Some(new_hash.clone())
        );
        c.execute_batch("ROLLBACK").unwrap();
        assert_eq!(read(&c, 7, &model).unwrap(), old);
        assert_eq!(
            published_checksum(&c, &model, &input).unwrap(),
            Some(old_hash)
        );
        assert_eq!(
            cached(&c, &model, &input).unwrap(),
            Some(CachedNative {
                bytes: new_bytes,
                checksum: new_hash,
            })
        );
    }

    #[test]
    fn failed_install_does_not_replace_valid_bytes_and_rejects_oversized_previous_blob() {
        atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
        let (c, model, input) = fixture();
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        let hash = crate::embedding_values::hash_bytes(&bytes);
        install(&c, &model, &input, &bytes, &hash).unwrap();
        let mut wrong_tokens = input.clone();
        wrong_tokens.token_count += 1;
        assert!(install(&c, &model, &wrong_tokens, &bytes, &hash).is_err());
        assert!(install(&c, &model, &input, &bytes, &[0; 32]).is_err());
        assert_eq!(read(&c, 7, &model).unwrap(), vector);
        let table = crate::schema::native_vector_table(&model).unwrap();
        // A synthetic malformed public relation exercises the allocation guard,
        // without touching any extension shadow storage or product database.
        c.execute_batch(&format!("DROP TABLE {table}; CREATE TABLE {table}(input_id INTEGER PRIMARY KEY,embedding BLOB); INSERT INTO {table} VALUES(7,zeroblob(8388608));")).unwrap();
        assert!(
            install(&c, &model, &input, &bytes, &hash)
                .unwrap_err()
                .to_string()
                .contains("previous native vector length mismatch")
        );
        assert_eq!(
            c.query_row(
                &format!("SELECT length(embedding) FROM {table} WHERE input_id=7"),
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            8388608
        );
    }
}
