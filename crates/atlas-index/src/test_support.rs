//! Reference-layer fixtures and artifact mutations. Runtime fixtures use the
//! runtime-owned workspace composition and supply its version to mutation helpers.
mod section_preservation;
pub use section_preservation::native_preservation_rows;
pub use section_preservation::reference_preservation_rows;

use crate::{IndexBuildInput, IndexError, SourceSearchProfileIdentity, SqliteIndexReader};
use atlas_storage::{Migration, Workspace};
use std::{collections::BTreeMap, path::Path};

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
        name: "manual-encounter-trackers",
        sql: atlas_local_state::MANUAL_TRACKERS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 24,
        name: "encounter-round-duration",
        sql: atlas_local_state::ENCOUNTER_CLOCK_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "encounter-clock-seconds/v1",
            apply: atlas_local_state::backfill_encounter_clock,
        }),
    },
    Migration {
        version: 25,
        name: "saved-state-integrity",
        sql: "",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "saved-state-integrity/v1",
            apply: atlas_local_state::backfill_saved_state_integrity,
        }),
    },
    Migration {
        version: 26,
        name: "turn-receipts",
        sql: atlas_local_state::TURN_RECEIPTS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 27,
        name: "input-owned-native-storage",
        sql: crate::INPUT_OWNED_STORAGE_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "input-owned-native-storage/v1",
            apply: crate::backfill_input_owned_storage,
        }),
    },
    Migration {
        version: 28,
        name: "canonical-selected-sections",
        sql: crate::SELECTED_SECTIONS_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "canonical-selected-sections/v1",
            apply: crate::backfill_selected_sections,
        }),
    },
    Migration {
        version: 29,
        name: "reference-component-reconciliation",
        sql: crate::COMPONENT_DIGEST_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 30,
        name: "remaster-information",
        sql: crate::REMASTER_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 31,
        name: "remove-remaster-preference",
        sql: crate::REMOVE_REMASTER_PREFERENCE_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 32,
        name: "compact-remaster-context",
        sql: crate::COMPACT_REMASTER_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 33,
        name: "reference-entries",
        sql: crate::REFERENCE_ENTRIES_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "reference-entries/v1",
            apply: crate::backfill_reference_entries,
        }),
    },
    Migration {
        version: 34,
        name: "retire-trait-similarity",
        sql: "",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "retire-trait-similarity/v1",
            apply: crate::retire_trait_similarity,
        }),
    },
    Migration {
        version: 35,
        name: "encounter-command-receipts",
        sql: atlas_local_state::ENCOUNTER_COMMAND_RECEIPTS_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: 36,
        name: "shared-trait-membership",
        sql: crate::TRAIT_MEMBERS_SCHEMA_SQL,
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "shared-trait-membership/v1",
            apply: crate::backfill_trait_members,
        }),
    },
    Migration {
        version: 37,
        name: "source-condition-override-payload",
        sql: "",
        backfill: None,
    },
    Migration {
        version: crate::NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_VERSION,
        name: "native-token-count-digest",
        sql: crate::NATIVE_TOKEN_COUNT_DIGEST_SCHEMA_SQL,
        backfill: None,
    },
    Migration {
        version: atlas_local_state::SAVED_STATE_CLOSEOUT_SCHEMA_VERSION,
        name: "saved-state-closeout",
        sql: "",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: false,
            id: "saved-state-closeout/v1",
            apply: atlas_local_state::backfill_saved_state_closeout,
        }),
    },
    Migration {
        version: crate::SEARCH_SCHEMA_CLOSEOUT_VERSION,
        name: "search-schema-closeout",
        sql: "",
        backfill: Some(atlas_storage::DataBackfill {
            reconstruct_tables: true,
            id: "search-schema-closeout/v1",
            apply: crate::backfill_search_schema_closeout,
        }),
    },
    Migration {
        version: atlas_local_state::MANUAL_TRACKER_ORDER_SCHEMA_VERSION,
        name: "manual-tracker-order",
        sql: atlas_local_state::MANUAL_TRACKER_ORDER_SCHEMA_SQL,
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

pub fn reference_repair(
    owner: &atlas_storage::UpdateOwner,
) -> Result<atlas_storage::ReferenceRepair<'_>, IndexError> {
    let (repair, ()) =
        owner.begin_reference_repair(TEST_MIGRATIONS, |_| Ok::<_, IndexError>(()))?;
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

fn publish_input_in_with_references(
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
    for value in collect_fixture_native_values(input, false)? {
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
    for value in collect_fixture_native_values(input, false)? {
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

fn publish_schema20_catalog(
    connection: &rusqlite::Connection,
    context: &mut crate::SourceArtifactBuildContext,
) -> Result<(), IndexError> {
    use crate::historical_similarity_admission::{
        SCHEMA20_QUERY_CATALOG_VERSION, schema20_catalog,
    };
    context.query_catalog_version = SCHEMA20_QUERY_CATALOG_VERSION;
    connection.execute("DELETE FROM query_field_catalog", [])?;
    for (field, definition) in schema20_catalog()? {
        connection.execute(
            "INSERT INTO query_field_catalog(field,definition_json) VALUES(?,?)",
            rusqlite::params![field, definition],
        )?;
    }
    Ok(())
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
        let ids = crate::historical_publication::records(connection, input)?;
        crate::historical_publication::search_units(connection, input, &ids)?;
        crate::traits::write_historical_traits(connection, input)?;
        if version == 20 {
            crate::similarity_projection::write_similarity_parts(connection, input)?;
        }
        let mut context = input.context.clone();
        if version == 19 {
            publish_schema20_catalog(connection, &mut context)?;
        }
        context.core_sections_recipe.clear();
        context.selected_sections_recipe.clear();
        context.remaster_inputs_identity.clear();
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

/// Publish the real schema-21 through schema-26 reference shape before sections.
pub fn publish_pre_section_fixture(
    maintenance: &atlas_storage::MaintenanceSession<'_>,
    input: &IndexBuildInput,
) -> Result<(), IndexError> {
    maintenance.publish(|connection| {
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if !(21..=26).contains(&version) {
            return Err(IndexError::Invalid(
                "pre-section fixtures require an actual schema-21 through schema-26 workspace"
                    .into(),
            ));
        }
        connection.execute_batch("PRAGMA defer_foreign_keys=ON")?;
        let ids = crate::historical_publication::records(connection, input)?;
        crate::historical_publication::search_units(connection, input, &ids)?;
        crate::traits::write_historical_traits(connection, input)?;
        crate::similarity_projection::write_similarity_parts(connection, input)?;
        crate::historical_core_projection::write_core_sections(connection, input)?;
        let mut context = input.context.clone();
        context.selected_sections_recipe.clear();
        context.remaster_inputs_identity.clear();
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

/// Stage the schema-32 catalog for a fixture with no remaster declarations.
pub fn stage_pre_glossary_catalog(
    workspace: &Workspace,
    remaster_inputs_identity: &str,
) -> Result<(), IndexError> {
    let write = workspace.write(32)?;
    let c = write.connection();
    let mut context = crate::read_stored_reference_context(c)?
        .ok_or_else(|| IndexError::Invalid("fixture has no reference context".into()))?;
    context.remaster_inputs_identity = remaster_inputs_identity.into();
    context.query_catalog_version = 9;
    c.execute(
        "UPDATE artifact_context SET context_hash=?,context_json=?",
        rusqlite::params![context.sha256()?, serde_json::to_string(&context)?],
    )?;
    c.execute_batch(
        "DELETE FROM query_field_catalog WHERE field IN
        ('actions','senses[].acuity','senses[].range','armor.group','embedded.equipment[].armor.group');",
    )?;
    write.commit()?;
    Ok(())
}

/// Seed only the record component of a deliberately historical schema fixture.
/// This does not produce a current readable artifact: ordinary publication must
/// use `publish_input` after the complete migration ledger is applied.
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
        let ids=crate::historical_publication::records(connection,input)?;
        crate::historical_publication::search_units(connection,input,&ids)?;
        // Modern projection columns are fixture scaffolding, not part of the
        // historical schema exercised by the embedding conversion/rollback.
        for (table, historical) in tables.iter().zip(historical_columns) {
            for column in columns(table)?.into_iter().rev().filter(|column| !historical.contains(column)) {
                connection.execute_batch(&format!("ALTER TABLE {table} DROP COLUMN {column}"))?;
            }
        }
        let mut context=input.context.clone();context.selected_sections_recipe.clear();
        context.remaster_inputs_identity.clear();context.core_sections_recipe.clear();
        publish_schema20_catalog(connection, &mut context)?;
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

/// Exact schema-2/9 checkpoint fixture; not a runtime compatibility path.
pub fn stage_historical_embedding_fixture(
    session: &atlas_storage::WriteSession,
    job: &str,
    reuse_key: &str,
    model: &crate::SourceSearchProfileIdentity,
    input: &atlas_embedding::PreparedNativeInput,
    vector: &[f32],
) -> Result<(), IndexError> {
    let bytes = crate::read_units::vector_bytes(vector);
    session.connection().execute("INSERT INTO embedding_checkpoints VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(reuse_key) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",rusqlite::params![reuse_key,serde_json::to_string(model)?,input.input_sha256,input.token_count,model.dimensions,bytes,crate::codec::sha256(&bytes),job])?;
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

/// Deliberately populate a historical/unknown-membership checkpoint. This fixture
/// path is absent from product builds; current execution must register and stage.
pub fn stage_historical_pending_native_fixture(
    session: &atlas_storage::WriteSession,
    job: &str,
    model: &crate::SourceSearchProfileIdentity,
    input: &atlas_embedding::NativeInputDescriptor,
    vector: &[f32],
) -> Result<(), IndexError> {
    crate::validation::check_vector(vector, model.dimensions)?;
    let id = crate::embedding_values::input_id(
        session.connection(),
        model,
        &input.input_sha256,
        input.token_count,
    )?;
    let bytes = crate::read_units::vector_bytes(vector);
    session.connection().execute("INSERT INTO pending_embedding_values VALUES(?,?,?,?) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,touched_job=excluded.touched_job",rusqlite::params![id,bytes,crate::embedding_values::hash_bytes(&bytes),job])?;
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
    collect_fixture_native_values(input, true)
}

fn collect_fixture_native_values<'a>(
    input: &'a IndexBuildInput,
    historical_traits: bool,
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
    if historical_traits {
        for seed in &input.trait_vectors {
            for chunk in &seed.chunks {
                accept(
                    &seed.profile_id,
                    &chunk.input_hash,
                    chunk.token_count,
                    &chunk.vector,
                )?;
            }
        }
        for seed in &input.trait_vectors {
            let definition = input
                .traits
                .iter()
                .find(|definition| definition.key == seed.key)
                .ok_or_else(|| IndexError::Invalid("trait seed has no definition".into()))?;
            crate::traits::validate_seed(
                definition,
                input.context.search_profile(&seed.profile_id)?,
                &seed.input_hash,
                &seed.vector,
                &seed.chunks,
            )?;
        }
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
