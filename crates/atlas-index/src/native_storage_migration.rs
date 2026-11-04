//! Main's occurrence-addressed vectors become one exact published value per input.
//! This callback uses public vec0 rows and never opens source files or model assets.
use crate::{IndexError, SourceSearchProfileIdentity};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;

pub const INPUT_OWNED_STORAGE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000008_input_owned_storage/up.sql");

pub fn backfill_input_owned_storage(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|error| {
        atlas_storage::StorageError::Migration(format!("input-owned native migration: {error}"))
    })
}

fn invalid(message: impl Into<String>) -> IndexError {
    IndexError::Invalid(format!("input-owned native migration: {}", message.into()))
}

struct Model {
    identity: SourceSearchProfileIdentity,
    json: String,
    bytes: usize,
}

fn models(c: &Connection) -> Result<BTreeMap<i64, Model>, IndexError> {
    let mut statement =
        c.prepare("SELECT model_id,identity_json FROM embedding_models ORDER BY model_id")?;
    let mut rows = statement.query([])?;
    let mut models = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let id = row.get(0)?;
        let json: String = row.get(1)?;
        let identity: SourceSearchProfileIdentity = serde_json::from_str(&json)?;
        let spec = atlas_embedding::search_profile(&identity.profile_id)
            .map_err(|error| invalid(error.to_string()))?;
        if identity.model_id != spec.model_spec().model_id
            || identity.dimensions != spec.model_spec().dimensions
            || identity.token_limit == 0
            || identity.token_limit > 32768
            || identity.dtype != "f32"
            || identity.normalization != "l2"
            || identity.distance_metric != "cosine"
            || !crate::codec::checked_hash(&identity.model_sha256)
            || !crate::codec::checked_hash(&identity.tokenizer_sha256)
            || !crate::codec::checked_hash(&identity.segmentation_tokenizer_sha256)
            || [
                &identity.provider,
                &identity.revision,
                &identity.pooling,
                &identity.unit_policy_version,
                &identity.tokenizer_id,
                &identity.output_name,
                &identity.execution_policy,
                &identity.input_recipe,
            ]
            .iter()
            .any(|value| value.is_empty())
            || identity
                .assets
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>()
                != spec
                    .model_spec()
                    .required_assets
                    .iter()
                    .map(|(name, _)| *name)
                    .collect()
            || identity.assets.is_empty()
            || identity
                .assets
                .iter()
                .any(|(name, hash)| name.is_empty() || !crate::codec::checked_hash(hash))
            || identity.assets.get("onnx/model.onnx") != Some(&identity.model_sha256)
            || identity.assets.get("tokenizer.json") != Some(&identity.tokenizer_sha256)
            || serde_json::to_string(&identity)? != json
        {
            return Err(invalid("invalid full model execution identity"));
        }
        let bytes = identity
            .dimensions
            .checked_mul(4)
            .ok_or_else(|| invalid("vector length overflow"))?;
        models.insert(
            id,
            Model {
                identity,
                json,
                bytes,
            },
        );
    }
    Ok(models)
}

fn check_bytes(bytes: &[u8], hash: &[u8], model: &Model) -> Result<(), IndexError> {
    if bytes.len() != model.bytes
        || hash.len() != 32
        || crate::embedding_values::hash_bytes(bytes) != hash
    {
        return Err(invalid("native bytes disagree with their length/checksum"));
    }
    crate::validation::check_vector(
        &crate::read_units::decode_vector(bytes)?,
        model.identity.dimensions,
    )
}

fn input(
    c: &Connection,
    id: i64,
    model_id: i64,
    model: &Model,
    published: bool,
) -> Result<Option<Vec<u8>>, IndexError> {
    let metadata: Option<(i64, usize, usize, Option<usize>)> = c.query_row(
        "SELECT model_id,token_count,length(input_hash),length(published_hash) FROM embedding_inputs WHERE input_id=?",
        [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
    ).optional()?;
    let Some((stored_model, tokens, hash_length, published_length)) = metadata else {
        return Err(invalid("native consumer has no input registry row"));
    };
    if stored_model != model_id
        || tokens == 0
        || tokens > model.identity.token_limit
        || hash_length != 32
        || published_length.is_some_and(|length| length != 32)
        || (published && published_length.is_none())
    {
        return Err(invalid("native input model/token/hash identity mismatch"));
    }
    Ok(c.query_row(
        "SELECT published_hash FROM embedding_inputs WHERE input_id=?",
        [id],
        |row| row.get(0),
    )?)
}

fn install(c: &Connection, table: &str, id: i64, bytes: &[u8]) -> Result<(), IndexError> {
    let existing: Option<usize> = c
        .query_row(
            &format!("SELECT length(embedding) FROM {table} WHERE input_id=?"),
            [id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(length) = existing {
        if length != bytes.len() {
            return Err(invalid(
                "duplicate published input has different numerical length",
            ));
        }
        let previous: Vec<u8> = c.query_row(
            &format!("SELECT embedding FROM {table} WHERE input_id=? AND length(embedding)=?"),
            params![id, bytes.len()],
            |row| row.get(0),
        )?;
        if previous != bytes {
            return Err(invalid(
                "duplicate published input has different numerical bytes",
            ));
        }
    } else {
        c.execute(
            &format!("INSERT INTO {table}(input_id,embedding) VALUES(?,?)"),
            params![id, bytes],
        )?;
    }
    Ok(())
}

fn check_centroids(
    c: &Connection,
    models: &BTreeMap<i64, Model>,
    profiles: &BTreeMap<i64, (String, String)>,
) -> Result<(), IndexError> {
    let mut statement = c.prepare("SELECT trait_key,profile_id,length(vector),length(vector_hash) FROM trait_vectors ORDER BY trait_key,profile_id")?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let profile: String = row.get(1)?;
        let (model_id, model) = models
            .iter()
            .find(|(id, model)| {
                model.identity.profile_id.as_str() == profile && profiles.contains_key(*id)
            })
            .ok_or_else(|| invalid("trait centroid has no active model profile"))?;
        if row.get::<_, usize>(2)? != model.bytes || row.get::<_, usize>(3)? != 32 {
            return Err(invalid("trait centroid has invalid length"));
        }
        let (bytes,hash): (Vec<u8>,Vec<u8>) = c.query_row(
            "SELECT vector,vector_hash FROM trait_vectors WHERE trait_key=? AND profile_id=? AND length(vector)=? AND length(vector_hash)=32",
            params![key,profile,model.bytes], |row|Ok((row.get(0)?,row.get(1)?)),
        )?;
        check_bytes(&bytes, &hash, model)?;
        let mut centroid = atlas_embedding::NormalizedCentroid::new(model.identity.dimensions)
            .map_err(|error| invalid(error.to_string()))?;
        let mut chunks = c.prepare("SELECT ordinal,input_id FROM trait_seed_chunks WHERE trait_key=? AND profile_id=? ORDER BY ordinal")?;
        let mut chunks = chunks.query(params![key, profile])?;
        let mut ordinal = 0usize;
        while let Some(chunk) = chunks.next()? {
            if chunk.get::<_, usize>(0)? != ordinal {
                return Err(invalid("trait chunk order has gaps"));
            }
            let id: i64 = chunk.get(1)?;
            input(c, id, *model_id, model, true)?;
            let table = &profiles[model_id].1;
            let value: Vec<u8> = c.query_row(
                &format!("SELECT embedding FROM {table} WHERE input_id=? AND length(embedding)=?"),
                params![id, model.bytes],
                |row| row.get(0),
            )?;
            centroid
                .push(&crate::read_units::decode_vector(&value)?)
                .map_err(|error| invalid(error.to_string()))?;
            ordinal += 1;
        }
        let expected = centroid
            .finish()
            .map_err(|error| invalid(error.to_string()))?;
        // Published centroid validation permits equal signed zeros. Keep its
        // float equality while retaining the original BLOB and checksum untouched.
        if expected != crate::read_units::decode_vector(&bytes)? {
            return Err(invalid(
                "trait centroid differs from its ordered published chunks",
            ));
        }
    }
    Ok(())
}

fn backfill(c: &Connection) -> Result<(), IndexError> {
    // A published empty profile has a vec0 table but need not have registered an
    // input/model yet. Give that active execution identity its stable registry row.
    let active = c
        .prepare("SELECT identity_json FROM search_profiles ORDER BY profile_id")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for json in active {
        c.execute(
            "INSERT INTO embedding_models(identity_json) VALUES(?) ON CONFLICT DO NOTHING",
            [json],
        )?;
    }
    let models = models(c)?;
    // Registry metadata can outlive a profile while it owns unpublished work.
    let mut inputs =
        c.prepare("SELECT input_id,model_id FROM embedding_inputs ORDER BY input_id")?;
    for row in inputs.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))? {
        let (id, model_id) = row?;
        let model = models
            .get(&model_id)
            .ok_or_else(|| invalid("input has no model registry row"))?;
        input(c, id, model_id, model, false)?;
    }
    drop(inputs);

    let mut profiles = BTreeMap::new();
    let profile_rows = c
        .prepare(
            "SELECT profile_id,identity_json,dimensions FROM search_profiles ORDER BY profile_id",
        )?
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, usize>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (profile, json, dimensions) in profile_rows {
        let (model_id, model) = models
            .iter()
            .find(|(_, model)| model.json == json)
            .ok_or_else(|| invalid("published profile has no exact model registry row"))?;
        if model.identity.profile_id.as_str() != profile || model.identity.dimensions != dimensions
        {
            return Err(invalid("published profile identity/geometry mismatch"));
        }
        let old = crate::schema::vector_table(&model.identity)?;
        let new = crate::schema::native_vector_table(&model.identity)?;
        let ddl: String = c.query_row(
            "SELECT sql FROM sqlite_schema WHERE name=? AND type='table'",
            [&old],
            |row| row.get(0),
        )?;
        let normalized = |sql: &str| {
            sql.split_whitespace()
                .collect::<String>()
                .to_ascii_lowercase()
        };
        if normalized(&ddl) != normalized(&crate::schema::vector_ddl(&model.identity)?) {
            return Err(invalid("published occurrence vector DDL differs"));
        }
        c.execute_batch(&crate::schema::native_vector_ddl(&model.identity)?)?;
        let mut statement = c.prepare(&format!("SELECT u.unit_id,u.input_id,length(v.embedding) FROM semantic_units u LEFT JOIN {old} v USING(unit_id) WHERE u.profile_id=? ORDER BY u.unit_id"))?;
        let mut rows = statement.query([&profile])?;
        while let Some(row) = rows.next()? {
            let unit: i64 = row.get(0)?;
            let id: i64 = row.get(1)?;
            let length: Option<usize> = row.get(2)?;
            if length != Some(model.bytes) {
                return Err(invalid("published occurrence vector missing or oversized"));
            }
            let hash = input(c, id, *model_id, model, true)?
                .ok_or_else(|| invalid("published checksum missing"))?;
            let bytes: Vec<u8> = c.query_row(
                &format!("SELECT embedding FROM {old} WHERE unit_id=? AND length(embedding)=?"),
                params![unit, model.bytes],
                |row| row.get(0),
            )?;
            check_bytes(&bytes, &hash, model)?;
            install(c, &new, id, &bytes)?;
        }
        drop(rows);
        drop(statement);
        let orphan: bool = c.query_row(&format!("SELECT EXISTS(SELECT 1 FROM {old} v WHERE NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.unit_id=v.unit_id AND u.profile_id=?))"), [&profile], |row| row.get(0))?;
        if orphan {
            return Err(invalid("unmapped occurrence vector row"));
        }
        profiles.insert(*model_id, (old, new));
    }
    let physical_old = c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name GLOB 'semantic_vectors_*' AND sql LIKE '%USING vec0%' ORDER BY name")?
        .query_map([], |row| row.get::<_,String>(0))?.collect::<Result<std::collections::BTreeSet<_>,_>>()?;
    let expected_old = profiles
        .values()
        .map(|(old, _)| old.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if physical_old != expected_old {
        return Err(invalid("unmapped published vector table"));
    }
    let missing_profile: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM semantic_units u LEFT JOIN search_profiles p USING(profile_id) WHERE p.profile_id IS NULL)", [], |row| row.get(0))?;
    if missing_profile {
        return Err(invalid("semantic occurrence has no published profile"));
    }

    let mut traits = c.prepare("SELECT t.input_id,i.model_id,length(t.vector) FROM trait_native_values t LEFT JOIN embedding_inputs i USING(input_id) ORDER BY t.input_id")?;
    let mut rows = traits.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let model_id: Option<i64> = row.get(1)?;
        let model_id =
            model_id.ok_or_else(|| invalid("trait native value has no registry input"))?;
        let model = models
            .get(&model_id)
            .ok_or_else(|| invalid("trait native value has no model"))?;
        let (_, table) = profiles
            .get(&model_id)
            .ok_or_else(|| invalid("trait native value has no published profile"))?;
        if row.get::<_, usize>(2)? != model.bytes {
            return Err(invalid("trait native value has invalid length"));
        }
        let has_consumer: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM trait_seed_chunks WHERE input_id=?)",
            [id],
            |row| row.get(0),
        )?;
        if !has_consumer {
            return Err(invalid("unmapped trait native value"));
        }
        let hash = input(c, id, model_id, model, true)?
            .ok_or_else(|| invalid("trait published checksum missing"))?;
        let bytes: Vec<u8> = c.query_row(
            "SELECT vector FROM trait_native_values WHERE input_id=? AND length(vector)=?",
            params![id, model.bytes],
            |row| row.get(0),
        )?;
        check_bytes(&bytes, &hash, model)?;
        install(c, table, id, &bytes)?;
    }
    drop(rows);
    drop(traits);

    // Validate every published input and the reverse row coverage independently
    // after copying, including inputs reached solely through ordered trait chunks.
    for (model_id, model) in &models {
        let expected: usize = c.query_row(
            "SELECT count(*) FROM embedding_inputs WHERE model_id=? AND published_hash IS NOT NULL",
            [model_id],
            |row| row.get(0),
        )?;
        let Some((_, table)) = profiles.get(model_id) else {
            if expected != 0 {
                return Err(invalid("published inputs have no active model profile"));
            }
            continue;
        };
        let count: usize = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })?;
        if count != expected {
            return Err(invalid("published input/value coverage differs"));
        }
        let mut statement = c.prepare("SELECT input_id FROM embedding_inputs WHERE model_id=? AND published_hash IS NOT NULL ORDER BY input_id")?;
        for row in statement.query_map([model_id], |row| row.get::<_, i64>(0))? {
            let id = row?;
            let hash = input(c, id, *model_id, model, true)?
                .ok_or_else(|| invalid("published checksum absent"))?;
            let length: Option<usize> = c
                .query_row(
                    &format!("SELECT length(embedding) FROM {table} WHERE input_id=?"),
                    [id],
                    |row| row.get(0),
                )
                .optional()?;
            if length != Some(model.bytes) {
                return Err(invalid("published registry value missing or oversized"));
            }
            let bytes: Vec<u8> = c.query_row(
                &format!("SELECT embedding FROM {table} WHERE input_id=? AND length(embedding)=?"),
                params![id, model.bytes],
                |row| row.get(0),
            )?;
            check_bytes(&bytes, &hash, model)?;
        }
    }
    let mut chunks = c.prepare("SELECT t.input_id,t.profile_id,i.model_id FROM trait_seed_chunks t LEFT JOIN embedding_inputs i USING(input_id) ORDER BY t.trait_key,t.profile_id,t.ordinal")?;
    let mut rows = chunks.query([])?;
    while let Some(row) = rows.next()? {
        let id = row.get(0)?;
        let profile: String = row.get(1)?;
        let model_id: Option<i64> = row.get(2)?;
        let model_id = model_id.ok_or_else(|| invalid("trait chunk has no input"))?;
        let model = models
            .get(&model_id)
            .ok_or_else(|| invalid("trait chunk has no model"))?;
        if model.identity.profile_id.as_str() != profile || !profiles.contains_key(&model_id) {
            return Err(invalid("trait chunk/model ownership mismatch"));
        }
        input(c, id, model_id, model, true)?;
    }
    drop(rows);
    drop(chunks);

    check_centroids(c, &models, &profiles)?;

    let old_cache_count: usize = c.query_row(
        "SELECT count(*) FROM legacy_pending_embedding_values",
        [],
        |row| row.get(0),
    )?;
    let mut pending = c.prepare("SELECT p.input_id,i.model_id,length(p.vector),length(p.vector_hash),coalesce(unixepoch(j.updated_at),0) FROM legacy_pending_embedding_values p LEFT JOIN embedding_inputs i USING(input_id) LEFT JOIN reference_update_jobs j ON j.job_key=p.touched_job ORDER BY p.input_id")?;
    let mut rows = pending.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let model_id: Option<i64> = row.get(1)?;
        let model_id = model_id.ok_or_else(|| invalid("recovery value has no input"))?;
        let model = models
            .get(&model_id)
            .ok_or_else(|| invalid("recovery value has no model"))?;
        if row.get::<_, usize>(2)? != model.bytes || row.get::<_, usize>(3)? != 32 {
            return Err(invalid("recovery vector/checksum has invalid length"));
        }
        input(c, id, model_id, model, false)?;
        let (bytes,hash): (Vec<u8>,Vec<u8>) = c.query_row("SELECT vector,vector_hash FROM legacy_pending_embedding_values WHERE input_id=? AND length(vector)=? AND length(vector_hash)=32", params![id,model.bytes], |row| Ok((row.get(0)?,row.get(1)?)))?;
        check_bytes(&bytes, &hash, model)?;
        let age: i64 = row.get(4)?;
        c.execute("INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at) VALUES(?,?,?,?)", params![id,bytes,hash,age])?;
    }
    drop(rows);
    drop(pending);
    let cache_count: usize =
        c.query_row("SELECT count(*) FROM pending_embedding_values", [], |row| {
            row.get(0)
        })?;
    if cache_count != old_cache_count {
        return Err(invalid("recovery cache coverage differs"));
    }
    let cache_differs: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM legacy_pending_embedding_values old LEFT JOIN pending_embedding_values new USING(input_id) WHERE new.input_id IS NULL OR old.vector IS NOT new.vector OR old.vector_hash IS NOT new.vector_hash)", [], |row| row.get(0))?;
    if cache_differs {
        return Err(invalid("recovery values changed during conversion"));
    }

    for (old, _) in profiles.values() {
        c.execute_batch(&format!("DROP TABLE {old}"))?;
    }
    c.execute_batch("DROP TABLE legacy_pending_embedding_values; DROP TABLE trait_native_values; DROP TABLE reference_update_jobs;")?;
    if c.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid(
            "foreign key coverage differs after native conversion",
        ));
    }
    Ok(())
}
