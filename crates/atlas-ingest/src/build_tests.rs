use crate::*;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture(PathBuf);
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
            embedding: None,
            reuse_embeddings: true,
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
        let reader = atlas_index::SqliteIndexReader::open_read_only(&report.output_path).unwrap();
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
                .join("BAAI/bge-small-en-v1.5/onnx/model.onnx")
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let mut options = fixture.options();
    options.embedding = Some(atlas_embedding::EmbeddingRuntimeConfig::default_model(
        cache,
    ));
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
        let prepared = crate::build::prepare_build(&options).unwrap();
        let identity = prepared
            .prepared
            .pending
            .iter()
            .filter(|pending| {
                matches!(
                    pending.unit.address,
                    atlas_domain::SourcePassageAddress::Identity {}
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(identity.len(), 1);
        assert!(identity[0].input.input.contains("items:aaaaaaaaaaaaaaaa"));
        assert!(identity[0].input.token_count <= 512);
        assert!(prepared.prepared.pending.iter().any(|pending| !matches!(
            pending.unit.address,
            atlas_domain::SourcePassageAddress::Identity {}
        )));
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
    let reader = atlas_index::SqliteIndexReader::open_read_only(&options.output_path).unwrap();
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
            .query_row("SELECT count(*) FROM semantic_models", [], |r| r
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
    let catalog = crate::source::localization::LocalizationCatalog::load(&fixture.0, "fr").unwrap();
    assert_eq!(catalog.localized_value("PF2E.Label"), Some("French"));
    assert_eq!(
        catalog.localized_value("PF2E.Fallback"),
        Some("English fallback")
    );
    assert_ne!(catalog.locale_sha256, catalog.english_sha256);
    let first = build_artifact(fixture.options()).unwrap();
    let bytes = fs::read(&first.output_path).unwrap();
    let mut options = fixture.options();
    options.locale = "unknown".into();
    assert!(build_artifact(options).is_err());
    assert_eq!(fs::read(first.output_path).unwrap(), bytes);
    assert!(crate::source::localization::LocalizationCatalog::load(&fixture.0, "../en").is_err());
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
        embedding: None,
        reuse_embeddings: false,
        embedding_batch_size: 16,
    })
    .unwrap();
    assert_eq!(report.record_count, 25641);
    assert_eq!(report.product_record_count, 25560);
    assert_eq!(report.source_report.quarantined_files, 0);
    assert!(report.source_report.identity_unavailable.is_empty());
    let validation = atlas_index::validate_artifact(&report.output_path).unwrap();
    assert_eq!(validation.checked_snapshots, 25641);
    let reader = atlas_index::SqliteIndexReader::open_read_only(&report.output_path).unwrap();
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
        atlas_index::SqliteIndexReader::open_read_only(evidence.join("full-lexical.sqlite"))
            .unwrap();
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
    assert_eq!(roots, 25641);
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
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let cache = ancestor.join(".cache/hf-models");
            cache
                .join("BAAI/bge-small-en-v1.5/onnx/model.onnx")
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let mut options = fixture.options();
    options.embedding = Some(atlas_embedding::EmbeddingRuntimeConfig::default_model(
        cache,
    ));
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
    let changed = build_artifact(options).unwrap();
    assert_ne!(first.source_fingerprint, changed.source_fingerprint);
    assert!(changed.inferred_inputs > 0);
    assert!(changed.reused_inputs > 0);
    assert_eq!(changed.semantic_unit_count, first.semantic_unit_count);
    atlas_index::validate_artifact(&changed.output_path).unwrap();
}
#[test]
#[ignore = "requires verified local BGE assets; run explicitly"]
fn actual_model_reuse_rejects_forged_input_associations_and_token_counts() {
    let fixture = Fixture::new();
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find_map(|ancestor| {
            let cache = ancestor.join(".cache/hf-models");
            cache
                .join("BAAI/bge-small-en-v1.5/onnx/model.onnx")
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let mut options = fixture.options();
    options.embedding = Some(atlas_embedding::EmbeddingRuntimeConfig::default_model(
        cache,
    ));
    build_artifact(options.clone()).unwrap();
    let source = fixture.0.join("packs/items/useful.json");
    let original = fs::read_to_string(&source).unwrap();
    fs::write(
        &source,
        original.replace("Full prose", "A completely different explanatory paragraph"),
    )
    .unwrap();
    let current = crate::build::prepare_build(&options).unwrap();
    let new_input = current
        .prepared
        .pending
        .iter()
        .find(|p| p.unit.field.is_some())
        .unwrap();
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    let (id,old_hash,old_count):(i64,String,usize)=db.query_row(
        "SELECT unit_id,input_hash,input_token_count FROM semantic_units WHERE field_path IS NOT NULL LIMIT 1",[],
        |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
    assert_ne!(old_hash, new_input.unit.input_hash);
    // The old, valid normalized vector is wrong for this newly requested text.
    // Hash syntax, token budget, addresses and offline structural checks pass.
    db.execute(
        "UPDATE semantic_units SET input_hash=?,input_token_count=? WHERE unit_id=?",
        rusqlite::params![
            new_input.unit.input_hash,
            new_input.unit.input_token_count,
            id
        ],
    )
    .unwrap();
    drop(db);
    atlas_index::validate_artifact(&options.output_path).unwrap();
    let corrupted = fs::read(&options.output_path).unwrap();
    let error = build_artifact(options.clone()).unwrap_err().to_string();
    assert!(error.contains("input association mismatch"), "{error}");
    assert_eq!(fs::read(&options.output_path).unwrap(), corrupted);
    let db = rusqlite::Connection::open(&options.output_path).unwrap();
    db.execute(
        "UPDATE semantic_units SET input_hash=?,input_token_count=? WHERE unit_id=?",
        rusqlite::params![old_hash, old_count, id],
    )
    .unwrap();
    db.execute("UPDATE semantic_units SET input_token_count=input_token_count+1 WHERE unit_kind='identity'",[]).unwrap();
    drop(db);
    atlas_index::validate_artifact(&options.output_path).unwrap();
    let corrupted = fs::read(&options.output_path).unwrap();
    let error = build_artifact(options).unwrap_err().to_string();
    assert!(error.contains("input association mismatch"), "{error}");
    assert_eq!(
        fs::read(fixture.0.join("artifact.sqlite")).unwrap(),
        corrupted
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
                    ("offGuard1", "cccccccccccccccc"),
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
                "SELECT count(*) FROM verified_aliases WHERE alias='Attack of Opportunity'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap()
            > 0
    );
    assert!(
        database
            .query_row(
                "SELECT count(*) FROM verified_aliases WHERE alias='flat-footed'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap()
            > 0
    );
    assert_eq!(
        database
            .query_row("SELECT count(*) FROM remaster_pairs", [], |r| r
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
            .query_row("SELECT count(*) FROM remaster_pairs", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
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
                .join("BAAI/bge-small-en-v1.5/onnx/model.onnx")
                .is_file()
                .then_some(cache)
        })
        .unwrap();
    let options = BuildArtifactOptions {
        source_root: root,
        output_path: PathBuf::from("unused-preparation-only"),
        manifest_path: None,
        locale: "en".into(),
        embedding: Some(atlas_embedding::EmbeddingRuntimeConfig::default_model(
            cache,
        )),
        reuse_embeddings: false,
        embedding_batch_size: 16,
    };
    let prepared = crate::build::prepare_build(&options).unwrap();
    assert_eq!(prepared.prepared.index.records.len(), 25641);
    let mut identities = std::collections::BTreeSet::new();
    let mut bytes = 0;
    let mut tokens = 0;
    let mut max_tokens = 0;
    let mut coverage = std::collections::BTreeMap::new();
    for pending in &prepared.prepared.pending {
        assert!(pending.unit.vector.is_empty());
        assert!(pending.input.token_count <= 512);
        assert!(
            pending.input.body_token_count <= 256
                || matches!(
                    pending.unit.address,
                    atlas_domain::SourcePassageAddress::Identity {}
                )
        );
        tokens += pending.input.token_count;
        max_tokens = max_tokens.max(pending.input.token_count);
        let range = match &pending.unit.address {
            atlas_domain::SourcePassageAddress::Identity {} => {
                assert!(identities.insert(pending.unit.record.clone()));
                continue;
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
            pending.unit.record.clone(),
            serde_json::to_string(&pending.unit.owners).unwrap(),
            pending.unit.field.clone(),
            range.0,
        );
        let end = coverage.entry(key).or_insert(0);
        assert!(range.1.start <= *end);
        assert!(range.1.end > *end);
        bytes += range.1.end - *end;
        *end = range.1.end;
    }
    assert_eq!(identities.len(), 25560);
    let mut selected_fields = 0;
    let mut selected_sections = 0;
    let mut selected_bytes = 0;
    for input in &prepared.prepared.index.records {
        let pack = prepared
            .prepared
            .index
            .packs
            .iter()
            .find(|pack| pack.pack_id == input.record.key().pack().as_str())
            .unwrap();
        let selected = atlas_record::source_record::select_record_search(
            &input.record,
            &input.content,
            prepared.prepared.index.context.audience,
            &pack.label,
            &prepared.prepared.index.context.used_trait_labels,
        )
        .unwrap();
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
    }
    assert!(coverage.is_empty());
    assert_eq!(bytes, selected_bytes);
    let evidence =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scratch/ingest-validation");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(evidence.join("full-semantic-preparation-census.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":prepared.prepared.index.records.len(),"identity_units":identities.len(),"selected_fields":selected_fields,"selected_sections":selected_sections,"selected_body_bytes":selected_bytes,"prepared_inputs":prepared.prepared.pending.len(),"exact_input_tokens":tokens,"maximum_input_tokens":max_tokens,"context_shortened":prepared.prepared.context_shortened,"policy_version":atlas_embedding::EMBEDDING_UNIT_POLICY_VERSION,"source_fingerprint":prepared.fingerprint.value,"context_sha256":prepared.prepared.index.context.sha256().unwrap(),"content_interpretation_version":prepared.prepared.index.context.content_interpretation_version,"content_selection_version":prepared.prepared.index.context.content_selection_version,"relationship_policy":prepared.prepared.index.context.relationship_policy,"duration_ms":started.elapsed().as_millis(),"inference_performed":false})).unwrap()).unwrap();
}
