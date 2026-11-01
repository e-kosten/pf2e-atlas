//! Reference-layer fixtures and artifact mutations. Runtime fixtures use the
//! runtime-owned workspace composition and supply its version to mutation helpers.
mod section_preservation;
pub use section_preservation::reference_preservation_rows;

use crate::{
    ArtifactValidationReport, IndexArtifactWriter, IndexBuildInput, IndexError, SqliteIndexReader,
    SqliteIndexWriter,
};
use atlas_storage::{Migration, Workspace};
use std::path::Path;

/// Explicit caller-supplied task-copy diagnostic. All projection writes roll back.
pub fn core_projection_pilot(path: &Path) -> Result<serde_json::Value, IndexError> {
    crate::core_projection_pilot::run(path)
}

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
            reconstruct_tables: false,
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
        name: "encounter-ability-state",
        sql: atlas_local_state::ABILITY_STATE_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 17,
        name: "stateless-strike-rolls",
        sql: atlas_local_state::STATELESS_STRIKE_ROLLS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 18,
        name: "reference-numeric-filters",
        sql: crate::REFERENCE_NUMERIC_FILTER_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 19,
        name: "physical-base-type",
        sql: crate::PHYSICAL_BASE_TYPE_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 20,
        name: "similarity-parts",
        sql: crate::SIMILARITY_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "typed-content-correspondence/v1",
            apply: crate::backfill_similarity_parts,
        }),
    },
    Migration {
        version: 21,
        name: "core-section-eligibility",
        sql: crate::CORE_SECTIONS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 22,
        name: "encounter-recovery",
        sql: atlas_local_state::ENCOUNTER_RECOVERY_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 23,
        name: "saved-state-integrity",
        sql: "",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "saved-state-integrity/v1",
            apply: atlas_local_state::backfill_saved_state_integrity,
        }),
    },
    Migration {
        version: 24,
        name: "turn-receipts",
        sql: atlas_local_state::TURN_RECEIPTS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 25,
        name: "canonical-selected-sections",
        sql: crate::SELECTED_SECTIONS_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "canonical-selected-sections/v1",
            apply: crate::backfill_selected_sections,
        }),
    },
    Migration {
        version: 26,
        name: "reference-component-reconciliation",
        sql: crate::COMPONENT_DIGEST_SCHEMA_SQL,
        backfill: None,
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

/// Populate the actual schema-19/20 reference shape for section-upgrade tests.
/// Runtime owns its migration prefix; this fixture never rewinds a ledger or saved schema.
pub fn publish_pre_core_fixture(
    maintenance: &atlas_storage::MaintenanceSession<'_>,
    input: &IndexBuildInput,
) -> Result<(), IndexError> {
    maintenance.publish(|connection| {
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if !matches!(version, 19 | 20) {
            return Err(IndexError::Invalid(
                "pre-Core fixtures require an actual schema-19 or schema-20 workspace".into(),
            ));
        }
        connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
        let (ids, _) = crate::writer::write_record_base(connection, input)?;
        crate::historical_publication::search_units(connection, input, &ids)?;
        crate::traits::write_traits_stream(connection, input, input)?;
        if version == 20 {
            crate::similarity_projection::write_similarity_parts(connection, input)?;
        }
        let mut context = input.context.clone();
        context.core_sections_recipe.clear();
        context.selected_sections_recipe.clear();
        connection.execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![context.sha256()?, serde_json::to_string(&context)?],
        )?;
        if connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
            return Err(IndexError::Invalid(
                "invalid pre-Core fixture references".into(),
            ));
        }
        Ok(())
    })
}

/// Publish the real schema-21 through schema-24 reference shape before sections.
pub fn publish_pre_section_fixture(
    maintenance: &atlas_storage::MaintenanceSession<'_>,
    input: &IndexBuildInput,
) -> Result<(), IndexError> {
    maintenance.publish(|connection| {
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if !(21..=24).contains(&version) {
            return Err(IndexError::Invalid(
                "pre-section fixtures require an actual schema-21 through schema-24 workspace"
                    .into(),
            ));
        }
        connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
        let (ids, _) = crate::writer::write_record_base(connection, input)?;
        crate::historical_publication::search_units(connection, input, &ids)?;
        crate::traits::write_traits_stream(connection, input, input)?;
        crate::similarity_projection::write_similarity_parts(connection, input)?;
        crate::historical_core_projection::write_core_sections(connection, input)?;
        let mut context = input.context.clone();
        context.selected_sections_recipe.clear();
        connection.execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![context.sha256()?, serde_json::to_string(&context)?],
        )?;
        if connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
            return Err(IndexError::Invalid(
                "invalid historical section fixture references".into(),
            ));
        }
        Ok(())
    })
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
        let tables = ["actor_projection", "physical_projection", "weapon_projection", "armor_projection", "shield_projection"];
        let columns = |table: &str| -> Result<Vec<String>, IndexError> {
            let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
            Ok(statement.query_map([], |row| row.get(1))?.collect::<Result<Vec<_>, _>>()?)
        };
        let historical_columns = tables.iter().map(|table| columns(table)).collect::<Result<Vec<_>, _>>()?;
        connection.execute_batch(crate::REFERENCE_NUMERIC_FILTER_SCHEMA_SQL)?;
        connection.execute_batch(crate::PHYSICAL_BASE_TYPE_SCHEMA_SQL)?;
        let (ids,_)=crate::writer::write_record_base(connection,input)?;
        crate::historical_publication::search_units(connection,input,&ids)?;
        // Modern projection columns are fixture scaffolding, not part of the
        // historical schema exercised by the embedding conversion/rollback.
        for (table, historical) in tables.iter().zip(historical_columns) {
            for column in columns(table)?.into_iter().rev().filter(|column| !historical.contains(column)) {
                connection.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN {column}"))?;
            }
        }
        let mut context=input.context.clone();context.selected_sections_recipe.clear();context.core_sections_recipe.clear();
        connection.execute("UPDATE artifact_context SET context_hash=?,context_json=?",rusqlite::params![context.sha256()?,serde_json::to_string(&context)?])?;
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

/// Model the one known predecessor contract in an isolated fixture, including
/// preparation/allocation dependencies. Production never decodes this contract.
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
    let previous = context.clone();
    update(&mut context);
    let preparation = context.preparation_sha256()?;
    crate::visit_persisted_inputs(c, &previous, &mut |mut source| {
        source.preparation_context_hash = preparation.clone();
        let (id, label): (i64, String) = c.query_row(
            "SELECT r.record_id,p.label FROM records r JOIN packs p USING(pack_id) WHERE r.key=?",
            [source.record.key().to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        c.execute(
            "UPDATE projection_allocations SET input_hash=? WHERE record_id=?",
            rusqlite::params![crate::publication_delta::root_digest(&source, &label)?, id],
        )?;
        Ok(())
    })?;
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

/// Exact schema-2/9 checkpoint fixture; not a runtime compatibility path.
pub fn stage_historical_embedding_fixture(
    session: &atlas_storage::WriteSession,
    job: &str,
    model: &crate::SourceSearchProfileIdentity,
    input: &atlas_embedding::PreparedNativeInput,
    vector: &[f32],
) -> Result<(), IndexError> {
    let bytes = crate::read_units::vector_bytes(vector);
    session.connection().execute("INSERT INTO embedding_checkpoints VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(reuse_key) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",rusqlite::params![input.reuse_key,serde_json::to_string(model)?,input.input_sha256,input.token_count,model.dimensions,bytes,crate::codec::sha256(&bytes),job])?;
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
