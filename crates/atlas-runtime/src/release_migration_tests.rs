use super::*;
use crate::test_support::{occurrence_vector_bits, sql_rows};

const LEDGER: &str = "SELECT version,name,checksum FROM schema_migrations ORDER BY version";

fn predecessor(path: &Path) -> Workspace {
    register_extensions().unwrap();
    let workspace = Workspace::at(path).unwrap();
    workspace.migrate(MIGRATIONS, |_| Ok(())).unwrap();
    workspace
}

#[test]
fn release_checkpoint_preserves_populated_schema_41_and_matches_fresh_creation() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = predecessor(&directory.path().join("old.sqlite"));
    let input = atlas_search::test_support::input_with_both_profiles(vec![
        atlas_search::test_support::record(
            "actors",
            "Actor",
            serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","name":"Goblin","type":"npc"}),
        ),
    ]);
    atlas_index::test_support::publish_input_in(&workspace, 41, &input).unwrap();
    let write = workspace.write(41).unwrap();
    write
        .connection()
        .execute_batch(
            "INSERT INTO principals VALUES(2,'other','Other');
         INSERT INTO saved_lists(id,owner_id,list_key,slug,name,created_at,updated_at)
         VALUES(17,2,'list','same','Retained list','before','before');
         INSERT INTO saved_list_tags(list_id,tag,created_at) VALUES(17,'retained','before');
         INSERT INTO parties(party_key,owner_id,name) VALUES('party',2,'Retained party');
         INSERT INTO encounters(id,owner_id,encounter_key,slug,name,status,created_at,updated_at)
         VALUES(28,2,'encounter','same','Retained encounter','draft','before','before');
         INSERT INTO encounter_participants(
             id,encounter_id,participant_key,participant_kind,position,display_name,
             initiative_order,created_at,updated_at)
         VALUES(39,28,'participant','pc',1,'Retained participant',1,'before','before');",
        )
        .unwrap();
    let facts = [
        "SELECT * FROM workspace",
        "SELECT * FROM saved_lists",
        "SELECT * FROM saved_list_tags",
        "SELECT * FROM parties",
        "SELECT * FROM encounters",
        "SELECT * FROM encounter_participants",
        "SELECT * FROM embedding_inputs ORDER BY input_id",
        "SELECT * FROM embedding_models ORDER BY model_id",
    ];
    let before = facts.map(|sql| sql_rows(write.connection(), sql));
    let old_ledger = sql_rows(write.connection(), LEDGER);
    let vectors = occurrence_vector_bits(write.connection());
    assert!(!vectors.is_empty());
    write.commit().unwrap();
    initialize_workspace(workspace.path()).unwrap();
    initialize_workspace(workspace.path()).unwrap();

    let current = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let c = current.connection();
    assert_eq!(facts.map(|sql| sql_rows(c, sql)), before);
    assert_eq!(occurrence_vector_bits(c), vectors);
    assert_eq!(
        sql_rows(historical_backup(&workspace).connection(), LEDGER),
        old_ledger
    );
    let fresh = initialize_workspace(directory.path().join("fresh.sqlite")).unwrap();
    let fresh_read = fresh.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    let fresh_c = fresh_read.connection();
    assert_eq!(sql_rows(c, LEDGER), sql_rows(fresh_c, LEDGER));
    assert_eq!(sql_rows(c, LEDGER).len(), 1);

    let schema = r#"SELECT type,name,tbl_name,
        replace(replace(sql,'CREATE TABLE "','CREATE TABLE '),'" (',' (')
        FROM sqlite_schema WHERE tbl_name NOT LIKE 'native_vectors_%' ORDER BY type,name"#;
    assert_eq!(sql_rows(c, schema), sql_rows(fresh_c, schema));
    assert!(sql_rows(c, "PRAGMA foreign_key_check").is_empty());
    drop(current);
    drop(fresh_read);
    for workspace in [&workspace, &fresh] {
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        for sql in [
            "INSERT INTO saved_lists(list_key,slug,name,created_at,updated_at) VALUES('new','new','New','now','now')",
            "INSERT INTO parties(party_key,name) VALUES('new','New')",
            "INSERT INTO encounters(encounter_key,slug,name,status,created_at,updated_at) VALUES('new','new','New','draft','now','now')",
        ] {
            let error = write.connection().execute(sql, []).unwrap_err();
            assert!(error.to_string().contains("owner_id"), "{error}");
        }
    }
}

#[test]
fn release_bridge_rejects_bad_history_and_rolls_back_schema_drift() {
    let directory = tempfile::tempdir().unwrap();
    for (label, sql, expected, backup) in [
        (
            "older",
            "PRAGMA user_version=40",
            "must be upgraded to schema 41",
            false,
        ),
        ("future", "PRAGMA user_version=43", "43", false),
        (
            "ledger",
            "UPDATE schema_migrations SET checksum='bad' WHERE version=39",
            "ledger differs",
            false,
        ),
        (
            "missing",
            "DELETE FROM schema_migrations WHERE version=39",
            "ledger differs",
            false,
        ),
        (
            "extra",
            "INSERT INTO schema_migrations(version,name,checksum) VALUES(0,'extra','extra')",
            "ledger differs",
            false,
        ),
        (
            "misnumbered",
            "UPDATE schema_migrations SET version=42 WHERE version=39",
            "ledger differs",
            false,
        ),
        (
            "unrelated",
            "PRAGMA application_id=0",
            "non-workspace database",
            false,
        ),
        (
            "missing-local",
            "DELETE FROM principals WHERE principal_key='local'",
            "missing its local principal",
            true,
        ),
        (
            "invalid-owner",
            "PRAGMA foreign_keys=OFF; INSERT INTO parties(party_key,owner_id,name) VALUES('invalid',99,'Invalid')",
            "foreign-key check failed",
            true,
        ),
        (
            "shape",
            "CREATE INDEX owner_drift ON encounters(owner_id)",
            "saved-owner tables or their dependencies differ",
            true,
        ),
    ] {
        let workspace = predecessor(&directory.path().join(format!("{label}.sqlite")));
        let c = rusqlite::Connection::open(workspace.path()).unwrap();
        c.execute_batch(sql).unwrap();
        let version = sql_rows(&c, "PRAGMA user_version");
        let history = sql_rows(&c, LEDGER);
        let error = initialize_workspace(workspace.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{label}: {error}");
        assert_eq!(workspace.upgrade_backup_path().is_file(), backup);
        assert_eq!(sql_rows(&c, "PRAGMA user_version"), version);
        assert_eq!(sql_rows(&c, LEDGER), history);
    }
}

#[test]
fn release_bridge_rolls_back_after_reconstruction_and_can_append() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = predecessor(&directory.path().join("old.sqlite"));
    let path = workspace.path();
    let error = workspace
        .migrate_history(RELEASE_HISTORY, |_| {
            Err(atlas_storage::StorageError::Migration(
                "injected failure".into(),
            ))
        })
        .unwrap_err();
    assert!(error.to_string().contains("injected failure"));
    let read = workspace.read(41).unwrap();
    assert_eq!(
        sql_rows(read.connection(), LEDGER),
        sql_rows(historical_backup(&workspace).connection(), LEDGER)
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT dflt_value FROM pragma_table_info('encounters') WHERE name='owner_id'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "1"
    );
    drop(read);
    let upgraded = initialize_workspace(path).unwrap();
    let fresh = initialize_workspace(directory.path().join("fresh.sqlite")).unwrap();
    let mut appended = RELEASE_HISTORY.migrations.to_vec();
    appended.push(Migration {
        version: 43,
        name: "future-fixture",
        sql: "CREATE TABLE future_fixture (id INTEGER PRIMARY KEY); INSERT INTO future_fixture VALUES(7);",
        backfill: None,
    });
    let history = MigrationHistory {
        migrations: &appended,
        ..RELEASE_HISTORY
    };
    for workspace in [upgraded, fresh] {
        workspace.migrate_history(history, |_| Ok(())).unwrap();
        workspace.migrate_history(history, |_| Ok(())).unwrap();
        let read = workspace.read(43).unwrap();
        assert_eq!(sql_rows(read.connection(), LEDGER).len(), 2);
        assert_eq!(
            sql_rows(read.connection(), "SELECT id FROM future_fixture").len(),
            1
        );
    }
}

#[test]
fn release_bridge_process_worker() {
    let Some(path) = std::env::var_os("ATLAS_RELEASE_TEST_WORKSPACE") else {
        return;
    };
    if std::env::var_os("ATLAS_RELEASE_TEST_INTERRUPT").is_some() {
        register_extensions().unwrap();
        let workspace = Workspace::at(path).unwrap();
        workspace
            .migrate_history(RELEASE_HISTORY, |_| std::process::exit(72))
            .unwrap();
    } else {
        initialize_workspace(path).unwrap();
    }
}

#[test]
fn release_bridge_survives_process_exit_and_cooperating_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = predecessor(&directory.path().join("old.sqlite"));
    let old_ledger = {
        let write = workspace.write(41).unwrap();
        write
            .connection()
            .execute("UPDATE workspace SET maintenance='pending maintenance'", [])
            .unwrap();
        let ledger = sql_rows(write.connection(), LEDGER);
        write.commit().unwrap();
        ledger
    };
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "workspace::release_migration_tests::release_bridge_process_worker",
        ])
        .env("ATLAS_RELEASE_TEST_WORKSPACE", workspace.path());
    let interrupted = command
        .env("ATLAS_RELEASE_TEST_INTERRUPT", "1")
        .output()
        .unwrap();
    assert_eq!(interrupted.status.code(), Some(72));
    {
        let connection = rusqlite::Connection::open(workspace.path()).unwrap();
        assert_eq!(
            connection
                .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
                .unwrap(),
            41
        );
        assert_eq!(sql_rows(&connection, LEDGER), old_ledger);
    }
    command.env_remove("ATLAS_RELEASE_TEST_INTERRUPT");
    let mut first = command.spawn().unwrap();
    let mut second = command.spawn().unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    // The backup correctly retains maintenance, so inspect it outside product admission.
    let backup = rusqlite::Connection::open_with_flags(
        workspace.upgrade_backup_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(sql_rows(&backup, LEDGER), old_ledger);
    assert_eq!(
        sql_rows(&backup, "SELECT maintenance FROM workspace"),
        sql_rows(
            &rusqlite::Connection::open(workspace.path()).unwrap(),
            "SELECT maintenance FROM workspace",
        )
    );
    assert_eq!(
        workspace.check_migration_history(RELEASE_HISTORY).unwrap(),
        WORKSPACE_SCHEMA_VERSION
    );
    assert!(matches!(
        workspace.read(WORKSPACE_SCHEMA_VERSION),
        Err(atlas_storage::StorageError::Maintenance(reason)) if reason == "pending maintenance"
    ));
}
