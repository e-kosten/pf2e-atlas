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
