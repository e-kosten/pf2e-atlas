// Separate binary: fork/exec briefly inherits other threads' file locks on Unix.
// Keep process-lifecycle checks separate from concurrent in-process unit fixtures.
use atlas_storage::*;
use std::process::Command;
const FIRST: Migration = Migration {
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
        "blocked" => assert!(matches!(workspace.read(1), Err(StorageError::Busy))),
        "write" => {
            let write = workspace.write(1).unwrap();
            write
                .connection()
                .execute("UPDATE saved SET title='process write'", [])
                .unwrap();
            write.commit().unwrap();
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
}
