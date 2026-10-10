use crate::*;
use atlas_embedding::{
    DEFAULT_EMBEDDING_MODEL, EmbeddingRuntimeConfig, required_embedding_model_cache_files,
};
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
fn all_four_model_assets_are_verified_without_inference_or_download() {
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
    assert_eq!(report.embedding.missing_files.len(), 4);
    assert!(!report.ready);
    let config = EmbeddingRuntimeConfig::new(DEFAULT_EMBEDDING_MODEL, &paths.embedding_cache_root);
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
    assert!(report.embedding.missing_files.is_empty());
    assert!(
        !report.embedding.ready,
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
