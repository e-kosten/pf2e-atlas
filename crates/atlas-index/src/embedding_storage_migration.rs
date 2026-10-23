//! Single upgrade from published schema 9, retaining exact native values and
//! unfinished job progress without decoding retired source snapshots.
use crate::{IndexError, SourceSearchProfileIdentity, embedding_values as values};
use rusqlite::{Connection, params};

pub const EMBEDDING_STORAGE_SCHEMA_SQL: &str =
    include_str!("../migrations/00000000000004_embedding_storage/up.sql");
pub fn backfill_embedding_storage(c: &Connection) -> atlas_storage::StorageResult<()> {
    backfill(c).map_err(|e| atlas_storage::StorageError::Migration(e.to_string()))
}
fn backfill(c: &Connection) -> Result<(), IndexError> {
    let models = c
        .prepare("SELECT identity_json FROM search_profiles ORDER BY profile_id")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for json in &models {
        let model: SourceSearchProfileIdentity = serde_json::from_str(json)?;
        let table = crate::schema::vector_table(&model)?;
        let mut s=c.prepare(&format!("SELECT u.unit_id,u.record_id,u.owners_json,u.field_path,u.section_json,u.chunk_ordinal,u.input_hash,u.input_token_count,u.unit_kind,v.embedding FROM legacy_semantic_units u LEFT JOIN {table} v USING(unit_id) WHERE profile_id=? ORDER BY unit_id"))?;
        let total: usize = c.query_row(
            "SELECT count(*) FROM legacy_semantic_units WHERE profile_id=?",
            [model.profile_id.as_str()],
            |r| r.get(0),
        )?;
        let mut checked = 0usize;
        let mut rows = s.query([model.profile_id.as_str()])?;
        while let Some(row) = rows.next()? {
            let unit: i64 = row.get(0)?;
            let record: i64 = row.get(1)?;
            let owners: String = row.get(2)?;
            let field: Option<String> = row.get(3)?;
            let address: String = row.get(4)?;
            let ordinal: usize = row.get(5)?;
            let hash: String = row.get(6)?;
            let tokens: usize = row.get(7)?;
            let kind: String = row.get(8)?;
            let bytes: Vec<u8> = row.get(9)?;
            let vector = crate::read_units::decode_vector(&bytes)?;
            let location = crate::semantic_locations::location_id(
                c,
                record,
                &owners,
                field.as_deref(),
                &address,
                ordinal,
            )?;
            let input = values::input_id(c, &model, &hash, tokens)?;
            let checksum = values::hash_bytes(&bytes);
            let published: Option<Vec<u8>> = c.query_row(
                "SELECT published_hash FROM embedding_inputs WHERE input_id=?",
                [input],
                |r| r.get(0),
            )?;
            if published.as_ref().is_some_and(|h| *h != checksum) {
                return Err(IndexError::Invalid(
                    "conflicting published native values".into(),
                ));
            }
            crate::validation::check_vector(&vector, model.dimensions)?;
            c.execute(
                "UPDATE embedding_inputs SET published_hash=? WHERE input_id=?",
                params![checksum, input],
            )?;
            c.execute(
                "INSERT INTO semantic_units VALUES(?,?,?,?,?,?)",
                params![
                    unit,
                    record,
                    model.profile_id.as_str(),
                    location,
                    input,
                    kind
                ],
            )?;
            checked += 1;
            if checked.is_multiple_of(1000) || checked == total {
                tracing::info!(target:"atlas_progress",phase="migrate_embeddings",profile=%model.profile_id,current=checked as u64,total=total as u64,unit="passages","Upgrading embedding storage");
            }
        }
        drop(rows);
        drop(s);
        crate::vector_allocation::compact(c, &model)?;
    }
    let mut s=c.prepare("SELECT trait_key,profile_id,input_hash,vector_json,chunks_json,vector_hash FROM legacy_trait_vectors")?;
    let mut rows = s.query([])?;
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let profile: String = row.get(1)?;
        let json: String = row.get(3)?;
        let hash: String = row.get(5)?;
        if crate::codec::sha256(json.as_bytes()) != hash {
            return Err(IndexError::Invalid("legacy trait checksum mismatch".into()));
        }
        let model: SourceSearchProfileIdentity = serde_json::from_str(&c.query_row(
            "SELECT identity_json FROM search_profiles WHERE profile_id=?",
            [&profile],
            |r| r.get::<_, String>(0),
        )?)?;
        let seed = crate::TraitVectorInput {
            key: key
                .parse()
                .map_err(|e| IndexError::Invalid(format!("invalid trait key: {e}")))?,
            profile_id: atlas_domain::SearchProfileId::new(profile),
            input_hash: row.get(2)?,
            vector: serde_json::from_str(&json)?,
            chunks: serde_json::from_str(&row.get::<_, String>(4)?)?,
        };
        // Existing document/native hashes must agree with every trait chunk.
        for chunk in &seed.chunks {
            let id = values::input_id(c, &model, &chunk.input_hash, chunk.token_count)?;
            let old: Option<Vec<u8>> = c.query_row(
                "SELECT published_hash FROM embedding_inputs WHERE input_id=?",
                [id],
                |r| r.get(0),
            )?;
            if old.is_some_and(|h| {
                h != values::hash_bytes(&crate::read_units::vector_bytes(&chunk.vector))
            }) {
                return Err(IndexError::Invalid(
                    "legacy trait/native values disagree".into(),
                ));
            }
        }
        let definition: crate::TraitDefinition = serde_json::from_str(&c.query_row(
            "SELECT definition_json FROM trait_definitions WHERE key=?",
            [&key],
            |r| r.get::<_, String>(0),
        )?)?;
        crate::traits::validate_seed(
            &definition,
            &model,
            &seed.input_hash,
            &seed.vector,
            &seed.chunks,
        )?;
        crate::traits::write_seed(c, &seed, &model)?;
    }
    drop(rows);
    drop(s);
    // A preparing job may contain a newer forced computation for an input whose
    // published vector remains valid. Preserve that staged checksum separately.
    let mut s=c.prepare("SELECT model_identity,input_hash,token_count,dimensions,vector,vector_hash,touched_job FROM embedding_checkpoints")?;
    let mut rows = s.query([])?;
    while let Some(row) = rows.next()? {
        let json: String = row.get(0)?;
        let hash: String = row.get(1)?;
        let tokens: usize = row.get(2)?;
        let dimensions: usize = row.get(3)?;
        let bytes: Vec<u8> = row.get(4)?;
        let checksum: String = row.get(5)?;
        let job: String = row.get(6)?;
        let model: SourceSearchProfileIdentity = serde_json::from_str(&json)?;
        if dimensions != model.dimensions || crate::codec::sha256(&bytes) != checksum {
            return Err(IndexError::Invalid(
                "legacy checkpoint checksum or dimensions mismatch".into(),
            ));
        }
        crate::validation::check_vector(
            &crate::read_units::decode_vector(&bytes)?,
            model.dimensions,
        )?;
        let id = values::input_id(c, &model, &hash, tokens)?;
        let published: Option<Vec<u8>> = c.query_row(
            "SELECT published_hash FROM embedding_inputs WHERE input_id=?",
            [id],
            |r| r.get(0),
        )?;
        let preparing:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM reference_update_jobs WHERE job_key=? AND state='preparing')",[&job],|r|r.get(0))?;
        if preparing {
            c.execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,?)",
                params![id, bytes, values::hash_bytes(&bytes), job],
            )?;
        } else if published
            .as_ref()
            .is_some_and(|h| *h != values::hash_bytes(&bytes))
        {
            return Err(IndexError::Invalid(
                "legacy published checkpoint disagrees with native value".into(),
            ));
        }
    }
    drop(rows);
    drop(s);
    c.execute_batch("DROP TABLE legacy_semantic_units; DROP TABLE legacy_trait_vectors; DROP TABLE embedding_checkpoints;
DELETE FROM embedding_inputs WHERE published_hash IS NULL AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=embedding_inputs.input_id);
DELETE FROM embedding_models WHERE NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id);")?;
    crate::semantic_locations::validate(c)?;
    Ok(())
}
