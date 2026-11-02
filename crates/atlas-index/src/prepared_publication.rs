//! One owned publication: stable published handles and only unpublished staged bytes.
use crate::native_values::NativeInput;
use crate::{
    ArtifactInstance, ArtifactValidationReport, BuildFileUsage, IndexError, PublicationSpool,
    SourceArtifactBuildContext, SourceSearchProfileIdentity,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{UpdateOwner, Workspace};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

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
    models: Vec<crate::native_values::NativeModelKey>,
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
        let models = crate::native_values::NativeModelKey::all(&metadata.context.search_profiles)?;
        let mut publication = Self {
            selected,
            selected_file,
            owner,
            workspace,
            schema,
            metadata,
            identities,
            models,
            cache_bytes,
            cache_limit: CACHE_PAYLOAD_LIMIT,
        };
        publication.evict_cache()?;
        Ok(publication)
    }
    pub fn context(&self) -> &SourceArtifactBuildContext {
        &self.metadata.context
    }
    pub fn product_record_count(&self) -> usize {
        self.identities
            .iter()
            .filter(|identity| identity.is_product)
            .count()
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
        let ordinal = self.model_key(model)?;
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let read = self.workspace.read(self.schema)?;
        crate::writer::check_expected(read.connection(), &self.metadata)?;
        let binding = crate::native_values::NativeModel::bind(
            read.connection(),
            &self.models[ordinal],
            false,
        )?;
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let mut output = Vec::with_capacity(inputs.len());
        for input in inputs {
            let input = NativeInput::new(model, input)?;
            if self.selection(ordinal, &input)?.is_some() {
                output.push(NativeValueSelection::AlreadySelected);
                continue;
            }
            if let Some(selected) = binding.select(&input, reuse_published)? {
                self.remember(
                    ordinal,
                    &input,
                    &selected.checksum,
                    selected.staged.as_deref(),
                )?;
                output.push(NativeValueSelection::Reused);
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
        let ordinal = self.model_key(model)?;
        if batch.is_empty() {
            return Ok(());
        }
        let mut encoded = Vec::with_capacity(batch.len());
        let mut batch_hashes = std::collections::BTreeMap::new();
        for (input, vector) in batch {
            let input = NativeInput::new(model, input)?;
            crate::validation::check_vector(vector, model.dimensions)?;
            let bytes = crate::read_units::vector_bytes(vector);
            let checksum = crate::embedding_values::hash_bytes(&bytes);
            if batch_hashes
                .insert(*input.hash(), (input.token_count(), checksum.clone()))
                .is_some_and(|old| old != (input.token_count(), checksum.clone()))
            {
                return Err(invalid("conflicting inferred values within batch"));
            }
            if let Some(expected) = self.selection(ordinal, &input)?
                && expected != checksum
            {
                return Err(invalid(
                    "attempted to replace an already selected native value",
                ));
            }
            encoded.push(EncodedNativeValue {
                input,
                bytes,
                checksum,
            });
        }
        let write = self.workspace.write(self.schema)?;
        crate::writer::check_expected(write.connection(), &self.metadata)?;
        let mut next_cache_bytes = self.cache_bytes;
        {
            let binding = crate::native_values::NativeModel::bind(
                write.connection(),
                &self.models[ordinal],
                true,
            )?;
            for value in &encoded {
                let id = binding.ensure_input(&value.input)?;
                let old: u64 = write.connection().query_row("SELECT coalesce((SELECT length(vector) FROM pending_embedding_values WHERE input_id=?),0)", [id], |r| r.get(0))?;
                write.connection().execute("INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at) VALUES(?,?,?,unixepoch()) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,cached_at=excluded.cached_at", params![id,value.bytes,value.checksum])?;
                next_cache_bytes = next_cache_bytes
                    .checked_sub(old)
                    .and_then(|n| n.checked_add(value.bytes.len() as u64))
                    .ok_or_else(|| invalid("cache payload accounting mismatch"))?;
            }
        }
        write.commit()?;
        self.cache_bytes = next_cache_bytes;
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        for value in &encoded {
            self.remember(ordinal, &value.input, &value.checksum, Some(&value.bytes))?;
        }
        selected_tx.commit()?;
        self.evict_cache()
    }
    pub fn publish(self) -> Result<ArtifactValidationReport, IndexError> {
        let maintenance = self
            .owner
            .maintenance(self.schema, "publishing reference update")?;
        maintenance
            .connection()
            .execute_batch("PRAGMA temp_store=FILE;")?;
        let prior = crate::ReferencePublicationExpectation::read(maintenance.connection())?;
        // This synchronous observation point follows durable maintenance admission
        // and precedes all publication mutations; guarded recovery authenticates it.
        tracing::info!(target:"atlas_progress",phase="validate_build_input",workspace_key=prior.instance().workspace_key(),reference_revision=prior.instance().reference_revision(),context_sha256=prior.context_sha256().unwrap_or(""),"Saving reference update");
        maintenance.publish(|connection| {
            connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
            crate::writer::check_expected(connection, &self.metadata)?;
            let reusable = crate::writer::prior_compatible(connection)?;
            let native_started = std::time::Instant::now();
            let delta_started = {
                let mut bindings = crate::native_values::NativeBindings::new(connection, &self.models, true)?;
                bindings.ensure_tables()?;
                self.install_staged(&mut bindings)?;
                tracing::info!(target:"atlas_build_diagnostic",stage="native_installation",elapsed_ms=native_started.elapsed().as_secs_f64()*1000.,"Installed selected unpublished values");
                let values = PublicationValues { selected: &self.selected, bindings: &bindings };
                let scratch = self.selected_file.path().parent().ok_or_else(|| invalid("publication scratch parent absent"))?;
                let delta_started = std::time::Instant::now();
                crate::stream_publication::write(connection, &self.metadata, &self.identities, &values, reusable, scratch)?;
                delta_started
            };
            connection.execute("DELETE FROM pending_embedding_values", [])?;
            crate::native_values::prune(connection)?;
            crate::native_allocation::compact_profiles(connection, &self.metadata.context)?;
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
        ordinal: usize,
        input: &NativeInput,
    ) -> Result<Option<Vec<u8>>, IndexError> {
        let row: Option<(usize, Vec<u8>)> = self.selected.prepare_cached("SELECT token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END FROM selected WHERE model=? AND input_hash=?")?.query_row(params![ordinal,input.hash()], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
        match row {
            Some((tokens, checksum)) if tokens == input.token_count() && checksum.len() == 32 => {
                Ok(Some(checksum))
            }
            Some(_) => Err(invalid("selected native identity mismatch")),
            None => Ok(None),
        }
    }
    fn remember(
        &self,
        ordinal: usize,
        input: &NativeInput,
        checksum: &[u8],
        bytes: Option<&[u8]>,
    ) -> Result<(), IndexError> {
        if let Some(old) = self.selection(ordinal, input)? {
            if old != checksum {
                return Err(invalid("selected native checksum changed"));
            }
            return Ok(());
        }
        self.selected
            .prepare_cached("INSERT INTO selected VALUES(?,?,?,?,?)")?
            .execute(params![
                ordinal,
                input.hash(),
                input.token_count(),
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
    fn install_staged(
        &self,
        bindings: &mut crate::native_values::NativeBindings<'_>,
    ) -> Result<(), IndexError> {
        let mut statement = self.selected.prepare("SELECT model,CASE WHEN length(input_hash)=32 THEN input_hash ELSE zeroblob(0) END,token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END,length(vector) FROM selected ORDER BY model,input_hash")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let model_key: usize = row.get(0)?;
            let hash: Vec<u8> = row.get(1)?;
            let binding = bindings.ordinal_mut(model_key)?;
            let model = binding.key.model();
            let input = NativeInput::from_hash(
                model,
                hash.try_into()
                    .map_err(|_| invalid("native input hash length mismatch"))?,
                row.get(2)?,
            )?;
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
                let bytes: Vec<u8> = self.selected.query_row("SELECT vector FROM selected WHERE model=? AND input_hash=? AND length(vector)=?", params![model_key,input.hash(),length], |r| r.get(0))?;
                binding.install(&input, &bytes, &expected)?;
            } else {
                let actual = binding.published_checksum(&input)?;
                if actual.as_ref() != Some(&expected) {
                    return Err(invalid("published selection changed before activation"));
                }
            }
        }
        Ok(())
    }
}
struct EncodedNativeValue {
    input: NativeInput,
    bytes: Vec<u8>,
    checksum: Vec<u8>,
}
pub(crate) struct PublicationValues<'a, 'session> {
    selected: &'a Connection,
    bindings: &'a crate::native_values::NativeBindings<'session>,
}
impl PublicationValues<'_, '_> {
    fn selected_checksum(
        &self,
        ordinal: usize,
        input: &NativeInput,
    ) -> Result<Vec<u8>, IndexError> {
        let (tokens,hash): (usize,Vec<u8>) = self.selected.prepare_cached("SELECT token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END FROM selected WHERE model=? AND input_hash=?")?
            .query_row(params![ordinal,input.hash()],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if tokens != input.token_count() || hash.len() != 32 {
            return Err(invalid("selected native identity mismatch"));
        }
        Ok(hash)
    }
    pub(crate) fn input_id(
        &self,
        profile: &atlas_domain::SearchProfileId,
        input: &NativeInputDescriptor,
    ) -> Result<i64, IndexError> {
        let binding = self.bindings.profile(profile)?;
        let input = NativeInput::new(binding.key.model(), input)?;
        let checksum = self.selected_checksum(binding.key.ordinal(), &input)?;
        binding.selected_input(&input, &checksum)
    }
    pub(crate) fn value(
        &self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Vec<f32>, IndexError> {
        let binding = self.bindings.profile(&model.profile_id)?;
        if binding.key.model() != model {
            return Err(invalid("native model differs from publication"));
        }
        let id = self.input_id(&model.profile_id, input)?;
        binding.read(id)
    }
}
fn bounded(length: usize) -> Result<(), IndexError> {
    if length > NATIVE_VALUE_BATCH_LIMIT {
        return Err(invalid("native value batch exceeds its bound"));
    }
    Ok(())
}
fn invalid(message: &str) -> IndexError {
    IndexError::Invalid(message.into())
}
fn operation(error: std::io::Error) -> IndexError {
    IndexError::Operation(format!("prepared publication: {error}"))
}

#[cfg(test)]
mod tests;
