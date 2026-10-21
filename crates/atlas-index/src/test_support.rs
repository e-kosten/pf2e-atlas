//! Reference-layer fixtures and artifact mutations. Runtime fixtures use the
//! runtime-owned workspace composition and supply its version to mutation helpers.
use crate::{
    ArtifactValidationReport, IndexArtifactWriter, IndexBuildInput, IndexError, SqliteIndexReader,
    SqliteIndexWriter,
};
use atlas_storage::{Migration, Workspace};
use std::path::Path;

pub const TEST_WORKSPACE_SCHEMA_VERSION: u32 = 8;
pub fn initialize_workspace(path: impl AsRef<Path>) -> Result<Workspace, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    let workspace = Workspace::at(path)?;
    workspace.migrate(
        &[
            Migration {
                version: 1,
                name: "reference-baseline",
                sql: crate::REFERENCE_SCHEMA_SQL,
            },
            Migration {
                version: 2,
                name: "saved-state-baseline",
                sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
            },
            Migration {
                version: 3,
                name: "lexical-alias-rows",
                sql: crate::LEXICAL_ALIAS_SCHEMA_SQL,
            },
            Migration {
                version: 4,
                name: "consumable-filters",
                sql: crate::CONSUMABLE_FILTER_SCHEMA_SQL,
            },
            Migration {
                version: 5,
                name: "encounter-status",
                sql: atlas_local_state::ENCOUNTER_STATUS_SCHEMA_SQL,
            },
            Migration {
                version: 6,
                name: "hazard-participation",
                sql: atlas_local_state::HAZARD_PARTICIPATION_SCHEMA_SQL,
            },
            Migration {
                version: 7,
                name: "encounter-parties",
                sql: atlas_local_state::PARTIES_SCHEMA_SQL,
            },
            Migration {
                version: 8,
                name: "trait-vocabulary",
                sql: crate::TRAIT_SCHEMA_SQL,
            },
        ],
        |_| Ok(()),
    )?;
    Ok(workspace)
}
pub fn open_reader(path: impl AsRef<Path>) -> Result<SqliteIndexReader, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    SqliteIndexReader::from_session(Workspace::at(path)?.read(TEST_WORKSPACE_SCHEMA_VERSION)?)
}
pub fn publish_input(
    path: impl AsRef<Path>,
    input: &IndexBuildInput,
) -> Result<ArtifactValidationReport, IndexError> {
    let workspace = initialize_workspace(path)?;
    let owner = workspace.update_owner()?;
    if !input.trait_vectors.is_empty() {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION)?;
        for seed in &input.trait_vectors {
            let model = input.context.search_profile(&seed.profile_id)?;
            let spec = atlas_embedding::search_profile(&seed.profile_id)
                .map_err(|e| IndexError::Invalid(e.to_string()))?
                .model_spec();
            let inputs = seed
                .chunks
                .iter()
                .map(|chunk| atlas_embedding::PreparedEmbeddingInput {
                    address: chunk.address.clone(),
                    input: chunk.input.clone(),
                    input_sha256: chunk.input_hash.clone(),
                    reuse_key: spec.document_reuse_key(&chunk.input),
                    token_count: chunk.token_count,
                    body_token_count: chunk.token_count,
                    overlap_token_count: 0,
                    context_shortened: false,
                })
                .collect::<Vec<_>>();
            let values = inputs
                .iter()
                .zip(&seed.chunks)
                .map(|(input, chunk)| (input, chunk.vector.as_slice()))
                .collect::<Vec<_>>();
            crate::save_embedding_checkpoints(&write, "fixture", model, &values)?;
        }
        write.commit()?;
    }
    let maintenance = owner.maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "test publication")?;
    SqliteIndexWriter::new(&maintenance).write(input)
}

/// Seed only the record component of a deliberately historical schema fixture.
/// This does not produce a current readable artifact: ordinary publication must
/// use `SqliteIndexWriter` after the complete migration ledger is applied.
pub fn publish_historical_record_fixture(
    maintenance: &atlas_storage::MaintenanceSession<'_>,
    input: &IndexBuildInput,
) -> Result<(), IndexError> {
    if !input.traits.is_empty() || !input.trait_vectors.is_empty() {
        return Err(IndexError::Invalid(
            "historical record fixtures cannot publish trait vocabulary".into(),
        ));
    }
    maintenance.publish(|connection| {
        connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
        crate::writer::write_record_rows(connection, input)
    })
}

/// Model an older preparation contract coherently, including the allocation
/// digests used to decide whether publication can retain a record's rows.
pub fn set_content_interpretation_version(
    path: impl AsRef<Path>,
    workspace_schema_version: u32,
    version: &str,
) -> Result<(), IndexError> {
    set_preparation_policy(path, workspace_schema_version, |context| {
        context.content_interpretation_version = version.into()
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
    let json = serde_json::to_string(&context)?;
    c.execute(
        "UPDATE artifact_context SET context_hash=?1,context_json=?2",
        [crate::codec::sha256(json.as_bytes()), json],
    )?;
    c.execute(
        "UPDATE prepared_content SET preparation_context_hash=?",
        [context.preparation_sha256()?],
    )?;
    for source in crate::read_persisted_inputs(c, &context)? {
        let (id, label): (i64, String) = c.query_row(
            "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?",
            [source.record.key().to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        c.execute(
            "UPDATE projection_allocations SET input_hash=? WHERE record_id=?",
            rusqlite::params![crate::publication_delta::root_digest(&source, &label)?, id],
        )?;
    }
    write.commit()?;
    Ok(())
}
