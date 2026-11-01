// Separate binary: fork/exec briefly inherits other threads' file locks on Unix.
// Keep process-lifecycle checks separate from concurrent in-process unit fixtures.
use atlas_storage::*;
use std::process::Command;
const FIRST: Migration = Migration {
    backfill: None,
    version: 1,
    name: "user-state",
    sql: "CREATE TABLE saved(id INTEGER PRIMARY KEY,title TEXT NOT NULL) STRICT; INSERT INTO saved VALUES(1,'retained');",
};
fn fixture() -> (tempfile::TempDir, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("atlas.sqlite")).unwrap();
    workspace.migrate(&[FIRST], |_| Ok(())).unwrap();
    (dir, workspace)
}

fn worker(workspace: &Workspace, mode: &str) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "process_worker", "--nocapture"])
        .env("ATLAS_STORAGE_TEST_PATH", workspace.path())
        .env("ATLAS_STORAGE_TEST_MODE", mode)
        .output()
        .unwrap()
}

#[test]
fn process_worker() {
    let Some(path) = std::env::var_os("ATLAS_STORAGE_TEST_PATH") else {
        return;
    };
    let workspace = Workspace::at(path).unwrap();
    match std::env::var("ATLAS_STORAGE_TEST_MODE").unwrap().as_str() {
        "mapping" => mapping_worker(&workspace),
        "blocked" => assert!(matches!(workspace.read(1), Err(StorageError::Busy))),
        "write" => {
            let write = workspace.write(1).unwrap();
            write
                .connection()
                .execute("UPDATE saved SET title='process write'", [])
                .unwrap();
            write.commit().unwrap();
        }
        "interrupt-reconstruction" => {
            let migration = Migration {
                version: 2,
                name: "interrupted-reconstruction",
                sql: "",
                backfill: Some(DataBackfill {
                    id: "interrupted-reconstruction/v1",
                    reconstruct_tables: true,
                    apply: |connection| {
                        connection.execute_batch("CREATE TABLE saved_rebuilt(id INTEGER PRIMARY KEY,title TEXT NOT NULL) STRICT; INSERT INTO saved_rebuilt SELECT * FROM saved; DROP TABLE saved;")?;
                        std::process::exit(72);
                    },
                }),
            };
            workspace.migrate(&[FIRST, migration], |_| Ok(())).unwrap();
        }
        "interrupt" => {
            let owner = workspace.update_owner().unwrap();
            let maintenance = owner.maintenance(1, "interrupted source update").unwrap();
            maintenance
                .connection()
                .execute_batch("BEGIN IMMEDIATE; UPDATE saved SET title='uncommitted';")
                .unwrap();
            // Exit deliberately bypasses destructors, simulating abrupt termination.
            std::process::exit(71);
        }
        _ => panic!("unknown test worker mode"),
    }
}

#[test]
fn process_admission_and_abrupt_exit_preserve_maintenance_and_data() {
    let (_dir, workspace) = fixture();
    assert!(worker(&workspace, "write").status.success());
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(1, "maintenance").unwrap();
    assert!(worker(&workspace, "blocked").status.success());
    maintenance
        .publish::<_, rusqlite::Error>(|_| Ok(()))
        .unwrap();
    drop(maintenance);
    drop(owner);
    assert_eq!(worker(&workspace, "interrupt").status.code(), Some(71));
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Maintenance(_))
    ));
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(1, "recover").unwrap();
    assert_eq!(
        maintenance
            .connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "process write"
    );
    maintenance
        .publish::<_, rusqlite::Error>(|_| Ok(()))
        .unwrap();
    drop(maintenance);
    drop(owner);
    let read = workspace.read(1).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 2);
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "process write"
    );
}

#[test]
fn bounded_mapping_lifecycle_in_isolated_process() {
    let (_dir, workspace) = fixture();
    let output = worker(&workspace, "mapping");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn mapping_worker(workspace: &Workspace) {
    use rusqlite::OptionalExtension;
    let effective = |connection: &rusqlite::Connection| {
        connection
            .query_row("PRAGMA main.mmap_size", [], |row| row.get::<_, u64>(0))
            .optional()
            .unwrap()
            .unwrap_or(0)
    };
    let max = {
        let connection = rusqlite::Connection::open(workspace.path()).unwrap();
        connection
            .execute_batch("PRAGMA journal_mode=WAL;")
            .unwrap();
        connection
            .pragma_update(Some("main"), "mmap_size", i64::MAX)
            .unwrap();
        effective(&connection)
    };
    // Failed admitted sessions must close/unmap before capacity becomes reusable.
    for _ in 0..16 {
        assert!(matches!(
            workspace.read(2),
            Err(StorageError::Schema { .. })
        ));
    }
    let requested = if usize::BITS >= 64 {
        std::fs::metadata(workspace.path())
            .unwrap()
            .len()
            .min(2 * 1024 * 1024 * 1024)
            .min(max)
    } else {
        0
    };
    let first = workspace.read(1).unwrap();
    let second = workspace.read(1).unwrap();
    let excess = workspace.read(1).unwrap();
    assert_eq!(effective(first.connection()), requested);
    assert_eq!(effective(second.connection()), requested);
    assert_eq!(effective(excess.connection()), 0);
    println!("requested={requested}, platform_max={max}");
    let title = |connection: &rusqlite::Connection| {
        connection
            .query_row("SELECT title FROM saved WHERE id=1", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap()
    };
    assert_eq!(title(first.connection()), "retained");
    workspace
        .write_action::<_, StorageError>(1, |write| {
            assert_eq!(effective(write.connection()), 0);
            write
                .connection()
                .execute("UPDATE saved SET title='committed' WHERE id=1", [])?;
            Ok(())
        })
        .unwrap();
    assert_eq!(title(first.connection()), "retained");
    drop(excess);
    let fresh = workspace.read(1).unwrap();
    assert_eq!(effective(fresh.connection()), 0);
    assert_eq!(title(fresh.connection()), "committed");
    drop(first);
    let replacement = workspace.read(1).unwrap();
    assert_eq!(effective(replacement.connection()), requested);
    drop((replacement, fresh, second));
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(1, "publication").unwrap();
    assert_eq!(effective(maintenance.connection()), 0);
    maintenance
        .publish::<_, rusqlite::Error>(|connection| {
            connection.execute("UPDATE saved SET title='published' WHERE id=1", [])?;
            Ok(())
        })
        .unwrap();
    drop(maintenance);
    drop(owner);
    let read = workspace.read(1).unwrap();
    assert_eq!(effective(read.connection()), requested);
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(title(read.connection()), "published");
    drop(read);
    let moved = workspace.path().with_extension("moved");
    std::fs::rename(workspace.path(), &moved).unwrap();
    std::fs::rename(&moved, workspace.path()).unwrap();
    {
        let connection = rusqlite::Connection::open(workspace.path()).unwrap();
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")
            .unwrap();
        assert_eq!(title(&connection), "published");
    }
    let second_migration = Migration {
        version: 2,
        name: "extra column",
        sql: "ALTER TABLE saved ADD COLUMN extra INTEGER NOT NULL DEFAULT 7;",
        backfill: None,
    };
    workspace
        .migrate(&[FIRST, second_migration], |_| Ok(()))
        .unwrap();
    let backup = rusqlite::Connection::open(workspace.upgrade_backup_path()).unwrap();
    assert_eq!(title(&backup), "published");
    assert_eq!(
        backup
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        1
    );
    drop(backup);
    let read = workspace.read(2).unwrap();
    assert_eq!(title(read.connection()), "published");
    assert_eq!(
        read.connection()
            .query_row("SELECT extra FROM saved", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        7
    );
}

#[test]
fn abrupt_reconstruction_exit_restores_previous_schema_data_and_enforcement() {
    let (_directory, workspace) = fixture();
    assert_eq!(
        worker(&workspace, "interrupt-reconstruction").status.code(),
        Some(72)
    );
    workspace.migrate(&[FIRST], |_| Ok(())).unwrap();
    let read = workspace.read(1).unwrap();
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved WHERE id=1", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "retained"
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(
        read.connection()
            .prepare("SELECT * FROM saved_rebuilt")
            .is_err()
    );
    assert!(
        read.connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, bool>(0))
            .unwrap()
    );
    assert!(workspace.upgrade_backup_path().exists());
}
