//! Reference-layer fixtures and artifact mutations. Runtime fixtures use the
//! runtime-owned workspace composition and supply its version to mutation helpers.
mod section_preservation;
pub use section_preservation::native_preservation_rows;
pub use section_preservation::reference_preservation_rows;

use crate::{IndexBuildInput, IndexError, SourceSearchProfileIdentity, SqliteIndexReader};
use atlas_storage::Workspace;
use std::{collections::BTreeMap, path::Path};

/// Explicit caller-supplied task-copy diagnostic. All projection writes roll back.
pub fn core_projection_pilot(path: &Path) -> Result<serde_json::Value, IndexError> {
    crate::core_projection_pilot::run(path)
}

// Compile the runtime-owned declaration for these fixtures without a runtime crate dependency.
#[path = "../../atlas-runtime/src/release_history.rs"]
mod release_history;
pub(crate) use release_history::RELEASE_HISTORY;

pub const TEST_WORKSPACE_SCHEMA_VERSION: u32 = RELEASE_HISTORY.target();

pub fn initialize_workspace(path: impl AsRef<Path>) -> Result<Workspace, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    let workspace = Workspace::at(path)?;
    workspace.migrate_history(RELEASE_HISTORY, |_| Ok(()))?;
    Ok(workspace)
}

pub fn reference_repair(
    owner: &atlas_storage::UpdateOwner,
) -> Result<atlas_storage::ReferenceRepair<'_>, IndexError> {
    let (repair, ()) =
        owner.begin_reference_repair(RELEASE_HISTORY, |_| Ok::<_, IndexError>(()))?;
    Ok(repair)
}
pub fn open_reader(path: impl AsRef<Path>) -> Result<SqliteIndexReader, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    SqliteIndexReader::from_session(Workspace::at(path)?.read(TEST_WORKSPACE_SCHEMA_VERSION)?)
}
pub fn publish_input(path: impl AsRef<Path>, input: &IndexBuildInput) -> Result<(), IndexError> {
    let workspace = initialize_workspace(path)?;
    publish_input_in(&workspace, TEST_WORKSPACE_SCHEMA_VERSION, input)
}

/// Publish explicit glossary fixtures through the same checked publication owner.
/// The caller supplies complete inventory/scope evidence in the input context.
pub fn publish_input_with_references(
    path: impl AsRef<Path>,
    input: &IndexBuildInput,
    entries: &[crate::ReferenceEntry],
    notices: &[crate::ReferenceNotice],
) -> Result<(), IndexError> {
    let workspace = initialize_workspace(path)?;
    publish_input_in_with_references(
        &workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        input,
        entries,
        notices,
    )
}

pub fn publish_input_in(
    workspace: &Workspace,
    schema: u32,
    input: &IndexBuildInput,
) -> Result<(), IndexError> {
    publish_input_in_with_references(workspace, schema, input, &[], &[])
}

pub fn publish_input_in_with_references(
    workspace: &Workspace,
    schema: u32,
    input: &IndexBuildInput,
    entries: &[crate::ReferenceEntry],
    notices: &[crate::ReferenceNotice],
) -> Result<(), IndexError> {
    let native = fixture_native_values(input)?;
    let owner = workspace.update_owner()?;
    let scratch = crate::BuildScratch::new(&owner)?;
    crate::recover_reference_publication(&owner, schema)?;
    let read = workspace.read(schema)?;
    let expected = crate::ArtifactInstance::new(read.workspace_key()?, read.reference_revision()?);
    drop(read);
    let mut metadata = spool_input_in(scratch.path(), input, expected)?;
    for entry in entries {
        metadata.reference_entries.push(entry)?;
    }
    for notice in notices {
        metadata.reference_notices.push(notice)?;
    }
    let mut abilities = crate::OwnedAbilityCatalog::default();
    for entry in entries {
        abilities.add_entry(entry);
    }
    for definition in &input.traits {
        abilities.add_trait(
            definition.key.clone().into(),
            definition.label_key.as_deref(),
            definition.description_key.as_deref(),
        );
    }
    for source in &input.records {
        if crate::reference_pack_applies(
            input.context.reference_preparation.as_ref(),
            "pf2e",
            source.record.key().pack().as_str(),
        ) {
            let occurrences = atlas_record::source_record::owned_action_localizations(
                &source.record,
                input.context.audience,
            );
            for evidence in abilities.project(occurrences) {
                metadata.owned_ability_evidence.push(&evidence)?;
            }
        }
    }
    let mut publication =
        crate::PreparedPublication::new_in(&owner, schema, scratch.path(), metadata)?;
    for model in &input.context.search_profiles {
        let values = native
            .iter()
            .filter(|value| value.model == model)
            .collect::<Vec<_>>();
        for window in values.chunks(crate::NATIVE_VALUE_BATCH_LIMIT) {
            let batch = window
                .iter()
                .map(|value| (&value.input, value.vector))
                .collect::<Vec<_>>();
            publication.stage_inferred(model, &batch)?;
        }
    }
    publication.publish()
}

/// Own concrete metadata for a real PreparedPublication fixture.
pub fn publication_spool_in(
    scratch: &Path,
    input: &IndexBuildInput,
    expected: crate::ArtifactInstance,
) -> Result<crate::PublicationSpool, IndexError> {
    fixture_native_values(input)?;
    spool_input_in(scratch, input, expected)
}

fn spool_input_in(
    scratch: &Path,
    input: &IndexBuildInput,
    expected: crate::ArtifactInstance,
) -> Result<crate::PublicationSpool, IndexError> {
    input
        .inventory
        .validate_parts(&input.context.source_fingerprint, &input.packs)?;
    let inventory = crate::CompleteSourceInventory::checked(
        &input.context.source_fingerprint,
        input.packs.iter().map(|pack| pack.pack_id.clone()),
        input.packs.iter().map(|pack| pack.pack_id.clone()),
    )?;
    let mut spool = crate::PublicationSpool::new_in(
        scratch,
        input.context.clone(),
        input.packs.clone(),
        inventory,
        expected,
    )?;
    for source in &input.records {
        let pack = input
            .packs
            .iter()
            .find(|pack| pack.pack_id == source.record.key().pack().as_str())
            .ok_or_else(|| IndexError::Invalid("record pack is absent".into()))?;
        let selection = atlas_record::source_record::select_record_search(
            &source.record,
            &source.content,
            input.context.audience,
            &pack.label,
            &input.context.used_trait_labels,
        )
        .map_err(|error| IndexError::Invalid(error.to_string()))?;
        spool.push_record(crate::PreparedRecordInput {
            source: crate::SourceArtifactRecordInput {
                record: atlas_record::source_record::SourceBackedRecord::new(
                    source.record.key().pack().as_str(),
                    atlas_foundry_model::decode_snapshot(&atlas_foundry_model::encode_snapshot(
                        source.record.source(),
                    )?)?,
                )
                .map_err(|_| IndexError::Invalid("fixture source identity differs".into()))?,
                source_path: source.source_path.clone(),
                content_hash: source.content_hash.clone(),
                preparation_context_hash: source.preparation_context_hash.clone(),
                content: serde_json::from_slice(&serde_json::to_vec(&source.content)?)?,
                relationships: serde_json::from_slice(&serde_json::to_vec(&source.relationships)?)?,
                diagnostics: source.diagnostics.clone(),
            },
            selection,
            lexical: input
                .lexical_units
                .iter()
                .filter(|row| row.record == *source.record.key())
                .cloned()
                .collect(),
        })?;
    }
    for row in &input.lexical_units {
        spool.lexical.push(row)?;
    }
    for unit in &input.semantic_units {
        spool.semantic.push(&crate::SemanticOccurrence {
            profile_id: unit.profile_id.clone(),
            record: unit.record.clone(),
            owners: unit.owners.clone(),
            field: unit.field.clone(),
            address: unit.address.clone(),
            chunk_ordinal: unit.chunk_ordinal,
            input: atlas_embedding::NativeInputDescriptor {
                input_sha256: unit.input_hash.clone(),
                token_count: unit.input_token_count,
            },
        })?;
    }
    for row in &input.remaster_context {
        spool.remaster_context.push(row)?;
    }
    for row in &input.aliases {
        spool.aliases.push(row)?;
    }
    spool.set_verified_duplicates(input.verified_duplicates.clone())?;
    for row in &input.remaster_pairs {
        spool.remaster_pairs.push(row)?;
    }
    for row in &input.traits {
        spool.traits.push(row)?;
    }
    Ok(spool)
}

/// Select/stage genuine fixture values through the real owned publisher.
pub fn prepare_publication_in<'owner>(
    owner: &'owner atlas_storage::UpdateOwner,
    schema: u32,
    scratch: &Path,
    input: &IndexBuildInput,
) -> Result<crate::PreparedPublication<'owner>, IndexError> {
    let mut publication = fixture_publication(owner, schema, scratch, input)?;
    for value in fixture_native_values(input)? {
        let selected =
            publication.select_native(value.model, std::slice::from_ref(&value.input))?;
        if selected == [crate::NativeValueSelection::Missing] {
            publication.stage_inferred(value.model, &[(&value.input, value.vector)])?;
        }
    }
    Ok(publication)
}

/// Inject explicit inference results so publication tests can exercise vector
/// replacement and rollback independently of ordinary selection policy.
#[cfg(test)]
pub(crate) fn prepare_publication_with_inferred_values<'owner>(
    owner: &'owner atlas_storage::UpdateOwner,
    schema: u32,
    scratch: &Path,
    input: &IndexBuildInput,
) -> Result<crate::PreparedPublication<'owner>, IndexError> {
    let mut publication = fixture_publication(owner, schema, scratch, input)?;
    for value in fixture_native_values(input)? {
        publication.stage_inferred(value.model, &[(&value.input, value.vector)])?;
    }
    Ok(publication)
}

pub(crate) fn fixture_publication<'owner>(
    owner: &'owner atlas_storage::UpdateOwner,
    schema: u32,
    scratch: &Path,
    input: &IndexBuildInput,
) -> Result<crate::PreparedPublication<'owner>, IndexError> {
    let workspace = Workspace::at(owner.path())?;
    let read = workspace.read(schema)?;
    let expected = crate::ArtifactInstance::new(read.workspace_key()?, read.reference_revision()?);
    drop(read);
    let metadata = spool_input_in(scratch, input, expected)?;
    crate::PreparedPublication::new_in(owner, schema, scratch, metadata)
}

pub fn set_content_interpretation_version(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, workspace_schema_version, |context| {
        context.content_interpretation_version = version.into()
    })
}

pub fn set_predecessor_source_contract(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
) -> Result<(), IndexError> {
    set_preparation_policy(path, workspace_schema_version, |context| {
        context.snapshot_version = 1;
        context.source_contract =
            "6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8".into();
    })
}

pub fn set_relationship_policy(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, workspace_schema_version, |context| {
        context.relationship_policy = version.into()
    })
}

pub fn set_content_selection_version(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, workspace_schema_version, |context| {
        context.content_selection_version = version.into()
    })
}

fn set_preparation_policy(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
    update: impl FnOnce(&mut crate::SourceArtifactBuildContext),
) -> Result<(), IndexError> {
    let workspace = Workspace::at(path)?;
    let write = workspace.write(workspace_schema_version)?;
    let c = write.connection();
    let mut context = crate::read_stored_reference_context(c)?
        .ok_or_else(|| IndexError::Invalid("fixture has no reference context".into()))?;
    update(&mut context);
    let preparation = context.preparation_sha256()?;
    let json = serde_json::to_string(&context)?;
    c.execute(
        "UPDATE artifact_context SET context_hash=?1,context_json=?2",
        [crate::codec::sha256(json.as_bytes()), json],
    )?;
    c.execute(
        "UPDATE prepared_content SET preparation_context_hash=?",
        [preparation],
    )?;
    write.commit()?;
    Ok(())
}

/// Deliberately encode a codec-valid altered retained snapshot for corruption
/// fixtures. Callers leave the source bytes and prior component proof unchanged.
pub fn altered_snapshot_for_fixture(
    source: &atlas_foundry_model::FoundryDocumentSource,
) -> Result<Vec<u8>, IndexError> {
    crate::codec::gzip(&atlas_foundry_model::encode_snapshot(source)?)
}

/// Change visible prose while retaining valid gzip, reference markers and all
/// existing cache sidecars. This deliberately leaves its prior proof stale.
pub fn alter_retained_html_for_fixture(
    connection: &rusqlite::Connection,
    key: &atlas_domain::RecordKey,
    field: &str,
    from: &str,
    to: &str,
) -> Result<(), IndexError> {
    let (id, bytes): (i64, Vec<u8>) = connection.query_row(
        "SELECT content_id,html_gzip FROM prepared_content WHERE record_id=(SELECT record_id FROM records WHERE key=?) AND field_path=?",
        rusqlite::params![key.to_string(), field],
        |row| Ok((row.get(0)?,row.get(1)?)),
    )?;
    let html = String::from_utf8(crate::codec::gunzip(&bytes)?)
        .map_err(|e| IndexError::Invalid(e.to_string()))?;
    if !html.contains(from) {
        return Err(IndexError::Invalid(
            "fixture replacement prose is absent".into(),
        ));
    }
    connection.execute(
        "UPDATE prepared_content SET html_gzip=? WHERE content_id=?",
        rusqlite::params![crate::codec::gzip(html.replace(from, to).as_bytes())?, id],
    )?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct FixtureNativeValue<'a> {
    pub model: &'a SourceSearchProfileIdentity,
    pub input: atlas_embedding::NativeInputDescriptor,
    pub vector: &'a [f32],
}

pub(crate) fn fixture_native_values<'a>(
    input: &'a IndexBuildInput,
) -> Result<Vec<FixtureNativeValue<'a>>, IndexError> {
    use atlas_embedding::NativeInputDescriptor;
    let mut selected = BTreeMap::<(String, String), FixtureNativeValue<'a>>::new();
    let mut accept = |profile: &atlas_domain::SearchProfileId,
                      hash: &str,
                      tokens: usize,
                      vector: &'a [f32]|
     -> Result<(), IndexError> {
        let model = input.context.search_profile(profile)?;
        crate::embedding_values::digest(hash)?;
        crate::validation::check_vector(vector, model.dimensions)?;
        if tokens == 0 || tokens > model.token_limit {
            return Err(IndexError::Invalid(
                "invalid fixture native token count".into(),
            ));
        }
        let key = (profile.to_string(), hash.to_owned());
        if let Some(old) = selected.get(&key) {
            if old.input.token_count != tokens
                || old.vector.len() != vector.len()
                || old
                    .vector
                    .iter()
                    .zip(vector)
                    .any(|(a, b)| a.to_bits() != b.to_bits())
            {
                return Err(IndexError::Invalid(
                    "duplicate native input values disagree".into(),
                ));
            }
        } else {
            selected.insert(
                key,
                FixtureNativeValue {
                    model,
                    input: NativeInputDescriptor {
                        input_sha256: hash.into(),
                        token_count: tokens,
                    },
                    vector,
                },
            );
        }
        Ok(())
    };
    for unit in &input.semantic_units {
        accept(
            &unit.profile_id,
            &unit.input_hash,
            unit.input_token_count,
            &unit.vector,
        )?;
    }
    Ok(selected.into_values().collect())
}

pub fn add_remaster_relationship(
    input: &mut crate::IndexBuildInput,
    old: &atlas_domain::RecordKey,
    new: &atlas_domain::RecordKey,
) -> Result<(), crate::IndexError> {
    input.remaster_pairs.push(crate::SourceRemasterPairInput {
        legacy: old.clone(),
        remaster: new.clone(),
        provenance: None,
    });
    Ok(())
}
