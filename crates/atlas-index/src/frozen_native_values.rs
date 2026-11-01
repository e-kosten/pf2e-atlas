//! Copy-before-mutation numerical publication input. This private temporary
//! SQLite file is disposable staging, never another persisted reference authority.
use crate::{
    ArtifactInstance, IndexError, NATIVE_VALUE_BATCH_LIMIT, NativeValueSource, PublicationSource,
    SourceSearchProfileIdentity,
};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{UpdateOwner, Workspace};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, io::Read};

// Keyset order belongs to the membership primary key, so each page stops before
// joining later memberships. Missing metadata remains a row and fails typed decode.
const SELECTED_NATIVE_PAGE_SQL: &str = "SELECT n.input_id,m.identity_json,lower(hex(i.input_hash)),i.token_count,CASE WHEN n.selected_hash IS NULL OR length(n.selected_hash)=32 THEN n.selected_hash ELSE zeroblob(0) END FROM native_target_inputs n LEFT JOIN embedding_inputs i USING(input_id) LEFT JOIN embedding_models m USING(model_id) WHERE n.job_key=? AND n.input_id>? ORDER BY n.input_id LIMIT 128";

fn io(error: impl std::fmt::Display) -> IndexError {
    IndexError::Operation(format!("frozen native values: {error}"))
}
fn file_digest(path: &std::path::Path) -> Result<Vec<u8>, IndexError> {
    let mut file = std::fs::File::open(path).map_err(io)?;
    let mut hash = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let n = file.read(&mut bytes).map_err(io)?;
        if n == 0 {
            break;
        }
        hash.update(&bytes[..n]);
    }
    Ok(hash.finalize().to_vec())
}

/// Exact values selected before publication can overwrite any document slot.
/// SQL connections here refer only to this temporary file, not the workspace.
pub struct FrozenNativeValues {
    connection: Connection,
    models: Vec<FrozenModel>,
    file: tempfile::NamedTempFile,
    file_digest: Vec<u8>,
    target: String,
    origin: ArtifactInstance,
    source_expected: Option<ArtifactInstance>,
    job: Option<String>,
    manifest: crate::native_manifest::NativeManifest,
    logical_bytes: u64,
}
struct FrozenModel {
    identity: SourceSearchProfileIdentity,
    id: i64,
    json: String,
}
struct Freezer {
    connection: Connection,
    file: tempfile::NamedTempFile,
    logical_bytes: u64,
    models: Vec<FrozenModel>,
}
impl Freezer {
    fn new() -> Result<Self, IndexError> {
        let file = tempfile::NamedTempFile::new().map_err(io)?;
        let connection = Connection::open(file.path())?;
        connection.execute_batch("PRAGMA cache_size=-1024; PRAGMA mmap_size=0; PRAGMA synchronous=OFF; CREATE TABLE frozen_models(model_id INTEGER PRIMARY KEY,identity_json TEXT NOT NULL UNIQUE) STRICT; CREATE TABLE native_values(model_id INTEGER NOT NULL REFERENCES frozen_models,input_hash BLOB NOT NULL,token_count INTEGER NOT NULL,checksum BLOB NOT NULL CHECK(length(checksum)=32),vector BLOB NOT NULL,PRIMARY KEY(model_id,input_hash)) WITHOUT ROWID,STRICT;")?;
        connection.execute_batch("BEGIN IMMEDIATE")?;
        Ok(Self {
            file,
            connection,
            logical_bytes: 0,
            models: Vec::new(),
        })
    }
    fn push(
        &mut self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
        vector: &[f32],
        expected: &[u8],
    ) -> Result<(), IndexError> {
        crate::validation::check_vector(vector, model.dimensions)?;
        if input.token_count == 0 || input.token_count > model.token_limit {
            return Err(IndexError::Invalid(
                "invalid frozen native token count".into(),
            ));
        }
        let bytes = crate::read_units::vector_bytes(vector);
        let checksum = crate::embedding_values::hash_bytes(&bytes);
        if checksum != expected {
            return Err(IndexError::Invalid(
                "frozen native checksum differs from selected value".into(),
            ));
        }
        // Only the finite requested model contracts are cached here; numerical
        // values remain in the bounded-page temporary file.
        let model_id =
            if let Some(stored) = self.models.iter().find(|stored| stored.identity == *model) {
                stored.id
            } else {
                let json = serde_json::to_string(model)?;
                self.connection.execute(
                    "INSERT INTO frozen_models(identity_json) VALUES(?) ON CONFLICT DO NOTHING",
                    [&json],
                )?;
                let id = self.connection.query_row(
                    "SELECT model_id FROM frozen_models WHERE identity_json=?",
                    [&json],
                    |r| r.get(0),
                )?;
                self.models.push(FrozenModel {
                    identity: model.clone(),
                    id,
                    json,
                });
                id
            };
        let hash = crate::embedding_values::digest(&input.input_sha256)?;
        let old: Option<(usize, Vec<u8>)> = self
            .connection
            .query_row(
                "SELECT token_count,checksum FROM native_values WHERE model_id=? AND input_hash=?",
                params![model_id, hash],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((tokens, old)) = old {
            if tokens != input.token_count || old != checksum {
                return Err(IndexError::Invalid(
                    "duplicate frozen native identities disagree".into(),
                ));
            }
            return Ok(());
        }
        self.connection.execute(
            "INSERT INTO native_values VALUES(?,?,?,?,?)",
            params![model_id, hash, input.token_count, checksum, bytes],
        )?;
        self.logical_bytes = self
            .logical_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| io("value bytes overflow"))?;
        Ok(())
    }
    fn seal(
        self,
        target: String,
        origin: ArtifactInstance,
        source_expected: Option<ArtifactInstance>,
        job: Option<String>,
        manifest: crate::native_manifest::NativeManifest,
    ) -> Result<FrozenNativeValues, IndexError> {
        self.connection.execute_batch("CREATE TABLE publication_origin(workspace_key TEXT NOT NULL,reference_revision INTEGER NOT NULL,target_hash TEXT NOT NULL,job_key TEXT,membership_digest BLOB NOT NULL,occurrences INTEGER NOT NULL,source_workspace_key TEXT,source_reference_revision INTEGER) STRICT;")?;
        self.connection.execute(
            "INSERT INTO publication_origin VALUES(?,?,?,?,?,?,?,?)",
            params![
                origin.workspace_key(),
                origin.reference_revision(),
                target,
                job,
                manifest.digest,
                manifest.occurrences,
                source_expected
                    .as_ref()
                    .map(ArtifactInstance::workspace_key),
                source_expected
                    .as_ref()
                    .map(ArtifactInstance::reference_revision)
            ],
        )?;
        self.connection.execute_batch("COMMIT")?;
        drop(self.connection);
        let digest = file_digest(self.file.path())?;
        let connection =
            Connection::open_with_flags(self.file.path(), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        connection
            .execute_batch("PRAGMA cache_size=-1024; PRAGMA mmap_size=0; PRAGMA query_only=ON;")?;
        let integrity: String = connection.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        if integrity != "ok" {
            return Err(io("temporary file integrity check failed"));
        }
        let model_count: usize =
            connection.query_row("SELECT count(*) FROM frozen_models", [], |row| row.get(0))?;
        if model_count != self.models.len() {
            return Err(io("frozen model identity coverage differs"));
        }
        for model in &self.models {
            let json: String = connection.query_row(
                "SELECT identity_json FROM frozen_models WHERE model_id=?",
                [model.id],
                |row| row.get(0),
            )?;
            if json != model.json
                || serde_json::from_str::<SourceSearchProfileIdentity>(&json)? != model.identity
            {
                return Err(io("frozen model identity differs"));
            }
        }
        Ok(FrozenNativeValues {
            models: self.models,
            file: self.file,
            connection,
            file_digest: digest,
            target,
            origin,
            source_expected,
            job,
            manifest,
            logical_bytes: self.logical_bytes,
        })
    }
}
impl FrozenNativeValues {
    pub fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }
    pub(crate) fn check_publication(
        &self,
        c: &Connection,
        source: &dyn PublicationSource,
        job: Option<&str>,
    ) -> Result<(), IndexError> {
        let actual = origin(c)?;
        if actual != self.origin {
            return Err(IndexError::Invalid(
                "frozen native values belong to another workspace generation".into(),
            ));
        }
        self.check(source, job)
    }
    pub fn temporary_file_bytes(&self) -> Result<u64, IndexError> {
        Ok(self.temporary_file_usage()?.file_bytes)
    }
    pub fn temporary_file_usage(&self) -> Result<crate::BuildFileUsage, IndexError> {
        crate::BuildFileUsage::of(self.file.as_file())
    }
    pub(crate) fn check(
        &self,
        source: &dyn PublicationSource,
        job: Option<&str>,
    ) -> Result<(), IndexError> {
        if self.target != source.context().sha256()?
            || self.job.as_deref() != job
            || source.expected_instance() != self.source_expected.as_ref()
            || self
                .source_expected
                .as_ref()
                .is_some_and(|expected| expected != &self.origin)
        {
            return Err(IndexError::Invalid(
                "frozen values belong to another publication target".into(),
            ));
        }
        let actual = crate::native_manifest::visit_native_manifest(source, &mut |_, _| Ok(()))?;
        if actual != self.manifest || file_digest(self.file.path())? != self.file_digest {
            return Err(IndexError::Invalid(
                "frozen numerical file or publication membership changed".into(),
            ));
        }
        Ok(())
    }
}
impl NativeValueSource for FrozenNativeValues {
    fn value<'a>(
        &'a self,
        model: &SourceSearchProfileIdentity,
        input: &NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, IndexError> {
        let expected = model
            .dimensions
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| IndexError::Invalid("frozen vector length overflow".into()))?;
        let stored = self
            .models
            .iter()
            .find(|stored| stored.identity == *model)
            .ok_or_else(|| io("frozen native model identity mismatch"))?;
        let hash = crate::embedding_values::digest(&input.input_sha256)?;
        let lengths: Option<(usize, usize)> = self.connection.prepare_cached(
            "SELECT length(v.vector),length(v.checksum) FROM native_values v JOIN frozen_models m USING(model_id) WHERE v.model_id=? AND m.identity_json=? AND v.input_hash=?"
        )?.query_row(params![stored.id, &stored.json, hash], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
        if lengths != Some((expected, 32)) {
            return Err(IndexError::Invalid(
                "frozen native value missing or length mismatch".into(),
            ));
        }
        let row: Option<(usize, Vec<u8>, Vec<u8>)> = self.connection.prepare_cached(
            "SELECT v.token_count,v.checksum,v.vector FROM native_values v JOIN frozen_models m USING(model_id) WHERE v.model_id=? AND m.identity_json=? AND v.input_hash=? AND length(v.vector)=? AND length(v.checksum)=32"
        )?.query_row(params![stored.id, &stored.json, hash, expected], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()?;
        let (tokens, checksum, bytes) = row
            .ok_or_else(|| IndexError::Invalid("selected frozen native input is missing".into()))?;
        if tokens != input.token_count || crate::embedding_values::hash_bytes(&bytes) != checksum {
            return Err(IndexError::Invalid(
                "frozen native identity or checksum mismatch".into(),
            ));
        }
        let vector = crate::read_units::decode_vector(&bytes)?;
        crate::validation::check_vector(&vector, model.dimensions)?;
        Ok(Cow::Owned(vector))
    }
}

/// Freeze every selected native value in bounded pages before obtaining final
/// maintenance admission or mutating profile tables, slots or occurrences.
pub fn freeze_native_values(
    owner: &UpdateOwner,
    workspace: &Workspace,
    schema: u32,
    job: &str,
    source: &dyn PublicationSource,
) -> Result<FrozenNativeValues, IndexError> {
    crate::native_targets::check_owner(owner, workspace)?;
    freeze_native_values_into(owner, workspace, schema, job, source, Freezer::new()?)
}

fn freeze_native_values_into(
    owner: &UpdateOwner,
    workspace: &Workspace,
    schema: u32,
    job: &str,
    source: &dyn PublicationSource,
    mut freezer: Freezer,
) -> Result<FrozenNativeValues, IndexError> {
    crate::native_targets::check_owner(owner, workspace)?;
    let manifest = crate::native_manifest::visit_native_manifest(source, &mut |_, _| Ok(()))?;
    let target = source.context().sha256()?;
    let actual_origin = {
        let read = workspace.read(schema)?;
        let c = read.connection();
        crate::native_selection::require_active(c, job)?;
        if let Some(expected) = source.expected_instance()
            && (read.workspace_key()? != expected.workspace_key()
                || read.reference_revision()? != expected.reference_revision())
        {
            return Err(IndexError::Invalid(
                "workspace generation changed before numerical freeze".into(),
            ));
        }
        let (digest,count,job_target,members,actual_members):(Vec<u8>,usize,String,usize,usize)=c.query_row("SELECT membership_digest,registered_count,target_hash,member_count,(SELECT count(*) FROM native_target_inputs WHERE job_key=?1) FROM native_update_targets JOIN reference_update_jobs USING(job_key) WHERE job_key=?1",[job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
        if members != actual_members {
            return Err(IndexError::Invalid(
                "selected native target member count changed".into(),
            ));
        }
        if digest != manifest.digest || count != manifest.occurrences || job_target != target {
            return Err(IndexError::Invalid(
                "native target differs from complete publication membership".into(),
            ));
        }
        origin(c)?
    };
    let mut after = 0i64;
    loop {
        let read = workspace.read(schema)?;
        let c = read.connection();
        crate::native_selection::require_active(c, job)?;
        let rows = c
            .prepare(SELECTED_NATIVE_PAGE_SQL)?
            .query_map(params![job, after], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, usize>(3)?,
                    r.get::<_, Option<Vec<u8>>>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        if rows.is_empty() {
            break;
        }
        crate::native_selection::bounded(rows.len())?;
        for (id, model, hash, tokens, expected) in rows {
            after = id;
            let model: SourceSearchProfileIdentity = serde_json::from_str(&model)?;
            let input = NativeInputDescriptor {
                input_sha256: hash,
                token_count: tokens,
            };
            let expected = expected.ok_or_else(|| {
                IndexError::Invalid("native target has an unselected input".into())
            })?;
            let value =
                crate::native_selection::load_value(c, &model, &input)?.ok_or_else(|| {
                    IndexError::Invalid("selected native value disappeared before freeze".into())
                })?;
            freezer.push(&model, &input, &value.vector, &expected)?;
        }
    }
    let frozen = freezer.seal(
        target,
        actual_origin,
        source.expected_instance().cloned(),
        Some(job.into()),
        manifest,
    )?;
    frozen.check(source, Some(job))?;
    let read = workspace.read(schema)?;
    let pending_bytes: u64 = read.connection().query_row(
        "SELECT coalesce(sum(length(vector)),0) FROM pending_embedding_values",
        [],
        |r| r.get(0),
    )?;
    tracing::info!(target:"atlas_build_diagnostic",logical_pending_bytes=pending_bytes,temporary_bound_value_bytes=frozen.logical_bytes(),temporary_numerical_file_bytes=frozen.temporary_file_bytes()?,native_value_batch_limit=NATIVE_VALUE_BATCH_LIMIT,"Frozen numerical publication values");
    Ok(frozen)
}

/// Genuine fixtures use the same frozen provider and the same writer. This
/// helper is absent from normal product builds and never reads workspace slots.
#[cfg(any(test, feature = "test-support"))]
pub fn freeze_fixture_native_values(
    source: &dyn PublicationSource,
    values: &dyn NativeValueSource,
    job: Option<&str>,
    origin: &ArtifactInstance,
) -> Result<FrozenNativeValues, IndexError> {
    let mut freezer = Freezer::new()?;
    let manifest = crate::native_manifest::visit_native_manifest(source, &mut |model, input| {
        let value = values.value(model, input)?;
        let checksum =
            crate::embedding_values::hash_bytes(&crate::read_units::vector_bytes(&value));
        freezer.push(model, input, &value, &checksum)
    })?;
    freezer.seal(
        source.context().sha256()?,
        origin.clone(),
        source.expected_instance().cloned(),
        job.map(str::to_owned),
        manifest,
    )
}

pub(crate) fn origin(c: &Connection) -> Result<ArtifactInstance, IndexError> {
    Ok(c.query_row(
        "SELECT workspace_key,reference_revision FROM workspace WHERE id=1",
        [],
        |r| Ok(ArtifactInstance::new(r.get(0)?, r.get(1)?)),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_support::*, *};
    fn descriptor(input: &IndexBuildInput) -> NativeInputDescriptor {
        NativeInputDescriptor {
            input_sha256: input.semantic_units[0].input_hash.clone(),
            token_count: input.semantic_units[0].input_token_count,
        }
    }
    #[test]
    fn temporary_sqlite_full_during_freeze_keeps_previous_publication_and_retry_work() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("native.sqlite");
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let job = crate::native_targets_tests::staged(&owner, &workspace, &input, false);
        let before = reference_preservation_rows(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
        )
        .unwrap();
        let freezer = Freezer::new().unwrap();
        freezer
            .connection
            .pragma_update(None, "max_page_count", 2)
            .unwrap();
        let error = freeze_native_values_into(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input,
            freezer,
        )
        .err()
        .unwrap();
        assert!(
            error.to_string().contains("database or disk is full"),
            "{error}"
        );
        assert_eq!(
            reference_preservation_rows(
                workspace
                    .read(TEST_WORKSPACE_SCHEMA_VERSION)
                    .unwrap()
                    .connection()
            )
            .unwrap(),
            before
        );
        assert_eq!(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            1
        );
        assert!(
            freeze_native_values(
                &owner,
                &workspace,
                TEST_WORKSPACE_SCHEMA_VERSION,
                &job,
                &input
            )
            .is_ok()
        );
    }
    #[test]
    fn oversized_temporary_value_is_rejected_before_fetch_and_file_corruption_blocks_publication() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("native.sqlite");
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let job = crate::native_targets_tests::staged(&owner, &workspace, &input, true);
        let frozen = freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input,
        )
        .unwrap();
        {
            let corrupt = Connection::open(frozen.file.path()).unwrap();
            corrupt
                .execute("UPDATE native_values SET vector=zeroblob(8388608)", [])
                .unwrap();
        }
        let error = frozen
            .value(&input.context.search_profiles[0], &descriptor(&input))
            .unwrap_err();
        assert!(error.to_string().contains("length mismatch"), "{error}");
        let maintenance = owner
            .maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "corrupt frozen values")
            .unwrap();
        assert!(
            SqliteIndexWriter::new(&maintenance)
                .write_job(&input, &frozen, &job, &input.context.sha256().unwrap())
                .is_err()
        );
        assert_eq!(
            maintenance
                .connection()
                .query_row("SELECT reference_revision FROM workspace", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn repeated_profile_lookups_keep_exact_model_binding_and_observe_model_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = initialize_workspace(dir.path().join("native.sqlite")).unwrap();
        let input = crate::profile_tests::both();
        let initial = origin(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
        )
        .unwrap();
        let frozen = freeze_fixture_native_values(&input, &input, None, &initial).unwrap();
        for _ in 0..2 {
            for unit in &input.semantic_units {
                let model = input.context.search_profile(&unit.profile_id).unwrap();
                let descriptor = NativeInputDescriptor {
                    input_sha256: unit.input_hash.clone(),
                    token_count: unit.input_token_count,
                };
                assert_eq!(
                    frozen.value(model, &descriptor).unwrap().as_ref(),
                    unit.vector.as_slice()
                );
            }
        }
        let model = &input.context.search_profiles[0];
        let unit = input
            .semantic_units
            .iter()
            .find(|unit| unit.profile_id == model.profile_id)
            .unwrap();
        let descriptor = NativeInputDescriptor {
            input_sha256: unit.input_hash.clone(),
            token_count: unit.input_token_count,
        };
        let mut changed = model.clone();
        changed.input_recipe.push_str("/changed");
        assert!(frozen.value(&changed, &descriptor).is_err());
        let corrupt = Connection::open(frozen.file.path()).unwrap();
        corrupt
            .execute(
                "UPDATE frozen_models SET identity_json=? WHERE identity_json=?",
                params![
                    serde_json::to_string(&changed).unwrap(),
                    serde_json::to_string(model).unwrap()
                ],
            )
            .unwrap();
        assert!(frozen.value(model, &descriptor).is_err());
        assert!(frozen.check(&input, None).is_err());
    }

    #[test]
    fn frozen_seal_binds_workspace_revision_and_source_expectation_including_empty_origin() {
        let dir = tempfile::tempdir().unwrap();
        let first = initialize_workspace(dir.path().join("one.sqlite")).unwrap();
        let second = initialize_workspace(dir.path().join("two.sqlite")).unwrap();
        let input = crate::foundation_tests::fixture(true);
        let initial = origin(
            first
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
        )
        .unwrap();
        assert_eq!(initial.reference_revision(), 0);
        let frozen = freeze_fixture_native_values(&input, &input, None, &initial).unwrap();
        let mut wrong = input.context.search_profiles[0].clone();
        wrong.input_recipe.push_str("/different-execution");
        assert!(frozen.value(&wrong, &descriptor(&input)).is_err());
        assert_eq!(
            frozen
                .connection
                .query_row("SELECT count(*) FROM frozen_models", [], |r| r
                    .get::<_, usize>(0))
                .unwrap(),
            1
        );
        assert!(
            frozen
                .check_publication(
                    first
                        .read(TEST_WORKSPACE_SCHEMA_VERSION)
                        .unwrap()
                        .connection(),
                    &input,
                    None
                )
                .is_ok()
        );
        assert!(
            frozen
                .check_publication(
                    second
                        .read(TEST_WORKSPACE_SCHEMA_VERSION)
                        .unwrap()
                        .connection(),
                    &input,
                    None
                )
                .is_err()
        );
        let expected = origin(
            second
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
        )
        .unwrap();
        let mut changed = PublicationSpool::new(
            input.context.clone(),
            input.packs.clone(),
            CompleteSourceInventory::checked(
                &input.context.source_fingerprint,
                input.packs.iter().map(|p| p.pack_id.clone()),
                input.packs.iter().map(|p| p.pack_id.clone()),
            )
            .unwrap(),
            expected,
        )
        .unwrap();
        input
            .semantic(&mut |unit| changed.semantic.push(unit))
            .unwrap();
        input
            .trait_seeds(&mut |seed| changed.trait_seeds.push(seed))
            .unwrap();
        let changed = changed.seal().unwrap();
        assert!(frozen.check(&changed, None).is_err());
        {
            let write = first.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            write
                .connection()
                .execute(
                    "UPDATE workspace SET reference_revision=reference_revision+1",
                    [],
                )
                .unwrap();
            write.commit().unwrap();
        }
        assert!(
            frozen
                .check_publication(
                    first
                        .read(TEST_WORKSPACE_SCHEMA_VERSION)
                        .unwrap()
                        .connection(),
                    &input,
                    None
                )
                .is_err()
        );
    }
}

#[cfg(test)]
#[path = "frozen_native_values/page_tests.rs"]
mod page_tests;
