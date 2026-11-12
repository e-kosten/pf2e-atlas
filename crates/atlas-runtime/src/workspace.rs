//! Composition of domain-owned schemas into one ordered workspace history.
use atlas_index::migrate_trait_membership_digests;
#[cfg(test)]
use atlas_storage::{Migration, MigrationHistory};
use atlas_storage::{StorageResult, Workspace};
use std::path::Path;

pub const WORKSPACE_SCHEMA_VERSION: u32 = RELEASE_HISTORY.target();

#[path = "release_history.rs"]
mod release_history;
pub(crate) use release_history::RELEASE_HISTORY;

/// Initialize or upgrade one workspace. Reference readiness is independent:
/// an empty reference corpus must not prevent creating saved lists or encounters.
pub fn initialize_workspace(path: impl AsRef<Path>) -> StorageResult<Workspace> {
    register_extensions()?;
    let workspace = Workspace::at(path)?;
    workspace.migrate_history_with_capture(
        RELEASE_HISTORY,
        |snapshot| crate::workspace_archive::capture_upgrade_archive(&workspace, snapshot),
        |_| Ok(()),
    )?;
    Ok(workspace)
}

pub(crate) fn check_workspace_history(workspace: &Workspace) -> StorageResult<()> {
    let version = workspace.check_migration_history(RELEASE_HISTORY)?;
    if version != WORKSPACE_SCHEMA_VERSION {
        return Err(atlas_storage::StorageError::Schema {
            expected: WORKSPACE_SCHEMA_VERSION,
            actual: version,
        });
    }
    Ok(())
}

pub(crate) fn begin_reference_repair(
    owner: &atlas_storage::UpdateOwner,
) -> Result<
    (
        atlas_storage::ReferenceRepair<'_>,
        atlas_index::RequestedReferenceConfig,
    ),
    String,
> {
    let (repair, configuration) = owner
        .begin_reference_repair(
            RELEASE_HISTORY,
            atlas_index::read_requested_reference_config,
        )
        .map_err(|e| e.to_string())?;
    let configuration = configuration
        .ok_or_else(|| "workspace has no saved reference build configuration".to_string())?;
    Ok((repair, configuration))
}

pub(crate) fn open_workspace_index(
    path: &Path,
) -> Result<atlas_index::SqliteIndexReader, atlas_index::IndexError> {
    register_extensions().map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    let workspace =
        Workspace::at(path).map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    let session = workspace
        .read(WORKSPACE_SCHEMA_VERSION)
        .map_err(|e| atlas_index::IndexError::Unavailable(e.to_string()))?;
    atlas_index::SqliteIndexReader::from_session(session)
}

pub(crate) fn register_extensions() -> StorageResult<()> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()
        .map_err(|e| atlas_storage::StorageError::Unsupported(e.to_string()))
}

#[cfg(test)]
#[path = "turn_receipts_tests.rs"]
mod turn_receipts_tests;

#[cfg(test)]
#[path = "saved_state_integrity_tests.rs"]
mod saved_state_integrity_tests;

#[cfg(test)]
#[path = "release_migration_tests.rs"]
mod release_migration_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_local_state::{LocalStateRead, LocalStateWrite, NewSavedList};

    #[test]
    fn runtime_automatically_upgrades_core_without_source_models_or_changed_reference_values() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("core-runtime.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        let input = atlas_search::test_support::input_with_both_profiles(vec![
            atlas_search::test_support::record(
                "actors",
                "Actor",
                serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Core creature","type":"npc","system":{"details":{"publicNotes":"<h2>Description</h2><p>A cold guardian.</p>","blurb":"Guardian"}}}),
            ),
        ]);
        atlas_index::test_support::publish_input_in(&workspace, WORKSPACE_SCHEMA_VERSION, &input)
            .unwrap();
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        LocalStateWrite::for_local(&write)
            .unwrap()
            .saved_lists()
            .create(NewSavedList {
                slug: "kept".into(),
                name: "Kept".into(),
                description: Some("Preserved source note".into()),
                tags: vec![],
            })
            .unwrap();
        let revision = write.reference_revision().unwrap();
        let before =
            atlas_index::test_support::reference_preservation_rows(write.connection()).unwrap();
        let ledger = crate::test_support::sql_rows(
            write.connection(),
            "SELECT version,name,checksum FROM schema_migrations ORDER BY version",
        );
        // Isolate the Core-only metadata contract from remaster preparation.
        let mut context = input.context.clone();
        context.core_sections_recipe.clear();
        write
            .connection()
            .execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                rusqlite::params![
                    serde_json::to_string(&context).unwrap(),
                    context.sha256().unwrap()
                ],
            )
            .unwrap();
        write
            .connection()
            .execute("DELETE FROM core_roots", [])
            .unwrap();
        assert_eq!(
            atlas_index::classify_reference_upgrade(write.connection()).unwrap(),
            atlas_index::ReferenceUpgrade::CoreProjections
        );
        write.commit().unwrap();
        let source = directory.path().join("absent-source");
        let models = directory.path().join("absent-models");
        let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
            path_mode: crate::AtlasPathMode::Global,
            overrides: crate::AtlasPathOverrides {
                source_root: Some(source.clone()),
                embedding_cache_root: Some(models.clone()),
                index_path: Some(path.clone()),
            },
            ..Default::default()
        })
        .unwrap();
        assert!(runtime.upgrade_workspace_if_present().unwrap());
        assert!(!source.exists());
        assert!(!models.exists());
        let reader = runtime.open_index().unwrap();
        assert_eq!(
            reader.workspace_session().reference_revision().unwrap(),
            revision + 1
        );
        let after = atlas_index::test_support::reference_preservation_rows(
            reader.workspace_session().connection(),
        )
        .unwrap();
        assert!(
            after == before,
            "unexpected changed logical tables: {:?}",
            before
                .keys()
                .chain(after.keys())
                .filter(|key| before.get(*key) != after.get(*key))
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert_eq!(
            crate::test_support::sql_rows(
                reader.workspace_session().connection(),
                "SELECT version,name,checksum FROM schema_migrations ORDER BY version",
            ),
            ledger
        );
        assert_eq!(
            reader
                .core_selection(
                    input.records[0].record.key(),
                    Some(&atlas_domain::SearchProfileId::new("bge-base"))
                )
                .unwrap()
                .representation,
            atlas_domain::CoreSectionRepresentation::IdentityAndProse
        );
        drop(reader);
        assert!(!runtime.upgrade_workspace_if_present().unwrap());
    }

    #[test]
    fn startup_can_initialize_current_workspace_during_publisher_preparation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("atlas.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        let _publisher = workspace.update_owner().unwrap();
        let reopened = initialize_workspace(&path).unwrap();
        assert_eq!(
            reopened
                .read(WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            0
        );
    }

    #[test]
    fn invalid_new_domain_migration_rolls_back_entire_fresh_baseline() {
        let directory = tempfile::tempdir().unwrap();
        let workspace = Workspace::at(directory.path().join("atlas.sqlite")).unwrap();
        let mut migrations = RELEASE_HISTORY.migrations.to_vec();
        migrations.push(Migration {
            backfill: None,
            version: WORKSPACE_SCHEMA_VERSION + 1,
            name: "invalid",
            sql: "INSERT INTO missing_table VALUES(1);",
        });
        assert!(
            workspace
                .migrate_history(
                    MigrationHistory {
                        migrations: &migrations,
                        ..RELEASE_HISTORY
                    },
                    |_| Ok(())
                )
                .is_err()
        );
        let connection = rusqlite::Connection::open(workspace.path()).unwrap();
        assert_eq!(connection.query_row("SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |row| row.get::<_, u32>(0)).unwrap(), 0);
        assert_eq!(
            connection
                .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
                .unwrap(),
            0
        );
        drop(connection);
        // A failed fresh initialization leaves no partially usable reference-only DB.
        let recovered = initialize_workspace(workspace.path()).unwrap();
        let read = recovered.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            LocalStateRead::for_local(&read)
                .unwrap()
                .saved_lists()
                .list()
                .unwrap()
                .is_empty()
        );
    }
}
