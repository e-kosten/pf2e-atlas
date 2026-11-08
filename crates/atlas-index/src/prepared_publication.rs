//! One owned publication: stable published handles and only unpublished staged bytes.
use crate::native_values::NativeInput;
use crate::{
    BuildFileUsage, IndexError, PublicationSpool, SourceArtifactBuildContext,
    SourceSearchProfileIdentity,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{ReferenceRepair, UpdateOwner, Workspace};
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
struct Rebuild<'owner> {
    repair: ReferenceRepair<'owner>,
    configuration: crate::RequestedReferenceConfig,
}

pub struct PreparedPublication<'owner> {
    selected: Connection,
    selected_file: tempfile::NamedTempFile,
    owner: &'owner UpdateOwner,
    workspace: Workspace,
    schema: u32,
    metadata: PublicationSpool,
    models: Vec<crate::native_values::NativeModelKey>,
    previous_layouts: Vec<Option<crate::native_values::NativeModelKey>>,
    verified_profiles: Vec<bool>,
    cache_bytes: u64,
    cache_limit: u64,
    rebuild: Option<Rebuild<'owner>>,
}
impl<'owner> PreparedPublication<'owner> {
    pub fn new_in(
        owner: &'owner UpdateOwner,
        schema: u32,
        scratch: &Path,
        metadata: PublicationSpool,
    ) -> Result<Self, IndexError> {
        Self::prepare(owner, schema, scratch, metadata, None)
    }

    /// Stage a complete rebuild privately, with no published or pending reuse.
    pub fn new_rebuild(
        repair: ReferenceRepair<'owner>,
        scratch: &Path,
        metadata: PublicationSpool,
        configuration: crate::RequestedReferenceConfig,
    ) -> Result<Self, IndexError> {
        Self::prepare(
            repair.owner(),
            repair.schema(),
            scratch,
            metadata,
            Some(Rebuild {
                repair,
                configuration,
            }),
        )
    }

    fn prepare(
        owner: &'owner UpdateOwner,
        schema: u32,
        scratch: &Path,
        metadata: PublicationSpool,
        rebuild: Option<Rebuild<'owner>>,
    ) -> Result<Self, IndexError> {
        let validation_started = std::time::Instant::now();
        crate::writer::validate_input(&metadata)?;
        tracing::info!(target:"atlas_build_diagnostic",stage="publication_input_validation",elapsed_ms=validation_started.elapsed().as_secs_f64()*1000.,"Completed publication input validation");
        Self::stage(owner, schema, scratch, metadata, rebuild)
    }

    fn stage(
        owner: &'owner UpdateOwner,
        schema: u32,
        scratch: &Path,
        metadata: PublicationSpool,
        rebuild: Option<Rebuild<'owner>>,
    ) -> Result<Self, IndexError> {
        let workspace = Workspace::at(owner.path())?;
        let (cache_bytes, previous) = if rebuild.is_some() {
            (0, None)
        } else {
            let read = workspace.read(schema)?;
            crate::writer::check_expected(read.connection(), &metadata)?;
            let (bytes, has_overlaps): (u64, bool) = read.connection().query_row(
                "SELECT coalesce(sum(length(p.vector)) FILTER (WHERE i.published_hash IS NULL),0),
                        count(i.published_hash)>0
                 FROM pending_embedding_values p LEFT JOIN embedding_inputs i USING(input_id)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let previous = crate::read_stored_reference_context(read.connection())?;
            drop(read);
            if has_overlaps {
                let write = workspace.write(schema)?;
                crate::writer::check_expected(write.connection(), &metadata)?;
                write.connection().execute(
                    "DELETE FROM pending_embedding_values WHERE EXISTS(
                         SELECT 1 FROM embedding_inputs i
                         WHERE i.input_id=pending_embedding_values.input_id AND i.published_hash IS NOT NULL)",
                    [],
                )?;
                write.commit()?;
            }
            (bytes, previous)
        };
        let previous_layouts = metadata
            .context
            .search_profiles
            .iter()
            .enumerate()
            .map(|(ordinal, model)| {
                previous
                    .as_ref()
                    .and_then(|context| {
                        context
                            .search_profiles
                            .iter()
                            .find(|old| *old != model && old.vector_compatible_with(model))
                    })
                    .map(|old| crate::native_values::NativeModelKey::new(ordinal, old))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
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
            verified_profiles: vec![false; models.len()],
            models,
            previous_layouts,
            cache_bytes,
            cache_limit: CACHE_PAYLOAD_LIMIT,
            rebuild,
        };
        if publication.rebuild.is_none() {
            publication.evict_cache()?;
        }
        Ok(publication)
    }
    pub fn context(&self) -> &SourceArtifactBuildContext {
        &self.metadata.context
    }
    pub fn product_record_count(&self) -> usize {
        self.metadata
            .identities()
            .iter()
            .filter(|identity| identity.is_product)
            .count()
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
    /// Load counts from the exact published model at this attempt's expected revision.
    pub fn verified_native_token_counts(
        &self,
        model: &SourceSearchProfileIdentity,
    ) -> Result<crate::VerifiedNativeTokenCounts, IndexError> {
        let ordinal = self.model_key(model)?;
        if self.rebuild.is_some() || self.schema < crate::NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION {
            return Ok(crate::VerifiedNativeTokenCounts::default());
        }
        let started = std::time::Instant::now();
        let read = self.workspace.read(self.schema)?;
        crate::writer::check_expected(read.connection(), &self.metadata)?;
        let counts =
            crate::native_token_counts::verify(read.connection(), &self.models[ordinal], true)?;
        tracing::info!(target: "atlas_build_diagnostic", stage = "native_token_count_verification",
            profile = %model.profile_id, elapsed_ms = started.elapsed().as_secs_f64() * 1000.,
            reuse_available = !counts.is_empty(), "Verified published native token counts");
        Ok(counts)
    }

    /// Record completion of the source ingest path that checked every selected
    /// descriptor against its tokenizer or the authenticated published counts.
    pub fn finish_verified_native_profile(
        &mut self,
        model: &SourceSearchProfileIdentity,
    ) -> Result<(), IndexError> {
        let ordinal = self.model_key(model)?;
        self.verified_profiles[ordinal] = true;
        Ok(())
    }

    pub fn append_semantic(
        &mut self,
        occurrence: crate::SemanticOccurrence,
    ) -> Result<(), IndexError> {
        self.metadata.semantic.push(&occurrence)
    }
    pub fn semantic_unit_count(&self) -> usize {
        self.metadata.semantic.len()
    }
    pub fn select_native(
        &mut self,
        model: &SourceSearchProfileIdentity,
        inputs: &[NativeInputDescriptor],
    ) -> Result<Vec<NativeValueSelection>, IndexError> {
        bounded(inputs.len())?;
        let ordinal = self.model_key(model)?;
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        if self.rebuild.is_some() {
            return inputs
                .iter()
                .map(|descriptor| {
                    let input = NativeInput::new(model, descriptor)?;
                    Ok(
                        if selected_checksum(&self.selected, ordinal, &input)?.is_some() {
                            NativeValueSelection::AlreadySelected
                        } else {
                            NativeValueSelection::Missing
                        },
                    )
                })
                .collect();
        }
        let read = self.workspace.read(self.schema)?;
        crate::writer::check_expected(read.connection(), &self.metadata)?;
        let binding = crate::native_values::NativeModel::bind(
            read.connection(),
            &self.models[ordinal],
            false,
        )?;
        let previous = self.previous_layouts[ordinal]
            .as_ref()
            .map(|key| crate::native_values::NativeModel::bind(read.connection(), key, false))
            .transpose()?;
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let mut output = Vec::with_capacity(inputs.len());
        for input in inputs {
            let input = NativeInput::new(model, input)?;
            if selected_checksum(&self.selected, ordinal, &input)?.is_some() {
                output.push(NativeValueSelection::AlreadySelected);
                continue;
            }
            let selected = match binding.select(&input)? {
                Some(selected) => Some(selected),
                None => previous
                    .as_ref()
                    .map(|old| old.copy_published(&input))
                    .transpose()?
                    .flatten(),
            };
            if let Some(selected) = selected {
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
            if vector.len() != model.dimensions {
                return Err(invalid("inferred vector dimension mismatch"));
            }
            let bytes = crate::read_units::vector_bytes(vector);
            let checksum = crate::embedding_values::hash_bytes(&bytes);
            if batch_hashes
                .insert(*input.hash(), (input.token_count(), checksum.clone()))
                .is_some_and(|old| old != (input.token_count(), checksum.clone()))
            {
                return Err(invalid("conflicting inferred values within batch"));
            }
            if let Some(expected) = selected_checksum(&self.selected, ordinal, &input)?
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
        if self.rebuild.is_none() {
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
                    let old: Option<u64> = write
                        .connection()
                        .prepare_cached(
                            "SELECT coalesce(length(p.vector),0) FROM embedding_inputs i
                         LEFT JOIN pending_embedding_values p USING(input_id)
                         WHERE i.input_id=? AND i.published_hash IS NULL",
                        )?
                        .query_row([id], |r| r.get(0))
                        .optional()?;
                    let Some(old) = old else { continue };
                    write.connection().execute("INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at) VALUES(?,?,?,unixepoch()) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,cached_at=excluded.cached_at", params![id,value.bytes,value.checksum])?;
                    next_cache_bytes = next_cache_bytes
                        .checked_sub(old)
                        .and_then(|n| n.checked_add(value.bytes.len() as u64))
                        .ok_or_else(|| invalid("cache payload accounting mismatch"))?;
                }
            }
            write.commit()?;
            self.cache_bytes = next_cache_bytes;
        }
        let selected_tx = rusqlite::Transaction::new_unchecked(
            &self.selected,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        for value in &encoded {
            self.remember(ordinal, &value.input, &value.checksum, Some(&value.bytes))?;
        }
        selected_tx.commit()?;
        if self.rebuild.is_none() {
            self.evict_cache()?;
        }
        Ok(())
    }
    pub fn publish(self) -> Result<(), IndexError> {
        let apply = |connection: &rusqlite::Connection| {
            connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
            tracing::info!(target:"atlas_progress",phase="validate_build_input","Saving reference update");
            let reusable = if self.rebuild.is_some() {
                crate::writer::check_workspace_revision(connection, &self.metadata)?;
                crate::reference_reset::clear(connection)?;
                false
            } else {
                crate::writer::check_expected(connection, &self.metadata)?;
                crate::writer::prior_compatible(connection)?
            };
            let native_started = std::time::Instant::now();
            let delta_started;
            {
                let mut bindings =
                    crate::native_values::NativeBindings::new(connection, &self.models, true)?;
                bindings.ensure_tables()?;
                self.install_staged(&mut bindings)?;
                tracing::info!(target:"atlas_build_diagnostic",stage="native_installation",elapsed_ms=native_started.elapsed().as_secs_f64()*1000.,"Installed selected unpublished values");
                let values = PublicationValues {
                    selected: &self.selected,
                    bindings: &bindings,
                };
                let scratch = self
                    .selected_file
                    .path()
                    .parent()
                    .ok_or_else(|| invalid("publication scratch parent absent"))?;
                delta_started = std::time::Instant::now();
                crate::stream_publication::write(
                    connection,
                    &self.metadata,
                    &values,
                    reusable,
                    scratch,
                )?;
            }
            connection.execute("DELETE FROM pending_embedding_values", [])?;
            crate::native_values::prune(connection)?;
            crate::native_allocation::compact_profiles(connection, &self.metadata.context)?;
            tracing::info!(target:"atlas_build_diagnostic",stage="publication_delta_and_prune",elapsed_ms=delta_started.elapsed().as_secs_f64()*1000.,"Completed reference reconciliation");
            if self.schema >= crate::NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION {
                let started = std::time::Instant::now();
                for (ordinal, key) in self.models.iter().enumerate() {
                    crate::native_token_counts::publish(
                        connection,
                        &self.selected,
                        ordinal,
                        key,
                        self.verified_profiles[ordinal],
                    )?;
                }
                tracing::info!(target: "atlas_build_diagnostic", stage = "native_token_count_publication",
                    elapsed_ms = started.elapsed().as_secs_f64() * 1000., "Published native token-count digests");
            }
            if let Some(rebuild) = &self.rebuild {
                crate::validate_reference_with_token_count_digests(connection)?;
                crate::requested_config::write_configuration(connection, &rebuild.configuration)?;
            }
            Ok(())
        };
        if let Some(rebuild) = &self.rebuild {
            rebuild.repair.publish(apply)
        } else {
            self.owner.publish_incremental(self.schema, apply)
        }
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
    fn remember(
        &self,
        ordinal: usize,
        input: &NativeInput,
        checksum: &[u8],
        bytes: Option<&[u8]>,
    ) -> Result<(), IndexError> {
        self.selected
            .prepare_cached(
                "INSERT INTO selected VALUES(?,?,?,?,?) ON CONFLICT(model,input_hash) DO NOTHING",
            )?
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
                write.connection().execute("DELETE FROM embedding_inputs WHERE input_id=? AND published_hash IS NULL AND NOT EXISTS(SELECT 1 FROM semantic_units u WHERE u.input_id=embedding_inputs.input_id) AND NOT EXISTS(SELECT 1 FROM pending_embedding_values p WHERE p.input_id=embedding_inputs.input_id)",[id])?;
                if let Some(model_id) = model_id {
                    write.connection().execute("DELETE FROM embedding_models WHERE model_id=? AND NOT EXISTS(SELECT 1 FROM embedding_inputs i WHERE i.model_id=embedding_models.model_id) AND NOT EXISTS(SELECT 1 FROM search_profiles p WHERE p.model_id=embedding_models.model_id)",[model_id])?;
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
    pub(crate) fn input_id(
        &self,
        profile: &atlas_domain::SearchProfileId,
        input: &NativeInputDescriptor,
    ) -> Result<i64, IndexError> {
        let binding = self.bindings.profile(profile)?;
        let input = NativeInput::new(binding.key.model(), input)?;
        let checksum = selected_checksum(self.selected, binding.key.ordinal(), &input)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        binding.selected_input(&input, &checksum)
    }
    #[cfg(test)]
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
fn selected_checksum(
    selected: &Connection,
    ordinal: usize,
    input: &NativeInput,
) -> Result<Option<Vec<u8>>, IndexError> {
    let row: Option<(usize, Vec<u8>)> = selected.prepare_cached("SELECT token_count,CASE WHEN length(selected_hash)=32 THEN selected_hash ELSE zeroblob(0) END FROM selected WHERE model=? AND input_hash=?")?.query_row(params![ordinal,input.hash()], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
    match row {
        Some((tokens, checksum)) if tokens == input.token_count() && checksum.len() == 32 => {
            Ok(Some(checksum))
        }
        Some(_) => Err(invalid("selected native identity mismatch")),
        None => Ok(None),
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
