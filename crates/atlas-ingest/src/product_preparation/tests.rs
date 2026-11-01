use super::*;
use atlas_index::PublicationSource;
use atlas_index::{NativeValueSource, SqliteIndexWriter};
use rusqlite::Connection;
use std::{borrow::Cow, fs};

struct SourceFixture(tempfile::TempDir);
impl SourceFixture {
    fn new() -> Self {
        let fixture = Self(tempfile::tempdir().unwrap());
        fs::create_dir_all(fixture.0.path().join("packs/items")).unwrap();
        fs::create_dir_all(fixture.0.path().join("static/lang")).unwrap();
        fs::write(
            fixture.0.path().join("module.json"),
            r#"{"packs":[{"name":"items","label":"Items","type":"Item","path":"packs/items"}]}"#,
        )
        .unwrap();
        fixture.locale("Original localized text");
        fixture.document(
            "a",
            "aaaaaaaaaaaaaaaa",
            "Dependent",
            "<h2>Rules</h2><p>@UUID[Compendium.pf2e.items.Item.Target] @Localize[Rules]</p>",
        );
        fixture.document(
            "b",
            "bbbbbbbbbbbbbbbb",
            "Target",
            "<p>Original target body.</p>",
        );
        fixture
    }
    fn locale(&self, value: &str) {
        fs::write(
            self.0.path().join("static/lang/en.json"),
            serde_json::json!({"Rules":value}).to_string(),
        )
        .unwrap();
    }
    fn document(&self, path: &str, id: &str, name: &str, body: &str) {
        fs::write(self.0.path().join(format!("packs/items/{path}.json")),serde_json::json!({"_id":id,"name":name,"type":"effect","system":{"description":{"value":body}}}).to_string()).unwrap();
    }
    fn options(&self, name: &str) -> BuildArtifactOptions {
        BuildArtifactOptions {
            source_root: self.0.path().to_owned(),
            output_path: self.0.path().join(format!("{name}.sqlite")),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 8,
        }
    }
    fn build(&self, name: &str) {
        let options = self.options(name);
        let workspace =
            atlas_index::test_support::initialize_workspace(&options.output_path).unwrap();
        crate::build_artifact(
            &workspace,
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            options,
        )
        .unwrap();
    }
}
fn components(path: &std::path::Path) -> Vec<(String, String)> {
    atlas_index::validate_artifact(path).unwrap();
    Connection::open(path).unwrap().prepare("SELECT r.key,d.digests_json FROM reference_component_digests d JOIN records r USING(record_id) ORDER BY r.key").unwrap().query_map([],|r|Ok((r.get(0)?,r.get(1)?))).unwrap().collect::<Result<_,_>>().unwrap()
}
#[test]
fn bounded_incremental_stream_matches_fresh_after_dependency_and_membership_changes() {
    let fixture = SourceFixture::new();
    fixture.build("incremental");
    let path = fixture.options("incremental").output_path;
    let c = Connection::open(&path).unwrap();
    c.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Kept notes','now','now'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'items:bbbbbbbbbbbbbbbb',0,'Retain target note','Target','now','now')").unwrap();
    let identity: i64 = c
        .query_row(
            "SELECT id FROM record_identity WHERE key='items:bbbbbbbbbbbbbbbb'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    for case in [
        "unchanged",
        "body",
        "add",
        "target removed",
        "reappear",
        "ambiguous name",
        "rename ambiguity",
        "locale",
        "renamed path",
        "changed key",
        "changed kind",
    ] {
        match case {
            "body" => fixture.document(
                "b",
                "bbbbbbbbbbbbbbbb",
                "Target",
                "<h2>Changed</h2><p>Changed body.</p>",
            ),
            "add" => fixture.document(
                "c",
                "cccccccccccccccc",
                "Other",
                "<p>Independent source.</p>",
            ),
            "target removed" => {
                fs::remove_file(fixture.0.path().join("packs/items/b.json")).unwrap()
            }
            "reappear" => {
                fixture.document("b", "bbbbbbbbbbbbbbbb", "Target", "<p>Returned body.</p>")
            }
            "ambiguous name" => {
                fixture.document("c", "cccccccccccccccc", "Target", "<p>Another target.</p>")
            }
            "rename ambiguity" => fixture.document(
                "c",
                "cccccccccccccccc",
                "Distinct",
                "<p>Another target.</p>",
            ),
            "locale" => fixture.locale("Changed localized text"),
            "renamed path" => fs::rename(
                fixture.0.path().join("packs/items/b.json"),
                fixture.0.path().join("packs/items/renamed.json"),
            )
            .unwrap(),
            "changed key" => fixture.document(
                "renamed",
                "ffffffffffffffff",
                "Target",
                "<p>New identity.</p>",
            ),
            "changed kind" => {
                fs::write(fixture.0.path().join("module.json"),r#"{"packs":[{"name":"items","label":"Items","type":"JournalEntry","path":"packs/items"}]}"#).unwrap();
                for (path, id) in [
                    ("a", "aaaaaaaaaaaaaaaa"),
                    ("renamed", "ffffffffffffffff"),
                    ("c", "cccccccccccccccc"),
                ] {
                    fs::write(
                        fixture.0.path().join(format!("packs/items/{path}.json")),
                        serde_json::json!({"_id":id,"name":"Changed document kind","pages":[]})
                            .to_string(),
                    )
                    .unwrap();
                }
            }
            _ => {}
        }
        fixture.build("incremental");
        fixture.build(case);
        assert_eq!(
            components(&path),
            components(&fixture.options(case).output_path),
            "{case}"
        );
        let saved: (String, String) = c
            .query_row(
                "SELECT l.name,i.note FROM saved_lists l JOIN saved_list_items i ON i.list_id=l.id",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            saved,
            ("Kept notes".into(), "Retain target note".into()),
            "{case}"
        );
        assert_eq!(
            c.query_row(
                "SELECT id FROM record_identity WHERE key='items:bbbbbbbbbbbbbbbb'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            identity,
            "{case}"
        );
    }
}
#[test]
fn observed_preparation_counts_and_frame_sizes_are_separate_from_structural_window() {
    let fixture = SourceFixture::new();
    let small = prepare_fixture(&fixture.options("small-counters")).unwrap();
    assert_eq!(small.work.selection_calls, 2);
    assert_eq!(small.work.reusable_record_read_calls, 0);
    assert!(small.work.largest_source_bytes > 0);
    assert_eq!(
        small.work.largest_prepared_frame_bytes,
        small.spool.records.max_decoded_frame_bytes()
    );
    assert!(small.work.largest_encoded_prepared_frame_bytes > 0);
    for ordinal in 0..32 {
        fixture.document(
            &format!("extra{ordinal}"),
            &format!("{ordinal:016}"),
            "Independent",
            "<p>Same bounded source body.</p>",
        );
    }
    let large = prepare_fixture(&fixture.options("large-counters")).unwrap();
    assert_eq!(large.work.selection_calls, 34);
    assert_eq!(large.spool.records.len(), 34);
    assert_eq!(
        large.work.largest_prepared_frame_bytes,
        large.spool.records.max_decoded_frame_bytes()
    );
    assert!(
        large.work.largest_prepared_frame_bytes < small.work.largest_prepared_frame_bytes * 2,
        "a frame contains corpus-sized metadata"
    );
}

struct NoNativeValues;
impl NativeValueSource for NoNativeValues {
    fn value<'a>(
        &'a self,
        _: &atlas_index::SourceSearchProfileIdentity,
        _: &atlas_embedding::NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, atlas_index::IndexError> {
        Err(atlas_index::IndexError::Invalid(
            "fixture has no native profile".into(),
        ))
    }
}
#[test]
fn stale_sealed_stream_cannot_remove_roots_or_advance_publication() {
    let fixture = SourceFixture::new();
    fixture.build("incremental");
    let options = fixture.options("incremental");
    fs::remove_file(fixture.0.path().join("packs/items/b.json")).unwrap();
    let prepared = prepare_fixture(&options).unwrap();
    let target = prepared.spool.context.sha256().unwrap();
    let sealed = prepared.spool.seal().unwrap();
    let workspace = atlas_storage::Workspace::at(&options.output_path).unwrap();
    let schema = atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    let (job, values) = {
        let owner = workspace.update_owner().unwrap();
        let job = atlas_index::register_native_target(&owner, &workspace, schema, &sealed).unwrap();
        let values = atlas_index::freeze_fixture_native_values(
            &sealed,
            &NoNativeValues,
            Some(&job),
            sealed.expected_instance().unwrap(),
        )
        .unwrap();
        (job, values)
    };
    fixture.document(
        "b",
        "bbbbbbbbbbbbbbbb",
        "Target",
        "<p>Restored before a newer publication.</p>",
    );
    fixture.build("incremental");
    let before = components(&options.output_path);
    let c = Connection::open(&options.output_path).unwrap();
    let revision: u64 = c
        .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
        .unwrap();
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner
        .maintenance(schema, "stale fixture publication")
        .unwrap();
    let error = SqliteIndexWriter::new(&maintenance)
        .write_job(&sealed, &values, &job, &target)
        .unwrap_err();
    assert!(
        error.to_string().contains("stale workspace revision"),
        "{error}"
    );
    drop(maintenance);
    drop(owner);
    assert_eq!(components(&options.output_path), before);
    assert_eq!(
        c.query_row("SELECT reference_revision FROM workspace", [], |r| r
            .get::<_, u64>(0))
            .unwrap(),
        revision
    );
}
#[test]
fn native_metadata_seal_binds_full_profile_context_and_expected_revision() {
    let fixture = SourceFixture::new();
    let mut prepared = prepare_fixture(&fixture.options("binding")).unwrap();
    prepared.check_execution_streams().unwrap();
    prepared.spool.context.lookup_version.push_str("-changed");
    assert!(
        prepared
            .check_execution_streams()
            .unwrap_err()
            .to_string()
            .contains("binding")
    );
}

#[test]
fn ordinary_same_source_build_rejects_valid_but_unattested_retained_payload_changes() {
    for corruption in ["snapshot name", "prepared prose"] {
        let fixture = SourceFixture::new();
        fixture.build("attested");
        let options = fixture.options("attested");
        let c = Connection::open(&options.output_path).unwrap();
        c.execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(1,'kept','kept','Kept proof notes','now','now')").unwrap();
        let revision: u64 = c
            .query_row("SELECT reference_revision FROM workspace", [], |r| r.get(0))
            .unwrap();
        let context: (String, String) = c
            .query_row(
                "SELECT context_hash,context_json FROM artifact_context",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        let proofs=c.prepare("SELECT record_id,recipe,digests_json FROM reference_component_digests ORDER BY record_id").unwrap().query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let original_source = fs::read(fixture.0.path().join("packs/items/a.json")).unwrap();
        if corruption == "snapshot name" {
            let mut raw: serde_json::Value = serde_json::from_slice(&original_source).unwrap();
            raw["name"] = serde_json::json!("Altered cached name");
            let source = atlas_foundry_model::admit_document_source(
                "Item",
                atlas_foundry_model::SourceContext::new("fixture", "packs/items/a.json", "$"),
                &serde_json::to_vec(&raw).unwrap(),
            )
            .unwrap()
            .model
            .unwrap();
            c.execute("UPDATE record_bodies SET snapshot=? WHERE record_id=(SELECT record_id FROM records WHERE key='items:aaaaaaaaaaaaaaaa')",[atlas_index::test_support::altered_snapshot_for_fixture(&source).unwrap()]).unwrap();
        } else {
            atlas_index::test_support::alter_retained_html_for_fixture(
                &c,
                &"items:aaaaaaaaaaaaaaaa".parse().unwrap(),
                "/system/description/value",
                "Original localized text",
                "Altered cached prose",
            )
            .unwrap();
        }
        assert!(
            c.query_row("SELECT maintenance FROM workspace", [], |r| r
                .get::<_, Option<String>>(0))
                .unwrap()
                .is_none(),
            "fixture must exercise ordinary reuse, not maintenance recovery"
        );
        let workspace = atlas_storage::Workspace::at(&options.output_path).unwrap();
        let error = crate::build_artifact(
            &workspace,
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
            options.clone(),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("component attestation"),
            "{corruption}: {error}"
        );
        assert_eq!(
            fs::read(fixture.0.path().join("packs/items/a.json")).unwrap(),
            original_source
        );
        assert_eq!(
            c.query_row("SELECT reference_revision FROM workspace", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            revision
        );
        assert_eq!(
            c.query_row(
                "SELECT context_hash,context_json FROM artifact_context",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            )
            .unwrap(),
            context
        );
        let after=c.prepare("SELECT record_id,recipe,digests_json FROM reference_component_digests ORDER BY record_id").unwrap().query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(after, proofs);
        assert_eq!(
            c.query_row("SELECT name FROM saved_lists WHERE id=1", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Kept proof notes"
        );
        assert!(
            c.query_row("SELECT maintenance FROM workspace", [], |r| r
                .get::<_, Option<String>>(0))
                .unwrap()
                .is_none()
        );
    }
}

fn uniform_source(count: usize) -> SourceFixture {
    let fixture = SourceFixture::new();
    for name in ["a", "b"] {
        fs::remove_file(fixture.0.path().join(format!("packs/items/{name}.json"))).unwrap();
    }
    for ordinal in 0..count {
        fixture.document(
            &format!("root{ordinal:03}"),
            &format!("{ordinal:016x}"),
            "Independent",
            "<p>Same bounded source body.</p>",
        );
    }
    fixture
}

#[test]
fn production_admission_batches_count_real_sessions_and_bound_simultaneous_payloads() {
    for count in [1usize, 32, 129] {
        let fixture = uniform_source(count);
        fixture.build("prior");
        let options = fixture.options("prior");
        let workspace = atlas_storage::Workspace::at(&options.output_path).unwrap();
        let reuse = ReferenceReuseReader::new(
            &workspace,
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap();
        for subset in [false, true] {
            if subset {
                for ordinal in (1..count).step_by(2) {
                    fixture.document(
                        &format!("root{ordinal:03}"),
                        &format!("{ordinal:016x}"),
                        "Independent",
                        "<p>Changed source body.</p>",
                    );
                }
            }
            let (prepared, metrics) =
                atlas_storage::diagnostics::capture_sql_work(|| prepare(&options, &reuse)).unwrap();
            let prepared = prepared.unwrap();
            let expected_records = if subset { count.div_ceil(2) } else { count };
            let batches = count.div_ceil(PREPARATION_REUSE_BATCH);
            assert_eq!(prepared.work.reusable_record_read_calls, expected_records);
            assert_eq!(prepared.work.reusable_record_read_batches, batches);
            assert_eq!(prepared.work.reusable_layout_read_calls, 0);
            // Identity enumeration opens one snapshot per128 rows, then one empty probe.
            assert_eq!(
                metrics.workspace_connections as usize,
                count.div_ceil(128) + 1 + batches
            );
            assert_eq!(prepared.source_report.reused_admissions, expected_records);
            assert_eq!(prepared.spool.records.len(), count);
            assert_eq!(prepared.work.selection_calls, count);
            let overlap = prepared.work.largest_admission_overlap;
            assert_eq!(overlap.raw_sources, count.min(PREPARATION_REUSE_BATCH));
            assert_eq!(
                overlap.retained_records,
                expected_records.min(if subset { 16 } else { 32 })
            );
            assert_eq!(overlap.prepared_records, 0);
            assert_eq!(overlap.layout_roots, 0);
            assert!(
                prepared.work.largest_admission_batch_source_bytes
                    <= prepared.work.largest_source_bytes * PREPARATION_REUSE_BATCH
            );
            eprintln!(
                "admission_batches records={count} subset={subset} reads={expected_records} batches={batches} workspace_connections={} overlap={overlap:?}",
                metrics.workspace_connections
            );
        }
    }
}

fn tokenizer_only_options(fixture: &SourceFixture) -> BuildArtifactOptions {
    let shared = std::path::PathBuf::from(
        std::env::var_os("ATLAS_TEST_MODEL_CACHE").expect("set pinned tokenizer cache"),
    );
    // Validate the consumed asset through its normal owner before copying it.
    TextEmbeddingTokenizer::load_baseline(&shared).unwrap();
    let private = fixture.0.path().join("tokenizer-only");
    let source = atlas_embedding::baseline_tokenizer_cache_file(&shared);
    let target = atlas_embedding::baseline_tokenizer_cache_file(&private);
    fs::create_dir_all(target.local_path.parent().unwrap()).unwrap();
    fs::copy(source.local_path, &target.local_path).unwrap();
    let mut options = fixture.options("native-structure");
    options.search_profiles = vec![atlas_embedding::EmbeddingRuntimeConfig::new(
        atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        private,
    )];
    assert!(
        !options.search_profiles[0]
            .model_dir()
            .join("onnx/model.onnx")
            .exists()
    );
    options
}

fn prepared_frames(prepared: &PreparedSourceBuild) -> Vec<String> {
    let mut output = Vec::new();
    prepared
        .spool
        .records
        .visit(&mut |frame| {
            output.push(serde_json::to_string(&frame)?);
            Ok(())
        })
        .unwrap();
    output
}

#[test]
#[ignore = "requires pinned BGE tokenizer only; private structural publication, no weights/inference"]
fn production_layout_batches_follow_active_phase_preserve_flags_and_bound_sessions() {
    for count in [1usize, 32, 129] {
        let fixture = uniform_source(count);
        let mut options = tokenizer_only_options(&fixture);
        crate::build::publish_fixture_without_inference(&options).unwrap();
        let workspace = atlas_storage::Workspace::at(&options.output_path).unwrap();
        let reuse = ReferenceReuseReader::new(
            &workspace,
            atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION,
        )
        .unwrap();
        let mut expected = None;
        let mut expected_native = None;
        for numerical in [true, false] {
            for layouts in [true, false] {
                options.reuse_embeddings = numerical;
                options.reuse_passage_layouts = layouts;
                let (prepared, metrics) =
                    atlas_storage::diagnostics::capture_sql_work(|| prepare(&options, &reuse))
                        .unwrap();
                let prepared = prepared.unwrap();
                let batches = count.div_ceil(PREPARATION_REUSE_BATCH);
                assert_eq!(prepared.work.reusable_record_read_calls, count);
                assert_eq!(prepared.work.reusable_record_read_batches, batches);
                assert_eq!(
                    prepared.work.reusable_layout_read_calls,
                    if layouts { count } else { 0 }
                );
                assert_eq!(
                    prepared.work.reusable_layout_read_batches,
                    if layouts { batches } else { 0 }
                );
                assert_eq!(
                    metrics.workspace_connections as usize,
                    count.div_ceil(128) + 1 + batches + if layouts { batches } else { 0 }
                );
                assert_eq!(prepared.source_report.reused_admissions, count);
                assert_eq!(prepared.spool.records.len(), count);
                assert_eq!(prepared.reused_sections > 0, layouts);
                assert!(prepared.work.largest_semantic_overlap.raw_sources <= 32);
                assert!(prepared.work.largest_semantic_overlap.retained_records <= 32);
                assert_eq!(
                    prepared.work.largest_semantic_overlap.prepared_records,
                    count.min(32)
                );
                assert_eq!(
                    prepared.work.largest_semantic_overlap.layout_roots,
                    if layouts { count.min(32) } else { 0 }
                );
                assert!(prepared.work.largest_semantic_overlap.total() <= 64);
                let frames = prepared_frames(&prepared);
                if let Some(expected) = &expected {
                    assert_eq!(&frames, expected);
                } else {
                    expected = Some(frames);
                }
                let mut native = Vec::new();
                let mut occurrences = Vec::new();
                prepared
                    .native_inputs
                    .visit(&mut |input| {
                        native.push(serde_json::to_string(&input)?);
                        Ok(())
                    })
                    .unwrap();
                prepared
                    .occurrences
                    .visit(&mut |occurrence| {
                        occurrences.push(serde_json::to_string(&occurrence)?);
                        Ok(())
                    })
                    .unwrap();
                let actual_native = (native, occurrences);
                if let Some(expected) = &expected_native {
                    assert_eq!(&actual_native, expected);
                } else {
                    expected_native = Some(actual_native);
                }
                eprintln!(
                    "layout_batches records={count} numerical={numerical} layouts={layouts} batches={batches} workspace_connections={} overlap={:?}",
                    metrics.workspace_connections, prepared.work.largest_semantic_overlap
                );
            }
        }
    }
}

#[test]
#[ignore = "requires pinned BGE tokenizer only; private structural publication, no weights/inference"]
fn excluded_changed_renamed_duplicates_do_not_read_corrupt_layouts_but_active_roots_do() {
    let fixture = uniform_source(3);
    let mut options = tokenizer_only_options(&fixture);
    crate::build::publish_fixture_without_inference(&options).unwrap();
    let c = Connection::open(&options.output_path).unwrap();
    assert_eq!(c.execute("UPDATE semantic_locations SET section_id=NULL,chunk_ordinal=NULL,chunk_start=NULL,chunk_end=NULL WHERE location_id=(SELECT min(su.location_id) FROM semantic_units su JOIN records r USING(record_id) WHERE r.key='items:0000000000000001' AND su.unit_kind='passage')", []).unwrap(),1);
    fixture.document(
        "root001",
        "0000000000000001",
        "Changed",
        "<p>Changed description.</p>",
    );
    fs::rename(
        fixture.0.path().join("packs/items/root001.json"),
        fixture.0.path().join("packs/items/renamed.json"),
    )
    .unwrap();
    fixture.document(
        "duplicate",
        "0000000000000001",
        "Duplicate",
        "<p>Collision evidence.</p>",
    );
    let prepared = prepare_fixture(&options).unwrap();
    assert_eq!(
        prepared
            .source_report
            .identity_unavailable
            .get("DuplicateId"),
        Some(&2)
    );
    assert_eq!(prepared.spool.records.len(), 2);
    assert_eq!(prepared.work.reusable_layout_read_calls, 2);
    assert!(prepared.reused_sections > 0);
    assert!(
        prepared_frames(&prepared)
            .iter()
            .all(|frame| !frame.contains("items:0000000000000001"))
    );
    fs::remove_file(fixture.0.path().join("packs/items/duplicate.json")).unwrap();
    for numerical in [true, false] {
        options.reuse_embeddings = numerical;
        let error = prepare_fixture(&options)
            .err()
            .expect("active corrupt layout must fail");
        assert!(
            error.to_string().contains("canonical section ownership"),
            "{error}"
        );
    }
    options.reuse_passage_layouts = false;
    assert_eq!(prepare_fixture(&options).unwrap().spool.records.len(), 3);
    options.reuse_passage_layouts = true;
    options.search_profiles.clear();
    let no_profiles = prepare_fixture(&options).unwrap();
    assert_eq!(no_profiles.work.reusable_layout_read_calls, 0);
    assert_eq!(no_profiles.spool.records.len(), 3);
}
