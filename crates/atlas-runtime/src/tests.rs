use crate::*;
use atlas_embedding::{EmbeddingRuntimeConfig, required_embedding_model_cache_files};
use atlas_index::{IndexArtifactWriter, SqliteIndexReader, SqliteIndexWriter};

use serde_json::json;
use std::fs;
fn paths(dir: &std::path::Path) -> ResolvedAtlasPaths {
    ResolvedAtlasPaths {
        mode: ResolvedPathMode::Global,
        repo_root: None,
        source_root: dir.join("source"),
        embedding_cache_root: dir.join("models"),
        index_path: dir.join("atlas.sqlite"),
        local_state_path: dir.join("state.sqlite"),
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
        SqliteIndexReader::open_read_only(&paths.index_path)
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
        SqliteIndexReader::open_read_only(&paths.index_path)
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
    let bytes = fs::read(&paths.index_path).unwrap();
    fs::write(paths.source_root.join("module.json"), "invalid").unwrap();
    let report = crate::setup::ensure_setup(
        &paths,
        RuntimeSetupOptions {
            force_rebuild: true,
            ..options(false)
        },
    );
    assert_eq!(report.exit_code_class(), SetupExitClass::RuntimeFailure);
    assert_eq!(fs::read(&paths.index_path).unwrap(), bytes);
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
    let service = runtime.open_retrieval_service_no_embeddings().unwrap();
    let result = service
        .list_records(atlas_search::ListRecordsRequest::new(
            None,
            atlas_search::SearchPage::default(),
        ))
        .unwrap();
    assert_eq!(result.total, 1);
    assert!(runtime.open_retrieval_service_for_stored_vectors().is_err());
    assert!(runtime.open_retrieval_service().is_err());
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
    SqliteIndexWriter::new(&paths.index_path)
        .write(&atlas_search::test_support::input(vec![record], true))
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
    assert!(runtime.open_retrieval_service_for_stored_vectors().is_ok());
    assert!(runtime.open_retrieval_service().is_err());
    assert!(runtime.validate_index_report(SetupTarget::Full).is_ok());
    assert!(runtime.check_index_report(SetupTarget::Records).is_ok());
    let reader = runtime.open_index().unwrap();
    let metadata = runtime.search_profile_readiness(&reader.context().search_profiles);
    assert_eq!(metadata.len(), 1);
    assert!(metadata[0].stored_vectors);
    assert!(!metadata[0].assets_ready);
    assert!(!metadata[0].usable);
    assert!(runtime.inference.loaded_profiles().is_empty());
}
#[test]
fn old_artifact_requires_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths(dir.path());
    source(&paths);
    fs::write(&paths.index_path, b"old incompatible artifact").unwrap();
    let report = crate::setup::ensure_setup(&paths, options(true));
    assert!(!report.ready);
    let report = crate::setup::ensure_setup(&paths, options(false));
    assert!(report.ready);
    assert!(report.build.is_some());
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
fn omitted_setup_preserves_readable_single_or_both_profiles_before_asset_work() {
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
        SqliteIndexWriter::new(&paths.index_path)
            .write(&input)
            .unwrap();
        let expected = input
            .context
            .search_profiles
            .iter()
            .map(|p| p.profile_id.clone())
            .collect::<Vec<_>>();
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
        let reader = SqliteIndexReader::open_read_only(&paths.index_path).unwrap();
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
    SqliteIndexWriter::new(&paths.index_path)
        .write(&input)
        .unwrap();
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        overrides: AtlasPathOverrides {
            index_path: Some(paths.index_path),
            embedding_cache_root: Some(paths.embedding_cache_root.clone()),
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
    let retrieval = runtime.open_retrieval_service().unwrap();
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
