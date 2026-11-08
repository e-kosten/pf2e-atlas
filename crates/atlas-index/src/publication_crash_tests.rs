use crate::{test_support::*, *};
use atlas_storage::{Migration, Workspace};
use rusqlite::{Connection, params, types::Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, Registry, layer::Context, prelude::*};

struct PauseAt {
    stage: String,
    signal: PathBuf,
}
impl Layer<Registry> for PauseAt {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, Registry>) {
        struct Stage<'a> {
            expected: &'a str,
            matches: bool,
        }
        impl Visit for Stage<'_> {
            fn record_str(&mut self, field: &Field, value: &str) {
                self.matches |= field.name() == "stage" && value == self.expected;
            }
            fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
        }
        let mut stage = Stage {
            expected: &self.stage,
            matches: false,
        };
        event.record(&mut stage);
        if stage.matches {
            std::fs::write(&self.signal, "ready").unwrap();
            loop {
                thread::park();
            }
        }
    }
}
fn kill_at_stage(test: &str, stage: &str, path: &Path, while_paused: impl FnOnce()) {
    let signal = path.with_extension("ready");
    let log = path.with_extension("child.log");
    struct OwnedChild(std::process::Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--nocapture", "--test-threads=1"])
            .env("ATLAS_TEST_CRASH_STAGE", stage)
            .env("ATLAS_TEST_CRASH_PATH", path)
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&log).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(45);
    while !signal.exists() {
        if let Some(status) = child.0.try_wait().unwrap() {
            panic!(
                "child exited {status} before {stage}: {}",
                std::fs::read_to_string(&log).unwrap()
            );
        }
        if Instant::now() >= deadline {
            child.0.kill().unwrap();
            child.0.wait().unwrap();
            panic!(
                "child did not reach {stage}: {}",
                std::fs::read_to_string(&log).unwrap()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    while_paused();
    child.0.kill().unwrap();
    assert!(!child.0.wait().unwrap().success());
}
fn crash_child() -> Option<(PathBuf, impl Subscriber + Send + Sync)> {
    let stage = std::env::var("ATLAS_TEST_CRASH_STAGE").ok()?;
    let path = PathBuf::from(std::env::var_os("ATLAS_TEST_CRASH_PATH").unwrap());
    let subscriber = Registry::default().with(PauseAt {
        stage,
        signal: path.with_extension("ready"),
    });
    Some((path, subscriber))
}
fn next_input() -> IndexBuildInput {
    let mut next = crate::foundation_tests::fixture(true);
    next.context.source_revision = Some("after-crash".into());
    next.records[0] = crate::foundation_tests::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Dragon Guard","type":"npc","system":{"details":{"level":{"value":9}},"attributes":{"hp":{"max":81},"ac":{"value":24}},"traits":{"value":["dragon"],"rarity":"rare","size":{"value":"lg"}}}}),
    );
    next.records[0].preparation_context_hash = next.context.preparation_sha256().unwrap();
    for unit in &mut next.semantic_units {
        unit.input_hash = crate::codec::sha256(unit.input_hash.as_bytes());
        unit.vector = vec![0.; next.context.search_profiles[0].dimensions];
        unit.vector[11] = 1.;
    }
    next
}
fn rows(c: &Connection) -> BTreeMap<String, Vec<Vec<Value>>> {
    let mut rows = reference_preservation_rows(c).unwrap();
    rows.remove("pending_embedding_values");
    // Retry batches can durably register unpublished inputs before activation.
    rows.get_mut("embedding_inputs")
        .unwrap()
        .retain(|input| input.last() != Some(&Value::Null));
    // Keep concrete row IDs and Core bindings as well as the migration oracle's
    // logical rows: a partial projection publication must not normalize away.
    let tables = c.prepare("SELECT name FROM sqlite_schema WHERE type='table' AND (name LIKE 'core_%' OR name IN ('selected_sections','semantic_locations')) ORDER BY name").unwrap()
        .query_map([], |row| row.get::<_,String>(0)).unwrap()
        .collect::<Result<Vec<_>,_>>().unwrap();
    for table in tables {
        let quoted = format!("\"{}\"", table.replace('"', "\"\""));
        let columns = c
            .prepare(&format!("SELECT * FROM {quoted}"))
            .unwrap()
            .column_count();
        let order = (1..=columns)
            .map(|column| column.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let values = c
            .prepare(&format!("SELECT * FROM {quoted} ORDER BY {order}"))
            .unwrap()
            .query_map([], |row| {
                (0..columns)
                    .map(|column| row.get(column))
                    .collect::<Result<Vec<Value>, _>>()
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        rows.insert(format!("raw:{table}"), values);
    }
    rows
}
fn save_list(workspace: &Workspace) {
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write.connection().execute_batch("INSERT INTO saved_lists(list_key,slug,name,created_at,updated_at) VALUES('keep','keep','Saved name','before','before')").unwrap();
    write.commit().unwrap();
}

#[test]
fn repair_crashes_commit_replacement_config_and_marker_together() {
    if let Some((path, subscriber)) = crash_child() {
        let _guard = tracing::subscriber::set_default(subscriber);
        let workspace = Workspace::at(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let repair = reference_repair(&owner).unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let expected = ReferenceReuseReader::fresh(&repair);
        let mut replacement = crate::foundation_tests::fixture(false);
        replacement.context.source_revision = Some("repaired".into());
        let spool = publication_spool_in(scratch.path(), &replacement, expected.expected().clone())
            .unwrap();
        PreparedPublication::new_rebuild(
            repair,
            scratch.path(),
            spool,
            RequestedReferenceConfig {
                source_root: path.with_file_name("corrected"),
                manifest_path: None,
                indexing_locale: "en".into(),
                search_profiles: vec![],
            },
        )
        .unwrap()
        .publish()
        .unwrap();
        panic!("requested repair crash milestone was not observed");
    }

    for stage in ["publication_before_commit", "publication_after_commit"] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("repair.sqlite");
        publish_input(&path, &crate::foundation_tests::fixture(false)).unwrap();
        let workspace = Workspace::at(&path).unwrap();
        save_list(&workspace);
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        write
            .connection()
            .execute_batch(
                "UPDATE artifact_context SET context_json='{}';
             UPDATE workspace SET maintenance='interrupted maintenance';",
            )
            .unwrap();
        write.commit().unwrap();
        kill_at_stage(
            "publication_crash_tests::repair_crashes_commit_replacement_config_and_marker_together",
            stage,
            &path,
            || {},
        );
        let committed = stage == "publication_after_commit";
        let inspect = Connection::open(&path).unwrap();
        let state: (u64, Option<String>, String, usize) = inspect
            .query_row(
                "SELECT reference_revision,maintenance,context_json,
                (SELECT count(*) FROM requested_reference_config)
             FROM workspace CROSS JOIN artifact_context",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(state.0, if committed { 2 } else { 1 });
        assert_eq!(
            state.1.as_deref(),
            (!committed).then_some("interrupted maintenance")
        );
        assert_eq!(state.2.contains("repaired"), committed);
        assert_eq!(state.3, usize::from(committed));
        assert_eq!(
            inspect
                .query_row("SELECT name FROM saved_lists", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "Saved name"
        );
        assert_eq!(
            workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).is_ok(),
            committed
        );
    }
}

#[test]
fn publication_crashes_preserve_exact_old_or_new_state_and_allow_the_next_build() {
    if let Some((path, subscriber)) = crash_child() {
        let _guard = tracing::subscriber::set_default(subscriber);
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        prepare_publication_with_inferred_values(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &next_input(),
        )
        .unwrap()
        .publish()
        .unwrap();
        panic!("requested crash milestone was not observed");
    }
    // Independent successful incremental publication supplies the exact new
    // state; expected rows are not calculated using publisher internals.
    let control_directory = tempfile::tempdir().unwrap();
    let control_path = control_directory.path().join("control.sqlite");
    publish_input(&control_path, &crate::foundation_tests::fixture(true)).unwrap();
    let control_workspace = initialize_workspace(&control_path).unwrap();
    save_list(&control_workspace);
    let owner = control_workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_with_inferred_values(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &next_input(),
    )
    .unwrap();
    let read = control_workspace
        .read(TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let expected_pending = reference_preservation_rows(read.connection())
        .unwrap()
        .remove("pending_embedding_values")
        .unwrap();
    assert!(!expected_pending.is_empty());
    drop(read);
    publication.publish().unwrap();
    let read = control_workspace
        .read(TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap();
    let after = rows(read.connection());
    let new_native = native_preservation_rows(read.connection()).unwrap();
    crate::validate_reference(read.connection()).unwrap();
    drop(read);
    for stage in [
        "native_installation",
        "publication_before_commit",
        "publication_after_commit",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("publication.sqlite");
        publish_input(&path, &crate::foundation_tests::fixture(true)).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        save_list(&workspace);
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let before = rows(read.connection());
        let old_native = native_preservation_rows(read.connection()).unwrap();
        assert_ne!(
            before, after,
            "the update must change published projections"
        );
        let committed = stage == "publication_after_commit";
        kill_at_stage(
            "publication_crash_tests::publication_crashes_preserve_exact_old_or_new_state_and_allow_the_next_build",
            stage,
            &path,
            || {
                assert_eq!(read.reference_revision().unwrap(), 1);
                assert_eq!(rows(read.connection()), before);
                assert_eq!(
                    native_preservation_rows(read.connection()).unwrap(),
                    old_native
                );
                let during = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
                assert_eq!(
                    during.reference_revision().unwrap(),
                    if committed { 2 } else { 1 }
                );
                assert_eq!(
                    rows(during.connection()),
                    if committed {
                        after.clone()
                    } else {
                        before.clone()
                    }
                );
            },
        );
        drop(read);
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(
            read.reference_revision().unwrap(),
            if committed { 2 } else { 1 }
        );
        let context = crate::read_stored_reference_context(read.connection())
            .unwrap()
            .unwrap();
        assert_eq!(
            context.source_revision.as_deref() == Some("after-crash"),
            committed
        );
        let pending = reference_preservation_rows(read.connection())
            .unwrap()
            .remove("pending_embedding_values")
            .unwrap();
        if committed {
            assert!(pending.is_empty());
        } else {
            assert_eq!(pending, expected_pending);
        }
        assert_eq!(
            read.connection()
                .query_row("SELECT hp_maximum FROM actor_projection", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            if committed { 81 } else { 80 },
        );
        if committed {
            let expected = next_input();
            for unit in &expected.semantic_units {
                let model = &expected.context.search_profiles[0];
                let input = atlas_embedding::NativeInputDescriptor {
                    input_sha256: unit.input_hash.clone(),
                    token_count: unit.input_token_count,
                };
                let id = crate::native_values::lookup_input(read.connection(), model, &input)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    crate::native_values::read(read.connection(), id, model).unwrap(),
                    unit.vector
                );
            }
            assert_eq!(
                native_preservation_rows(read.connection()).unwrap(),
                new_native
            );
            assert_eq!(rows(read.connection()), after);
        } else {
            assert_eq!(rows(read.connection()), before);
            assert_eq!(
                native_preservation_rows(read.connection()).unwrap(),
                old_native
            );
        }
        crate::validate_reference(read.connection()).unwrap();
        drop(read);
        publish_input(&path, &next_input()).unwrap();
        crate::validate_artifact(&path).unwrap();
        assert_eq!(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            if committed { 3 } else { 2 }
        );
    }
}

fn packing_workspace(path: &Path) -> Workspace {
    let workspace = Workspace::at(path).unwrap();
    workspace.migrate(&[Migration { version: 1, name: "packing-crash-fixture", backfill: None,
        sql: "CREATE TABLE saved(value TEXT); INSERT INTO saved VALUES('kept'); CREATE TABLE built(value TEXT); INSERT INTO built VALUES('old'); CREATE TABLE pending(value TEXT); INSERT INTO pending VALUES('useful'); CREATE TABLE embedding_models(model_id INTEGER PRIMARY KEY,identity_json TEXT); CREATE TABLE embedding_inputs(input_id INTEGER PRIMARY KEY,model_id INTEGER,published_hash BLOB);" }], |_| Ok(())).unwrap();
    workspace
}
#[test]
fn killed_repack_restores_vector_bytes_allocation_and_reopens_admission() {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let model = crate::foundation_tests::fixture(true)
        .context
        .search_profiles
        .remove(0);
    let table = crate::schema::native_vector_table(&model).unwrap();
    if let Some((path, subscriber)) = crash_child() {
        let _guard = tracing::subscriber::set_default(subscriber);
        let workspace = Workspace::at(path).unwrap();
        let owner = workspace.update_owner().unwrap();
        owner
            .publish_incremental(1, |c| -> Result<(), IndexError> {
                c.execute("UPDATE built SET value='new'", [])?;
                c.execute("DELETE FROM pending", [])?;
                crate::native_allocation::compact_profiles(
                    c,
                    &crate::foundation_tests::fixture(true).context,
                )?;
                Ok(())
            })
            .unwrap();
        panic!("requested repack crash milestone was not observed");
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("packing.sqlite");
    let workspace = packing_workspace(&path);
    let write = workspace.write(1).unwrap();
    let c = write.connection();
    c.execute_batch(&crate::schema::native_vector_ddl(&model).unwrap())
        .unwrap();
    c.execute(
        "INSERT INTO embedding_models VALUES(1,?)",
        [serde_json::to_string(&model).unwrap()],
    )
    .unwrap();
    for id in 1..=4096 {
        let mut vector = vec![0.; model.dimensions];
        vector[id as usize % 3] = 1.;
        let bytes = crate::read_units::vector_bytes(&vector);
        c.execute(
            "INSERT INTO embedding_inputs VALUES(?,1,?)",
            params![id, crate::embedding_values::hash_bytes(&bytes)],
        )
        .unwrap();
        c.execute(
            &format!("INSERT INTO {table}(input_id,embedding) VALUES(?,?)"),
            params![id, bytes],
        )
        .unwrap();
    }
    for id in (2..=4096).step_by(2) {
        c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [id])
            .unwrap();
        c.execute("DELETE FROM embedding_inputs WHERE input_id=?", [id])
            .unwrap();
    }
    let before = native_preservation_rows(c).unwrap();
    let blocks: usize = c
        .query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |row| {
            row.get(0)
        })
        .unwrap();
    write.commit().unwrap();
    kill_at_stage(
        "publication_crash_tests::killed_repack_restores_vector_bytes_allocation_and_reopens_admission",
        "native_repack_replaced",
        &path,
        || {},
    );
    let read = workspace.read(1).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 0);
    assert_eq!(native_preservation_rows(read.connection()).unwrap(), before);
    assert_eq!(
        read.connection()
            .query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |row| {
                row.get::<_, usize>(0)
            })
            .unwrap(),
        blocks
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT value FROM saved", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "kept"
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT value FROM built", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "old"
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT value FROM pending", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "useful"
    );
    drop(read);
    workspace
        .update_owner()
        .unwrap()
        .publish(1, |c| -> Result<(), IndexError> {
            crate::native_allocation::compact_profiles(
                c,
                &crate::foundation_tests::fixture(true).context,
            )?;
            Ok(())
        })
        .unwrap();
    let read = workspace.read(1).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(native_preservation_rows(read.connection()).unwrap(), before);
    assert!(
        read.connection()
            .query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |row| {
                row.get::<_, usize>(0)
            })
            .unwrap()
            < blocks
    );
}
