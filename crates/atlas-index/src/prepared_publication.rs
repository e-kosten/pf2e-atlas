//! One owned publication: stable published handles and only unpublished staged bytes.
use crate::{
    ArtifactInstance, ArtifactValidationReport, BuildFileUsage, IndexError, NativeValueSource,
    PublicationSpool, SourceArtifactBuildContext, SourceSearchProfileIdentity,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{UpdateOwner, Workspace};
use rusqlite::{Connection, OptionalExtension, params};
use std::{borrow::Cow, path::Path};

pub const NATIVE_VALUE_BATCH_LIMIT: usize = 128;
const CACHE_PAYLOAD_LIMIT: u64 = 1024 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeValueSelection {
    Missing,
    Reused,
    AlreadySelected,
}
pub struct PreparedPublication<'owner> {
    selected: Connection,
    selected_file: tempfile::NamedTempFile,
    owner: &'owner UpdateOwner,
    workspace: Workspace,
    schema: u32,
    metadata: PublicationSpool,
    identities: Vec<crate::writer::CheckedRootIdentity>,
    cache_bytes: u64,
    cache_limit: u64,
}
impl<'owner> PreparedPublication<'owner> {
    pub fn new_in(
        owner: &'owner UpdateOwner,
        schema: u32,
        scratch: &Path,
        metadata: PublicationSpool,
    ) -> Result<Self, IndexError> {
        let validation_started = std::time::Instant::now();
        let identities = crate::writer::validate_input(&metadata)?;
        tracing::info!(target:"atlas_build_diagnostic",stage="publication_input_validation",elapsed_ms=validation_started.elapsed().as_secs_f64()*1000.,"Completed publication input validation");
        let workspace = Workspace::at(owner.path())?;
        let read = workspace.read(schema)?;
        crate::writer::check_expected(read.connection(), &metadata)?;
        let cache_bytes = read.connection().query_row(
            "SELECT coalesce(sum(length(vector)),0) FROM pending_embedding_values",
            [],
            |r| r.get(0),
        )?;
        drop(read);
        let selected_file = tempfile::NamedTempFile::new_in(scratch).map_err(operation)?;
        let selected = Connection::open(selected_file.path())?;
        selected.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=OFF;
            CREATE TABLE selected(model INTEGER NOT NULL,input_hash BLOB NOT NULL CHECK(length(input_hash)=32),token_count INTEGER NOT NULL,
            selected_hash BLOB NOT NULL CHECK(length(selected_hash)=32),vector BLOB,
            PRIMARY KEY(model,input_hash)) WITHOUT ROWID,STRICT;")?;
        selected.set_prepared_statement_cache_capacity(32);
        let mut publication = Self {
            selected,
            selected_file,
            owner,
            workspace,
            schema,
            metadata,
            identities,
            cache_bytes,
            cache_limit: CACHE_PAYLOAD_LIMIT,
        };
        publication.evict_cache()?;
        Ok(publication)
    }
    pub fn context(&self) -> &SourceArtifactBuildContext {
        &self.metadata.context
    }
    pub fn expected(&self) -> &ArtifactInstance {
        &self.metadata.expected
    }
    pub fn file_usage(&self) -> Result<BuildFileUsage, IndexError> {
        self.metadata
            .file_usage()?
            .combine(BuildFileUsage::of(self.selected_file.as_file())?)
    }
    pub fn logical_native_bytes(&self) -> Result<u64, IndexError> {
        Ok(self.selected.query_row(
            "SELECT coalesce(sum(length(vector)),0) FROM selected",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn temporary_native_bytes(&self) -> Result<u64, IndexError> {
        Ok(BuildFileUsage::of(self.selected_file.as_file())?.file_bytes)
    }
    pub fn select_native(
        &mut self,
        model: &SourceSearchProfileIdentity,
        inputs: &[NativeInputDescriptor],
        reuse_published: bool,
    ) -> Result<Vec<NativeValueSelection>, IndexError> {
        bounded(inputs.len())?;
        self.model_key(model)?;
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let read = self.workspace.read(self.schema)?;
        crate::writer::check_expected(read.connection(), &self.metadata)?;
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let mut output = Vec::with_capacity(inputs.len());
        for input in inputs {
            check_input(model, input)?;
            if self.selection(model, input)?.is_some() {
                output.push(NativeValueSelection::AlreadySelected);
                continue;
            }
            if let Some((bytes, checksum)) =
                crate::native_values::cached(read.connection(), model, input)?
            {
                self.remember(model, input, &checksum, Some(&bytes))?;
                output.push(NativeValueSelection::Reused);
            } else if reuse_published {
                if let Some(checksum) =
                    crate::native_values::published_checksum(read.connection(), model, input)?
                {
                    let id = crate::native_values::lookup_input(read.connection(), model, input)?
                        .ok_or_else(|| invalid("published native input disappeared"))?;
                    crate::native_values::read(read.connection(), id, model)?;
                    self.remember(model, input, &checksum, None)?;
                    output.push(NativeValueSelection::Reused);
                } else {
                    output.push(NativeValueSelection::Missing);
                }
            } else {
                output.push(NativeValueSelection::Missing);
            }
        }
        selected_tx.commit()?;
        Ok(output)
    }
    pub fn stage_inferred(
        &mut self,
        model: &SourceSearchProfileIdentity,
        batch: &[(&NativeInputDescriptor, &[f32])],
    ) -> Result<(), IndexError> {
        bounded(batch.len())?;
        self.model_key(model)?;
        if batch.is_empty() {
            return Ok(());
        }
        let mut batch_hashes = std::collections::BTreeMap::new();
        for (input, vector) in batch {
            let checksum =
                crate::embedding_values::hash_bytes(&crate::read_units::vector_bytes(vector));
            if batch_hashes
                .insert(
                    input.input_sha256.as_str(),
                    (input.token_count, checksum.clone()),
                )
                .is_some_and(|old| old != (input.token_count, checksum))
            {
                return Err(invalid("conflicting inferred values within batch"));
            }
            check_input(model, input)?;
            crate::validation::check_vector(vector, model.dimensions)?;
            if let Some(expected) = self.selection(model, input)? {
                if expected
                    != crate::embedding_values::hash_bytes(&crate::read_units::vector_bytes(vector))
                {
                    return Err(invalid(
                        "attempted to replace an already selected native value",
                    ));
                }
            }
        }
        let write = self.workspace.write(self.schema)?;
        crate::writer::check_expected(write.connection(), &self.metadata)?;
        let mut next_cache_bytes = self.cache_bytes;
        for (input, vector) in batch {
            let id = crate::embedding_values::input_id(
                write.connection(),
                model,
                &input.input_sha256,
                input.token_count,
            )?;
            let bytes = crate::read_units::vector_bytes(vector);
            let checksum = crate::embedding_values::hash_bytes(&bytes);
            let old: u64 = write.connection().query_row("SELECT coalesce((SELECT length(vector) FROM pending_embedding_values WHERE input_id=?),0)", [id], |r| r.get(0))?;
            write.connection().execute("INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at) VALUES(?,?,?,unixepoch()) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,cached_at=excluded.cached_at", params![id,bytes,checksum])?;
            next_cache_bytes = next_cache_bytes
                .checked_sub(old)
                .and_then(|n| n.checked_add(bytes.len() as u64))
                .ok_or_else(|| invalid("cache payload accounting mismatch"))?;
        }
        write.commit()?;
        self.cache_bytes = next_cache_bytes;
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        for (input, vector) in batch {
            let bytes = crate::read_units::vector_bytes(vector);
            self.remember(
                model,
                input,
                &crate::embedding_values::hash_bytes(&bytes),
                Some(&bytes),
            )?;
        }
        selected_tx.commit()?;
        self.evict_cache()
    }
    pub fn publish(self) -> Result<ArtifactValidationReport, IndexError> {
        let maintenance = self
            .owner
            .maintenance(self.schema, "publishing reference update")?;
        let prior = crate::ReferencePublicationExpectation::read(maintenance.connection())?;
        // This synchronous observation point follows durable maintenance admission
        // and precedes all publication mutations; guarded recovery authenticates it.
        tracing::info!(target:"atlas_progress",phase="validate_build_input",workspace_key=prior.instance().workspace_key(),reference_revision=prior.instance().reference_revision(),context_sha256=prior.context_sha256().unwrap_or(""),"Saving reference update");
        maintenance.publish(|connection| {
            connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
            crate::writer::check_expected(connection, &self.metadata)?;
            let reusable = crate::writer::prior_compatible(connection)?;
            let native_started = std::time::Instant::now();
            self.install_staged(connection)?;
            crate::native_values::ensure_profiles(connection,&self.metadata.context)?;
            tracing::info!(target:"atlas_build_diagnostic",stage="native_installation",elapsed_ms=native_started.elapsed().as_secs_f64()*1000.,"Installed selected unpublished values");
            let values = PublicationValues { connection, selected: &self.selected, models: &self.metadata.context.search_profiles };
            let scratch = self.selected_file.path().parent().ok_or_else(|| invalid("publication scratch parent absent"))?;
            let delta_started = std::time::Instant::now();
            crate::stream_publication::write(connection, &self.metadata, &self.identities, &values, reusable, scratch)?;
            connection.execute("DELETE FROM pending_embedding_values", [])?;
            crate::native_values::prune(connection)?;
            tracing::info!(target:"atlas_build_diagnostic",stage="publication_delta_and_prune",elapsed_ms=delta_started.elapsed().as_secs_f64()*1000.,"Completed reference reconciliation");
            let validation_started = std::time::Instant::now();
            let report = crate::validation::validate_reference(connection)?;
            tracing::info!(target:"atlas_build_diagnostic",stage="publication_exact_validation",elapsed_ms=validation_started.elapsed().as_secs_f64()*1000.,"Completed publication validation");
            Ok(report)
        })
    }
    fn model_key(&self, model: &SourceSearchProfileIdentity) -> Result<usize, IndexError> {
        let key = self
            .context()
            .search_profiles
            .iter()
            .position(|p| p.profile_id == model.profile_id)
            .ok_or_else(|| invalid("native model absent from publication"))?;
        if &self.context().search_profiles[key] != model {
            return Err(invalid("native selection model differs from publication"));
        }
        Ok(key)
    }
    fn selection(
        &self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Option<Vec<u8>>, IndexError> {
        let row: Option<(usize, Vec<u8>)> = self.selected.prepare_cached("SELECT token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END FROM selected WHERE model=? AND input_hash=?")?.query_row(params![self.model_key(model)?,crate::embedding_values::digest(&input.input_sha256)?], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
        match row {
            Some((tokens, checksum)) if tokens == input.token_count && checksum.len() == 32 => {
                Ok(Some(checksum))
            }
            Some(_) => Err(invalid("selected native identity mismatch")),
            None => Ok(None),
        }
    }
    fn remember(
        &self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
        checksum: &[u8],
        bytes: Option<&[u8]>,
    ) -> Result<(), IndexError> {
        if let Some(old) = self.selection(model, input)? {
            if old != checksum {
                return Err(invalid("selected native checksum changed"));
            }
            return Ok(());
        }
        self.selected
            .prepare_cached("INSERT INTO selected VALUES(?,?,?,?,?)")?
            .execute(params![
                self.model_key(model)?,
                crate::embedding_values::digest(&input.input_sha256)?,
                input.token_count,
                checksum,
                bytes
            ])?;
        Ok(())
    }
    fn evict_cache(&mut self) -> Result<(), IndexError> {
        while self.cache_bytes > self.cache_limit {
            let write = self.workspace.write(self.schema)?;
            let mut next_cache_bytes = self.cache_bytes;
            let oldest = write.connection().prepare("SELECT p.input_id,length(p.vector),i.model_id FROM pending_embedding_values p LEFT JOIN embedding_inputs i USING(input_id) ORDER BY p.cached_at,p.input_id LIMIT 128")?.query_map([], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,u64>(1)?,r.get::<_,Option<i64>>(2)?)))?.collect::<Result<Vec<_>,_>>()?;
            if oldest.is_empty() {
                self.cache_bytes = 0;
                break;
            }
            for (id, bytes, model_id) in oldest {
                write.connection().execute(
                    "DELETE FROM pending_embedding_values WHERE input_id=?",
                    [id],
                )?;
                // Owned staging is keyed by full identity, not this disposable ID.
                write.connection().execute("DELETE FROM embedding_inputs WHERE input_id=? AND published_hash IS NULL AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM trait_seed_chunks t WHERE t.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=embedding_inputs.input_id)",[id])?;
                if let Some(model_id) = model_id {
                    write.connection().execute("DELETE FROM embedding_models WHERE model_id=? AND NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id) AND NOT EXISTS(SELECT 1 FROM search_profiles p WHERE p.identity_json=embedding_models.identity_json)",[model_id])?;
                }
                next_cache_bytes = next_cache_bytes
                    .checked_sub(bytes)
                    .ok_or_else(|| invalid("cache payload accounting mismatch"))?;
                if next_cache_bytes <= self.cache_limit {
                    break;
                }
            }
            write.commit()?;
            self.cache_bytes = next_cache_bytes;
        }
        Ok(())
    }
    fn install_staged(&self, connection: &Connection) -> Result<(), IndexError> {
        let mut statement = self.selected.prepare("SELECT model,CASE WHEN length(input_hash)=32 THEN input_hash ELSE zeroblob(0) END,token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END,length(vector) FROM selected ORDER BY model,input_hash")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let model_key: usize = row.get(0)?;
            let hash: Vec<u8> = row.get(1)?;
            let model = self
                .context()
                .search_profiles
                .get(model_key)
                .ok_or_else(|| invalid("selected model ordinal invalid"))?;
            let input = NativeInputDescriptor {
                input_sha256: hex_hash(&hash)?,
                token_count: row.get(2)?,
            };
            let expected: Vec<u8> = row.get(3)?;
            if expected.len() != 32 {
                return Err(invalid("selected native checksum length mismatch"));
            }
            if let Some(length) = row.get::<_, Option<usize>>(4)? {
                if length
                    != model
                        .dimensions
                        .checked_mul(4)
                        .ok_or_else(|| invalid("native geometry overflow"))?
                {
                    return Err(invalid("staged vector length mismatch"));
                }
                let bytes: Vec<u8> = self.selected.query_row("SELECT vector FROM selected WHERE model=? AND input_hash=? AND length(vector)=?", params![model_key,hash,length], |r| r.get(0))?;
                crate::native_values::check_bytes(&bytes, &expected, &model)?;
                crate::native_values::install(connection, &model, &input, &bytes, &expected)?;
            } else {
                let actual = crate::native_values::published_checksum(connection, &model, &input)?;
                if actual.as_ref() != Some(&expected) {
                    return Err(invalid("published selection changed before activation"));
                }
            }
        }
        Ok(())
    }
}
struct PublicationValues<'a> {
    connection: &'a Connection,
    selected: &'a Connection,
    models: &'a [SourceSearchProfileIdentity],
}
impl NativeValueSource for PublicationValues<'_> {
    fn selected_checksum(
        &self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Vec<u8>, IndexError> {
        let model_key = self
            .models
            .iter()
            .position(|stored| stored == model)
            .ok_or_else(|| invalid("native model differs from publication"))?;
        let (tokens,hash): (usize,Vec<u8>) = self.selected.prepare_cached("SELECT token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END FROM selected WHERE model=? AND input_hash=?")?
            .query_row(params![model_key,crate::embedding_values::digest(&input.input_sha256)?],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if tokens != input.token_count || hash.len() != 32 {
            return Err(invalid("selected native identity mismatch"));
        }
        Ok(hash)
    }
    fn value<'a>(
        &'a self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, IndexError> {
        check_input(model, input)?;
        let expected = self.selected_checksum(model, input)?;
        let id = crate::native_values::lookup_input(self.connection, model, input)?
            .ok_or_else(|| invalid("selected native input missing"))?;
        if crate::native_values::published_checksum(self.connection, model, input)?.as_ref()
            != Some(&expected)
        {
            return Err(invalid("selected native checksum mismatch"));
        }
        Ok(Cow::Owned(crate::native_values::read(
            self.connection,
            id,
            model,
        )?))
    }
}
fn bounded(length: usize) -> Result<(), IndexError> {
    if length > NATIVE_VALUE_BATCH_LIMIT {
        return Err(invalid("native value batch exceeds its bound"));
    }
    Ok(())
}
fn check_input(
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Result<(), IndexError> {
    crate::embedding_values::digest(&input.input_sha256)?;
    if input.token_count == 0 || input.token_count > model.token_limit {
        return Err(invalid("invalid native token count"));
    }
    Ok(())
}
fn hex_hash(bytes: &[u8]) -> Result<String, IndexError> {
    if bytes.len() != 32 {
        return Err(invalid("native input hash length mismatch"));
    }
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}
fn operation(error: std::io::Error) -> IndexError {
    IndexError::Operation(format!("prepared publication: {error}"))
}

#[cfg(test)]
mod tests;
