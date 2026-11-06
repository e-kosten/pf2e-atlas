use crate::*;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture(PathBuf);

/// An explicit smoke cache is authoritative. Native loaders still verify every
/// consumed asset; an invalid override must never select another local cache.
fn native_smoke_cache(model: atlas_embedding::EmbeddingModelId) -> PathBuf {
    let marker = match model {
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15 => "onnx/model.onnx",
        atlas_embedding::EmbeddingModelId::EmbeddingGemma300M => "onnx/model.onnx_data",
    };
    let has_model = |root: &Path| {
        atlas_embedding::EmbeddingRuntimeConfig::new(model, root)
            .model_dir()
            .join(marker)
            .is_file()
    };
    if let Some(root) = std::env::var_os("ATLAS_TEST_MODEL_CACHE") {
        let root = PathBuf::from(root);
        assert!(
            has_model(&root),
            "ATLAS_TEST_MODEL_CACHE lacks pinned {} assets beneath {}",
            model.as_str(),
            root.display()
        );
        return root;
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|parent| parent.join(".cache/hf-models"))
        .find(|root| has_model(root))
        .expect("these ignored native smoke tests require the pinned local model cache")
}

/// A thread-scoped observer keeps deterministic phase-boundary mutations inside
/// the actual product path, without adding build hooks or production callbacks.
struct BuildTraceObserver<F>(F);
impl<F: Fn(&tracing::Event<'_>) + Send + Sync + 'static> tracing::Subscriber
    for BuildTraceObserver<F>
{
    // Parallel tests use independent thread-local dispatchers. Keep callsites
    // dynamically enabled through the current dispatcher instead of caching this
    // short-lived observer's interest for another test's subscriber lifetime.
    fn register_callsite(
        &self,
        _: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        tracing::subscriber::Interest::sometimes()
    }
    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::TRACE)
    }
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        (self.0)(event);
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}
#[derive(Default)]
struct TraceFields(std::collections::BTreeMap<String, String>);
impl tracing::field::Visit for TraceFields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
}

#[test]
fn captured_git_revision_matches_the_published_manifest_after_checkout_advances() {
    let fixture = Fixture::new();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(&fixture.0)
            .args([
                "-c",
                "user.name=Atlas test",
                "-c",
                "user.email=atlas-test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=.git/no-hooks",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(&["init"]);
    git(&["add", "."]);
    git(&["commit", "-m", "Original source"]);
    let original_revision = git(&["rev-parse", "HEAD"]);

    let destination = tempfile::tempdir().unwrap();
    let mut options = fixture.options();
    options.output_path = destination.path().join("artifact.sqlite");
    let captured = CapturedBuildInputs::capture(options.clone()).unwrap();
    let manifest = fixture.0.join("module.json");
    let changed = fs::read_to_string(&manifest)
        .unwrap()
        .replace(r#""label":"Items""#, r#""label":"Renamed Items""#);
    fs::write(&manifest, changed).unwrap();
    git(&["add", "module.json"]);
    git(&["commit", "-m", "Rename pack"]);
    assert_ne!(git(&["rev-parse", "HEAD"]), original_revision);

    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path).unwrap();
    captured
        .build(
            &workspace,
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap();
    let reader = atlas_index::test_support::open_reader(&options.output_path).unwrap();
    assert_eq!(
        reader.context().source_revision.as_deref(),
        Some(original_revision.as_str())
    );
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    let label: String = db
        .query_row("SELECT label FROM packs WHERE pack_id='items'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(label, "Items");
}

#[test]
fn captured_inputs_are_consumed_once_and_later_changes_belong_to_the_next_build() {
    const CHILD: &str = "ATLAS_TEST_CAPTURED_ONCE_CHILD";
    const COMPLETE: &str = "captured-once: owned and unconsumed mutation cases verified";
    // Isolate phase-boundary mutations from process-global callsite registration
    // in other concurrent fixtures, while exercising the actual build/publisher.
    if std::env::var_os(CHILD).as_deref() != Some(std::ffi::OsStr::new("1")) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env(CHILD, "1")
            .args(["--exact", "build_tests::captured_inputs_are_consumed_once_and_later_changes_belong_to_the_next_build", "--nocapture"])
            .output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "{}\n{}\n{stderr}",
            output.status,
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(stderr.lines().any(|line| line == COMPLETE), "{stderr}");
        return;
    }
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    fn rows(path: &Path) -> Vec<(String, String, String)> {
        rusqlite::Connection::open(path)
            .unwrap()
            .prepare("SELECT key,content_hash,name FROM records ORDER BY key")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }
    for phase in ["source_capture", "captured_document_batch"] {
        for changed in ["document", "catalog", "manifest"] {
            // The manifest is consumed by capture; the other files are not yet
            // consumed at source_capture. The second boundary owns all tiny docs.
            for remove in [false, true] {
                let fixture = Fixture::new();
                fs::write(fixture.0.join("module.json"), r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"alias","label":"Alias","type":"Item","path":"packs/items"},{"name":"macros","label":"Macros","type":"Macro","path":"packs/macros"}]}"#).unwrap();
                let options = fixture.options();
                let before = build_artifact(options.clone()).unwrap();
                let expected = rows(&options.output_path);
                let before_context = atlas_index::test_support::open_reader(&options.output_path)
                    .unwrap()
                    .context()
                    .clone();
                let path = fixture.0.join(match changed {
                    "document" => "packs/items/useful.json",
                    "catalog" => "static/lang/en.json",
                    _ => "module.json",
                });
                let bytes = match changed {
                    "document" => {
                        br#"{"_id":"cccccccccccccccc","name":"Next document","type":"effect"}"#
                            .to_vec()
                    }
                    "catalog" => br#"{"PF2E":{"Label":"Next catalog"}}"#.to_vec(),
                    _ => b"{invalid".to_vec(),
                };
                let observed = Arc::new(AtomicBool::new(false));
                let changed_once = Arc::clone(&observed);
                let added = fixture.0.join("packs/items/added.json");
                let observer = BuildTraceObserver(move |event: &tracing::Event<'_>| {
                    let mut fields = TraceFields::default();
                    event.record(&mut fields);
                    if event.metadata().target() == "atlas_build_diagnostic"
                        && fields.0.get("stage").is_some_and(|stage| stage == phase)
                        && !changed_once.swap(true, Ordering::SeqCst)
                    {
                        if remove {
                            fs::remove_file(&path).unwrap();
                        } else {
                            fs::write(&path, &bytes).unwrap();
                        }
                        fs::write(&added, r#"{"_id":"dddddddddddddddd","name":"Next added document","type":"effect"}"#).unwrap();
                    }
                });
                let result =
                    tracing::subscriber::with_default(observer, || build_artifact(options.clone()));
                assert!(
                    observed.load(Ordering::SeqCst),
                    "{phase}/{changed}/{remove}: {result:?}"
                );
                let consumed = phase == "captured_document_batch" || changed == "manifest";
                if consumed {
                    let report = result.unwrap();
                    assert_eq!(report.source_fingerprint, before.source_fingerprint);
                    assert_eq!(
                        rows(&options.output_path),
                        expected,
                        "{phase}/{changed}/{remove}"
                    );
                    let context = atlas_index::test_support::open_reader(&options.output_path)
                        .unwrap()
                        .context()
                        .clone();
                    assert_eq!(
                        context.english_catalog_sha256,
                        before_context.english_catalog_sha256
                    );
                    assert_eq!(report.record_count, 3, "two pack bindings and a Macro");
                    // Ordinary next capture observes the mutations/new inventory.
                    let next = build_artifact(options.clone());
                    if changed == "manifest" || (changed == "catalog" && remove) {
                        assert!(next.is_err());
                    } else {
                        let next = next.unwrap();
                        assert_ne!(next.source_fingerprint, report.source_fingerprint);
                        assert_ne!(rows(&options.output_path), expected);
                    }
                } else {
                    assert!(
                        matches!(result, Err(IngestError::SourceUnavailable(_))),
                        "{result:?}"
                    );
                    assert_eq!(rows(&options.output_path), expected);
                    assert_eq!(
                        atlas_index::test_support::open_reader(&options.output_path)
                            .unwrap()
                            .context()
                            .source_fingerprint,
                        before.source_fingerprint
                    );
                }
                atlas_index::validate_artifact(&options.output_path).unwrap();
            }
        }
    }
    for phase in ["source_capture", "source_preparation"] {
        for remove in [false, true] {
            let fixture = Fixture::new();
            let directory = fixture.0.join("src/module/migration/migrations");
            fs::create_dir_all(&directory).unwrap();
            fs::create_dir_all(fixture.0.join("packs/spells")).unwrap();
            fs::write(fixture.0.join("packs/spells/new.json"),r#"{"_id":"cccccccccccccccc","name":"Useful spell","type":"spell","system":{"description":{"value":"<p>Spell body.</p>"}}}"#).unwrap();
            let manifest_path = fixture.0.join("module.json");
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
            manifest["packs"].as_array_mut().unwrap().push(serde_json::json!({"name":"spells-srd","label":"Spells","type":"Item","path":"packs/spells"}));
            fs::write(manifest_path, manifest.to_string()).unwrap();
            let path = directory.join("887-redirect-spell-links.ts");
            fs::write(&path,r#"class M { #spells=[{from:{_id:"dddddddddddddddd",name:"Old name"},to:{_id:"cccccccccccccccc",name:"Useful spell"}}]; }"#).unwrap();
            let options = fixture.options();
            let before = build_artifact(options.clone()).unwrap();
            let aliases = |path: &Path| -> Vec<String> {
                rusqlite::Connection::open(path)
                    .unwrap()
                    .prepare("SELECT alias FROM remaster_aliases ORDER BY alias")
                    .unwrap()
                    .query_map([], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<_, _>>()
                    .unwrap()
            };
            assert_eq!(aliases(&options.output_path), ["Old name"]);
            let observed = Arc::new(AtomicBool::new(false));
            let changed_once = Arc::clone(&observed);
            let observer = BuildTraceObserver(move |event: &tracing::Event<'_>| {
                let mut fields = TraceFields::default();
                event.record(&mut fields);
                if event.metadata().target() == "atlas_build_diagnostic"
                    && fields.0.get("stage").is_some_and(|stage| stage == phase)
                    && !changed_once.swap(true, Ordering::SeqCst)
                {
                    if remove {
                        fs::remove_file(&path).unwrap();
                    } else {
                        fs::write(&path,r#"class M { #spells=[{from:{_id:"dddddddddddddddd",name:"Next old name"},to:{_id:"cccccccccccccccc",name:"Useful spell"}}]; }"#).unwrap();
                    }
                }
            });
            let result =
                tracing::subscriber::with_default(observer, || build_artifact(options.clone()));
            assert!(
                observed.load(Ordering::SeqCst),
                "alias/{phase}/{remove}: {result:?}"
            );
            if phase == "source_capture" {
                assert!(
                    matches!(result, Err(IngestError::SourceUnavailable(_))),
                    "{result:?}"
                );
            } else {
                assert_eq!(
                    result.unwrap().source_fingerprint,
                    before.source_fingerprint
                );
            }
            assert_eq!(aliases(&options.output_path), ["Old name"]);
            if phase == "source_preparation" {
                let next = build_artifact(options.clone()).unwrap();
                assert_ne!(next.source_fingerprint, before.source_fingerprint);
                assert_eq!(
                    aliases(&options.output_path),
                    if remove {
                        Vec::<String>::new()
                    } else {
                        vec!["Next old name".to_owned()]
                    }
                );
            }
            atlas_index::validate_artifact(&options.output_path).unwrap();
        }
    }
    eprintln!("{COMPLETE}");
}

#[test]
#[ignore = "requires ATLAS_TEST_MODEL_CACHE pinned BGE tokenizer only; no model inference"]
fn bounded_cached_native_windows_deduplicate_without_loading_weights() {
    let fixture = Fixture::new();
    let cache = PathBuf::from(
        std::env::var_os("ATLAS_TEST_MODEL_CACHE").expect("set ATLAS_TEST_MODEL_CACHE"),
    );
    let source_config = atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        cache,
    );
    let tokenizer_cache = fixture.0.join("tokenizer-only");
    let config =
        atlas_embedding::EmbeddingRuntimeConfig::new(source_config.model, &tokenizer_cache);
    fs::create_dir_all(config.model_dir()).unwrap();
    fs::copy(
        source_config.model_dir().join("tokenizer.json"),
        config.model_dir().join("tokenizer.json"),
    )
    .unwrap();
    assert!(!config.model_dir().join("onnx/model.onnx").exists());
    let mut options = fixture.options();
    options.search_profiles = vec![config];
    let long_trait = atlas_foundry_model::generated::trait_definitions()
        .iter()
        .find(|definition| definition.description_key.is_some())
        .unwrap();
    let catalog_path = fixture.0.join("static/lang/en.json");
    let mut catalog: serde_json::Value =
        serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog[long_trait.description_key.unwrap()] =
        serde_json::json!(format!("<p>{}</p>", "one ".repeat(50_000)));
    fs::write(catalog_path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("packs/items/useful.json")).unwrap())
            .unwrap();
    for ordinal in 0..140 {
        let mut duplicate = original.clone();
        duplicate["_id"] = serde_json::json!(format!("{ordinal:016x}"));
        fs::write(
            fixture
                .0
                .join(format!("packs/items/duplicate-{ordinal}.json")),
            serde_json::to_vec(&duplicate).unwrap(),
        )
        .unwrap();
    }
    crate::build::publish_fixture_without_inference(&options).unwrap();
    let prepared = crate::product_preparation::prepare_fixture(&options).unwrap();
    assert!(prepared.native_inputs.len() > atlas_index::NATIVE_VALUE_BATCH_LIMIT * 2);
    let mut unique = std::collections::BTreeSet::new();
    let model = &prepared.spool.context().search_profiles[0];
    prepared
        .native_inputs
        .visit(&mut |frame| {
            unique.insert((
                serde_json::to_string(model)?,
                frame.input.input_sha256,
                frame.input.token_count,
            ));
            Ok(())
        })
        .unwrap();
    assert!(unique.len() < prepared.native_inputs.len());
    let mut expected_trait_hashes = Vec::new();
    prepared
        .native_inputs
        .visit(&mut |frame| {
            if let crate::product_preparation::RawOccurrence::Trait { key, .. } = frame.occurrence
                && key.identifier() == long_trait.identifier
            {
                expected_trait_hashes.push(frame.input.input_sha256);
            }
            Ok(())
        })
        .unwrap();
    assert!(expected_trait_hashes.len() > atlas_index::NATIVE_VALUE_BATCH_LIMIT);
    assert!(
        expected_trait_hashes
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            < expected_trait_hashes.len()
    );

    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = std::sync::Arc::clone(&events);
    let observer = BuildTraceObserver(move |event: &tracing::Event<'_>| {
        if event.metadata().target() == "atlas_progress" {
            let mut fields = TraceFields::default();
            event.record(&mut fields);
            captured.lock().unwrap().push(fields.0);
        }
    });
    let report =
        tracing::subscriber::with_default(observer, || build_artifact(options.clone())).unwrap();
    let events = events.lock().unwrap();
    let phase = "bge-base:reuse_selection";
    let rows = events
        .iter()
        .filter(|event| event.get("phase").is_some_and(|value| value == phase))
        .collect::<Vec<_>>();
    let counts = rows
        .iter()
        .map(|event| event["current"].parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(counts.first(), Some(&0));
    assert_eq!(counts.last(), Some(&prepared.native_inputs.len()));
    assert!(
        counts.windows(2).all(|pair| pair[0] < pair[1]),
        "aggregate progress must not reset between windows"
    );
    assert!(
        rows.iter()
            .all(|event| event["total"].parse::<usize>().unwrap() == prepared.native_inputs.len())
    );
    assert!(events.iter().all(|event| {
        !event
            .get("message")
            .is_some_and(|message| message.contains("Generating") || message.contains("Loading"))
    }));
    assert_eq!(report.inferred_inputs, 0);
    assert_eq!(report.reused_inputs, unique.len());
    assert_eq!(report.profiles[0].reused_inputs, unique.len());
    atlas_index::validate_artifact(&options.output_path).unwrap();
    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path).unwrap();
    let schema = atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    {
        let read = workspace.read(schema).unwrap();
        let actual_hashes = read.connection().prepare("SELECT lower(hex(i.input_hash)) FROM trait_seed_chunks t JOIN embedding_inputs i USING(input_id) WHERE trait_key=? ORDER BY ordinal").unwrap()
            .query_map([format!("trait:{}", long_trait.identifier)], |row| row.get::<_, String>(0)).unwrap()
            .collect::<rusqlite::Result<Vec<_>>>().unwrap();
        assert_eq!(actual_hashes, expected_trait_hashes);
    }
    let before_revision = workspace
        .read(schema)
        .unwrap()
        .reference_revision()
        .unwrap();
    let before_context = atlas_index::test_support::open_reader(&options.output_path)
        .unwrap()
        .context()
        .sha256()
        .unwrap();
    options.reuse_embeddings = false;
    assert!(
        build_artifact(options.clone()).is_err(),
        "forced inference must require missing weights"
    );
    // Recovery cache metadata may change, but the published
    // reference and its ordinary queryable row count remain intact.
    assert_eq!(
        workspace
            .read(schema)
            .unwrap()
            .reference_revision()
            .unwrap(),
        before_revision
    );
    let reader = atlas_index::test_support::open_reader(&options.output_path).unwrap();
    assert_eq!(reader.context().sha256().unwrap(), before_context);
    assert_eq!(reader.statistics().unwrap().records, report.record_count);
    assert_eq!(
        reader.context().source_fingerprint,
        report.source_fingerprint
    );
    drop(reader);
    options
        .search_profiles
        .push(atlas_embedding::EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::EmbeddingGemma300M,
            fixture.0.join("missing-second-profile"),
        ));
    assert!(crate::build::publish_fixture_without_inference(&options).is_err());
    let read = workspace.read(schema).unwrap();
    assert_eq!(read.reference_revision().unwrap(), before_revision);
    assert!(
        read.connection()
            .query_row("SELECT count(*) FROM pending_embedding_values", [], |row| {
                row.get::<_, usize>(0)
            })
            .unwrap()
            > 0
    );
    atlas_index::validate_reference(read.connection()).unwrap();
}

#[test]
#[ignore = "requires the pinned BGE tokenizer; no model inference"]
fn published_passage_replay_matches_fresh_selection_after_dependency_changes() {
    let fixture = Fixture::new();
    let mut options = fixture.options();
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        std::env::var_os("ATLAS_TEST_MODEL_CACHE").expect("set ATLAS_TEST_MODEL_CACHE"),
    )];
    let source_path = fixture.0.join("packs/items/useful.json");
    let target_path = fixture.0.join("packs/items/target.json");
    let description = format!(
        "<h2>Definition</h2><p>@UUID[Compendium.pf2e.items.Item.cccccccccccccccc] @Localize[PF2E.Label] {}</p>",
        "Unicode 🐉 prose and more details. ".repeat(350)
    );
    let source = serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Useful record","type":"effect","system":{"description":{"value":description}}});
    let target = serde_json::json!({"_id":"cccccccccccccccc","name":"Target","type":"effect","system":{"description":{"value":"<p>Target prose</p>"}}});
    fs::write(&source_path, serde_json::to_vec(&source).unwrap()).unwrap();
    fs::write(&target_path, serde_json::to_vec(&target).unwrap()).unwrap();
    crate::build::publish_fixture_without_inference(&options).unwrap();
    let initial = crate::product_preparation::prepare_fixture(&options).unwrap();
    let initial_signatures = crate::passage_benchmark::signatures(&initial);
    drop(initial);
    let fresh_directory = tempfile::tempdir().unwrap();
    let original_manifest = fs::read(fixture.0.join("module.json")).unwrap();
    let original_locale = fs::read(fixture.0.join("static/lang/en.json")).unwrap();
    for case in [
        "unchanged",
        "body",
        "reference",
        "localization",
        "pack label",
    ] {
        fs::write(&source_path, serde_json::to_vec(&source).unwrap()).unwrap();
        fs::write(&target_path, serde_json::to_vec(&target).unwrap()).unwrap();
        fs::write(fixture.0.join("module.json"), &original_manifest).unwrap();
        fs::write(fixture.0.join("static/lang/en.json"), &original_locale).unwrap();
        match case {
            "body" => {
                let mut changed = source.clone();
                changed["system"]["description"]["value"] =
                    serde_json::json!(format!("{description}<p>Added body text.</p>"));
                fs::write(&source_path, serde_json::to_vec(&changed).unwrap()).unwrap();
            }
            "reference" => {
                let mut changed = target.clone();
                changed["name"] = serde_json::json!("Renamed target");
                fs::write(&target_path, serde_json::to_vec(&changed).unwrap()).unwrap();
            }
            "localization" => {
                fs::write(
                    fixture.0.join("static/lang/en.json"),
                    r#"{"PF2E":{"Label":"Changed localized text"}}"#,
                )
                .unwrap();
            }
            "pack label" => {
                let mut changed: serde_json::Value =
                    serde_json::from_slice(&original_manifest).unwrap();
                changed["packs"][0]["label"] = serde_json::json!("Renamed pack");
                fs::write(
                    fixture.0.join("module.json"),
                    serde_json::to_vec(&changed).unwrap(),
                )
                .unwrap();
            }
            _ => {}
        }
        let mut fresh_options = options.clone();
        fresh_options.output_path = fresh_directory.path().join(format!("{case}.sqlite"));
        fresh_options.reuse_passage_layouts = false;
        let fresh = crate::product_preparation::prepare_fixture(&fresh_options).unwrap();
        let replay = crate::product_preparation::prepare_fixture(&options).unwrap();
        let fresh_signatures = crate::passage_benchmark::signatures(&fresh);
        let replay_signatures = crate::passage_benchmark::signatures(&replay);
        assert_eq!(fresh_signatures, replay_signatures, "{case}");
        if case == "unchanged" {
            assert!(replay.reused_sections > 0);
            assert_eq!(fresh_signatures, initial_signatures);
        } else {
            assert_ne!(
                fresh_signatures, initial_signatures,
                "{case} must exercise a changed input"
            );
        }
    }
}
fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path)
        .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
    crate::build_artifact(
        &workspace,
        atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        options,
    )
}
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scratch/ingest-tests")
            .join(format!(
                "{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
        fs::create_dir_all(path.join("packs/items")).unwrap();
        fs::create_dir_all(path.join("packs/macros")).unwrap();
        fs::create_dir_all(path.join("static/lang")).unwrap();
        fs::write(
            path.join("static/lang/en.json"),
            r#"{"PF2E":{"Label":"English","Fallback":"English fallback"}}"#,
        )
        .unwrap();
        fs::write(path.join("module.json"),r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"macros","label":"Macros","type":"Macro","path":"packs/macros"}]}"#).unwrap();
        fs::write(path.join("packs/items/useful.json"),r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Useful record","type":"effect","system":{"description":{"value":"<h2>Defined Affliction</h2><p>Full prose @Check[type:reflex|dc:23] and @Damage[2d6].</p>"},"level":{"value":"quoted"}}}"#).unwrap();
        fs::write(path.join("packs/items/bad.json"), "{bad").unwrap();
        fs::write(path.join("packs/macros/tool.json"),r#"{"_id":"bbbbbbbbbbbbbbbb","name":"Tool macro","type":"script","command":"console.log('tool');"}"#).unwrap();
        Self(path)
    }
    fn options(&self) -> BuildArtifactOptions {
        BuildArtifactOptions {
            source_root: self.0.clone(),
            output_path: self.0.join("artifact.sqlite"),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 16,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn missing_null_and_invalid_names_preserve_root_and_nameless_owned_prose() {
    let fixture = Fixture::new();
    for name in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!(42)),
    ] {
        let mut source = serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"effect","system":{"description":{"value":"<p>Still useful root prose.</p>"}}});
        if let Some(name) = name {
            source["name"] = name;
        }
        fs::write(
            fixture.0.join("packs/items/useful.json"),
            serde_json::to_vec(&source).unwrap(),
        )
        .unwrap();
        let report = build_artifact(fixture.options()).unwrap();
        let reader = atlas_index::test_support::open_reader(&report.output_path).unwrap();
        let key = "items:aaaaaaaaaaaaaaaa".parse().unwrap();
        assert!(
            reader
                .read_source_record_for_inspection(&key)
                .unwrap()
                .is_some()
        );
        let db = rusqlite::Connection::open(&report.output_path).unwrap();
        let identity: String = db
            .query_row(
                "SELECT identity_terms FROM lexical_units WHERE unit_kind='root_name'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(identity, key.to_string());
        assert_eq!(
            atlas_index::validate_artifact(&report.output_path)
                .unwrap()
                .roots,
            2
        );
    }
    fs::write(fixture.0.join("module.json"), r#"{"packs":[{"name":"items","label":"Actors","type":"Actor","path":"packs/items"},{"name":"macros","label":"Macros","type":"Macro","path":"packs/macros"}]}"#).unwrap();
    fs::write(fixture.0.join("packs/items/useful.json"), r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Parent","type":"npc","items":[{"_id":"cccccccccccccccc","type":"action","system":{"description":{"value":"<h2>Nameless embedded definition</h2><p>Owned prose survives without an invented name.</p>"}}}]}"#).unwrap();
    let report = build_artifact(fixture.options()).unwrap();
    let db = rusqlite::Connection::open(&report.output_path).unwrap();
    let label: String = db
        .query_row(
            "SELECT definition_terms FROM lexical_units WHERE unit_kind='heading'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(label, "Nameless embedded definition");
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM lexical_units WHERE unit_kind='owned_name'",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        atlas_index::validate_artifact(&report.output_path)
            .unwrap()
            .roots,
        2
    );
}
#[test]
#[ignore = "requires verified local tokenizer; run explicitly"]
fn actual_tokenizer_prepares_identity_for_unavailable_authored_names() {
    let fixture = Fixture::new();
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let cache = ancestor.join(".cache/hf-models");
            cache
                .join(format!(
                    "BAAI/bge-base-en-v1.5/{}/onnx/model.onnx",
                    atlas_embedding::embedding_model_spec(
                        atlas_embedding::EmbeddingModelId::BgeBaseEnV15
                    )
                    .model_revision
                ))
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let mut options = fixture.options();
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        cache,
    )];
    for name in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!(42)),
    ] {
        let mut source = serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"effect","system":{"description":{"value":"<p>Still useful prose.</p>"}}});
        if let Some(name) = name {
            source["name"] = name;
        }
        fs::write(
            fixture.0.join("packs/items/useful.json"),
            serde_json::to_vec(&source).unwrap(),
        )
        .unwrap();
        let prepared = crate::product_preparation::prepare_fixture(&options).unwrap();
        let mut identities = std::collections::BTreeSet::new();
        let mut has_passage = false;
        prepared
            .native_inputs
            .visit(&mut |frame| {
                if let crate::product_preparation::RawOccurrence::Document { attribution } =
                    frame.occurrence
                {
                    if matches!(
                        attribution.address,
                        atlas_domain::SourcePassageAddress::Identity {}
                    ) {
                        assert_eq!(attribution.record.to_string(), "items:aaaaaaaaaaaaaaaa");
                        assert!(frame.input.input.contains("items:aaaaaaaaaaaaaaaa"));
                        assert!(frame.input.token_count <= 512);
                        identities.insert(attribution.record);
                    } else {
                        has_passage = true;
                    }
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(identities.len(), 1);
        assert!(has_passage);
    }
}
#[test]
fn verified_trait_labels_use_selected_locale_and_english_fallback_without_guessing_unknowns() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("static/lang/en.json"),
        r#"{"PF2E":{"TraitUndead":"Undead","TraitAcid":"Acid","TraitFire":"Unused"}}"#,
    )
    .unwrap();
    fs::write(
        fixture.0.join("static/lang/fr.json"),
        r#"{"PF2E":{"TraitUndead":"Mort-vivant"}}"#,
    )
    .unwrap();
    fs::write(fixture.0.join("packs/items/useful.json"),r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Useful record","type":"effect","system":{"traits":{"value":["undead","acid","future-homebrew"]}}}"#).unwrap();
    let mut options = fixture.options();
    options.locale = "fr".into();
    build_artifact(options.clone()).unwrap();
    let reader = atlas_index::test_support::open_reader(&options.output_path).unwrap();
    assert_eq!(
        reader.context().used_trait_labels,
        std::collections::BTreeMap::from([
            ("acid".into(), "Acid".into()),
            ("undead".into(), "Mort-vivant".into())
        ])
    );
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    let vocabulary: String = db
        .query_row(
            "SELECT structured_terms FROM lexical_units WHERE unit_kind='root_name'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(vocabulary.contains("future-homebrew"));
    assert!(vocabulary.contains("Mort-vivant"));
    assert!(vocabulary.contains("Acid"));
    assert!(!vocabulary.contains("Unused"));
}
#[test]
fn lexical_build_preserves_partial_records_and_keeps_macros_developer_only() {
    let fixture = Fixture::new();
    let report = build_artifact(fixture.options()).unwrap();
    assert_eq!(report.record_count, 2);
    assert_eq!(report.product_record_count, 1);
    assert_eq!(report.semantic_unit_count, 0);
    assert_eq!(report.source_report.quarantined_files, 1);
    assert!(report.source_report.partial_documents > 0);
    let database = rusqlite::Connection::open(&report.output_path).unwrap();
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM records", [], |r| r.get::<_, usize>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM search_profiles", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        database
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='semantic_vectors'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(database.query_row("SELECT count(*) FROM lexical_units u JOIN records r USING(record_id) WHERE r.document_kind='Macro'",[],|r|r.get::<_,usize>(0)).unwrap(),0);
    let definitions: String = database
        .query_row(
            "SELECT definition_terms FROM lexical_units WHERE unit_kind='heading'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(definitions, "Defined Affliction");
    let content: usize = database
        .query_row(
            "SELECT count(*) FROM prepared_content WHERE outcome='prepared'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(content > 0);
    let validation = atlas_index::validate_artifact(&report.output_path).unwrap();
    assert_eq!(validation.roots, 2);
}
#[test]
fn fingerprint_tracks_bytes_bad_files_definitions_rename_add_remove_and_locale() {
    let fixture = Fixture::new();
    let fingerprint = || compute_source_fingerprint(&fixture.0, None).unwrap().value;
    let initial = fingerprint();
    fs::write(fixture.0.join("packs/items/bad.json"), "{different bad").unwrap();
    let bad = fingerprint();
    assert_ne!(initial, bad);
    fs::rename(
        fixture.0.join("packs/items/bad.json"),
        fixture.0.join("packs/items/renamed.json"),
    )
    .unwrap();
    let renamed = fingerprint();
    assert_ne!(bad, renamed);
    fs::write(fixture.0.join("packs/items/_folders.json"), "[]").unwrap();
    let added = fingerprint();
    assert_ne!(renamed, added);
    fs::remove_file(fixture.0.join("packs/items/_folders.json")).unwrap();
    assert_eq!(fingerprint(), renamed);
    fs::write(fixture.0.join("static/lang/en.json"), "{}").unwrap();
    assert_ne!(fingerprint(), renamed);
    let manifest = fs::read_to_string(fixture.0.join("module.json")).unwrap();
    let before = fingerprint();
    fs::write(
        fixture.0.join("module.json"),
        manifest.replace("Items", "Changed label"),
    )
    .unwrap();
    assert_ne!(fingerprint(), before);
}
#[test]
fn indexing_locale_falls_back_to_english_and_missing_catalog_preserves_artifact() {
    use atlas_record::source_content::LocalizationResolver;
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("static/lang/fr.json"),
        r#"{"PF2E":{"Label":"French"}}"#,
    )
    .unwrap();
    let catalog =
        crate::source::localization::LocalizationCatalog::load(&fixture.0, "fr", None).unwrap();
    assert_eq!(catalog.localized_value("PF2E.Label"), Some("French"));
    assert_eq!(
        catalog.localized_value("PF2E.Fallback"),
        Some("English fallback")
    );
    assert_ne!(catalog.locale_sha256, catalog.english_sha256);
    let first = build_artifact(fixture.options()).unwrap();
    let before = atlas_index::validate_artifact(&first.output_path).unwrap();
    let mut options = fixture.options();
    options.locale = "unknown".into();
    request_test_configuration(&options);
    let error = build_artifact(options).unwrap_err().to_string();
    assert!(error.contains("unknown.json"), "{error}");
    assert_eq!(
        atlas_index::validate_artifact(first.output_path).unwrap(),
        before
    );
    assert!(
        crate::source::localization::LocalizationCatalog::load(&fixture.0, "../en", None).is_err()
    );
}

#[test]
fn english_catalog_changes_reprepare_unchanged_documents_and_failures_preserve_publication() {
    use atlas_record::source_record::SourceContentStatus;
    let fixture = Fixture::new();
    let manifest_path = fixture.0.join("module.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["languages"] = serde_json::json!([
        {"lang":"en","path":"lang/en.json"},
        {"lang":"en","path":"lang/re-en.json"}
    ]);
    fs::write(&manifest_path, manifest.to_string()).unwrap();
    let rules = fixture.0.join("static/lang/re-en.json");
    fs::write(
        &rules,
        r#"{"Rules":"<h2>Original rules</h2><p>Original English passage.</p>"}"#,
    )
    .unwrap();
    let document = fixture.0.join("packs/items/useful.json");
    let mut authored: serde_json::Value =
        serde_json::from_slice(&fs::read(&document).unwrap()).unwrap();
    authored["system"]["description"]["value"] = serde_json::json!("@Localize[Rules]");
    fs::write(&document, authored.to_string()).unwrap();
    build_artifact(fixture.options()).unwrap();
    let original = crate::product_preparation::prepare_fixture(&fixture.options()).unwrap();
    let original_hash = original.spool.context().preparation_sha256().unwrap();
    drop(original);
    fs::write(
        &rules,
        r#"{"Rules":"<h2>Updated rules</h2><p>Updated English passage.</p>"}"#,
    )
    .unwrap();
    let updated = crate::product_preparation::prepare_fixture(&fixture.options()).unwrap();
    assert_eq!(updated.source_report.reused_admissions, 2);
    assert_eq!(updated.source_report.reused_preparations, 0);
    assert_ne!(
        updated.spool.context().preparation_sha256().unwrap(),
        original_hash
    );
    let mut checked = 0;
    updated
        .spool
        .records(&mut |frame| {
            for outcome in &frame.source.content {
                if let SourceContentStatus::Prepared(content) = &outcome.status
                    && content.locator.field == "/system/description/value"
                {
                    assert!(content.text.contains("Updated English passage."));
                    assert!(!content.html.contains("Original English passage."));
                    assert!(!content.html.contains("@Localize"));
                    checked += 1;
                }
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(checked, 1);
    let published = build_artifact(fixture.options()).unwrap();
    let before = atlas_index::validate_artifact(&published.output_path).unwrap();
    fs::remove_file(rules).unwrap();
    assert!(build_artifact(fixture.options()).is_err());
    assert_eq!(
        atlas_index::validate_artifact(published.output_path).unwrap(),
        before
    );
}
#[test]
#[ignore = "requires the pinned full PF2e source export; run explicitly"]
fn actual_complete_source_build_has_typed_projection_content_and_navigation_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let path = ancestor.join("scratch/source-contracts-full/pf2e");
            path.join("static/lang/en.json").is_file().then_some(path)
        })
        .unwrap();
    let evidence =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scratch/ingest-validation");
    fs::create_dir_all(&evidence).unwrap();
    let report = build_artifact(BuildArtifactOptions {
        source_root: root,
        output_path: evidence.join("full-lexical.sqlite"),
        manifest_path: None,
        locale: "en".into(),
        search_profiles: vec![],
        reuse_embeddings: false,
        reuse_passage_layouts: false,
        embedding_batch_size: 16,
    })
    .unwrap();
    assert_eq!(report.record_count, 29894);
    assert_eq!(report.product_record_count, 29813);
    assert_eq!(report.source_report.quarantined_files, 0);
    assert!(report.source_report.identity_unavailable.is_empty());
    let validation = atlas_index::validate_artifact(&report.output_path).unwrap();
    assert_eq!(validation.checked_snapshots, 29894);
    let reader = atlas_index::test_support::open_reader(&report.output_path).unwrap();
    fs::write(evidence.join("full-lexical-census.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":report.record_count,"product_records":report.product_record_count,"source_report":report.source_report,"validation":validation,"duration_ms":report.build_duration_ms,"source_fingerprint":report.source_fingerprint,"context":reader.context()})).unwrap()).unwrap();
    eprintln!(
        "full source artifact validated at {}",
        report.output_path.display()
    );
}
#[test]
#[ignore = "requires the pinned source export and completed full lexical artifact; run explicitly"]
fn complete_artifact_snapshots_equal_independently_reloaded_source_models() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let path = ancestor.join("scratch/source-contracts-full/pf2e");
            path.join("static/lang/en.json").is_file().then_some(path)
        })
        .unwrap();
    let evidence =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scratch/ingest-validation");
    let reader =
        atlas_index::test_support::open_reader(evidence.join("full-lexical.sqlite")).unwrap();
    let fingerprint = compute_source_fingerprint(&source_root, None).unwrap();
    assert_eq!(reader.context().source_fingerprint, fingerprint.value);
    let loaded = load_foundry_documents(&source_root, None).unwrap();
    let mut roots = 0;
    let mut macros = 0;
    let mut partial = 0;
    for pack in loaded.packs {
        assert!(pack.discovery_failure.is_none());
        assert!(pack.quarantined_files.is_empty());
        for document in pack.documents {
            partial += usize::from(!document.admission.diagnostics.is_empty());
            macros += usize::from(document.provenance.document_type == "Macro");
            let expected = atlas_record::source_record::SourceBackedRecord::new(
                &pack.metadata.name,
                document.admission.model.unwrap(),
            )
            .unwrap();
            let actual = reader
                .read_source_record_for_inspection(expected.key())
                .unwrap()
                .unwrap();
            assert!(
                actual == expected,
                "artifact/source mismatch for {}",
                expected.key()
            );
            roots += 1;
        }
    }
    assert_eq!(roots, 29894);
    assert_eq!(macros, 81);
    assert_eq!(reader.statistics().unwrap().records, roots);
    fs::write(
        evidence.join("full-snapshot-source-equality.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "compared_roots": roots,
            "developer_macros": macros,
            "partial_source_documents": partial,
            "mismatches": 0,
            "source_fingerprint": fingerprint.value,
            "context": reader.context(),
            "basis": "Independently reload and admit original source files, then compare every typed model including retained invalid/additional data with its public checked artifact reader result."
        })).unwrap(),
    ).unwrap();
}
#[test]
#[ignore = "requires verified local BGE assets; run explicitly"]
fn actual_model_build_reuses_exact_inputs_with_fresh_source_attribution() {
    let fixture = Fixture::new();
    let cache = native_smoke_cache(atlas_embedding::EmbeddingModelId::BgeBaseEnV15);
    let mut options = fixture.options();
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        cache,
    )];
    let first = build_artifact(options.clone()).unwrap();
    assert!(first.inferred_inputs > 0);
    assert_eq!(first.reused_inputs, 0);
    let second = build_artifact(options.clone()).unwrap();
    assert_eq!(second.inferred_inputs, 0);
    assert_eq!(second.reused_inputs, first.inferred_inputs);
    assert_eq!(first.semantic_unit_count, second.semantic_unit_count);
    let path = fixture.0.join("packs/items/useful.json");
    let value = fs::read_to_string(&path).unwrap();
    fs::write(path, value.replace("Full prose", "Changed full prose")).unwrap();
    let changed = build_artifact(options.clone()).unwrap();
    assert_ne!(first.source_fingerprint, changed.source_fingerprint);
    assert_eq!(changed.inferred_inputs, 1);
    assert!(changed.reused_inputs > 0);
    assert_eq!(changed.semantic_unit_count, first.semantic_unit_count);
    atlas_index::validate_artifact(&changed.output_path).unwrap();
    // An independently addressable root with identical selected input must reuse
    // the existing values without merging either root's filtered witnesses.
    let mut duplicate: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.0.join("packs/items/useful.json")).unwrap(),
    )
    .unwrap();
    duplicate["_id"] = serde_json::json!("cccccccccccccccc");
    fs::write(
        fixture.0.join("packs/items/duplicate.json"),
        serde_json::to_vec(&duplicate).unwrap(),
    )
    .unwrap();
    let shared = build_artifact(options.clone()).unwrap();
    assert_eq!(shared.inferred_inputs, 0);
    assert_eq!(shared.reused_inputs, first.inferred_inputs);
    assert_eq!(shared.semantic_unit_count, changed.semantic_unit_count * 2);
    let reader = atlas_index::test_support::open_reader(&shared.output_path).unwrap();
    let profile = atlas_domain::SearchProfileId::new("bge-base");
    let original = atlas_domain::RecordKey::parse("items:aaaaaaaaaaaaaaaa").unwrap();
    let other = atlas_domain::RecordKey::parse("items:cccccccccccccccc").unwrap();
    let vector = reader
        .seed_identity_vector(&profile, &original)
        .unwrap()
        .unwrap()
        .vector;
    let all = atlas_index::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for root in [original, other] {
        let hits = reader
            .vector_candidates(atlas_index::SourceVectorCandidateRequest {
                profile: &profile,
                vector: &vector,
                query: &all,
                keys: Some(std::slice::from_ref(&root)),
                exclude: None,
                content_scope: atlas_domain::SearchContentScope::All,
                limit: 100,
            })
            .unwrap();
        assert_eq!(hits.len(), changed.semantic_unit_count);
        for hit in hits {
            assert_eq!(hit.location.record, root);
            assert!(ids.insert(hit.location.unit_id));
            if hit.location.field.is_some() {
                assert!(reader.recover_passage(&hit.location).unwrap().is_some());
            }
        }
    }
    drop(reader);
    options.reuse_embeddings = false;
    let forced = build_artifact(options).unwrap();
    assert_eq!(forced.reused_inputs, 0);
    assert_eq!(forced.inferred_inputs, first.inferred_inputs);
    atlas_index::validate_artifact(&forced.output_path).unwrap();
}

#[test]
#[ignore = "requires verified local BGE Base/Gemma assets; bounded tiny source only"]
fn actual_profile_builds_share_preparation_isolate_reuse_and_publish_atomically() {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};
    let cache = native_smoke_cache(EmbeddingModelId::EmbeddingGemma300M);
    let fixture = Fixture::new();
    let mut options = fixture.options();
    options.search_profiles = vec![EmbeddingRuntimeConfig::new(
        EmbeddingModelId::BgeBaseEnV15,
        &cache,
    )];
    let base = build_artifact(options.clone()).unwrap();
    assert_eq!(base.profiles.len(), 1);
    let first = atlas_index::test_support::open_reader(&options.output_path).unwrap();
    let prep = first.context().preparation_sha256().unwrap();
    let generation = first.context().sha256().unwrap();
    drop(first);
    options.search_profiles.push(EmbeddingRuntimeConfig::new(
        EmbeddingModelId::EmbeddingGemma300M,
        &cache,
    ));
    request_test_configuration(&options);
    // The second profile has a valid tokenizer but no inference weights. Shared
    // admission and ALL native validation must complete before BGE starts, then
    // BGE checkpoints must survive the later Gemma inference failure.
    let source_path = fixture.0.join("packs/items/useful.json");
    let source = fs::read_to_string(&source_path).unwrap();
    fs::write(
        &source_path,
        source.replace("Full prose", "Changed full prose"),
    )
    .unwrap();
    let partial = EmbeddingRuntimeConfig::new(
        EmbeddingModelId::EmbeddingGemma300M,
        fixture.0.join("gemma-tokenizer-only"),
    );
    fs::create_dir_all(partial.model_dir()).unwrap();
    fs::copy(
        options.search_profiles[1]
            .model_dir()
            .join("tokenizer.json"),
        partial.model_dir().join("tokenizer.json"),
    )
    .unwrap();
    let valid_gemma = options.search_profiles[1].clone();
    options.search_profiles[1] = partial;
    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path).unwrap();
    let schema = atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    let revision = workspace
        .read(schema)
        .unwrap()
        .reference_revision()
        .unwrap();
    assert!(build_artifact(options.clone()).is_err());
    assert_eq!(
        workspace
            .read(schema)
            .unwrap()
            .reference_revision()
            .unwrap(),
        revision
    );
    assert_eq!(
        atlas_index::test_support::open_reader(&options.output_path)
            .unwrap()
            .context()
            .sha256()
            .unwrap(),
        generation
    );
    {
        let read = workspace.read(schema).unwrap();
        let pending: usize = read
            .connection()
            .query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(pending > 0, "completed BGE work survives Gemma failure");
        let models: usize = read.connection().query_row("SELECT count(DISTINCT i.model_id) FROM pending_embedding_values p JOIN embedding_inputs i USING(input_id)", [], |r|r.get(0)).unwrap();
        assert_eq!(
            models, 1,
            "only the completed BGE work is cached before Gemma inference fails"
        );
    }
    options.search_profiles[1] = valid_gemma;
    let both = build_artifact(options.clone()).unwrap();
    assert_eq!(both.profiles[0].inferred_inputs, 0);
    assert!(both.profiles[0].reused_inputs > 0);
    assert!(both.profiles[1].inferred_inputs > 0);
    assert_eq!(both.semantic_unit_count, base.semantic_unit_count * 2);
    let reader = atlas_index::test_support::open_reader(&options.output_path).unwrap();
    assert_eq!(reader.context().preparation_sha256().unwrap(), prep);
    assert_ne!(reader.context().sha256().unwrap(), generation);
    assert_eq!(reader.statistics().unwrap().prepared_fields, 1);
    drop(reader);
    let before = atlas_index::validate_artifact(&options.output_path).unwrap();
    options.search_profiles[1].cache_root = fixture.0.join("missing-gemma-cache");
    assert!(build_artifact(options.clone()).is_err());
    assert_eq!(
        atlas_index::validate_artifact(&options.output_path).unwrap(),
        before
    );
    // Gemma-only needs the small shared BGE tokenizer dependency, not BGE weights.
    let gemma_target = gemma_only_cache(&cache, &fixture.0);
    options.search_profiles = vec![gemma_target];
    request_test_configuration(&options);
    let gemma = build_artifact(options).unwrap();
    assert_eq!(gemma.profiles.len(), 1);
    assert_eq!(gemma.profiles[0].profile_id.as_str(), "embeddinggemma");
    assert_eq!(gemma.inferred_inputs, 0);
    assert!(gemma.reused_inputs > 0);
    assert_eq!(gemma.semantic_unit_count, base.semantic_unit_count);
    atlas_index::validate_artifact(&gemma.output_path).unwrap();
    eprintln!(
        "tiny native builds: Base-only -> both (Base reused) -> missing Gemma failure preserves published reference -> Gemma-only without BGE weights (Gemma reused)"
    );
}

// Requested configuration is committed before a build, just as runtime setup does.
fn request_test_configuration(options: &BuildArtifactOptions) {
    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path).unwrap();
    let _owner = workspace.update_owner().unwrap();
    let session = workspace
        .write(atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    atlas_index::write_requested_reference_config(
        &session,
        &atlas_index::RequestedReferenceConfig {
            source_root: options.source_root.clone(),
            manifest_path: options.manifest_path.clone(),
            indexing_locale: options.locale.clone(),
            search_profiles: options
                .search_profiles
                .iter()
                .map(|config| atlas_embedding::profile_for_model(config.model).profile_id())
                .collect(),
        },
    )
    .unwrap();
    session.commit().unwrap();
}

fn gemma_only_cache(cache: &Path, fixture_root: &Path) -> atlas_embedding::EmbeddingRuntimeConfig {
    use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig};
    let isolated = fixture_root.join("gemma-only-cache");
    let base_token = atlas_embedding::baseline_tokenizer_cache_file(&isolated);
    fs::create_dir_all(base_token.local_path.parent().unwrap()).unwrap();
    fs::copy(
        atlas_embedding::baseline_tokenizer_cache_file(cache).local_path,
        &base_token.local_path,
    )
    .unwrap();
    let gemma_source = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, cache);
    let gemma_target = EmbeddingRuntimeConfig::new(EmbeddingModelId::EmbeddingGemma300M, &isolated);
    for file in atlas_embedding::required_embedding_model_cache_files(&gemma_target) {
        fs::create_dir_all(file.local_path.parent().unwrap()).unwrap();
        fs::hard_link(
            gemma_source.model_dir().join(file.source_path),
            &file.local_path,
        )
        .unwrap();
    }
    assert!(
        !isolated
            .join(format!(
                "BAAI/bge-base-en-v1.5/{}/onnx/model.onnx",
                atlas_embedding::embedding_model_spec(
                    atlas_embedding::EmbeddingModelId::BgeBaseEnV15
                )
                .model_revision
            ))
            .exists()
    );
    gemma_target
}

#[test]
#[ignore = "requires verified local Gemma assets; bounded fresh tiny source only"]
fn actual_gemma_only_fresh_build_needs_baseline_tokenizer_but_no_base_weights() {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .map(|p| p.join(".cache/hf-models"))
        .find(|p| {
            p.join(format!(
                "onnx-community/embeddinggemma-300m-ONNX/{}/onnx/model.onnx_data",
                atlas_embedding::embedding_model_spec(
                    atlas_embedding::EmbeddingModelId::EmbeddingGemma300M
                )
                .model_revision
            ))
            .is_file()
        })
        .unwrap();
    let fixture = Fixture::new();
    let mut options = fixture.options();
    options.search_profiles = vec![gemma_only_cache(&cache, &fixture.0)];
    options.reuse_embeddings = false;
    let report = build_artifact(options).unwrap();
    assert_eq!(report.profiles.len(), 1);
    assert_eq!(report.profiles[0].profile_id.as_str(), "embeddinggemma");
    assert!(report.inferred_inputs > 0);
    assert_eq!(report.reused_inputs, 0);
    let reader = atlas_index::test_support::open_reader(&report.output_path).unwrap();
    assert_eq!(reader.context().search_profiles.len(), 1);
    assert_eq!(
        reader.context().search_profiles[0].profile_id.as_str(),
        "embeddinggemma"
    );
    atlas_index::validate_artifact(&report.output_path).unwrap();
    eprintln!(
        "fresh Gemma-only without BGE weights: inferred={}, reused={}, semantic_units={}",
        report.inferred_inputs, report.reused_inputs, report.semantic_unit_count
    );
}
#[test]
#[ignore = "requires verified local BGE assets; run explicitly"]
fn actual_model_reuse_rejects_corrupted_published_input_metadata() {
    // Snapshot public logical rows, including native vec0/FTS rows; extension
    // shadow tables are deliberately excluded. This checks the poisoned state,
    // rather than misreporting it as a healthy artifact after failed admission.
    fn state(
        db: &rusqlite::Connection,
    ) -> std::collections::BTreeMap<String, Vec<Vec<rusqlite::types::Value>>> {
        let tables = db.prepare("SELECT name,wr FROM pragma_table_list WHERE schema='main' AND type IN ('table','virtual') AND (name NOT LIKE 'sqlite_%' OR name='sqlite_sequence') AND (name NOT GLOB 'native_vectors_*' OR type='virtual') AND (name NOT GLOB 'lexical_fts*' OR name='lexical_fts') ORDER BY name").unwrap()
            .query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,bool>(1)?))).unwrap()
            .collect::<Result<Vec<_>,_>>().unwrap();
        tables
            .into_iter()
            .map(|(table, without_rowid)| {
                let quoted = table.replace('"', "\"\"");
                let columns = if without_rowid { "*" } else { "rowid,*" };
                let select = format!("SELECT {columns} FROM \"{quoted}\"");
                let count = db.prepare(&select).unwrap().column_count();
                let order = (1..=count)
                    .map(|column| column.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let mut statement = db.prepare(&format!("{select} ORDER BY {order}")).unwrap();
                let values = statement
                    .query_map([], |row| (0..count).map(|column| row.get(column)).collect())
                    .unwrap()
                    .collect::<Result<_, _>>()
                    .unwrap();
                (table, values)
            })
            .collect()
    }
    let fixture = Fixture::new();
    let cache = native_smoke_cache(atlas_embedding::EmbeddingModelId::BgeBaseEnV15);
    let mut options = fixture.options();
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        cache,
    )];
    build_artifact(options.clone()).unwrap();
    atlas_index::validate_artifact(&options.output_path).unwrap();
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    db.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(71,'kept','kept','Saved native proof','before','after'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(71,'items:aaaaaaaaaaaaaaaa',0,'Keep this note','Useful record','before','after');").unwrap();
    let healthy = state(&db);
    let changed = db
        .execute("UPDATE embedding_inputs SET token_count=token_count+1", [])
        .unwrap();
    assert!(changed > 0);
    let poisoned = state(&db);
    assert_ne!(poisoned["embedding_inputs"], healthy["embedding_inputs"]);
    for (table, values) in &healthy {
        if table != "embedding_inputs" {
            assert_eq!(&poisoned[table], values, "corruption touched {table}");
        }
    }
    drop(db);
    // A mistaken cache miss would try to load unavailable weights. Requiring the
    // matched-input token error proves rejection precedes inference and publish.
    let original = &options.search_profiles[0];
    let tokenizer_only = atlas_embedding::EmbeddingRuntimeConfig::new(
        original.model,
        fixture.0.join("tokenizer-only-rejection"),
    );
    fs::create_dir_all(tokenizer_only.model_dir()).unwrap();
    fs::copy(
        original.model_dir().join("tokenizer.json"),
        tokenizer_only.model_dir().join("tokenizer.json"),
    )
    .unwrap();
    assert!(!tokenizer_only.model_dir().join("onnx/model.onnx").exists());
    options.search_profiles = vec![tokenizer_only];
    let error = build_artifact(options.clone()).unwrap_err().to_string();
    assert!(
        error.contains("native input token count mismatch"),
        "{error}"
    );
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    assert_eq!(
        state(&db),
        poisoned,
        "failed reuse changed the already-poisoned registry, native values, reference or saved state"
    );
}

#[test]
fn verified_name_aliases_never_invent_legacy_record_pairs() {
    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &to.join(entry.file_name()));
            } else {
                let mut text = fs::read_to_string(entry.path()).unwrap();
                // The historical fixture predates checked sixteen-character IDs.
                for (before, after) in [
                    ("attackOpportunity1", "aaaaaaaaaaaaaaaa"),
                    ("reactiveStrike1", "bbbbbbbbbbbbbbbb"),
                    ("offGuard1", "AJh5ex99aV6VTggg"),
                    ("flatFooted1", "dddddddddddddddd"),
                    ("aliasCarrier1", "eeeeeeeeeeeeeeee"),
                    ("journal1", "ffffffffffffffff"),
                    ("embeddedLegacy1", "gggggggggggggggg"),
                    ("page1", "hhhhhhhhhhhhhhhh"),
                ] {
                    text = text.replace(before, after);
                }
                fs::write(to.join(entry.file_name()), text).unwrap();
            }
        }
    }
    let fixture = Fixture::new();
    copy_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/foundry-source/remaster-links"),
        &fixture.0,
    );
    let report = build_artifact(fixture.options()).unwrap();
    let database = rusqlite::Connection::open(&report.output_path).unwrap();
    assert!(
        database
            .query_row(
                "SELECT count(*) FROM remaster_aliases WHERE alias='Attack of Opportunity'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap()
            > 0
    );
    assert!(
        database
            .query_row(
                "SELECT count(*) FROM remaster_aliases WHERE alias='Flat-Footed'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap()
            > 0
    );
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM remaster_preferences", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    let journal = fixture.0.join("packs/journals/remaster-changes.json");
    let text = fs::read_to_string(&journal).unwrap();
    fs::write(
        journal,
        text.replace(
            "<td>Attack of Opportunity</td>",
            "<td>@UUID[Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa]{Attack of Opportunity}</td>",
        ),
    )
    .unwrap();
    let explicit = build_artifact(fixture.options()).unwrap();
    let database = rusqlite::Connection::open(explicit.output_path).unwrap();
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM remaster_preferences", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
    );
    drop(database);
    let reopened = atlas_index::test_support::open_reader(&report.output_path).unwrap();
    let information = reopened
        .remaster_record_information(&"actions:aaaaaaaaaaaaaaaa".parse().unwrap(), &[], 20)
        .unwrap();
    assert_eq!(information.declarations.len(), 1);
    let provenance = information.declarations[0].citations[0]
        .locator
        .clone()
        .unwrap();
    assert_eq!(provenance.record.to_string(), "journals:ffffffffffffffff");
    assert_eq!(provenance.field, "/text/content");
    assert!(
        reopened
            .read_source_record(&provenance.record)
            .unwrap()
            .unwrap()
            .authored_content_at(&provenance)
            .value()
            .unwrap()
            .contains("@UUID")
    );
    drop(reopened);
    // Model the published v6 component: same checked source and opaque journal evidence.
    atlas_index::test_support::set_relationship_policy(
        &report.output_path,
        atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        "typed-authored-occurrences/v6",
    )
    .unwrap();
    let before = rusqlite::Connection::open(&report.output_path).unwrap();
    before.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,description,created_at,updated_at) VALUES(1,'saved','saved','Saved editions','Retain notes','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actions:aaaaaaaaaaaaaaaa',0,'Legacy reference','Attack of Opportunity','now','now'); INSERT INTO encounters(id,encounter_key,slug,name,note,status,created_at,updated_at,revision) VALUES(1,'encounter','encounter','Saved encounter','Keep this note','running','now','now',4);").unwrap();
    let identity: i64 = before
        .query_row(
            "SELECT record_id FROM records WHERE key='actions:aaaaaaaaaaaaaaaa'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let workspace: String = before
        .query_row("SELECT workspace_key FROM workspace", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        atlas_index::classify_reference_upgrade(&before).unwrap(),
        atlas_index::ReferenceUpgrade::SourcePreparation
    );
    assert!(atlas_index::test_support::open_reader(&report.output_path).is_err());
    drop(before);
    let upgraded = build_artifact(fixture.options()).unwrap();
    let reopened = atlas_index::test_support::open_reader(&upgraded.output_path).unwrap();
    assert_eq!(
        reopened
            .remaster_record_information(&"actions:aaaaaaaaaaaaaaaa".parse().unwrap(), &[], 20)
            .unwrap()
            .declarations[0]
            .citations[0]
            .locator
            .clone()
            .unwrap(),
        provenance
    );
    assert_eq!(reopened.artifact_instance().workspace_key(), workspace);
    let c = rusqlite::Connection::open(&upgraded.output_path).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT record_id FROM records WHERE key='actions:aaaaaaaaaaaaaaaa'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        identity
    );
    let saved: (String,String,String,String,i64) = c.query_row("SELECT l.name,l.description,i.note,e.note,e.revision FROM saved_lists l JOIN saved_list_items i ON i.list_id=l.id CROSS JOIN encounters e", [], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        saved,
        (
            "Saved editions".into(),
            "Retain notes".into(),
            "Legacy reference".into(),
            "Keep this note".into(),
            4
        )
    );
}
#[test]
#[ignore = "requires pinned full source and verified local tokenizer; run explicitly"]
fn actual_complete_source_semantic_preparation_covers_every_selected_body_and_identity() {
    let started = std::time::Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let root = ancestor.join("scratch/source-contracts-full/pf2e");
            root.join("static/lang/en.json").is_file().then_some(root)
        })
        .unwrap();
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let cache = ancestor.join(".cache/hf-models");
            cache
                .join(format!(
                    "BAAI/bge-base-en-v1.5/{}/onnx/model.onnx",
                    atlas_embedding::embedding_model_spec(
                        atlas_embedding::EmbeddingModelId::BgeBaseEnV15
                    )
                    .model_revision
                ))
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let options = BuildArtifactOptions {
        source_root: root,
        output_path: temporary.path().join("preparation.sqlite"),
        manifest_path: None,
        locale: "en".into(),
        search_profiles: vec![atlas_embedding::EmbeddingRuntimeConfig::new(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
            cache,
        )],
        reuse_embeddings: false,
        reuse_passage_layouts: false,
        embedding_batch_size: 16,
    };
    let prepared = crate::product_preparation::prepare_fixture(&options).unwrap();
    assert_eq!(prepared.spool.record_count(), 29894);
    let mut identities = std::collections::BTreeSet::new();
    let mut bytes = 0;
    let mut tokens = 0;
    let mut max_tokens = 0;
    let mut coverage = std::collections::BTreeMap::new();
    let mut document_units = 0usize;
    prepared
        .native_inputs
        .visit(&mut |frame| {
            let crate::product_preparation::RawOccurrence::Document { attribution } =
                frame.occurrence
            else {
                return Ok(());
            };
            document_units += 1;
            assert!(frame.input.token_count <= 512);
            tokens += frame.input.token_count;
            max_tokens = max_tokens.max(frame.input.token_count);
            let range = match &attribution.address {
                atlas_domain::SourcePassageAddress::Identity {} => {
                    assert!(identities.insert(attribution.record));
                    return Ok(());
                }
                atlas_domain::SourcePassageAddress::HtmlSection {
                    section_ordinal,
                    chunk_bytes,
                    ..
                }
                | atlas_domain::SourcePassageAddress::PlainSection {
                    section_ordinal,
                    chunk_bytes,
                    ..
                } => (*section_ordinal, *chunk_bytes),
            };
            let key = (
                attribution.record,
                serde_json::to_string(&attribution.owners).unwrap(),
                attribution.field,
                range.0,
            );
            let end = coverage.entry(key).or_insert(0);
            assert!(range.1.start <= *end);
            assert!(range.1.end > *end);
            bytes += range.1.end - *end;
            *end = range.1.end;
            Ok(())
        })
        .unwrap();
    assert_eq!(document_units, prepared.document_units);
    assert_eq!(identities.len(), 29813);
    let mut signatures = crate::passage_benchmark::signatures(&prepared);
    let mut aliases = std::collections::BTreeMap::<_, Vec<String>>::new();
    prepared
        .spool
        .lexical
        .visit(&mut |alias| {
            if alias.kind == atlas_index::SourceLexicalUnitKind::RootAlias {
                aliases
                    .entry(alias.record)
                    .or_default()
                    .push(alias.alias_terms);
            }
            Ok(())
        })
        .unwrap();
    let tokenizer = atlas_embedding::TextEmbeddingTokenizer::load_baseline(
        &options.search_profiles[0].cache_root,
    )
    .unwrap();
    let mut selected_fields = 0;
    let mut selected_sections = 0;
    let mut selected_bytes = 0;
    prepared
        .spool
        .records(&mut |frame| {
            let input = frame.source;
            let pack = prepared
                .spool
                .packs()
                .iter()
                .find(|pack| pack.pack_id == input.record.key().pack().as_str())
                .unwrap();
            let mut selected = atlas_record::source_record::select_record_search(
                &input.record,
                &input.content,
                prepared.spool.context().audience,
                &pack.label,
                &prepared.spool.context().used_trait_labels,
            )
            .unwrap();
            selected
                .identity_optional
                .extend(aliases.remove(input.record.key()).unwrap_or_default());
            assert_eq!(
                crate::passage_benchmark::fresh_record_signature(&selected, &tokenizer),
                signatures.remove(input.record.key()),
                "{}",
                input.record.key()
            );
            selected_fields += selected.fields.len();
            for field in &selected.fields {
                for section in &field.sections {
                    selected_sections += 1;
                    selected_bytes += section.text.len();
                    let key = (
                        selected.root.clone(),
                        serde_json::to_string(&field.locator.owners).unwrap(),
                        Some(field.locator.field.clone()),
                        section.section_ordinal,
                    );
                    assert_eq!(
                        coverage.remove(&key),
                        Some(section.text.len()),
                        "{} {} {}",
                        selected.root,
                        field.locator.field,
                        section.section_ordinal
                    );
                }
            }
            Ok(())
        })
        .unwrap();
    assert!(signatures.is_empty());
    assert!(aliases.is_empty());
    assert!(coverage.is_empty());
    assert_eq!(bytes, selected_bytes);
    let evidence =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scratch/ingest-validation");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(evidence.join("full-semantic-preparation-census.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":prepared.spool.record_count(),"identity_units":identities.len(),"selected_fields":selected_fields,"selected_sections":selected_sections,"selected_body_bytes":selected_bytes,"prepared_inputs":document_units,"exact_input_tokens":tokens,"maximum_input_tokens":max_tokens,"context_shortened":prepared.context_shortened,"policy_version":atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION,"source_fingerprint":prepared.fingerprint.value,"context_sha256":prepared.spool.context().sha256().unwrap(),"content_interpretation_version":prepared.spool.context().content_interpretation_version,"content_selection_version":prepared.spool.context().content_selection_version,"relationship_policy":prepared.spool.context().relationship_policy,"duration_ms":started.elapsed().as_millis(),"inference_performed":false})).unwrap()).unwrap();
}

#[test]
fn incremental_preparation_reuses_admission_and_replays_missing_and_renamed_targets() {
    let fixture = Fixture::new();
    let path = fixture.0.join("packs/items/useful.json");
    let mut source: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    source["system"]["description"]["value"] = serde_json::json!(
        "<p>@UUID[Compendium.pf2e.items.Item.cccccccccccccccc] @UUID[Compendium.pf2e.items.Item.Target]</p>"
    );
    fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    let check_targets = |names: [Option<&str>; 2]| {
        use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
        let reader = atlas_index::test_support::open_reader(fixture.options().output_path).unwrap();
        let references = reader
            .read_relationships(&atlas_index::SourceRelationshipRequest {
                record: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
                owners: Some(vec![]),
                field: Some("/system/description/value".into()),
                direction: atlas_index::SourceRelationshipDirection::Outgoing,
                limit: 8,
            })
            .unwrap();
        assert!(!references.truncated);
        assert_eq!(references.occurrences.len(), 2);
        for ((reference, target), name) in references
            .occurrences
            .iter()
            .zip([
                "Compendium.pf2e.items.Item.cccccccccccccccc",
                "Compendium.pf2e.items.Item.Target",
            ])
            .zip(names)
        {
            assert_eq!(reference.authored_target.as_deref(), Some(target));
            assert_eq!(
                reference
                    .details
                    .label
                    .as_ref()
                    .unwrap()
                    .target_name
                    .as_deref(),
                name
            );
            assert_eq!(
                reference.resolution,
                match name {
                    Some(_) =>
                        ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                            key: "items:cccccccccccccccc".parse().unwrap(),
                        }),
                    None => ContentReferenceResolution::Unresolved,
                }
            );
        }
    };
    let first = build_artifact(fixture.options()).unwrap();
    assert_eq!(first.source_report.reused_admissions, 0);
    check_targets([None, None]);
    let unchanged = build_artifact(fixture.options()).unwrap();
    assert_eq!(unchanged.source_report.reused_admissions, 2);
    assert_eq!(unchanged.source_report.reused_preparations, 2);
    assert_eq!(
        unchanged.source_report.admission_diagnostics,
        first.source_report.admission_diagnostics
    );
    let target = fixture.0.join("packs/items/target.json");
    fs::write(&target,r#"{"_id":"cccccccccccccccc","name":"Target","type":"effect","system":{"description":{"value":"<p>Target prose</p>"}}}"#).unwrap();
    let added = build_artifact(fixture.options()).unwrap();
    assert_eq!(added.source_report.reused_admissions, 2);
    // The referencing source is byte-identical, but its previously unresolved targets changed.
    assert_eq!(added.source_report.reused_preparations, 1);
    check_targets([Some("Target"), Some("Target")]);
    let all_unchanged = build_artifact(fixture.options()).unwrap();
    assert_eq!(all_unchanged.source_report.reused_preparations, 3);
    let duplicate = fixture.0.join("packs/items/duplicate-name.json");
    fs::write(&duplicate,r#"{"_id":"dddddddddddddddd","name":"Target","type":"effect","system":{"description":{"value":"<p>Another target</p>"}}}"#).unwrap();
    let ambiguous = build_artifact(fixture.options()).unwrap();
    assert_eq!(ambiguous.source_report.reused_admissions, 3);
    assert_eq!(ambiguous.source_report.reused_preparations, 2);
    check_targets([Some("Target"), None]);
    fs::remove_file(duplicate).unwrap();
    let unique_again = build_artifact(fixture.options()).unwrap();
    assert_eq!(unique_again.source_report.reused_preparations, 2);
    check_targets([Some("Target"), Some("Target")]);
    fs::write(&target,r#"{"_id":"cccccccccccccccc","name":"Renamed target","type":"effect","system":{"description":{"value":"<p>Target prose</p>"}}}"#).unwrap();
    let renamed = build_artifact(fixture.options()).unwrap();
    assert_eq!(renamed.source_report.reused_admissions, 2);
    // Stable-key display name changed, and the authored name lookup became unresolved.
    assert_eq!(renamed.source_report.reused_preparations, 1);
    check_targets([Some("Renamed target"), None]);
}

#[test]
fn changed_preparation_reuses_admission_without_consuming_obsolete_html() {
    let fixture = Fixture::new();
    let options = fixture.options();
    build_artifact(options.clone()).unwrap();
    let connection = rusqlite::Connection::open(&options.output_path).unwrap();
    assert!(
        connection
            .execute(
                "UPDATE prepared_content SET html_gzip=X'00' WHERE html_gzip IS NOT NULL",
                [],
            )
            .unwrap()
            > 0
    );
    assert!(crate::product_preparation::prepare_fixture(&options).is_err());
    fs::write(
        fixture.0.join("static/lang/en.json"),
        r#"{"PF2E":{"Label":"Changed catalog"}}"#,
    )
    .unwrap();
    let prepared = crate::product_preparation::prepare_fixture(&options).unwrap();
    assert_eq!(
        prepared.source_report.reused_admissions,
        prepared.spool.record_count()
    );
    assert_eq!(prepared.source_report.reused_preparations, 0);
}

#[test]
fn captured_document_change_is_fatal_even_when_original_bytes_return() {
    let fixture = Fixture::new();
    let capture = crate::source_fingerprint::SourceCapture::capture(&fixture.0, None).unwrap();
    let path = fixture.0.join("packs/items/useful.json");
    let original = fs::read(&path).unwrap();
    let checked = capture.read_verified(&path).unwrap();
    assert_eq!(checked.bytes, original);
    assert_eq!(checked.sha256, {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(&original))
    });
    fs::write(
        &path,
        r#"{"_id":"aaaaaaaaaaaaaaaa","name":"Transient","type":"effect"}"#,
    )
    .unwrap();
    let result = capture.read(&path);
    fs::write(&path, original).unwrap();
    assert_eq!(
        capture.fingerprint,
        compute_source_fingerprint(&fixture.0, None).unwrap()
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("source changed after capture")
    );
}

#[test]
fn captured_document_read_rejects_temporarily_missing_document() {
    let fixture = Fixture::new();
    let capture = crate::source_fingerprint::SourceCapture::capture(&fixture.0, None).unwrap();
    let path = fixture.0.join("packs/items/useful.json");
    let original = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    let result = capture.read(&path);
    fs::write(&path, original).unwrap();
    assert_eq!(
        capture.fingerprint,
        compute_source_fingerprint(&fixture.0, None).unwrap()
    );
    assert!(result.is_err());
}

#[test]
fn unconsumed_localization_and_alias_inputs_reject_transient_changes() {
    let fixture = Fixture::new();
    let migration = fixture.0.join("src/module/migration/migrations");
    fs::create_dir_all(&migration).unwrap();
    let alias_path = migration.join("850-flat-footed-to-off-guard.ts");
    fs::write(&alias_path, "// original").unwrap();
    for path in [fixture.0.join("static/lang/en.json"), alias_path] {
        let capture = crate::source_fingerprint::SourceCapture::capture(&fixture.0, None).unwrap();
        let original = fs::read(&path).unwrap();
        fs::write(&path, "{}").unwrap();
        let result = if path.ends_with("en.json") {
            crate::source::localization::LocalizationCatalog::load_captured(
                &fixture.0,
                "en",
                None,
                Some(&capture),
            )
            .map(|_| ())
        } else {
            crate::remaster_information::captured_inputs(&fixture.0, &capture, "fixture")
                .map(|_| ())
        };
        fs::write(&path, original).unwrap();
        assert_eq!(
            capture.fingerprint,
            compute_source_fingerprint(&fixture.0, None).unwrap()
        );
        assert!(
            result.is_err(),
            "{} must be checked against capture",
            path.display()
        );
    }
}

#[test]
fn aliased_pack_paths_reuse_only_the_published_pack_and_preserve_manifest_order() {
    let fixture = Fixture::new();
    build_artifact(fixture.options()).unwrap();
    fs::write(fixture.0.join("module.json"), r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"alias","label":"Alias","type":"Item","path":"packs/items"}]}"#).unwrap();
    let prepared = crate::product_preparation::prepare_fixture(&fixture.options()).unwrap();
    assert_eq!(prepared.source_report.reused_admissions, 1);
    assert_eq!(prepared.source_report.reused_preparations, 1);
    let mut keys = Vec::new();
    prepared
        .spool
        .records(&mut |frame| {
            assert_eq!(frame.source.source_path, "packs/items/useful.json");
            keys.push(frame.selection.root.to_string());
            Ok(())
        })
        .unwrap();
    assert_eq!(keys, ["items:aaaaaaaaaaaaaaaa", "alias:aaaaaaaaaaaaaaaa"]);
    let loaded = load_foundry_documents(&fixture.0, None).unwrap();
    assert_eq!(loaded.packs.len(), 2);
    assert_eq!(
        loaded.packs[0].documents.len(),
        loaded.packs[1].documents.len()
    );
    assert!(
        loaded.packs[1]
            .documents
            .iter()
            .all(|d| d.provenance.pack_name == "alias")
    );
}

#[test]
fn direct_preparation_rejects_incompatible_system_but_developer_loading_remains_available() {
    let fixture = Fixture::new();
    let published = build_artifact(fixture.options()).unwrap();
    let before = atlas_index::validate_artifact(&published.output_path).unwrap();
    let path = fixture.0.join("module.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["id"] = serde_json::json!("pf2e");
    manifest["version"] = serde_json::json!("7.9.1");
    fs::write(&path, manifest.to_string()).unwrap();
    assert!(crate::load_foundry_documents(&fixture.0, None).is_ok());
    let result = crate::product_preparation::prepare_fixture(&fixture.options());
    assert!(
        matches!(result, Err(crate::IngestError::SourceIncompatible(message)) if message.contains("7.9.1") && message.contains(atlas_foundry_model::generated::REFERENCE_SOURCE_VERSION))
    );
    assert_eq!(
        atlas_index::validate_artifact(published.output_path).unwrap(),
        before
    );
    let fresh = tempfile::tempdir().unwrap();
    let mut fresh_options = fixture.options();
    fresh_options.output_path = fresh.path().join("incompatible.sqlite");
    assert!(
        matches!(crate::product_preparation::prepare_fixture(&fresh_options), Err(crate::IngestError::SourceIncompatible(message)) if message.contains("7.9.1"))
    );
    let workspace = atlas_storage::Workspace::at(&fresh_options.output_path).unwrap();
    let read = workspace
        .read(atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    assert_eq!(read.reference_revision().unwrap(), 0);
    assert!(
        atlas_index::read_stored_reference_context(read.connection())
            .unwrap()
            .is_none()
    );
}

#[test]
fn trait_vocabulary_is_localized_persisted_and_keeps_exact_variants() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("static/lang/en.json"),r#"{"PF2E.TraitAgile":"Agile","PF2E.TraitDescriptionAgile":"<p>English definition.</p><script>bad()</script>","PF2E.TraitDescriptionDeadly":"<p>Shared exact authored definition.</p>","PF2E.TraitDeadlyD8":"Deadly d8","PF2E.TraitDeadlyD12":"Deadly d12"}"#).unwrap();
    fs::write(
        fixture.0.join("static/lang/fr.json"),
        r#"{"PF2E.TraitAgile":"Agile traduit"}"#,
    )
    .unwrap();
    let mut options = fixture.options();
    options.locale = "fr".into();
    let report = build_artifact(options).unwrap();
    fs::remove_dir_all(fixture.0.join("packs")).unwrap();
    fs::remove_dir_all(fixture.0.join("static")).unwrap();
    let reader = atlas_index::test_support::open_reader(&report.output_path).unwrap();
    let agile = reader
        .trait_definition(&atlas_domain::TraitKey::new("agile").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(agile.label, "Agile traduit");
    assert!(
        agile
            .description_html
            .as_ref()
            .unwrap()
            .contains("English definition.")
    );
    assert!(!agile.description_html.as_ref().unwrap().contains("script"));
    let d8 = reader
        .trait_definition(&atlas_domain::TraitKey::new("deadly-d8").unwrap())
        .unwrap()
        .unwrap();
    let d12 = reader
        .trait_definition(&atlas_domain::TraitKey::new("deadly-d12").unwrap())
        .unwrap()
        .unwrap();
    assert_ne!(d8.key, d12.key);
    assert_eq!(d8.description_html, d12.description_html);
    let amp = reader
        .trait_definition(&atlas_domain::TraitKey::new("amp").unwrap())
        .unwrap()
        .unwrap();
    assert!(amp.description_html.is_none());
    assert!(
        reader
            .trait_definition(&atlas_domain::TraitKey::new("common").unwrap())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        reader
            .workspace_session()
            .connection()
            .query_row("SELECT count(*) FROM trait_definitions", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        atlas_foundry_model::generated::trait_definitions().len()
    );
}

#[test]
fn complete_discovery_preserves_active_publication_when_a_declared_pack_is_missing() {
    let fixture = Fixture::new();
    let first = build_artifact(fixture.options()).unwrap();
    assert_eq!(first.admission.discovered_files, 3);
    assert_eq!(first.admission.admitted_records, 2);
    assert_eq!(first.admission.quarantined_files, 1);
    assert_eq!(first.admission.exclusions[0].path, "packs/items/bad.json");
    let db = rusqlite::Connection::open(fixture.options().output_path).unwrap();
    db.execute("INSERT INTO saved_lists(list_key,slug,name,created_at,updated_at) VALUES('keep','keep','Keep','now','now')", []).unwrap();
    let before: (u64, String, i64) = db.query_row("SELECT reference_revision,(SELECT context_json FROM artifact_context),(SELECT record_id FROM records WHERE pack_id='macros') FROM workspace", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    fs::rename(
        fixture.0.join("packs/macros"),
        fixture.0.join("held-macros"),
    )
    .unwrap();
    for _ in 0..2 {
        let error = build_artifact(fixture.options()).unwrap_err().to_string();
        assert!(error.contains("packs/macros"), "{error}");
        assert_eq!(before, db.query_row("SELECT reference_revision,(SELECT context_json FROM artifact_context),(SELECT record_id FROM records WHERE pack_id='macros') FROM workspace", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap());
        assert_eq!(
            db.query_row(
                "SELECT count(*) FROM saved_lists WHERE list_key='keep'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            atlas_index::test_support::open_reader(fixture.options().output_path)
                .unwrap()
                .statistics()
                .unwrap()
                .records,
            2
        );
    }
    fs::rename(
        fixture.0.join("held-macros"),
        fixture.0.join("packs/macros"),
    )
    .unwrap();
    build_artifact(fixture.options()).unwrap();
    assert_eq!(
        db.query_row(
            "SELECT record_id FROM records WHERE pack_id='macros'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        before.2
    );
}

#[test]
fn missing_pack_from_outset_is_rejected_before_tokenizer_or_inference() {
    let fixture = Fixture::new();
    fs::remove_dir_all(fixture.0.join("packs/macros")).unwrap();
    let mut options = fixture.options();
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        fixture.0.join("missing-models"),
    )];
    let error = build_artifact(options).unwrap_err().to_string();
    assert!(error.contains("packs/macros"), "{error}");
    let diagnostic = load_foundry_documents(&fixture.0, None).unwrap().report();
    assert_eq!(diagnostic.counts.unavailable_packs, 1);
    let db = rusqlite::Connection::open(fixture.options().output_path).unwrap();
    assert_eq!(
        db.query_row("SELECT reference_revision FROM workspace", [], |r| r
            .get::<_, u64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn complete_empty_pack_and_intentional_removal_publish_and_returning_keys_reuse_identity() {
    let fixture = Fixture::new();
    build_artifact(fixture.options()).unwrap();
    let db = rusqlite::Connection::open(fixture.options().output_path).unwrap();
    let identity: i64 = db
        .query_row(
            "SELECT id FROM record_identity WHERE key='macros:bbbbbbbbbbbbbbbb'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let bytes = fs::read(fixture.0.join("packs/macros/tool.json")).unwrap();
    fs::remove_file(fixture.0.join("packs/macros/tool.json")).unwrap();
    let empty = build_artifact(fixture.options()).unwrap();
    assert_eq!(empty.record_count, 1);
    assert_eq!(
        db.query_row(
            "SELECT availability FROM record_identity WHERE key='macros:bbbbbbbbbbbbbbbb'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "unavailable"
    );
    fs::write(fixture.0.join("packs/macros/tool.json"), bytes).unwrap();
    build_artifact(fixture.options()).unwrap();
    assert_eq!(
        db.query_row(
            "SELECT id FROM record_identity WHERE key='macros:bbbbbbbbbbbbbbbb'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        identity
    );
    let manifest =
        r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"}]}"#;
    fs::write(fixture.0.join("module.json"), manifest).unwrap();
    fs::remove_dir_all(fixture.0.join("packs/macros")).unwrap();
    let removed = build_artifact(fixture.options()).unwrap();
    assert_eq!(removed.pack_count, 1);
    assert_eq!(removed.record_count, 1);
    fs::remove_file(fixture.0.join("packs/items/useful.json")).unwrap();
    fs::remove_file(fixture.0.join("packs/items/bad.json")).unwrap();
    let entirely_empty = build_artifact(fixture.options()).unwrap();
    assert_eq!(entirely_empty.record_count, 0);
    assert_eq!(entirely_empty.admission.discovered_files, 0);
}

#[test]
fn pack_path_replaced_by_a_file_is_unavailable_and_keeps_the_active_context() {
    let fixture = Fixture::new();
    build_artifact(fixture.options()).unwrap();
    fs::remove_dir_all(fixture.0.join("packs/macros")).unwrap();
    fs::write(fixture.0.join("packs/macros"), "not a directory").unwrap();
    let error = build_artifact(fixture.options()).unwrap_err().to_string();
    assert!(error.contains("packs/macros"), "{error}");
    assert_eq!(
        atlas_index::test_support::open_reader(fixture.options().output_path)
            .unwrap()
            .statistics()
            .unwrap()
            .records,
        2
    );
}

#[test]
fn duplicate_identity_exclusions_and_partial_admissions_are_explained_by_the_active_summary() {
    let fixture = Fixture::new();
    let initial = build_artifact(fixture.options()).unwrap();
    assert_eq!(initial.admission.partial_records, 1);
    fs::copy(
        fixture.0.join("packs/items/useful.json"),
        fixture.0.join("packs/items/duplicate.json"),
    )
    .unwrap();
    let report = build_artifact(fixture.options()).unwrap();
    assert_eq!(report.admission.admitted_records, 1);
    assert_eq!(report.admission.partial_records, 0);
    assert_eq!(report.admission.quarantined_files, 1);
    assert_eq!(report.admission.excluded_documents, 2);
    assert_eq!(
        report
            .admission
            .exclusions
            .iter()
            .filter(|e| e.reason == "DuplicateId")
            .count(),
        2
    );
    let reader = atlas_index::test_support::open_reader(fixture.options().output_path).unwrap();
    assert_eq!(reader.context().admission.as_ref(), Some(&report.admission));
    assert_eq!(reader.context().preparation_sha256().unwrap(), {
        let mut context = reader.context().clone();
        context.admission = None;
        context.preparation_sha256().unwrap()
    });
}

#[cfg(unix)]
#[test]
fn unreadable_declared_pack_fails_with_pack_identity_and_preserves_publication() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    build_artifact(fixture.options()).unwrap();
    let path = fixture.0.join("packs/macros");
    let original = fs::metadata(&path).unwrap().permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    let cannot_read = fs::read_dir(&path).is_err();
    let result = build_artifact(fixture.options());
    fs::set_permissions(&path, original).unwrap();
    if cannot_read {
        let error = result.unwrap_err().to_string();
        assert!(error.contains("macros"), "{error}");
        assert_eq!(
            atlas_index::test_support::open_reader(fixture.options().output_path)
                .unwrap()
                .statistics()
                .unwrap()
                .records,
            2
        );
    }
}

#[test]
fn wholly_quarantined_or_unaddressable_source_preserves_active_publication_and_saved_state() {
    for quarantine in [true, false] {
        let fixture = Fixture::new();
        build_artifact(fixture.options()).unwrap();
        let db = rusqlite::Connection::open(fixture.options().output_path).unwrap();
        db.execute("INSERT INTO saved_lists(list_key,slug,name,created_at,updated_at) VALUES('keep','keep','Keep','now','now')", []).unwrap();
        let before: (u64, String) = db.query_row("SELECT reference_revision,(SELECT context_json FROM artifact_context) FROM workspace", [], |r| Ok((r.get(0)?,r.get(1)?))).unwrap();
        let roots_before = db
            .prepare("SELECT record_id,key,content_hash FROM records ORDER BY key")
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        fs::write(
            fixture.0.join("packs/items/useful.json"),
            if quarantine {
                "{broken"
            } else {
                r#"{"name":"Unaddressable item","type":"effect"}"#
            },
        )
        .unwrap();
        fs::write(
            fixture.0.join("packs/macros/tool.json"),
            if quarantine {
                "{broken"
            } else {
                r#"{"name":"Unaddressable macro","type":"script","command":""}"#
            },
        )
        .unwrap();
        let diagnostic = load_foundry_documents(&fixture.0, None).unwrap().report();
        assert_eq!(diagnostic.counts.discovered_files, 3);
        assert_eq!(
            diagnostic.counts.quarantined_files,
            if quarantine { 3 } else { 1 }
        );
        let error = build_artifact(fixture.options()).unwrap_err();
        assert!(matches!(error, IngestError::NoRecordsLoaded), "{error}");
        assert_eq!(before, db.query_row("SELECT reference_revision,(SELECT context_json FROM artifact_context) FROM workspace", [], |r| Ok((r.get(0)?,r.get(1)?))).unwrap());
        assert_eq!(
            roots_before,
            db.prepare("SELECT record_id,key,content_hash FROM records ORDER BY key")
                .unwrap()
                .query_map([], |r| Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?
                )))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        );
        assert_eq!(
            db.query_row(
                "SELECT name FROM saved_lists WHERE list_key='keep'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "Keep"
        );
        let reader = atlas_index::test_support::open_reader(fixture.options().output_path).unwrap();
        let query =
            atlas_index::validate_query(&atlas_domain::query::QueryPredicate::boolean(true))
                .unwrap();
        let hits = reader
            .lexical_root_candidates(
                "Useful",
                &query,
                None,
                atlas_domain::SearchContentScope::All,
            )
            .unwrap();
        assert_eq!(
            hits.iter()
                .map(|h| h.record.to_string())
                .collect::<Vec<_>>(),
            ["items:aaaaaaaaaaaaaaaa"]
        );
        assert_eq!(
            reader
                .context()
                .admission
                .as_ref()
                .unwrap()
                .admitted_records,
            2
        );
    }
}

#[test]
fn retired_preparation_and_predecessor_skip_unreadable_cached_dtos_before_source_rebuild() {
    for cause in ["relationship", "localization", "predecessor"] {
        let fixture = Fixture::new();
        let first = build_artifact(fixture.options()).unwrap();
        if cause == "predecessor" {
            atlas_index::test_support::set_predecessor_source_contract(
                &first.output_path,
                atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            )
            .unwrap();
        } else if cause == "relationship" {
            atlas_index::test_support::set_relationship_policy(
                &first.output_path,
                atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
                "typed-authored-occurrences/v6",
            )
            .unwrap();
        } else {
            let database = rusqlite::Connection::open(&first.output_path).unwrap();
            let mut context = atlas_index::read_stored_reference_context(&database)
                .unwrap()
                .unwrap();
            context.localization_policy = "selected-source-catalog/english-fallback/v1".into();
            database
                .execute(
                    "UPDATE artifact_context SET context_hash=?,context_json=? WHERE singleton=1",
                    rusqlite::params![
                        context.sha256().unwrap(),
                        serde_json::to_string(&context).unwrap()
                    ],
                )
                .unwrap();
        }
        let database = rusqlite::Connection::open(&first.output_path).unwrap();
        assert_eq!(
            atlas_index::classify_reference_upgrade(&database).unwrap(),
            if cause == "predecessor" {
                atlas_index::ReferenceUpgrade::SourceReplacement
            } else {
                atlas_index::ReferenceUpgrade::SourcePreparation
            }
        );
        // Invalid cached bytes must never be decoded by the source-backed path.
        database
            .execute("UPDATE record_bodies SET snapshot=x'00'", [])
            .unwrap();
        database.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Kept','now','now')").unwrap();
        let revision: i64 = database
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        drop(database);
        let workspace = atlas_storage::Workspace::at(&first.output_path).unwrap();
        let owner = workspace.update_owner().unwrap();
        // Dropping a real unfinished publication keeps durable maintenance set.
        drop(
            owner
                .maintenance(
                    atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
                    "interrupted retired reference publication",
                )
                .unwrap(),
        );
        drop(owner);
        assert!(atlas_index::test_support::open_reader(&first.output_path).is_err());
        let database = rusqlite::Connection::open(&first.output_path).unwrap();
        assert!(
            database
                .query_row("SELECT maintenance FROM workspace", [], |r| r
                    .get::<_, Option<String>>(0))
                .unwrap()
                .is_some()
        );
        drop(database);
        // Source absence remains a truthful acquisition error, not a DTO error.
        let source = fixture.0.join("packs/items");
        let hidden = fixture.0.join("hidden-items");
        fs::rename(&source, &hidden).unwrap();
        let failure = build_artifact(fixture.options()).unwrap_err().to_string();
        assert!(failure.contains("items"), "{failure}");
        let database = rusqlite::Connection::open(&first.output_path).unwrap();
        assert_eq!(
            database
                .query_row("SELECT reference_revision FROM workspace", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            revision
        );
        assert_eq!(
            database
                .query_row("SELECT name FROM saved_lists WHERE id=1", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "Kept"
        );
        assert!(
            database
                .query_row("SELECT maintenance FROM workspace", [], |r| r
                    .get::<_, Option<String>>(0))
                .unwrap()
                .is_none()
        );
        assert!(atlas_index::test_support::open_reader(&first.output_path).is_err());
        drop(database);
        fs::rename(hidden, source).unwrap();
        let rebuilt = build_artifact(fixture.options()).unwrap();
        let reader = atlas_index::test_support::open_reader(&rebuilt.output_path).unwrap();
        assert_eq!(
            reader
                .similarity_part_addresses(&"items:aaaaaaaaaaaaaaaa".parse().unwrap())
                .unwrap()
                .len(),
            1
        );
        assert!(
            reader
                .read_source_record_for_inspection(&"items:aaaaaaaaaaaaaaaa".parse().unwrap())
                .unwrap()
                .is_some()
        );
        let database = rusqlite::Connection::open(&first.output_path).unwrap();
        assert_eq!(
            database
                .query_row("SELECT name FROM saved_lists WHERE id=1", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "Kept"
        );
    }
}

#[test]
fn bounded_product_preparation_keeps_excluded_collisions_and_stable_owned_destinations() {
    use atlas_record::source_content::{ContentReferenceResolution, ContentReferenceTarget};
    use atlas_record::source_record::SourceContentStatus;
    for cause in ["raw name", "missing ID name", "duplicate ID"] {
        for reverse in [false, true] {
            let fixture = Fixture::new();
            fs::remove_file(fixture.0.join("packs/items/bad.json")).unwrap();
            fs::create_dir_all(fixture.0.join("packs/journals")).unwrap();
            fs::write(fixture.0.join("module.json"), r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"},{"name":"journals","label":"Journals","type":"JournalEntry","path":"packs/journals"}]}"#).unwrap();
            let targets = [
                "Compendium.pf2e.items.Item.Same",
                "Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa",
                "Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb",
                "Compendium.pf2e.journals.JournalEntry.cccccccccccccccc.JournalEntryPage.dddddddddddddddd",
            ];
            let markup = targets
                .iter()
                .map(|target| format!("@UUID[{target}]"))
                .collect::<Vec<_>>()
                .join(" ");
            fs::write(fixture.0.join("packs/items/useful.json"), serde_json::to_vec(&serde_json::json!({"_id":"ffffffffffffffff","name":"Referring source","type":"effect","system":{"description":{"value":markup}}})).unwrap()).unwrap();
            let valid = serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","name":"Same"});
            let excluded = match cause {
                "raw name" => {
                    serde_json::json!({"_id":"bbbbbbbbbbbbbbbb","type":"unknown-family","name":"Same"})
                }
                "missing ID name" => serde_json::json!({"type":"spell","name":"Same"}),
                _ => {
                    serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"unknown-family","name":"Same"})
                }
            };
            for (name, source) in if reverse {
                [("z-target.json", &valid), ("a-excluded.json", &excluded)]
            } else {
                [("a-target.json", &valid), ("z-excluded.json", &excluded)]
            } {
                fs::write(
                    fixture.0.join("packs/items").join(name),
                    serde_json::to_vec(source).unwrap(),
                )
                .unwrap();
            }
            fs::write(fixture.0.join("packs/journals/owned.json"), r#"{"_id":"cccccccccccccccc","name":"Owned pages","pages":[{"_id":"dddddddddddddddd","name":"Stable page","type":"text","text":{"format":1,"content":"<p>Stable owned prose.</p>"}}]}"#).unwrap();
            let prepared = crate::product_preparation::prepare_fixture(&fixture.options()).unwrap();
            assert!(prepared.native_inputs.is_empty());
            assert_eq!(
                prepared.source_report.raw_only_documents,
                usize::from(cause != "missing ID name")
            );
            if cause == "missing ID name" {
                assert_eq!(
                    prepared.source_report.identity_unavailable.get("MissingId"),
                    Some(&1)
                );
            }
            if cause == "duplicate ID" {
                assert_eq!(
                    prepared
                        .source_report
                        .identity_unavailable
                        .get("DuplicateId"),
                    Some(&1)
                );
            }
            let publication = &prepared.spool;
            assert_eq!(publication.semantic.len(), 0);
            let mut roots = Vec::new();
            let mut references = Vec::new();
            publication
                .records(&mut |frame| {
                    let source = frame.source;
                    roots.push(source.record.key().to_string());
                    if source.record.key().to_string() == "items:ffffffffffffffff" {
                        for outcome in &source.content {
                            if let SourceContentStatus::Prepared(content) = &outcome.status {
                                references.extend(content.references.iter().map(|reference| {
                                    (
                                        reference.authored_target.clone(),
                                        reference.resolution.clone(),
                                    )
                                }));
                            }
                        }
                    }
                    Ok(())
                })
                .unwrap();
            let expected = [
                ContentReferenceResolution::Unresolved,
                if cause == "duplicate ID" {
                    ContentReferenceResolution::Unresolved
                } else {
                    ContentReferenceResolution::Resolved(ContentReferenceTarget::Record {
                        key: "items:aaaaaaaaaaaaaaaa".parse().unwrap(),
                    })
                },
                ContentReferenceResolution::Unresolved,
                ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {
                    key: "journals:cccccccccccccccc".parse().unwrap(),
                    owners: vec![atlas_domain::OwnedContentLocator {
                        collection: "/pages".into(),
                        identity: atlas_domain::OwnedContentIdentity::Stable(
                            "dddddddddddddddd".into(),
                        ),
                    }],
                }),
            ];
            assert_eq!(
                references,
                targets
                    .into_iter()
                    .zip(expected)
                    .map(|(target, resolution)| (target.to_owned(), resolution))
                    .collect::<Vec<_>>(),
                "{cause}, reverse={reverse}"
            );
            roots.sort();
            let mut expected_roots = vec!["items:ffffffffffffffff", "journals:cccccccccccccccc"];
            if cause != "duplicate ID" {
                expected_roots.insert(0, "items:aaaaaaaaaaaaaaaa");
            }
            assert_eq!(roots, expected_roots, "{cause}, reverse={reverse}");
        }
    }
}
