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
    if bytes.len() != expected_bytes(model)?
        || checksum.len() != 32
        || crate::embedding_values::hash_bytes(bytes) != checksum
    {
        return Err(invalid("native vector length or checksum mismatch"));
    }
    crate::validation::check_vector(&crate::read_units::decode_vector(bytes)?, model.dimensions)
}

pub(crate) fn lookup_input(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<i64>, IndexError> {
    let row: Option<(i64,usize)> = c.prepare_cached("SELECT i.input_id,i.token_count FROM embedding_inputs i JOIN embedding_models m USING(model_id) WHERE m.identity_json=? AND i.input_hash=?")?
        .query_row(params![serde_json::to_string(model)?,crate::embedding_values::digest(&input.input_sha256)?], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
    match row {
        Some((id, tokens))
            if tokens == input.token_count && tokens > 0 && tokens <= model.token_limit =>
        {
            Ok(Some(id))
        }
        Some(_) => Err(invalid("native input token count mismatch")),
        None => Ok(None),
    }
}

fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}

pub(crate) fn published_checksum(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<Vec<u8>>, IndexError> {
    let Some(id) = lookup_input(c, model, input)? else {
        return Ok(None);
    };
    let length: Option<usize> = c.query_row(
        "SELECT length(published_hash) FROM embedding_inputs WHERE input_id=?",
        [id],
        |r| r.get(0),
    )?;
    match length {
        None => {
            let consumers: bool = c.prepare_cached("SELECT EXISTS(SELECT 1 FROM semantic_units WHERE input_id=?1) OR EXISTS(SELECT 1 FROM trait_seed_chunks WHERE input_id=?1)")?
                .query_row([id], |row| row.get(0))?;
            let table = crate::schema::native_vector_table(model)?;
            let exists: bool = c.prepare_cached("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)")?
                .query_row([&table], |row| row.get(0))?;
            let native: bool = exists && c.prepare_cached(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE input_id=?)"))?
                .query_row([id], |row| row.get(0))?;
            if consumers || native {
                return Err(invalid("published native checksum missing for existing value or consumer"));
            }
            Ok(None)
        },
        Some(32) => Ok(Some(c.query_row("SELECT published_hash FROM embedding_inputs WHERE input_id=? AND length(published_hash)=32",[id],|r|r.get(0))?)),
        Some(_) => Err(invalid("published native checksum length mismatch")),
    }
}

/// Reads only the supplied connection; shape is checked before allocating a BLOB.
pub(crate) fn read(
    c: &Connection,
    id: i64,
    model: &SourceSearchProfileIdentity,
) -> Result<Vec<f32>, IndexError> {
    let table = crate::schema::native_vector_table(model)?;
    let shape: (Option<usize>,Option<usize>,usize) = c.prepare_cached(&format!("SELECT length(v.embedding),length(i.published_hash),i.token_count FROM embedding_inputs i JOIN embedding_models m USING(model_id) LEFT JOIN {table} v USING(input_id) WHERE i.input_id=? AND m.identity_json=?"))?
        .query_row(params![id,serde_json::to_string(model)?],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    let length = expected_bytes(model)?;
    if shape.0 != Some(length) || shape.1 != Some(32) || shape.2 == 0 || shape.2 > model.token_limit
    {
        return Err(invalid("published native geometry or token count mismatch"));
    }
    let (bytes,checksum): (Vec<u8>,Vec<u8>) = c.prepare_cached(&format!("SELECT v.embedding,i.published_hash FROM embedding_inputs i JOIN {table} v USING(input_id) WHERE input_id=? AND length(v.embedding)=? AND length(i.published_hash)=32"))?
        .query_row(params![id,length],|r|Ok((r.get(0)?,r.get(1)?)))?;
    check_bytes(&bytes, &checksum, model)?;
    crate::read_units::decode_vector(&bytes)
}

/// Install inside the caller's transaction. It can roll back bytes and registry together.
pub(crate) fn install(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
    bytes: &[u8],
    checksum: &[u8],
) -> Result<i64, IndexError> {
    check_bytes(bytes, checksum, model)?;
    if input.token_count == 0 || input.token_count > model.token_limit {
        return Err(invalid("invalid native input token count"));
    }
    let id = crate::embedding_values::input_id(c, model, &input.input_sha256, input.token_count)?;
    let table = crate::schema::native_vector_table(model)?;
    let exists: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)",
        [&table],
        |r| r.get(0),
    )?;
    if !exists {
        c.execute_batch(&crate::schema::native_vector_ddl(model)?)?;
    }
    let previous_length: Option<usize> = c
        .prepare_cached(&format!(
            "SELECT length(embedding) FROM {table} WHERE input_id=?"
        ))?
        .query_row([id], |r| r.get(0))
        .optional()?;
    let expected_length = expected_bytes(model)?;
    if previous_length.is_some_and(|length| length != expected_length) {
        return Err(invalid("previous native vector length mismatch"));
    }
    let previous: Option<Vec<u8>> = if previous_length.is_some() {
        Some(
            c.prepare_cached(&format!(
                "SELECT embedding FROM {table} WHERE input_id=? AND length(embedding)=?"
            ))?
            .query_row(params![id, expected_length], |r| r.get(0))?,
        )
    } else {
        None
    };
    if previous.as_deref() != Some(bytes) {
        if previous.is_some() {
            c.execute(
                &format!("UPDATE {table} SET embedding=? WHERE input_id=?"),
                params![bytes, id],
            )?;
        } else {
            c.execute(
                &format!("INSERT INTO {table}(input_id,embedding) VALUES(?,?)"),
                params![id, bytes],
            )?;
        }
    }
    c.execute("UPDATE embedding_inputs SET published_hash=? WHERE input_id=? AND published_hash IS NOT ?1",params![checksum,id])?;
    Ok(id)
}

pub(crate) fn cached(
    c: &Connection,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<Option<(Vec<u8>, Vec<u8>)>, IndexError> {
    let Some(id) = lookup_input(c, model, input)? else {
        return Ok(None);
    };
    let shape: Option<(usize,usize)> = c.prepare_cached("SELECT length(vector),length(vector_hash) FROM pending_embedding_values WHERE input_id=?")?
        .query_row([id], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
    let Some((length, hash_length)) = shape else {
        return Ok(None);
    };
    if length != expected_bytes(model)? || hash_length != 32 {
        return Err(invalid("cached native length mismatch"));
    }
    let (bytes, checksum): (Vec<u8>,Vec<u8>) = c.prepare_cached("SELECT vector,vector_hash FROM pending_embedding_values WHERE input_id=? AND length(vector)=? AND length(vector_hash)=32")?
        .query_row(params![id,length], |r| Ok((r.get(0)?,r.get(1)?)))?;
    check_bytes(&bytes, &checksum, model)?;
    Ok(Some((bytes, checksum)))
}

/// Corrupt NULL publication checksums never authorize deleting live associations.
pub(crate) fn prune_registry(c: &Connection) -> Result<(), IndexError> {
    c.execute_batch("DELETE FROM embedding_inputs WHERE published_hash IS NULL
        AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id)
        AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=embedding_inputs.input_id)
        AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=embedding_inputs.input_id);
        DELETE FROM embedding_models WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id)
        AND NOT EXISTS(SELECT 1 FROM search_profiles p WHERE p.identity_json=embedding_models.identity_json);")?;
    Ok(())
}

pub(crate) fn ensure_profiles(
    c: &Connection,
    context: &crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    for model in &context.search_profiles {
        crate::embedding_values::model_id(c, model)?;
        let table = crate::schema::native_vector_table(model)?;
        let exists: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?)",
            [table],
            |r| r.get(0),
        )?;
        if !exists {
            c.execute_batch(&crate::schema::native_vector_ddl(model)?)?;
        }
    }
    Ok(())
}

/// Resolve an already installed selected input without materializing its vector.
pub(crate) fn selected_input(
    c: &Connection,
    values: &dyn crate::NativeValueSource,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<i64, IndexError> {
    let expected = values.selected_checksum(model, input)?;
    if expected.len() != 32 || published_checksum(c, model, input)?.as_ref() != Some(&expected) {
        return Err(invalid(
            "selected native checksum differs from installed input",
        ));
    }
    lookup_input(c, model, input)?.ok_or_else(|| invalid("selected native input missing"))
}

/// Run after every occurrence and ordered trait consumer, before final validation.
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
        let mut obsolete = c.prepare("SELECT input_id FROM embedding_inputs i WHERE model_id=? AND published_hash IS NOT NULL AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id)")?;
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
            let referenced: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM embedding_inputs i WHERE model_id=? AND (EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=i.input_id) OR EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=i.input_id)))",[id],|r|r.get(0))?;
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
            CREATE TABLE semantic_units(input_id INTEGER);
            CREATE TABLE trait_seed_chunks(input_id INTEGER);") .unwrap();
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
    fn exact_model_input_and_tokens_are_required() {
        let (c, model, input) = fixture();
        assert_eq!(lookup_input(&c, &model, &input).unwrap(), Some(7));
        let mut changed = input.clone();
        changed.token_count += 1;
        assert!(lookup_input(&c, &model, &changed).is_err());
        changed.input_sha256 = "b".repeat(64);
        assert_eq!(lookup_input(&c, &model, &changed).unwrap(), None);
        let mut other_model = model.clone();
        other_model.revision.push_str("-other");
        assert_eq!(lookup_input(&c, &other_model, &input).unwrap(), None);
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
    fn cache_missing_corrupt_and_oversized_rows_are_distinct() {
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
        assert_eq!(cached(&c, &model, &input).unwrap(), Some((bytes, hash)));
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
    }

    #[test]
    fn registry_gc_protects_document_trait_and_pending_even_with_null_checksum() {
        let (c, _, _) = fixture();
        c.execute_batch("CREATE TABLE pending_embedding_values(input_id INTEGER);
            INSERT INTO embedding_inputs VALUES(8,1,randomblob(32),3,NULL),(9,1,randomblob(32),3,NULL),(10,1,randomblob(32),3,NULL);
            INSERT INTO semantic_units VALUES(7); INSERT INTO trait_seed_chunks VALUES(8);
            INSERT INTO pending_embedding_values VALUES(9);").unwrap();
        prune_registry(&c).unwrap();
        let remaining = c
            .prepare("SELECT input_id FROM embedding_inputs ORDER BY input_id")
            .unwrap()
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(remaining, [7, 8, 9]);
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
            Some((new_bytes, new_hash))
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
