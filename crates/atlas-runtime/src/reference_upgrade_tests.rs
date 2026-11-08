//! The glossary owner upgrade preserves retained records, traits and saved state.
use super::*;
use crate::test_support::{occurrence_vector_bits, sql_rows};
use atlas_local_state::{LocalStateWrite, NewEncounter};

fn create_encounter(workspace: &Workspace, version: u32) {
    let write = workspace.write(version).unwrap();
    LocalStateWrite::for_local(&write)
        .unwrap()
        .encounters()
        .create(NewEncounter {
            slug: "glossary-upgrade".into(),
            name: "Kept encounter".into(),
            description: None,
            note: Some("Keep this note".into()),
        })
        .unwrap();
    write.commit().unwrap();
}

fn retained_rows(
    c: &rusqlite::Connection,
    queries: &[&str],
) -> Vec<Vec<Vec<rusqlite::types::Value>>> {
    queries.iter().map(|sql| sql_rows(c, sql)).collect()
}

fn assert_retained_rows(
    c: &rusqlite::Connection,
    queries: &[&str],
    before: &[Vec<Vec<rusqlite::types::Value>>],
) {
    assert_eq!(queries.len(), before.len());
    for (sql, expected) in queries.iter().zip(before) {
        assert_eq!(&sql_rows(c, sql), expected, "{sql}");
    }
}

#[test]
fn glossary_upgrade_is_atomic_and_keeps_source_free_trait_and_record_reads() {
    let (_directory, workspace, input) = super::native_storage_tests::main24(1);
    workspace.migrate(&MIGRATIONS[..32], |_| Ok(())).unwrap();
    atlas_index::test_support::stage_pre_glossary_catalog(
        &workspace,
        &input.context.remaster_inputs_identity,
    )
    .unwrap();
    create_encounter(&workspace, 32);
    let read = workspace.read(32).unwrap();
    let durable = [
        "SELECT * FROM trait_vectors ORDER BY trait_key,profile_id",
        "SELECT * FROM trait_seed_chunks ORDER BY trait_key,profile_id,ordinal",
        "SELECT * FROM saved_lists ORDER BY id",
        "SELECT * FROM encounters ORDER BY id",
    ];
    let rollback = [
        "SELECT * FROM schema_migrations ORDER BY version",
        "SELECT * FROM trait_definitions ORDER BY key",
    ];
    let before = retained_rows(read.connection(), &durable);
    let before_rollback = retained_rows(read.connection(), &rollback);
    let before_native = occurrence_vector_bits(read.connection());
    let before_generation = read.reference_revision().unwrap();
    assert!(before.iter().all(|rows| !rows.is_empty()));
    assert!(!before_rollback[1].is_empty());
    drop(read);

    let failure = workspace.migrate(MIGRATIONS, |_| {
        Err(atlas_storage::StorageError::Migration(
            "injected glossary validation failure".into(),
        ))
    });
    assert!(
        failure
            .unwrap_err()
            .to_string()
            .contains("injected glossary validation failure")
    );
    let read = workspace.read(32).unwrap();
    assert_retained_rows(read.connection(), &rollback, &before_rollback);
    assert_retained_rows(read.connection(), &durable, &before);
    assert_eq!(read.reference_revision().unwrap(), before_generation);
    drop(read);

    let parent = workspace.path().parent().unwrap();
    let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
        path_mode: crate::AtlasPathMode::Global,
        overrides: crate::AtlasPathOverrides {
            index_path: Some(workspace.path().to_owned()),
            source_root: Some(parent.join("source")),
            embedding_cache_root: Some(parent.join("models")),
        },
        ..Default::default()
    })
    .unwrap();
    runtime.upgrade_workspace_if_present().unwrap();
    let reader = open_workspace_index(workspace.path()).unwrap();
    assert_eq!(
        reader.reference_coverage(),
        atlas_index::ReferenceCoverage::TraitsOnly
    );
    let session = reader.workspace_session();
    let c = session.connection();
    assert_retained_rows(c, &durable[2..], &before[2..]);
    assert_eq!(occurrence_vector_bits(c), before_native);
    assert_eq!(session.reference_revision().unwrap(), before_generation + 2);
    assert!(
        !c.prepare("SELECT 1 FROM sqlite_schema WHERE name='trait_definitions'")
            .unwrap()
            .exists([])
            .unwrap()
    );
    assert!(matches!(
        reader.reference_summary(&"sense:darkvision".parse().unwrap()),
        Err(atlas_index::IndexError::Unavailable(_))
    ));
    let key = atlas_domain::TraitKey::new("fire").unwrap();
    let definition = reader.trait_definition(&key).unwrap().unwrap();
    assert_eq!(definition, input.traits[0]);
    assert!(
        reader
            .read_summary(input.records[0].record.key())
            .unwrap()
            .is_some()
    );
    assert!(
        reader
            .read_source_record(input.records[0].record.key())
            .unwrap()
            .is_some()
    );
    reader.require_vectors().unwrap();
    assert!(runtime.inference.loaded_profiles().is_empty());
    // The fixture has no external source tree or model package at any point.
    assert!(!workspace.path().parent().unwrap().join("source").exists());
    assert!(!workspace.path().parent().unwrap().join("models").exists());
}

fn joined_paths(root: &std::path::Path) -> crate::ResolvedAtlasPaths {
    crate::ResolvedAtlasPaths {
        mode: crate::ResolvedPathMode::Global,
        repo_root: None,
        source_root: root.join("source"),
        managed_source_root: root.join("managed-source"),
        source_root_explicit: true,
        embedding_cache_root: root.join("models"),
        index_path: root.join("atlas.sqlite"),
    }
}

/// Matching numerical inputs are prepared by the real public owners. Values are
/// deterministic fixture data; no model executes while constructing the old artifact.
fn joined_native_input(
    paths: &crate::ResolvedAtlasPaths,
    config: &atlas_embedding::EmbeddingRuntimeConfig,
) -> atlas_index::IndexBuildInput {
    use atlas_embedding::{
        PassageSection, PreparedEmbeddingBatch, TextEmbeddingTokenizer, prepare_embedding_section,
    };
    let lexical = crate::setup::ensure_setup(
        paths,
        crate::RuntimeSetupOptions {
            target: crate::SetupTarget::Records,
            offline: true,
            ..Default::default()
        },
    );
    assert!(lexical.ready, "{lexical:?}");
    let reader = open_workspace_index(&paths.index_path).unwrap();
    let connection = reader.workspace_session().connection();
    let records =
        atlas_index::collect_persisted_inputs_for_fixture(connection, reader.context()).unwrap();
    let mut context = reader.context().clone();
    context.reference_preparation = None;
    let model =
        atlas_index::SourceSearchProfileIdentity::from_spec(atlas_embedding::SEARCH_PROFILES[0]);
    context.search_profiles = vec![model.clone()];
    assert_eq!(context.trait_policy, "trait-definition-centroid/v1");
    let baseline = TextEmbeddingTokenizer::load_baseline(&config.cache_root).unwrap();
    let native = TextEmbeddingTokenizer::load(config).unwrap();
    let vector = |hash: &str| {
        let mut value = vec![0.; model.dimensions];
        value[usize::from(u16::from_str_radix(&hash[..4], 16).unwrap()) % model.dimensions] = 1.;
        value
    };
    let checked = |input: atlas_embedding::PreparedEmbeddingInput| {
        let batch = PreparedEmbeddingBatch::prepare(&native, vec![input.into()]).unwrap();
        batch.inputs()[0].clone()
    };
    let mut lexical_units = Vec::new();
    let mut semantic_units = Vec::new();
    for record in &records {
        let label = if record.record.key().pack().as_str() == "creatures" {
            "Creatures"
        } else {
            "General abilities"
        };
        let selection = atlas_record::source_record::select_record_search(
            &record.record,
            &record.content,
            context.audience,
            label,
            &context.used_trait_labels,
        )
        .unwrap();
        // This fixture deliberately has only root identity prose, so no private
        // context-assembly or passage-layout policy is copied into the test.
        assert!(selection.fields.is_empty());
        let identity = std::iter::once(selection.identity_required.as_str())
            .chain(selection.identity_optional.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        // Every selected optional token fits; this fixture needs no budget selection
        // or private passage-context assembly to establish the exact identity input.
        assert!(
            baseline.count_tokens(&identity, false).unwrap()
                <= atlas_embedding::IDENTITY_TOKEN_BUDGET
        );
        lexical_units.extend(atlas_index::project_lexical_units(&selection, &[]));
        for input in
            prepare_embedding_section(&baseline, PassageSection::Identity { text: &identity })
                .unwrap()
        {
            let address = input.address.clone();
            let input = checked(input);
            semantic_units.push(atlas_index::SourceSemanticUnitInput {
                profile_id: model.profile_id.clone(),
                record: record.record.key().clone(),
                owners: vec![],
                field: None,
                address,
                chunk_ordinal: 0,
                input_token_count: input.token_count,
                vector: vector(&input.input_sha256),
                input_hash: input.input_sha256,
            });
        }
    }
    let mut traits = Vec::new();
    let mut trait_vectors = Vec::new();
    for authored in atlas_foundry_model::generated::trait_definitions() {
        let key = atlas_domain::TraitKey::new(authored.identifier).unwrap();
        let definition = reader.trait_definition(&key).unwrap().unwrap();
        let text = format!(
            "{}\n{}",
            definition.label,
            definition
                .prepared
                .as_ref()
                .map(|p| p.text.as_str())
                .unwrap_or("")
        );
        let mut chunks = Vec::new();
        for input in prepare_embedding_section(
            &baseline,
            PassageSection::Plain {
                text: &text,
                context: &definition.label,
                selection_version: &context.trait_policy,
                section_ordinal: 0,
                label: Some(&definition.label),
            },
        )
        .unwrap()
        {
            let address = input.address.clone();
            let input = checked(input);
            chunks.push(atlas_index::TraitSeedChunk {
                input: input.input,
                input_hash: input.input_sha256.clone(),
                token_count: input.token_count,
                vector: vector(&input.input_sha256),
                address,
            });
        }
        let hashes = chunks
            .iter()
            .map(|c| c.input_hash.as_str())
            .collect::<Vec<_>>()
            .join(":");
        trait_vectors.push(atlas_index::TraitVectorInput {
            key: key.clone(),
            profile_id: model.profile_id.clone(),
            input_hash: atlas_search::test_support::hash(
                format!("{}:{hashes}", context.trait_policy).as_bytes(),
            ),
            vector: atlas_embedding::normalized_centroid(
                &chunks
                    .iter()
                    .map(|c| c.vector.as_slice())
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            chunks,
        });
        traits.push(definition);
    }
    assert!(trait_vectors.iter().any(|seed| seed.chunks.len() > 1));
    let packs = vec![
        atlas_index::IndexBuildPack {
            pack_id: "creatures".into(),
            label: "Creatures".into(),
            document_kind: "Actor".into(),
            declared_path: "packs/creatures".into(),
        },
        atlas_index::IndexBuildPack {
            pack_id: "bestiary-ability-glossary-srd".into(),
            label: "General abilities".into(),
            document_kind: "Item".into(),
            declared_path: "packs/abilities".into(),
        },
    ];
    let inventory = atlas_index::CompleteSourceInventory::checked(
        &context.source_fingerprint,
        packs.iter().map(|p| p.pack_id.clone()),
        packs.iter().map(|p| p.pack_id.clone()),
    )
    .unwrap();
    atlas_index::IndexBuildInput {
        inventory,
        traits,
        trait_vectors,
        records,
        packs,
        context,
        aliases: vec![],
        remaster_pairs: vec![],
        remaster_context: vec![],
        lexical_units,
        semantic_units,
    }
}

#[test]
#[ignore = "requires verified local BGE assets; F3 joined upgrade/setup proof executes zero inference"]
fn populated_glossary_upgrade_then_explicit_setup_reuses_all_native_inputs() {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};
    let shared = std::env::var_os("ATLAS_TEST_MODEL_CACHE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            crate::paths::resolve_atlas_paths(
                crate::AtlasPathMode::Global,
                crate::AtlasPathOverrides::default(),
            )
            .unwrap()
            .embedding_cache_root
        });
    let shared_config = EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeBaseEnV15, &shared);
    atlas_embedding::validate_embedding_model_cache(&shared_config).unwrap();
    let preparation = tempfile::tempdir().unwrap();
    let prepared_paths = joined_paths(preparation.path());
    std::fs::create_dir_all(prepared_paths.source_root.join("packs/abilities")).unwrap();
    std::fs::create_dir_all(prepared_paths.source_root.join("packs/creatures")).unwrap();
    std::fs::create_dir_all(prepared_paths.source_root.join("static/lang")).unwrap();
    std::fs::write(prepared_paths.source_root.join("module.json"), r#"{"id":"pf2e","version":"8.6.1","packs":[{"name":"bestiary-ability-glossary-srd","label":"General abilities","type":"Item","path":"packs/abilities"},{"name":"creatures","label":"Creatures","type":"Actor","path":"packs/creatures"}]}"#).unwrap();
    let fire = format!(
        "<p>{}</p>",
        "A synthetic flame changes the target and remains visible. ".repeat(100)
    );
    std::fs::write(
        prepared_paths.source_root.join("static/lang/en.json"),
        serde_json::to_vec(
            &serde_json::json!({"PF2E":{"TraitFire":"Fire","TraitDescriptionFire":fire}}),
        )
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        prepared_paths
            .source_root
            .join("packs/abilities/darkvision.json"),
        r#"{"_id":"qCCLZhnp2HhP3Ex6","name":"Darkvision fixture","type":"action","system":{}}"#,
    )
    .unwrap();
    std::fs::write(prepared_paths.source_root.join("packs/creatures/one.json"), r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Keeper fixture","type":"npc","system":{"perception":{"senses":[{"type":"darkvision","acuity":"precise"}]}}}"#).unwrap();
    let input = joined_native_input(&prepared_paths, &shared_config);
    let expected_units = input.semantic_units.len();
    assert_eq!(expected_units, 2);
    let expected_profile = input.context.search_profiles.clone();
    let (_old_directory, workspace, input) = super::native_storage_tests::main24_input(input);
    workspace.migrate(&MIGRATIONS[..29], |_| Ok(())).unwrap();
    create_encounter(&workspace, 29);
    let before_native = occurrence_vector_bits(workspace.read(29).unwrap().connection());
    workspace.migrate(&MIGRATIONS[..32], |_| Ok(())).unwrap();
    atlas_index::test_support::stage_pre_glossary_catalog(
        &workspace,
        &input.context.remaster_inputs_identity,
    )
    .unwrap();
    let parent = workspace.path().parent().unwrap();
    let mut paths = joined_paths(parent);
    paths.index_path = workspace.path().to_owned();
    assert!(!paths.source_root.exists());
    assert!(!paths.embedding_cache_root.exists());
    let runtime = crate::AtlasRuntime::resolve(crate::AtlasRuntimeOptions {
        path_mode: crate::AtlasPathMode::Global,
        overrides: crate::AtlasPathOverrides {
            index_path: Some(paths.index_path.clone()),
            source_root: Some(paths.source_root.clone()),
            embedding_cache_root: Some(paths.embedding_cache_root.clone()),
        },
        ..Default::default()
    })
    .unwrap();
    runtime.upgrade_workspace_if_present().unwrap();
    assert!(!paths.source_root.exists());
    assert!(!paths.embedding_cache_root.exists());
    let reader = open_workspace_index(&paths.index_path).unwrap();
    assert_eq!(
        reader.reference_coverage(),
        atlas_index::ReferenceCoverage::TraitsOnly
    );
    assert_eq!(reader.context().search_profiles, expected_profile);
    reader.require_vectors().unwrap();
    let assert_reads = |reader: &atlas_index::SqliteIndexReader| {
        for record in &input.records {
            assert!(
                reader
                    .read_source_record(record.record.key())
                    .unwrap()
                    .is_some()
            );
        }
        assert_eq!(
            reader
                .trait_definition(&atlas_domain::TraitKey::new("fire").unwrap())
                .unwrap()
                .unwrap(),
            *input
                .traits
                .iter()
                .find(|t| t.key.to_string() == "trait:fire")
                .unwrap()
        );
    };
    assert_reads(&reader);
    let c = reader.workspace_session().connection();
    assert_eq!(occurrence_vector_bits(c), before_native);
    let native_sql = format!(
        "SELECT input_id,embedding FROM native_vectors_{} ORDER BY input_id",
        atlas_search::test_support::hash(&serde_json::to_vec(&expected_profile[0]).unwrap())
    );
    let queries = [
        "SELECT * FROM embedding_inputs ORDER BY input_id",
        &native_sql,
        "SELECT * FROM saved_lists ORDER BY id",
        "SELECT * FROM encounters ORDER BY id",
    ];
    let before = retained_rows(c, &queries);
    assert_eq!(before[0].len(), before[1].len());
    assert!(before.iter().all(|rows| !rows.is_empty()));
    let darkvision = "sense:darkvision".parse().unwrap();
    assert!(matches!(
        reader.reference_entry(&darkvision),
        Err(atlas_index::IndexError::Unavailable(_))
    ));
    let old_generation = reader.workspace_session().reference_revision().unwrap();
    drop(reader);
    // Source and verified cache become available only for explicit setup. A private
    // disposable copy leaves shared assets and installed data untouched.
    std::fs::rename(&prepared_paths.source_root, &paths.source_root).unwrap();
    let isolated =
        EmbeddingRuntimeConfig::new(EmbeddingModelId::BgeBaseEnV15, &paths.embedding_cache_root);
    for file in atlas_embedding::required_embedding_model_cache_files(&isolated) {
        std::fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
        std::fs::copy(
            shared_config.model_dir().join(file.source_path),
            file.local_path,
        )
        .unwrap();
    }
    let options = || crate::RuntimeSetupOptions {
        offline: true,
        search_profiles: vec![expected_profile[0].profile_id.clone()],
        ..Default::default()
    };
    // Fail inside the actual publication transaction, after native reuse selection.
    // This private fixture trigger is removed before retry; no production fault hook.
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write.connection().execute_batch("CREATE TRIGGER reject_glossary_fixture BEFORE INSERT ON reference_entries WHEN NEW.key='sense:darkvision' BEGIN SELECT RAISE(ABORT,'injected glossary publication failure'); END;").unwrap();
    write.commit().unwrap();
    let failed = crate::setup::ensure_setup(&paths, options());
    assert!(!failed.ready);
    assert!(
        failed
            .actions
            .iter()
            .any(|a| a.kind == crate::SetupActionKind::BuildIndex
                && a.status == crate::SetupActionStatus::Failed
                && a.reason
                    .as_deref()
                    .is_some_and(|r| r.contains("injected glossary publication failure"))),
        "{failed:?}"
    );
    let retained = open_workspace_index(&paths.index_path).unwrap();
    assert_eq!(
        retained.reference_coverage(),
        atlas_index::ReferenceCoverage::TraitsOnly
    );
    assert_eq!(
        retained.workspace_session().reference_revision().unwrap(),
        old_generation
    );
    assert_retained_rows(retained.workspace_session().connection(), &queries, &before);
    assert_reads(&retained);
    drop(retained);
    let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute_batch("DROP TRIGGER reject_glossary_fixture;")
        .unwrap();
    write.commit().unwrap();
    let report = crate::setup::ensure_setup(&paths, options());
    assert!(report.ready, "{report:?}");
    let build = report.build.unwrap();
    assert_eq!(build.inferred_inputs, 0);
    assert_eq!(build.semantic_unit_count, expected_units);
    assert_eq!(build.profiles.len(), 1);
    assert_eq!(build.profiles[0].profile_id, expected_profile[0].profile_id);
    assert_eq!(build.reused_inputs, before[0].len());
    assert!(build.reused_inputs > 784);
    let reader = open_workspace_index(&paths.index_path).unwrap();
    assert_eq!(
        reader.reference_coverage(),
        atlas_index::ReferenceCoverage::Complete
    );
    assert_eq!(reader.context().search_profiles, expected_profile);
    assert_eq!(
        reader.context().source_fingerprint,
        input.context.source_fingerprint
    );
    assert_reads(&reader);
    let sense = reader.reference_entry(&darkvision).unwrap().unwrap();
    assert!(
        matches!(sense.backing, atlas_index::ReferenceBacking::Record { key } if key.to_string() == "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6")
    );
    let c = reader.workspace_session().connection();
    assert_eq!(occurrence_vector_bits(c), before_native);
    assert_retained_rows(c, &queries, &before);
    let preparation = reader.context().reference_preparation.as_ref().unwrap();
    assert_eq!(
        sql_rows(c, "SELECT key FROM reference_entries").len(),
        preparation.expected_entry_count
    );
    let published_generation = reader.workspace_session().reference_revision().unwrap();
    assert_eq!(published_generation, old_generation + 1);
    drop(reader);
    let repeated = crate::setup::ensure_setup(&paths, options());
    assert!(repeated.ready, "{repeated:?}");
    assert!(repeated.build.is_none());
    assert_eq!(
        workspace
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        published_generation
    );
    println!(
        "F3 exact native reuse: {} inputs, {} semantic occurrences, {} ordered trait chunks; zero inferred inputs",
        build.reused_inputs,
        expected_units,
        before[2].len()
    );
}
