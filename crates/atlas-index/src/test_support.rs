//! Reference-layer fixtures and artifact mutations. Runtime fixtures use the
//! runtime-owned workspace composition and supply its version to mutation helpers.
use crate::{
    ArtifactValidationReport, IndexArtifactWriter, IndexBuildInput, IndexError, SqliteIndexReader,
    SqliteIndexWriter,
};
use atlas_storage::{Migration, Workspace};
use std::path::Path;

pub const TEST_WORKSPACE_SCHEMA_VERSION: u32 = TEST_MIGRATIONS.len() as u32;
pub(crate) const TEST_MIGRATIONS: &[Migration] = &[
    Migration {
        backfill: None,
        version: 1,
        name: "reference-baseline",
        sql: crate::REFERENCE_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 2,
        name: "saved-state-baseline",
        sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 3,
        name: "lexical-alias-rows",
        sql: crate::LEXICAL_ALIAS_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 4,
        name: "consumable-filters",
        sql: crate::CONSUMABLE_FILTER_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 5,
        name: "encounter-status",
        sql: atlas_local_state::ENCOUNTER_STATUS_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 6,
        name: "hazard-participation",
        sql: atlas_local_state::HAZARD_PARTICIPATION_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 7,
        name: "encounter-parties",
        sql: atlas_local_state::PARTIES_SCHEMA_SQL,
    },
    Migration {
        backfill: None,
        version: 8,
        name: "trait-vocabulary",
        sql: crate::TRAIT_SCHEMA_SQL,
    },
    Migration {
        version: 9,
        name: "condition-system",
        sql: atlas_local_state::CONDITION_SYSTEM_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 10,
        name: "embedding-storage",
        sql: crate::EMBEDDING_STORAGE_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            id: "native-embedding-storage/v1",
            apply: crate::backfill_embedding_storage,
        }),
    },
    Migration {
        version: 11,
        name: "persistent-preparation",
        sql: atlas_local_state::PERSISTENT_PREPARATION_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 12,
        name: "turn-resources",
        sql: atlas_local_state::TURN_RESOURCES_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 13,
        name: "consumed-condition-applications",
        sql: atlas_local_state::CONSUMED_CONDITION_APPLICATIONS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 14,
        name: "party-member-levels",
        sql: atlas_local_state::PARTY_LEVELS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 15,
        name: "encounter-difficulty",
        sql: atlas_local_state::ENCOUNTER_DIFFICULTY_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 16,
        name: "similarity-parts",
        sql: crate::SIMILARITY_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            id: "typed-content-correspondence/v1",
            apply: crate::backfill_similarity_parts,
        }),
    },
];
pub fn initialize_workspace(path: impl AsRef<Path>) -> Result<Workspace, IndexError> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| IndexError::Operation(e.to_string()))?;
    let workspace = Workspace::at(path)?;
    workspace.migrate(TEST_MIGRATIONS, |_| Ok(()))?;
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
        // Build with current production projection code, then retain only the
        // historical physical shape under test. No legacy runtime writer.
        connection.execute_batch(crate::TRAIT_SCHEMA_SQL)?;
        connection.execute_batch(crate::EMBEDDING_STORAGE_SCHEMA_SQL)?;
        crate::writer::write_record_rows(connection, input)?;
        connection.execute_batch("INSERT INTO legacy_semantic_units SELECT u.unit_id,u.record_id,u.profile_id,l.owners_json,u.unit_kind,l.field_path,l.section_json,l.chunk_ordinal,i.token_count,lower(hex(i.input_hash)) FROM semantic_units u JOIN semantic_locations l USING(location_id) JOIN embedding_inputs i USING(input_id);
DROP TABLE semantic_units;
ALTER TABLE legacy_semantic_units RENAME TO semantic_units;
CREATE INDEX semantic_units_root ON semantic_units(record_id,profile_id,unit_id);
CREATE INDEX semantic_units_input_hash ON semantic_units(input_hash,unit_id);
CREATE UNIQUE INDEX semantic_identity_root ON semantic_units(record_id,profile_id) WHERE unit_kind='identity';
CREATE INDEX semantic_reuse_input ON semantic_units(profile_id,input_hash,input_token_count);
DROP TABLE trait_seed_chunks; DROP TABLE trait_vectors; DROP TABLE legacy_trait_vectors;
DROP TABLE trait_memberships; DROP TABLE trait_references; DROP TABLE trait_definitions;
DROP TABLE pending_embedding_values; DROP TABLE trait_native_values;
DROP TABLE embedding_inputs; DROP TABLE embedding_models; DROP TABLE semantic_locations;")?;
        Ok(())
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

/// Exact schema-2/9 checkpoint fixture; not a runtime compatibility path.
pub fn stage_historical_embedding_fixture(
    session: &atlas_storage::WriteSession,
    job: &str,
    model: &crate::SourceSearchProfileIdentity,
    input: &atlas_embedding::PreparedEmbeddingInput,
    vector: &[f32],
) -> Result<(), IndexError> {
    let bytes = crate::read_units::vector_bytes(vector);
    session.connection().execute("INSERT INTO embedding_checkpoints VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(reuse_key) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",rusqlite::params![input.reuse_key,serde_json::to_string(model)?,input.input_sha256,input.token_count,model.dimensions,bytes,crate::codec::sha256(&bytes),job])?;
    Ok(())
}
