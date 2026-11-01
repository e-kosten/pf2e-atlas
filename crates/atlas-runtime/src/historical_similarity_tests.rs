//! Released19→20 admission and rollback, without applying any later migration.
use super::{MIGRATIONS, register_extensions};
use atlas_index::IndexBuildInput;
use atlas_storage::Workspace;
use rusqlite::{Connection, params, types::Value};

#[derive(Debug, PartialEq)]
struct TableSnapshot {
    table: String,
    rows: Vec<Vec<Value>>,
}

fn tables(c: &Connection) -> Vec<String> {
    c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND name<>'schema_migrations' ORDER BY name")
        .unwrap().query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap()
}

fn snapshot(c: &Connection, names: &[String]) -> Vec<TableSnapshot> {
    names
        .iter()
        .map(|name| {
            let mut statement = c
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let columns = statement.column_count();
            let rows = statement
                .query_map([], |row| (0..columns).map(|n| row.get(n)).collect())
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            TableSnapshot {
                table: name.clone(),
                rows,
            }
        })
        .collect()
}

fn ledger(c: &Connection) -> Vec<(u32, String, String)> {
    c.prepare("SELECT version,name,checksum FROM schema_migrations ORDER BY version")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn context(c: &Connection) -> serde_json::Value {
    let text: String = c
        .query_row("SELECT context_json FROM artifact_context", [], |row| {
            row.get(0)
        })
        .unwrap();
    serde_json::from_str(&text).unwrap()
}

fn set_context(c: &Connection, value: &serde_json::Value) {
    let text = serde_json::to_string(value).unwrap();
    c.execute(
        "UPDATE artifact_context SET context_json=?,context_hash=?",
        params![text, atlas_search::test_support::hash(text.as_bytes())],
    )
    .unwrap();
}

fn fixture(current_core: bool) -> (tempfile::TempDir, Workspace, IndexBuildInput) {
    register_extensions().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(directory.path().join("released19.sqlite")).unwrap();
    workspace.migrate(&MIGRATIONS[..19], |_| Ok(())).unwrap();
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "hazards",
            "Actor",
            serde_json::json!({
                "_id":"aaaaaaaaaaaaaaaa", "name":"Hazard", "type":"hazard",
                "system":{"details":{"description":"<p>A narrow fissure</p>","reset":"<p>Returns at dawn</p>"}}
            }),
        ),
    ]);
    {
        let owner = workspace.update_owner().unwrap();
        let maintenance = owner
            .maintenance(19, "genuine released19 reference fixture")
            .unwrap();
        atlas_index::test_support::publish_pre_core_fixture(&maintenance, &input).unwrap();
    }
    let write = workspace.write(19).unwrap();
    let mut stored = context(write.connection());
    assert!(stored.get("selected_sections_recipe").is_none());
    assert!(stored.get("admission").is_none());
    if current_core {
        stored["core_sections_recipe"] = serde_json::json!(atlas_domain::CORE_SECTIONS_RECIPE);
        set_context(write.connection(), &stored);
    }
    write.connection().execute_batch("INSERT INTO saved_lists(id,list_key,slug,name,created_at,updated_at) VALUES(91,'kept','kept','Saved exact bytes','before','after');").unwrap();
    write.commit().unwrap();
    (directory, workspace, input)
}

fn assert_prefix20_preserved(workspace: &Workspace) {
    let read = workspace.read(19).unwrap();
    let names = tables(read.connection());
    let before = snapshot(read.connection(), &names);
    let before_ledger = ledger(read.connection());
    let vectors = crate::test_support::occurrence_vector_bits(read.connection());
    drop(read);
    workspace.migrate(&MIGRATIONS[..20], |_| Ok(())).unwrap();
    let read = workspace.read(20).unwrap();
    assert_eq!(snapshot(read.connection(), &names), before);
    assert_eq!(
        crate::test_support::occurrence_vector_bits(read.connection()),
        vectors
    );
    let after = ledger(read.connection());
    assert_eq!(&after[..19], before_ledger.as_slice());
    assert_eq!(after.len(), 20);
    assert_eq!(after[19].1, "similarity-parts");
    let mut wanted = names;
    wanted.push("similarity_parts".into());
    wanted.sort();
    assert_eq!(tables(read.connection()), wanted);
    assert_eq!(
        read.connection()
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        20
    );
}

#[test]
fn authentic_missing_new_fields_preserve_parts_saved_vectors_context_and_prefix() {
    for current_core in [false, true] {
        let (_directory, workspace, input) = fixture(current_core);
        // Independently spell the preparation recipe used by main's selector.
        let context = &input.context;
        let shared = serde_json::json!({
            "snapshot_version":context.snapshot_version,"source_contract":context.source_contract,
            "indexing_locale":context.indexing_locale,"locale_catalog_sha256":context.locale_catalog_sha256,
            "english_catalog_sha256":context.english_catalog_sha256,"localization_policy":context.localization_policy,
            "audience":context.audience,"content_interpretation_version":context.content_interpretation_version,
            "content_selection_version":context.content_selection_version,"asset_policy":context.asset_policy,
            "relationship_policy":context.relationship_policy,
        });
        let preparation = atlas_search::test_support::hash(&serde_json::to_vec(&shared).unwrap());
        assert_eq!(preparation, context.preparation_sha256().unwrap());
        assert_prefix20_preserved(&workspace);
        let read = workspace.read(20).unwrap();
        let actual = read.connection().prepare("SELECT selector,field_path,body_hash,role FROM similarity_parts ORDER BY field_path").unwrap()
            .query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?)))
            .unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        let expected = [
            (
                "/system/details/description",
                "A narrow fissure",
                "description",
            ),
            ("/system/details/reset", "Returns at dawn", "reset"),
        ]
        .into_iter()
        .map(|(field, text, role)| {
            let body = atlas_search::test_support::hash(text.as_bytes());
            let selector = atlas_search::test_support::hash(
                &serde_json::to_vec(&(
                    input.records[0].record.key(),
                    Vec::<serde_json::Value>::new(),
                    field,
                    0usize,
                    &input.records[0].content_hash,
                    &preparation,
                    &body,
                ))
                .unwrap(),
            );
            (
                format!("part:{selector}"),
                field.to_owned(),
                body,
                role.to_owned(),
            )
        })
        .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn current_declared_catalog_profile_and_historical_layout_fail_atomically() {
    for fault in [
        "catalog_missing",
        "catalog_changed",
        "profile_identity",
        "profile_manifest",
        "profile_dimensions",
        "native_missing",
        "native_extra",
        "native_ddl",
        "selected_empty",
        "selected_unknown",
        "selected_null",
        "admission",
        "unknown_field",
        "context_hash",
    ] {
        let (_directory, workspace, input) = fixture(true);
        let write = workspace.write(19).unwrap();
        let c = write.connection();
        let table = format!(
            "semantic_vectors_{}",
            atlas_search::test_support::hash(
                &serde_json::to_vec(&input.context.search_profiles[0]).unwrap()
            )
        );
        match fault {
            "catalog_missing" => {
                assert_eq!(
                    c.execute("DELETE FROM query_field_catalog WHERE field='kind'", [])
                        .unwrap(),
                    1
                );
            }
            "catalog_changed" => {
                assert_eq!(
                    c.execute(
                        "UPDATE query_field_catalog SET definition_json='{}' WHERE field='kind'",
                        []
                    )
                    .unwrap(),
                    1
                );
            }
            "profile_identity" => {
                let mut identity = serde_json::to_value(&input.context.search_profiles[0]).unwrap();
                identity["revision"] = serde_json::json!("different-full-identity");
                c.execute(
                    "UPDATE search_profiles SET identity_json=? WHERE profile_id=?",
                    params![
                        serde_json::to_string(&identity).unwrap(),
                        input.context.search_profiles[0].profile_id.as_str()
                    ],
                )
                .unwrap();
            }
            "profile_manifest" => {
                let mut stored = context(c);
                stored["search_profiles"].as_array_mut().unwrap().pop();
                set_context(c, &stored);
            }
            "profile_dimensions" => {
                c.execute("UPDATE search_profiles SET dimensions=dimensions+1", [])
                    .unwrap();
            }
            "native_missing" => {
                c.execute_batch(&format!("DROP TABLE {table}")).unwrap();
            }
            "native_extra" => {
                c.execute_batch("CREATE VIRTUAL TABLE semantic_vectors_unexpected USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[768] distance_metric=cosine)").unwrap();
            }
            "native_ddl" => {
                c.execute_batch(&format!("DROP TABLE {table}; CREATE VIRTUAL TABLE {table} USING vec0(unit_id INTEGER PRIMARY KEY,embedding FLOAT[32] distance_metric=cosine)")).unwrap();
            }
            "context_hash" => {
                c.execute(
                    "UPDATE artifact_context SET context_hash=printf('%064d',0)",
                    [],
                )
                .unwrap();
            }
            other => {
                let mut stored = context(c);
                let (field, value) = match other {
                    "selected_empty" => ("selected_sections_recipe", serde_json::json!("")),
                    "selected_unknown" => (
                        "selected_sections_recipe",
                        serde_json::json!("unknown/v999"),
                    ),
                    "selected_null" => ("selected_sections_recipe", serde_json::Value::Null),
                    "admission" => ("admission", serde_json::Value::Null),
                    _ => ("unrecognized_main_field", serde_json::json!(true)),
                };
                stored[field] = value;
                set_context(c, &stored);
            }
        }
        let names = tables(c);
        let before = snapshot(c, &names);
        let before_ledger = ledger(c);
        write.commit().unwrap();
        let error = workspace
            .migrate(&MIGRATIONS[..20], |_| Ok(()))
            .unwrap_err();
        assert!(!error.to_string().is_empty(), "{fault}");
        let read = workspace.read(19).unwrap();
        assert_eq!(tables(read.connection()), names, "{fault}");
        assert_eq!(snapshot(read.connection(), &names), before, "{fault}");
        assert_eq!(ledger(read.connection()), before_ledger, "{fault}");
        assert_eq!(
            read.connection()
                .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
                .unwrap(),
            19,
            "{fault}"
        );
    }
}

#[test]
fn main_query_lexical_and_core_upgrade_branches_defer_open_checks() {
    for branch in ["query", "lexical", "core"] {
        let (_directory, workspace, _) = fixture(true);
        let write = workspace.write(19).unwrap();
        let mut stored = context(write.connection());
        match branch {
            "query" => stored["query_catalog_version"] = serde_json::json!(7),
            "lexical" => {
                stored["lexical_selection_version"] = serde_json::json!("curated-lexical/v1")
            }
            _ => stored["core_sections_recipe"] = serde_json::json!(""),
        }
        set_context(write.connection(), &stored);
        assert_eq!(
            write
                .connection()
                .execute("DELETE FROM query_field_catalog WHERE field='kind'", [])
                .unwrap(),
            1
        );
        write.commit().unwrap();
        assert_prefix20_preserved(&workspace);
        let read = workspace.read(20).unwrap();
        assert!(
            read.connection()
                .query_row("SELECT count(*) FROM similarity_parts", [], |row| row
                    .get::<_, usize>(0))
                .unwrap()
                > 0
        );
    }
}

#[test]
fn retired_preparation_defers_without_decoding_poisoned_record_bodies() {
    let (_directory, workspace, _) = fixture(true);
    let write = workspace.write(19).unwrap();
    let mut stored = context(write.connection());
    stored["localization_policy"] =
        serde_json::json!("selected-source-catalog/english-fallback/v1");
    set_context(write.connection(), &stored);
    write
        .connection()
        .execute("UPDATE record_bodies SET snapshot=zeroblob(4)", [])
        .unwrap();
    write.commit().unwrap();
    assert_prefix20_preserved(&workspace);
    let read = workspace.read(20).unwrap();
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM similarity_parts", [], |row| row
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
}
