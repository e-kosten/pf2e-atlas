use crate::test_support::{open_reader, publish_input};
use crate::*;
use atlas_embedding::{EmbeddingRuntimeConfig, required_embedding_model_cache_files};

use serde_json::json;
use std::fs;
fn paths(dir: &std::path::Path) -> ResolvedAtlasPaths {
    ResolvedAtlasPaths {
        managed_source_root: dir.join("managed-source"),
        source_root_explicit: true,
        mode: ResolvedPathMode::Global,
        repo_root: None,
        source_root: dir.join("source"),
        embedding_cache_root: dir.join("models"),
        index_path: dir.join("atlas.sqlite"),
    }
}

#[test]
fn startup_backfills_filter_catalog_without_source_or_inference() {
    assert_projection_backfill(true, false, crate::WORKSPACE_SCHEMA_VERSION);
}

#[test]
fn startup_backfills_lexical_rows_without_source_or_inference() {
    assert_projection_backfill(false, true, crate::WORKSPACE_SCHEMA_VERSION);
}

#[test]
fn startup_backfills_query_and_lexical_rows_together() {
    assert_projection_backfill(true, true, crate::WORKSPACE_SCHEMA_VERSION);
}

#[test]
fn startup_migrates_populated_schema_two_and_preserves_pending_trait_preparation() {
    assert_projection_backfill(false, true, 2);
}

fn assert_projection_backfill(query_changed: bool, lexical_changed: bool, initial_schema: u32) {
    use atlas_local_state::*;
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    let record = atlas_search::test_support::record(
        "actors",
        "Actor",
        json!({
            "_id":"testCreature1000","type":"npc","name":"Test Creature",
            "system":{"details":{"publicNotes":"<h2>Frozen Grave</h2><p>A patient guardian.</p>"},"abilities":{"str":{"mod":0}},"skills":{"athletics":{"base":25}}},
            "items":[{"_id":"dddddddddddddddd","name":"Flame Lance","type":"spell",
                "system":{"traits":{"value":["fire"]}}}]
        }),
    );
    let input = atlas_search::test_support::input(vec![record], true);
    if initial_schema == 2 {
        crate::workspace::register_extensions().unwrap();
        let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
        workspace
            .migrate(
                &[
                    atlas_storage::Migration {
                        backfill: None,
                        version: 1,
                        name: "reference-baseline",
                        sql: atlas_index::REFERENCE_SCHEMA_SQL,
                    },
                    atlas_storage::Migration {
                        backfill: None,
                        version: 2,
                        name: "saved-state-baseline",
                        sql: atlas_local_state::LOCAL_STATE_SCHEMA_SQL,
                    },
                ],
                |_| Ok(()),
            )
            .unwrap();
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner.maintenance(2, "old workspace fixture").unwrap();
        atlas_index::test_support::publish_historical_record_fixture(&maintenance, &input).unwrap();
        assert!(
            !maintenance
                .connection()
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='trait_definitions')",
                    [],
                    |r| r.get::<_, bool>(0)
                )
                .unwrap()
        );
    } else {
        publish_input(&paths.index_path, &input).unwrap();
    }
    let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
    let write = workspace.write(initial_schema).unwrap();
    let state = LocalStateWrite::new(&write);
    let list = state
        .saved_lists()
        .create(NewSavedList {
            slug: "kept-list".into(),
            name: "Kept list".into(),
            description: Some("Personal notes".into()),
            tags: vec!["campaign".into()],
        })
        .unwrap();
    let key = input.records[0].record.key().clone();
    state
        .saved_lists()
        .add_resolved_item(
            &list.list_key,
            ResolvedSavedListItem {
                record_key: key.clone(),
                title_snapshot: "Test Creature".into(),
                kind_snapshot: Some("creature".into()),
                note: Some("Use next session".into()),
            },
        )
        .unwrap();
    let encounter = Encounter {
        encounter_key: "historical-encounter".into(),
        slug: "kept-encounter".into(),
        name: "Kept encounter".into(),
        description: None,
        note: Some("In progress".into()),
        status: EncounterStatus::Draft,
        round_number: 1,
        current_turn_participant_key: None,
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
        revision: 0,
        difficulty_reference: Default::default(),
    };
    // Seed historical rows directly: current domain readers require newer columns.
    write
        .connection()
        .execute(
            "INSERT INTO encounters(encounter_key,slug,name,note,status,created_at,updated_at)
         VALUES (?1,?2,?3,?4,'draft',?5,?6)",
            rusqlite::params![
                encounter.encounter_key,
                encounter.slug,
                encounter.name,
                encounter.note,
                encounter.created_at,
                encounter.updated_at
            ],
        )
        .unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO encounter_participants (
            encounter_id,participant_key,record_key,participant_kind,participant_variant,
            position,display_name,record_title_snapshot,record_kind_snapshot,side,
            initiative,initiative_order,max_hp,current_hp,temporary_hp,note,
            hp_origin,variant_origin,created_at,updated_at)
         SELECT id,'kept-participant',?2,'creature','elite',0,'Wounded creature',
            'Test Creature','creature','enemy',21,0,40,13,5,'Do not reset HP',
            'explicit','explicit','before','before'
         FROM encounters WHERE encounter_key=?1",
            rusqlite::params![encounter.encounter_key, key.to_string()],
        )
        .unwrap();
    let participant_rows =
        |connection: &rusqlite::Connection| {
            let mut statement = connection.prepare(
            "SELECT participant_key,record_key,participant_kind,participant_variant,position,
             display_name,record_title_snapshot,record_kind_snapshot,side,initiative,
             initiative_order,max_hp,current_hp,temporary_hp,defeated,hidden,note,
             hp_origin,variant_origin,created_at,updated_at,revision
             FROM encounter_participants ORDER BY participant_key"
        ).unwrap();
            let column_count = statement.column_count();
            statement
                .query_map([], |row| {
                    (0..column_count)
                        .map(|column| row.get::<_, rusqlite::types::Value>(column))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
    let saved_list = state
        .saved_lists()
        .get_with_items(&list.list_key)
        .unwrap()
        .unwrap();
    let saved_participants = participant_rows(write.connection());
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let checkpoint = atlas_embedding::PreparedEmbeddingInput {
        address: unit.address.clone(),
        input: String::new(),
        input_sha256: unit.input_hash.clone(),
        reuse_key: "backfill-preservation".into(),
        token_count: unit.input_token_count,
        body_token_count: unit.input_token_count,
        overlap_token_count: 0,
        context_shortened: false,
    };
    let job = atlas_index::start_reference_job(&write, "next-preparation").unwrap();
    if initial_schema == 2 {
        atlas_index::test_support::stage_historical_embedding_fixture(
            &write,
            &job,
            model,
            &checkpoint,
            &unit.vector,
        )
        .unwrap();
    } else {
        atlas_index::stage_embedding_values(&write, &job, model, &[(&checkpoint, &unit.vector)])
            .unwrap();
    }
    // Inspect stored vector bytes, not a retrieval score that could hide replacement.
    let vector_table: String = write.connection().query_row(
        "SELECT name FROM sqlite_schema WHERE type='table' AND sql LIKE 'CREATE VIRTUAL TABLE%USING vec0%'", [], |r| r.get(0),
    ).unwrap();
    let stored_vectors = |connection: &rusqlite::Connection| {
        connection
            .prepare(&format!(
                "SELECT unit_id,embedding FROM {vector_table} ORDER BY unit_id"
            ))
            .unwrap()
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    };
    let vectors_before = stored_vectors(write.connection());
    assert!(!vectors_before.is_empty());
    let before = write.reference_revision().unwrap();
    if query_changed {
        write
            .connection()
            .execute("DELETE FROM query_field_catalog WHERE field='kind'", [])
            .unwrap();
    }
    if lexical_changed {
        write
            .connection()
            .execute(
                "UPDATE lexical_units SET identity_terms='obsolete inherited name',input_hash=printf('%064x',unit_id)",
                [],
            )
            .unwrap();
        write.connection().execute(
            "INSERT INTO verified_aliases(record_id,alias,alias_lookup_key,evidence_json) SELECT record_id,'Snow Keeper','snow keeper','\"source redirect\"' FROM records",
            [],
        ).unwrap();
        write
            .connection()
            .execute(
                "UPDATE lexical_units SET alias_terms='Snow Keeper' WHERE unit_kind='root_name'",
                [],
            )
            .unwrap();
    }
    let json: String = write
        .connection()
        .query_row("SELECT context_json FROM artifact_context", [], |row| {
            row.get(0)
        })
        .unwrap();
    let mut context: atlas_index::SourceArtifactBuildContext = serde_json::from_str(&json).unwrap();
    if query_changed {
        context.query_catalog_version = 3;
    }
    if lexical_changed {
        context.lexical_selection_version = "curated-lexical/v1".into();
    }
    let mut historical_context = serde_json::to_value(&context).unwrap();
    if initial_schema == 2 {
        // Genuine pre-trait workspaces have no vocabulary preparation contract.
        historical_context
            .as_object_mut()
            .unwrap()
            .remove("trait_policy");
    }
    let json = serde_json::to_string(&historical_context).unwrap();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?1,context_json=?2",
            [atlas_search::test_support::hash(json.as_bytes()), json],
        )
        .unwrap();

    write.commit().unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            source_root: Some(paths.source_root.clone()),
            embedding_cache_root: Some(paths.embedding_cache_root.clone()),
            index_path: Some(paths.index_path.clone()),
        },
        ..Default::default()
    })
    .unwrap();
    let historical = initial_schema == 2;
    assert_eq!(runtime.upgrade_workspace_if_present().unwrap(), !historical);
    let expected_revision = before + u64::from(!historical);
    if historical {
        assert!(runtime.open_index().is_err());
        assert!(runtime.read_workspace(None).is_err());
    } else {
        let read = runtime.read_workspace(None).unwrap();
        assert_eq!(read.reference_revision().unwrap(), expected_revision);
    }
    assert!(!runtime.upgrade_workspace_if_present().unwrap());
    {
        let read = workspace.read(crate::WORKSPACE_SCHEMA_VERSION).unwrap();
        let state = LocalStateRead::new(&read);
        assert_eq!(
            state
                .saved_lists()
                .get_with_items(&list.list_key)
                .unwrap()
                .unwrap(),
            saved_list
        );
        assert_eq!(participant_rows(read.connection()), saved_participants);
        let saved_encounter = state
            .encounters()
            .get_with_participants(&encounter.encounter_key)
            .unwrap()
            .unwrap();
        assert_eq!(saved_encounter.encounter, encounter);
        assert!(!saved_encounter.participants[0].hazard_disabled);
        assert_eq!(stored_vectors(read.connection()), vectors_before);
        assert_eq!(read.reference_revision().unwrap(), expected_revision);
        if historical {
            assert_eq!(
                atlas_index::classify_reference_upgrade(read.connection()).unwrap(),
                atlas_index::ReferenceUpgrade::SourcePreparation
            );
            let context = atlas_index::read_stored_reference_context(read.connection())
                .unwrap()
                .unwrap();
            assert!(context.trait_policy.is_empty());
            assert_eq!(context.lexical_selection_version, "curated-lexical/v1");
            assert_eq!(
                read.connection()
                    .query_row("SELECT count(*) FROM trait_definitions", [], |r| r
                        .get::<_, usize>(0))
                    .unwrap(),
                0
            );
            assert_eq!(
                read.connection()
                    .query_row("SELECT max(version) FROM schema_migrations", [], |r| r
                        .get::<_, u32>(0))
                    .unwrap(),
                crate::WORKSPACE_SCHEMA_VERSION
            );
        } else {
            for phrase in ["Test Creature", "Flame Lance", "Frozen Grave"]
                .into_iter()
                .chain(lexical_changed.then_some("Snow Keeper"))
            {
                let matches: i64 = read
                    .connection()
                    .query_row(
                        "SELECT count(*) FROM lexical_fts WHERE lexical_fts MATCH ?",
                        [phrase],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert!(matches > 0, "missing migrated FTS match: {phrase}");
            }
            let reader = runtime.open_index().unwrap();
            let query =
                atlas_index::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
            for phrase in ["Test Creature", "Frozen Grave"]
                .into_iter()
                .chain(lexical_changed.then_some("Snow Keeper"))
            {
                let matches = reader
                    .lexical_candidates(
                        phrase,
                        &query,
                        None,
                        atlas_domain::SearchContentScope::Core,
                        100,
                    )
                    .unwrap();
                assert!(!matches.is_empty(), "missing rebuilt Core match: {phrase}");
                assert!(matches.iter().all(|hit| hit.location.record == key));
            }
            assert!(
                reader
                    .lexical_candidates(
                        "Flame Lance",
                        &query,
                        None,
                        atlas_domain::SearchContentScope::Core,
                        100,
                    )
                    .unwrap()
                    .is_empty(),
                "a creature's carried spell is not Core lexical content"
            );
            if lexical_changed {
                let aliases: i64 = read.connection().query_row(
                "SELECT count(*) FROM lexical_units WHERE unit_kind='root_alias' AND alias_terms='Snow Keeper'",
                [], |row| row.get(0),
            ).unwrap();
                assert_eq!(aliases, 1);
            }
        }
        let checkpoints =
            atlas_index::read_reusable_embeddings(&read, model, std::slice::from_ref(&checkpoint))
                .unwrap();
        assert_eq!(checkpoints[&checkpoint.reuse_key], unit.vector);
    }
    assert!(!paths.source_root.exists());
    {
        let read = workspace.read(crate::WORKSPACE_SCHEMA_VERSION).unwrap();
        if historical {
            // Schema upgrade cannot make old content preparation queryable.
            let states: (String, String) = read
                .connection()
                .query_row(
                    "SELECT ability_str_state,skill_athletics_modifier_state FROM actor_projection",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(states, ("missing".into(), "missing".into()));
        } else {
            let facts: (i64, i64) = read
                .connection()
                .query_row(
                    "SELECT ability_str,skill_athletics_modifier FROM actor_projection",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(facts, (0, 25));
        }
    }
    assert!(!paths.embedding_cache_root.exists());
    assert!(runtime.inference.loaded_profiles().is_empty());
    if !historical {
        // A current-version catalog with missing rows is corruption, not an upgrade.
        let write = workspace.write(crate::WORKSPACE_SCHEMA_VERSION).unwrap();
        write
            .connection()
            .execute("DELETE FROM query_field_catalog WHERE field='kind'", [])
            .unwrap();
        write.commit().unwrap();
        assert!(runtime.upgrade_workspace_if_present().is_err());
        assert_eq!(
            workspace
                .read(crate::WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            before + 1
        );
    }
}
fn source(paths: &ResolvedAtlasPaths) {
    fs::create_dir_all(paths.source_root.join("packs/items")).unwrap();
    fs::create_dir_all(paths.source_root.join("static/lang")).unwrap();
    fs::write(paths.source_root.join("static/lang/en.json"), "{}").unwrap();
    fs::write(
        paths.source_root.join("module.json"),
        r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"}]}"#,
    )
    .unwrap();
    fs::write(paths.source_root.join("packs/items/one.json"),r#"{"_id":"aaaaaaaaaaaaaaaa","name":"One","type":"effect","system":{"level":{"value":1},"description":{"value":"<h2>First definition</h2><p>Some prose.</p>"}}}"#).unwrap();
}
fn options(check: bool) -> RuntimeSetupOptions {
    RuntimeSetupOptions {
        target: SetupTarget::Records,
        check,
        offline: true,
        ..Default::default()
    }
}

#[test]
fn setup_reprepares_known_old_content_policy_but_readers_remain_blocked_until_publication() {
    for previous in [
        "foundry-content/v6",
        "foundry-content/v10",
        "foundry-content/v11",
        "foundry-content/v12",
        "foundry-content/v15",
        "foundry-content/v16",
        "foundry-content/v17",
        "foundry-content/v18",
        "foundry-content/v19",
        "foundry-content/v20",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let paths = paths(directory.path());
        source(&paths);
        assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
        let current_content_policy = open_reader(&paths.index_path)
            .unwrap()
            .context()
            .content_interpretation_version
            .clone();
        let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
        let before = workspace
            .read(crate::WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap();
        atlas_index::test_support::set_content_interpretation_version(
            &paths.index_path,
            crate::WORKSPACE_SCHEMA_VERSION,
            previous,
        )
        .unwrap();
        assert!(
            !atlas_index::ensure_reference_projections(&workspace, crate::WORKSPACE_SCHEMA_VERSION)
                .unwrap()
        );
        assert!(open_reader(&paths.index_path).is_err());
        let check = crate::setup::ensure_setup(&paths, options(true));
        assert!(!check.ready);
        assert!(open_reader(&paths.index_path).is_err());
        let rebuilt = crate::setup::ensure_setup(&paths, options(false));
        assert!(rebuilt.ready, "{:?}", rebuilt.actions);
        assert!(rebuilt.build.is_some());
        let reader = open_reader(&paths.index_path).unwrap();
        assert_eq!(
            reader.workspace_session().reference_revision().unwrap(),
            before + 1
        );
        assert_eq!(
            reader.context().content_interpretation_version,
            current_content_policy
        );
        assert!(!paths.embedding_cache_root.exists());
    }
}

#[test]
fn stale_build_configuration_is_rejected_before_reading_source() {
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    let original =
        crate::source_configuration::resolve_configuration(&paths, &options(false), None).unwrap();
    let newer = crate::source_configuration::resolve_configuration(
        &paths,
        &RuntimeSetupOptions {
            locale: Some("fr".into()),
            ..options(false)
        },
        None,
    )
    .unwrap();
    let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
    let error = atlas_ingest::build_artifact(
        &workspace,
        crate::WORKSPACE_SCHEMA_VERSION,
        atlas_ingest::BuildArtifactOptions {
            source_root: original.source_root,
            output_path: paths.index_path.clone(),
            manifest_path: original.manifest_path,
            locale: original.indexing_locale,
            search_profiles: vec![],
            reuse_embeddings: true,
            embedding_batch_size: 32,
        },
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("configuration changed before the build started"),
        "{error}"
    );
    let read = workspace.read(crate::WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        atlas_index::read_requested_reference_config(&read).unwrap(),
        Some(newer)
    );
    assert_eq!(read.reference_revision().unwrap(), 0);
    assert!(!paths.source_root.exists());
    assert!(!paths.embedding_cache_root.exists());
}
#[test]
fn explicit_setup_freshness_tracks_actual_bytes_and_preserves_locale() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    fs::write(paths.source_root.join("static/lang/fr.json"), "{}").unwrap();
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            locale: Some("fr".into()),
            ..options(false)
        },
    );
    assert!(report.ready, "{:?}", report.actions);
    assert!(report.build.is_some());
    assert_eq!(
        open_reader(&paths.index_path)
            .unwrap()
            .context()
            .indexing_locale,
        "fr"
    );
    let report = crate::setup::ensure_setup(&paths, options(true));
    assert!(report.ready);
    assert!(report.build.is_none());
    fs::write(
        paths.source_root.join("packs/items/one.json"),
        r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Two","type":"effect"}"#,
    )
    .unwrap();
    let report = crate::setup::ensure_setup(&paths, options(true));
    assert!(!report.ready);
    assert!(
        report.actions.iter().any(
            |a| a.kind == SetupActionKind::BuildIndex && a.status == SetupActionStatus::Planned
        )
    );
    let report = crate::setup::ensure_setup(&paths, options(false));
    assert!(report.ready);
    assert!(report.build.is_some());
    assert_eq!(
        open_reader(&paths.index_path)
            .unwrap()
            .context()
            .indexing_locale,
        "fr"
    );
    fs::write(
        paths.source_root.join("static/lang/en.json"),
        r#"{"changed":true}"#,
    )
    .unwrap();
    assert!(!crate::setup::ensure_setup(&paths, options(true)).ready);
}
#[test]
fn failed_rebuild_preserves_prior_artifact_and_product_reads_need_no_source() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    let (generation, context, record) = {
        let reader = open_reader(&paths.index_path).unwrap();
        (
            reader.artifact_instance().generation(),
            reader.context().clone(),
            reader
                .read_source_record(&"items:aaaaaaaaaaaaaaaa".parse().unwrap())
                .unwrap(),
        )
    };
    fs::write(paths.source_root.join("module.json"), "invalid").unwrap();
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            force_rebuild: true,
            ..options(false)
        },
    );
    assert_eq!(report.exit_code_class(), SetupExitClass::RuntimeFailure);
    {
        let reader = open_reader(&paths.index_path).unwrap();
        assert_eq!(reader.artifact_instance().generation(), generation);
        assert_eq!(reader.context(), &context);
        assert_eq!(
            reader
                .read_source_record(&"items:aaaaaaaaaaaaaaaa".parse().unwrap())
                .unwrap(),
            record
        );
    }
    fs::remove_dir_all(&paths.source_root).unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: None,
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides {
            source_root: Some(paths.source_root),
            embedding_cache_root: Some(paths.embedding_cache_root),
            index_path: Some(paths.index_path),
        },
    })
    .unwrap();
    let service = runtime.read_workspace(None).unwrap();
    let result = service
        .retrieval()
        .list_records(atlas_search::ListRecordsRequest::new(
            None,
            atlas_search::SearchPage::default(),
        ))
        .unwrap();
    assert_eq!(result.total, 1);
    assert!(runtime.open_search_index().is_err());
    assert!(runtime.prepare_query(None, "test query").is_err());
}
#[test]
fn all_required_model_assets_are_verified_without_inference_or_download() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            target: SetupTarget::Full,
            check: true,
            offline: true,
            ..Default::default()
        },
    );
    assert_eq!(report.search_profiles[0].missing_files.len(), 5);
    assert!(!report.ready);
    let config = EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        &paths.embedding_cache_root,
    );
    for asset in required_embedding_model_cache_files(&config) {
        fs::create_dir_all(asset.local_path.parent().unwrap()).unwrap();
        fs::write(asset.local_path, b"corrupted cached bytes").unwrap();
    }
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            target: SetupTarget::Full,
            check: true,
            offline: true,
            ..Default::default()
        },
    );
    assert!(report.search_profiles[0].missing_files.is_empty());
    assert!(
        !report.search_profiles[0].ready,
        "presence is insufficient without checksum verification"
    );
}
#[test]
fn stored_vector_runtime_needs_no_query_model_but_full_mode_does() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    let record = atlas_search::test_support::record(
        "items",
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"One","type":"effect"}),
    );
    publish_input(
        &paths.index_path,
        &atlas_search::test_support::input(vec![record], true),
    )
    .unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
        path_mode: AtlasPathMode::Global,
        overrides: AtlasPathOverrides {
            source_root: Some(paths.source_root),
            embedding_cache_root: Some(paths.embedding_cache_root),
            index_path: Some(paths.index_path),
        },
    })
    .unwrap();
    let validation = runtime.validate_index_report(SetupTarget::Full).unwrap();
    assert_eq!(validation.roots, 1);
    assert_eq!(validation.checked_snapshots, 1);
    assert_eq!(validation.semantic_units, 1);
    let statistics = runtime.check_index_report(SetupTarget::Records).unwrap();
    assert_eq!(statistics.records, 1);
    assert_eq!(statistics.product_records, 1);
    assert_eq!(statistics.semantic_units, 1);
    assert_eq!(
        runtime
            .prepare_query(None, "test query")
            .err()
            .expect("missing model assets must prevent query inference")
            .kind(),
        atlas_search::SearchErrorKind::EmbeddingUnavailable
    );
    let reader = runtime.open_index().unwrap();
    let metadata = runtime.search_profile_readiness(&reader.context().search_profiles);
    assert_eq!(metadata.len(), 1);
    assert!(metadata[0].stored_vectors);
    assert!(!metadata[0].assets_ready);
    assert!(!metadata[0].usable);
    assert!(runtime.inference.loaded_profiles().is_empty());
}
#[test]
fn incompatible_database_is_not_replaced_by_setup() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    fs::write(&paths.index_path, b"old incompatible artifact").unwrap();
    let report = crate::setup::ensure_setup(&paths, options(true));
    assert!(!report.ready);
    let report = crate::setup::ensure_setup(&paths, options(false));
    assert!(!report.ready);
    assert!(report.build.is_none());
    assert_eq!(
        fs::read(&paths.index_path).unwrap(),
        b"old incompatible artifact"
    );
}

#[test]
fn invalid_profile_selection_cannot_fetch_assets_or_replace_the_artifact() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    let original = fs::read(&paths.index_path).unwrap();
    for ids in [vec!["bge-base", "bge-base"], vec!["unknown"]] {
        let report = crate::setup::ensure_setup(
            &paths,
            RuntimeSetupOptions {
                target: SetupTarget::Full,
                search_profiles: ids
                    .into_iter()
                    .map(atlas_domain::SearchProfileId::new)
                    .collect(),
                check: false,
                offline: false,
                force_rebuild: true,
                ..Default::default()
            },
        );
        assert!(!report.ready);
        assert!(report.build.is_none());
        assert!(
            report
                .actions
                .iter()
                .chain(report.checks.iter())
                .any(|action| action.status == SetupActionStatus::Failed)
        );
        assert!(
            !report
                .actions
                .iter()
                .any(|action| action.kind == SetupActionKind::FetchSource)
        );
        assert!(!paths.embedding_cache_root.exists());
        assert_eq!(fs::read(&paths.index_path).unwrap(), original);
    }
}

#[test]
fn unsupported_runtime_default_fails_without_opening_assets_or_artifacts() {
    let result = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(atlas_domain::SearchProfileId::new("unknown")),
        ..Default::default()
    });
    let error = result.err().expect("unknown runtime default must fail");
    assert_eq!(
        error.kind(),
        &crate::RuntimeErrorKind::UnsupportedSearchProfile
    );
    assert!(error.to_string().contains("unknown"));
}

#[test]
fn omitted_default_resolves_stored_manifest_and_explicit_missing_still_errors() {
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions::default()).unwrap();
    let base = atlas_domain::SearchProfileId::new("bge-base");
    let gemma = atlas_domain::SearchProfileId::new("embeddinggemma");
    let mut input = atlas_search::test_support::input_with_both_profiles(vec![]);
    assert_eq!(
        runtime
            .resolve_default_search_profile(&input.context)
            .unwrap(),
        base
    );
    input
        .context
        .search_profiles
        .retain(|p| p.profile_id == gemma);
    assert_eq!(
        runtime
            .resolve_default_search_profile(&input.context)
            .unwrap(),
        gemma
    );
    let explicit = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: Some(base),
        ..Default::default()
    })
    .unwrap();
    assert!(
        explicit
            .resolve_default_search_profile(&input.context)
            .is_err()
    );
}

#[test]
fn omitted_setup_preserves_requested_single_or_both_profiles_before_asset_work() {
    for gemma_only in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let paths = paths(directory.path());
        let record = atlas_search::test_support::record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"One","type":"effect"}),
        );
        let mut input = atlas_search::test_support::input_with_both_profiles(vec![record]);
        if gemma_only {
            input
                .context
                .search_profiles
                .retain(|p| p.profile_id.as_str() == "embeddinggemma");
            input
                .semantic_units
                .retain(|p| p.profile_id.as_str() == "embeddinggemma");
        }
        publish_input(&paths.index_path, &input).unwrap();
        let expected = input
            .context
            .search_profiles
            .iter()
            .map(|p| p.profile_id.clone())
            .collect::<Vec<_>>();
        let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
        let write = workspace.write(crate::WORKSPACE_SCHEMA_VERSION).unwrap();
        atlas_index::write_requested_reference_config(
            &write,
            &atlas_index::RequestedReferenceConfig {
                source_root: paths.source_root.clone(),
                manifest_path: None,
                indexing_locale: "en".into(),
                search_profiles: expected.clone(),
            },
        )
        .unwrap();
        write.commit().unwrap();
        let report = crate::setup::ensure_setup(
            &paths,
            RuntimeSetupOptions {
                check: true,
                offline: true,
                ..Default::default()
            },
        );
        assert_eq!(
            report
                .search_profiles
                .iter()
                .map(|p| p.profile_id.clone())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(!paths.embedding_cache_root.exists());
        let reader = open_reader(&paths.index_path).unwrap();
        assert_eq!(
            reader.context().search_profiles,
            input.context.search_profiles
        );
    }
}

#[test]
#[ignore = "requires verified local Gemma assets; bounded query/session recovery"]
fn actual_gemma_only_implicit_default_retains_query_usability_after_disk_assets_disappear() {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|p| p.join(".cache/hf-models"))
        .find(|p| {
            EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, p)
                .model_dir()
                .join("onnx/model.onnx_data")
                .is_file()
        })
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    let isolated = EmbeddingRuntimeConfig::new(
        EmbeddingModelId::EmbeddingGemma300M,
        &paths.embedding_cache_root,
    );
    let source = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, cache);
    for file in required_embedding_model_cache_files(&isolated) {
        fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
        fs::hard_link(source.model_dir().join(file.source_path), file.local_path).unwrap();
    }
    let mut input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","name":"One","type":"effect"}),
        ),
    ]);
    input
        .context
        .search_profiles
        .retain(|p| p.profile_id.as_str() == "embeddinggemma");
    input
        .semantic_units
        .retain(|p| p.profile_id.as_str() == "embeddinggemma");
    publish_input(&paths.index_path, &input).unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: Some(paths.index_path),
            embedding_cache_root: Some(paths.embedding_cache_root.clone()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let prepared = runtime.prepare_query(None, "forest guardian").unwrap();
    let operation = runtime.read_workspace(Some(prepared)).unwrap();
    let retrieval = operation.retrieval();
    assert_eq!(
        retrieval.default_search_profile().as_str(),
        "embeddinggemma"
    );
    let ready = runtime.search_profile_readiness(&input.context.search_profiles);
    assert_eq!(ready.len(), 1);
    assert!(ready[0].assets_ready && ready[0].usable);
    fs::remove_dir_all(isolated.model_dir()).unwrap();
    let retained = runtime.search_profile_readiness(&input.context.search_profiles);
    assert!(!retained[0].assets_ready);
    assert!(
        retained[0].usable,
        "loaded verified session survives unlinking its private disk view"
    );
    assert_eq!(
        runtime
            .inference
            .embed_query(
                &atlas_domain::SearchProfileId::new("embeddinggemma"),
                "forest guardian"
            )
            .unwrap()
            .len(),
        768
    );
    assert!(
        !atlas_embedding::EmbeddingRuntimeConfig::new(
            EmbeddingModelId::BgeBaseEnV15,
            paths.embedding_cache_root
        )
        .model_dir()
        .exists()
    );
}

#[test]
fn fresh_omitted_setup_selects_base_without_installing_during_check() {
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            check: true,
            offline: true,
            ..Default::default()
        },
    );
    assert_eq!(report.search_profiles.len(), 1);
    assert_eq!(report.search_profiles[0].profile_id.as_str(), "bge-base");
    assert!(!paths.embedding_cache_root.exists());
}

#[test]
fn current_reference_startup_and_reading_continue_during_publisher_preparation() {
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    let record = atlas_search::test_support::record(
        "actors",
        "Actor",
        json!({"_id":"testCreature1000","type":"npc","name":"Test Creature"}),
    );
    publish_input(
        &paths.index_path,
        &atlas_search::test_support::input(vec![record], true),
    )
    .unwrap();
    let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
    let before = workspace
        .read(crate::WORKSPACE_SCHEMA_VERSION)
        .unwrap()
        .reference_revision()
        .unwrap();
    let _publisher = workspace.update_owner().unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            source_root: Some(paths.source_root),
            embedding_cache_root: Some(paths.embedding_cache_root),
            index_path: Some(paths.index_path),
        },
        ..Default::default()
    })
    .unwrap();
    assert!(!runtime.upgrade_workspace_if_present().unwrap());
    assert_eq!(
        runtime
            .read_workspace(None)
            .unwrap()
            .reference_revision()
            .unwrap(),
        before
    );
}

#[test]
fn incompatible_supplied_source_preserves_published_reference_and_saved_state() {
    use atlas_local_state::{LocalStateRead, LocalStateWrite, NewEncounter, NewSavedList};
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    source(&paths);
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    LocalStateWrite::new(&write)
        .saved_lists()
        .create(NewSavedList {
            slug: "keep".into(),
            name: "Keep".into(),
            description: None,
            tags: vec![],
        })
        .unwrap();
    LocalStateWrite::new(&write)
        .encounters()
        .create(NewEncounter {
            slug: "keep-encounter".into(),
            name: "Keep encounter".into(),
            description: None,
            note: None,
        })
        .unwrap();
    write.commit().unwrap();
    let before = workspace
        .read(WORKSPACE_SCHEMA_VERSION)
        .unwrap()
        .reference_revision()
        .unwrap();
    let manifest = paths.source_root.join("module.json");
    let mismatched = r#"{"id":"pf2e","version":"7.9.1","packs":[]}"#;
    fs::write(&manifest, mismatched).unwrap();
    for check in [true, false] {
        let mut opts = options(check);
        opts.offline = false;
        let report = crate::setup::ensure_setup(&paths, opts);
        assert!(!report.ready);
        assert!(report.readiness.source.reason.unwrap().contains("7.9.1"));
        assert!(
            !report
                .actions
                .iter()
                .any(|a| a.kind == SetupActionKind::FetchSource)
        );
        assert_eq!(fs::read_to_string(&manifest).unwrap(), mismatched);
    }
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), before);
    assert_eq!(
        LocalStateRead::new(&read)
            .saved_lists()
            .list()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        LocalStateRead::new(&read)
            .encounters()
            .list()
            .unwrap()
            .len(),
        1
    );
    assert!(crate::workspace::open_workspace_index(&paths.index_path).is_ok());
}

#[test]
fn missing_managed_source_in_check_or_offline_mode_is_never_created() {
    for (check, offline) in [(true, false), (false, true)] {
        let temp = tempfile::tempdir().unwrap();
        let mut paths = paths(temp.path());
        paths.source_root_explicit = false;
        paths.managed_source_root = paths.source_root.clone();
        let mut opts = options(check);
        opts.offline = offline;
        let report = crate::setup::ensure_setup(&paths, opts);
        assert!(!report.ready);
        assert!(!paths.source_root.exists());
        assert!(
            report
                .actions
                .iter()
                .any(|a| a.kind == SetupActionKind::FetchSource
                    && a.status
                        == if offline {
                            SetupActionStatus::Blocked
                        } else {
                            SetupActionStatus::Planned
                        })
        );
    }
}

#[test]
fn setup_reprepares_prerequisites_from_old_selection_policy() {
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    source(&paths);
    fs::write(paths.source_root.join("packs/items/one.json"), r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Feat","type":"feat","system":{"prerequisites":{"value":[{"value":"<strong>expert</strong> in Medicine"}]}}}"#).unwrap();
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    atlas_index::test_support::set_content_selection_version(
        &paths.index_path,
        crate::WORKSPACE_SCHEMA_VERSION,
        "structural-source-sections/v2",
    )
    .unwrap();
    assert!(open_reader(&paths.index_path).is_err());
    assert!(!crate::setup::ensure_setup(&paths, options(true)).ready);
    let rebuilt = crate::setup::ensure_setup(&paths, options(false));
    assert!(rebuilt.ready, "{:?}", rebuilt.actions);
    assert!(rebuilt.build.is_some());
    let reader = open_reader(&paths.index_path).unwrap();
    assert_eq!(
        reader.context().content_selection_version,
        "structural-source-sections/v4"
    );
    let report = atlas_index::validate_reference(reader.workspace_session().connection()).unwrap();
    assert_eq!(report.prepared_fields, 1);
}

#[test]
fn explicit_setup_replaces_prior_model_and_preserves_saved_state() {
    use atlas_local_state::{LocalStateRead, LocalStateWrite, NewEncounter, NewSavedList};
    let directory = tempfile::tempdir().unwrap();
    let paths = paths(directory.path());
    source(&paths);
    assert!(crate::setup::ensure_setup(&paths, options(false)).ready);
    let workspace = atlas_storage::Workspace::at(&paths.index_path).unwrap();
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    let state = LocalStateWrite::new(&write);
    let list = state
        .saved_lists()
        .create(NewSavedList {
            slug: "keep-upgrade".into(),
            name: "Keep through model upgrade".into(),
            description: Some("Personal notes".into()),
            tags: vec!["campaign".into()],
        })
        .unwrap();
    let encounter = state
        .encounters()
        .create(NewEncounter {
            slug: "keep-upgrade".into(),
            name: "Active encounter".into(),
            description: None,
            note: Some("Do not reset".into()),
        })
        .unwrap();
    let mut context = atlas_index::read_stored_reference_context(write.connection())
        .unwrap()
        .unwrap();
    context.source_contract =
        "6bf64da835272af22c53db729cb28d18a87e8dee999c11b154fb3e772befdeb8".into();
    let json = serde_json::to_string(&context).unwrap();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?1,context_json=?2",
            [atlas_search::test_support::hash(json.as_bytes()), json],
        )
        .unwrap();
    let before = write.reference_revision().unwrap();
    write.commit().unwrap();
    assert!(crate::workspace::open_workspace_index(&paths.index_path).is_err());
    assert!(!crate::setup::ensure_setup(&paths, options(true)).ready);
    // An unsuccessful preparation retains the old publication and saved domains.
    fs::write(
        paths.source_root.join("packs/items/one.json"),
        "invalid JSON",
    )
    .unwrap();
    assert!(!crate::setup::ensure_setup(&paths, options(false)).ready);
    assert_eq!(
        workspace
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        before
    );
    source(&paths);
    let report = crate::setup::ensure_setup(&paths, options(false));
    assert!(report.ready, "{report:?}");
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert!(read.reference_revision().unwrap() > before);
    let state = LocalStateRead::new(&read);
    assert_eq!(state.saved_lists().list().unwrap(), vec![list]);
    assert_eq!(state.encounters().list().unwrap(), vec![encounter]);
    assert_eq!(
        atlas_index::read_stored_reference_context(read.connection())
            .unwrap()
            .unwrap()
            .source_contract,
        atlas_foundry_model::SOURCE_CONTRACT_ID
    );
    drop(read);
    assert!(crate::workspace::open_workspace_index(&paths.index_path).is_ok());
}
