//! Real released-prefix migration, source-preparation readiness and saved-state proof.
use super::*;
use serde_json::json;

#[test]
fn remaster_v29_upgrade_rolls_back_then_preserves_state_until_source_reprepare() {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("upgrade.sqlite");
    let old = Workspace::at(&path).unwrap();
    old.migrate(&MIGRATIONS[..26], |_| Ok(())).unwrap();
    let records=[("aaaaaaaaaaaaaaaa","Attack of Opportunity"),("bbbbbbbbbbbbbbbb","Reactive Strike")].map(|(id,name)|atlas_search::test_support::record("actions","Item",json!({"_id":id,"name":name,"type":"action","system":{"description":{"value":"<p>Current retained text.</p>"}}})));
    let mut input = atlas_search::test_support::input(records.into(), false);
    input.context.remaster_inputs_identity.clear();
    {
        let owner = old.update_owner().unwrap();
        let maintenance = owner
            .maintenance(26, "authentic old schema fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_section_fixture(&maintenance, &input).unwrap();
    }
    old.migrate(&MIGRATIONS[..29], |_| Ok(())).unwrap();
    atlas_index::test_support::set_relationship_policy(&path, 29, "typed-authored-occurrences/v7")
        .unwrap();
    let snapshot = |c: &rusqlite::Connection| {
        crate::test_support::sql_rows(
            c,
            "SELECT l.list_key,i.record_key,i.note,i.position,i.record_title_snapshot,e.encounter_key,e.note,e.revision FROM saved_lists l JOIN saved_list_items i ON i.list_id=l.id CROSS JOIN encounters e ORDER BY i.position",
        )
    };
    let saved;
    let key;
    let before_revision;
    {
        let write = old.write(29).unwrap();
        write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,description,created_at,updated_at) VALUES(1,'kept','kept','Kept','Keep notes','before','before'); INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,added_at,updated_at) VALUES(1,'actions:aaaaaaaaaaaaaaaa',0,'Exact old selection','Attack of Opportunity','before','before'); INSERT INTO encounters(id,encounter_key,slug,name,note,status,created_at,updated_at,revision) VALUES(1,'enc','enc','Encounter','Keep encounter','running','before','before',7);").unwrap();
        saved = snapshot(write.connection());
        key = write
            .connection()
            .query_row("SELECT workspace_key FROM workspace", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap();
        before_revision = write.reference_revision().unwrap();
        write.commit().unwrap();
    }
    let failed = old.migrate(MIGRATIONS, |_| {
        Err(atlas_storage::StorageError::Migration(
            "deliberate post-schema validation failure".into(),
        ))
    });
    assert!(failed.is_err());
    {
        let read = old.read(29).unwrap();
        assert_eq!(snapshot(read.connection()), saved);
        assert!(
            read.connection()
                .prepare("SELECT 1 FROM remaster_pairs")
                .is_ok()
        );
        assert!(
            read.connection()
                .prepare("SELECT 1 FROM remaster_declarations")
                .is_err()
        );
        assert_eq!(read.reference_revision().unwrap(), before_revision);
    }
    assert!(
        std::fs::read_dir(directory.path())
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().contains("backup")),
        "migration creates a retained backup before mutation"
    );
    let upgraded = upgrade_historical_fixture(&path).unwrap();
    {
        let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(snapshot(read.connection()), saved);
        assert_eq!(
            atlas_index::classify_reference_upgrade(read.connection()).unwrap(),
            atlas_index::ReferenceUpgrade::SourcePreparation
        );
        assert_eq!(read.reference_revision().unwrap(), before_revision + 1);
        assert!(
            read.connection()
                .prepare("SELECT 1 FROM remaster_pairs")
                .is_ok()
        );
    }
    assert!(
        open_workspace_index(&path).is_err(),
        "readiness cannot silently serve incomplete remaster metadata"
    );
    // No external source is available yet: a second normal startup still preserves state.
    initialize_workspace(&path).unwrap();
    assert_eq!(
        upgraded
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        before_revision + 1
    );
    let source = directory.path().join("source");
    std::fs::create_dir_all(source.join("packs/actions")).unwrap();
    std::fs::create_dir_all(source.join("packs/journals")).unwrap();
    std::fs::create_dir_all(source.join("static/lang")).unwrap();
    std::fs::write(source.join("module.json"),r#"{"packs":[{"name":"actions","label":"Actions","type":"Item","path":"packs/actions"},{"name":"journals","label":"Journals","type":"JournalEntry","path":"packs/journals"}]}"#).unwrap();
    std::fs::write(source.join("static/lang/en.json"), "{}").unwrap();
    for (id, name) in [
        ("aaaaaaaaaaaaaaaa", "Attack of Opportunity"),
        ("bbbbbbbbbbbbbbbb", "Reactive Strike"),
    ] {
        std::fs::write(source.join(format!("packs/actions/{id}.json")),json!({"_id":id,"name":name,"type":"action","system":{"description":{"value":"<p>Current retained text.</p>"}}}).to_string()).unwrap();
    }
    std::fs::write(source.join("packs/journals/remaster.json"),json!({"_id":"cccccccccccccccc","name":"Remaster Changes","pages":[{"_id":"dddddddddddddddd","name":"Class Features","type":"text","text":{"format":1,"content":"<table><tr><th>Old</th><th>Status</th><th>New</th></tr><tr><td>@UUID[Compendium.pf2e.actions.Item.aaaaaaaaaaaaaaaa]{Attack of Opportunity}</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.bbbbbbbbbbbbbbbb]{Reactive Strike}</td></tr></table>"}}]}).to_string()).unwrap();
    atlas_ingest::build_artifact(
        &upgraded,
        WORKSPACE_SCHEMA_VERSION,
        atlas_ingest::BuildArtifactOptions {
            source_root: source.clone(),
            output_path: path.clone(),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_passage_layouts: true,
            embedding_batch_size: 8,
        },
    )
    .unwrap();
    let reader = open_workspace_index(&path).unwrap();
    assert_eq!(reader.artifact_instance().workspace_key(), key);
    assert_eq!(
        reader
            .read_remaster_links(&"actions:aaaaaaaaaaaaaaaa".parse().unwrap())
            .unwrap()
            .len(),
        1
    );
    drop(reader);
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(snapshot(read.connection()), saved);
    assert!(read.reference_revision().unwrap() > before_revision);
    let current_revision = read.reference_revision().unwrap();
    drop(read);
    let paths = crate::ResolvedAtlasPaths {
        managed_source_root: directory.path().join("managed"),
        source_root_explicit: true,
        mode: crate::ResolvedPathMode::Global,
        repo_root: None,
        source_root: source,
        embedding_cache_root: directory.path().join("models"),
        index_path: path.clone(),
    };
    let options = crate::RuntimeSetupOptions {
        target: crate::SetupTarget::Records,
        offline: true,
        ..Default::default()
    };
    let no_op = crate::setup::ensure_setup(&paths, options.clone());
    assert!(no_op.ready, "{no_op:?}");
    assert!(
        no_op.build.is_none(),
        "normal current setup skips publication"
    );
    assert_eq!(
        upgraded
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        current_revision
    );
    let reader = open_workspace_index(&path).unwrap();
    let mut older_context = reader.context().clone();
    let upstream_fingerprint = older_context.source_fingerprint.clone();
    let preparation = older_context.preparation_sha256().unwrap();
    older_context.remaster_inputs_identity = "0".repeat(64);
    assert_eq!(
        older_context.preparation_sha256().unwrap(),
        preparation,
        "curated metadata is independent of content rendering"
    );
    drop(reader);
    let write = upgraded.write(WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute(
            "UPDATE artifact_context SET context_hash=?,context_json=?",
            rusqlite::params![
                older_context.sha256().unwrap(),
                serde_json::to_string(&older_context).unwrap()
            ],
        )
        .unwrap();
    write.commit().unwrap();
    assert!(
        open_workspace_index(&path).is_err(),
        "a changed compiled curated identity closes normal offline reference admission"
    );
    let check = crate::setup::ensure_setup(
        &paths,
        crate::RuntimeSetupOptions {
            check: true,
            ..options.clone()
        },
    );
    assert!(!check.ready);
    assert_eq!(
        upgraded
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        current_revision
    );
    let refreshed = crate::setup::ensure_setup(&paths, options.clone());
    assert!(refreshed.ready, "{refreshed:?}");
    assert!(
        refreshed.build.is_some(),
        "curated-only identity change requires normal publication despite unchanged upstream source"
    );
    let reader = open_workspace_index(&path).unwrap();
    assert_eq!(reader.context().source_fingerprint, upstream_fingerprint);
    let next_revision = reader.artifact_instance().reference_revision();
    assert!(next_revision > current_revision);
    drop(reader);
    assert!(crate::setup::ensure_setup(&paths, options).build.is_none());
    let read = upgraded.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), next_revision);
    assert_eq!(snapshot(read.connection()), saved);
}
