use super::*;

const FIRST: Migration = Migration {
    backfill: None,
    version: 1,
    name: "user-state",
    sql: "CREATE TABLE saved(id INTEGER PRIMARY KEY,title TEXT NOT NULL) STRICT; INSERT INTO saved VALUES(1,'retained');",
};
pub(super) fn fixture() -> (tempfile::TempDir, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("atlas.sqlite")).unwrap();
    workspace.migrate(&[FIRST], |_| Ok(())).unwrap();
    (dir, workspace)
}

#[test]
fn read_session_allows_local_validation_tables_but_cannot_change_saved_data() {
    let (_dir, workspace) = fixture();
    let read = workspace.read(1).unwrap();
    read.connection()
        .execute_batch("CREATE TEMP TABLE validation_probe(value INTEGER);")
        .unwrap();
    assert!(
        read.connection()
            .execute("UPDATE saved SET title='changed'", [])
            .is_err()
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
}

#[test]
fn abandoned_maintenance_requires_validated_recovery_without_new_revision() {
    let (_dir, workspace) = fixture();
    let owner = workspace.update_owner().unwrap();
    drop(owner.maintenance(1, "interrupted publication").unwrap());
    drop(owner);
    // Opening the known schema authenticates its ledger but cannot reopen it.
    workspace.migrate(&[FIRST], |_| Ok(())).unwrap();
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Maintenance(_))
    ));
    let owner = workspace.update_owner().unwrap();
    let rejected: StorageResult<bool> = owner.recover_maintenance(1, |_| {
        Err(StorageError::Migration("invalid committed state".into()))
    });
    assert!(rejected.is_err());
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Maintenance(_))
    ));
    let recovered: StorageResult<bool> = owner.recover_maintenance(1, |connection| {
        assert_eq!(
            connection.query_row("SELECT title FROM saved", [], |row| row.get::<_, String>(0))?,
            "retained"
        );
        Ok(())
    });
    assert!(recovered.unwrap());
    assert_eq!(workspace.read(1).unwrap().reference_revision().unwrap(), 0);
}

#[test]
fn pending_schema_upgrade_retains_interrupted_maintenance_until_recovery() {
    let (_dir, workspace) = fixture();
    let owner = workspace.update_owner().unwrap();
    drop(owner.maintenance(1, "interrupted publication").unwrap());
    drop(owner);
    let second = Migration {
        backfill: None,
        version: 2,
        name: "additional field",
        sql: "ALTER TABLE saved ADD COLUMN note TEXT;",
    };
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    assert!(matches!(
        workspace.read(2),
        Err(StorageError::Maintenance(_))
    ));
    let owner = workspace.update_owner().unwrap();
    let result: StorageResult<bool> = owner.recover_maintenance(2, |connection| {
        assert_eq!(
            connection.query_row("SELECT title FROM saved", [], |row| row.get::<_, String>(0))?,
            "retained"
        );
        Ok(())
    });
    assert!(result.unwrap());
    assert_eq!(workspace.read(2).unwrap().reference_revision().unwrap(), 0);
}

#[test]
fn failed_upgrade_rolls_back_schema_data_and_ledger_and_keeps_backup() {
    let (_dir, workspace) = fixture();
    let second = Migration {
        backfill: None,
        version: 2,
        name: "backfill",
        sql: "ALTER TABLE saved ADD COLUMN extra INTEGER; UPDATE saved SET extra=7,title='bad';",
    };
    assert!(
        workspace
            .migrate(&[FIRST, second], |_| Err(StorageError::Migration(
                "failed invariant".into()
            )))
            .is_err()
    );
    let reader = workspace.read(1).unwrap();
    assert_eq!(
        reader
            .connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
    assert_eq!(
        reader
            .connection()
            .query_row(
                "SELECT count(*) FROM pragma_table_info('saved') WHERE name='extra'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        reader
            .connection()
            .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    let backup = rusqlite::Connection::open(workspace.upgrade_backup_path()).unwrap();
    assert_eq!(
        backup
            .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "delete"
    );
    assert_eq!(
        backup
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        backup
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
}

#[test]
fn successful_upgrade_rejects_old_process_and_changed_migration_history() {
    let (_dir, workspace) = fixture();
    let second = Migration {
        backfill: None,
        version: 2,
        name: "backfill",
        sql: "ALTER TABLE saved ADD COLUMN extra INTEGER; UPDATE saved SET extra=7;",
    };
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Schema { actual: 2, .. })
    ));
    assert_eq!(
        workspace
            .read(2)
            .unwrap()
            .connection()
            .query_row("SELECT extra FROM saved", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        7
    );
    let changed = Migration {
        backfill: None,
        version: 1,
        name: "user-state",
        sql: "SELECT 1",
    };
    assert!(matches!(
        workspace.migrate(&[changed], |_| Ok(())),
        Err(StorageError::Schema { .. })
    ));
    let second = Migration {
        backfill: None,
        version: 2,
        name: "backfill",
        sql: "ALTER TABLE saved ADD COLUMN extra INTEGER; UPDATE saved SET extra=7;",
    };
    assert!(matches!(
        workspace.migrate(&[changed, second], |_| Ok(())),
        Err(StorageError::Migration(_))
    ));
}

#[test]
fn maintenance_waits_for_readers_and_failed_publication_requires_recovery() {
    let (_dir, workspace) = fixture();
    let owner = workspace.update_owner().unwrap();
    let read = workspace.read(1).unwrap();
    assert!(matches!(
        owner.maintenance(1, "source update"),
        Err(StorageError::Busy)
    ));
    drop(read);
    let maintenance = owner.maintenance(1, "source update").unwrap();
    assert!(matches!(workspace.read(1), Err(StorageError::Busy)));
    let failed: Result<(), rusqlite::Error> = maintenance.publish(|c| {
        c.execute("UPDATE saved SET title='bad'", [])?;
        c.cache_flush()?;
        c.execute("INSERT INTO saved VALUES(1,'duplicate')", [])?;
        Ok(())
    });
    assert!(failed.is_err());
    drop(maintenance);
    drop(owner);
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Maintenance(_))
    ));
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(1, "resume").unwrap();
    let recovered: Result<(), rusqlite::Error> = maintenance.publish(|c| {
        assert_eq!(
            c.query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))?,
            "retained"
        );
        c.execute("UPDATE saved SET title='published'", [])?;
        Ok(())
    });
    recovered.unwrap();
    drop(maintenance);
    drop(owner);
    let read = workspace.read(1).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "published"
    );
}

#[test]
fn bounded_snapshot_is_coherent_and_new_operation_observes_committed_state() {
    let (_dir, workspace) = fixture();
    let reader = workspace.read(1).unwrap();
    let writer = workspace.write(1).unwrap();
    writer
        .connection()
        .execute("UPDATE saved SET title='new'", [])
        .unwrap();
    writer.commit().unwrap();
    assert_eq!(
        reader
            .connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
    drop(reader);
    assert_eq!(
        workspace
            .read(1)
            .unwrap()
            .connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "new"
    );
    let writer = workspace.write(1).unwrap();
    writer
        .connection()
        .execute("UPDATE saved SET title='dropped'", [])
        .unwrap();
    drop(writer);
    assert_eq!(
        workspace
            .read(1)
            .unwrap()
            .connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "new"
    );
}

#[test]
fn unknown_databases_are_not_initialized_or_erased() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("unknown.sqlite")).unwrap();
    let connection = rusqlite::Connection::open(workspace.path()).unwrap();
    connection
        .execute_batch("CREATE TABLE important(value TEXT); INSERT INTO important VALUES('keep');")
        .unwrap();
    assert!(matches!(
        workspace.migrate(&[FIRST], |_| Ok(())),
        Err(StorageError::Unsupported(_))
    ));
    assert_eq!(
        connection
            .query_row("SELECT value FROM important", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "keep"
    );
}

#[test]
fn flushed_vec_and_fts_participate_in_the_same_rollback_as_user_state() {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let workspace = Workspace::at(dir.path().join("atlas.sqlite")).unwrap();
    let vector = Migration {
        backfill: None,
        version: 2,
        name: "search",
        sql: "CREATE VIRTUAL TABLE vectors USING vec0(id INTEGER PRIMARY KEY,embedding float[4]); CREATE VIRTUAL TABLE text_search USING fts5(name);",
    };
    workspace.migrate(&[FIRST, vector], |_| Ok(())).unwrap();
    let write = workspace.write(2).unwrap();
    write.connection().execute_batch("INSERT INTO vectors VALUES(1,zeroblob(16)); INSERT INTO text_search VALUES('sample'); UPDATE saved SET title='bad';").unwrap();
    write.connection().execute_batch("CREATE VIRTUAL TABLE new_vectors USING vec0(id INTEGER PRIMARY KEY,embedding float[4]); INSERT INTO new_vectors VALUES(1,zeroblob(16));").unwrap();
    write.connection().cache_flush().unwrap();
    drop(write);
    let read = workspace.read(2).unwrap();
    assert!(
        !read
            .connection()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='new_vectors')",
                [],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM vectors", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM text_search WHERE text_search MATCH 'sample'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
}

#[test]
fn composed_action_rolls_back_on_error_and_unwind() {
    let (_dir, workspace) = fixture();
    let failed: StorageResult<()> = workspace.write_action(1, |session| {
        session
            .connection()
            .execute("UPDATE saved SET title='partial'", [])?;
        Err(StorageError::Migration("action failed".into()))
    });
    assert!(failed.is_err());
    let panic = std::panic::catch_unwind(|| {
        let _: StorageResult<()> = workspace.write_action(1, |session| {
            session
                .connection()
                .execute("UPDATE saved SET title='unwound'", [])?;
            panic!("action panicked");
        });
    });
    assert!(panic.is_err());
    assert_eq!(
        workspace
            .read(1)
            .unwrap()
            .connection()
            .query_row("SELECT title FROM saved", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
    let succeeded: StorageResult<()> = workspace.write_action(1, |session| {
        session
            .connection()
            .execute("UPDATE saved SET title='complete'", [])?;
        Ok(())
    });
    succeeded.unwrap();
    assert_eq!(
        workspace
            .read(1)
            .unwrap()
            .connection()
            .query_row("SELECT title FROM saved", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "complete"
    );
}

#[test]
fn successive_upgrades_replace_backup_with_latest_pre_upgrade_state() {
    let (_dir, workspace) = fixture();
    let second = Migration {
        backfill: None,
        version: 2,
        name: "second",
        sql: "UPDATE saved SET title='second';",
    };
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    let third = Migration {
        backfill: None,
        version: 3,
        name: "third",
        sql: "UPDATE saved SET title='third';",
    };
    workspace
        .migrate(&[FIRST, second, third], |_| Ok(()))
        .unwrap();
    let backup = rusqlite::Connection::open(workspace.upgrade_backup_path()).unwrap();
    assert_eq!(
        backup
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        backup
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "second"
    );
}

#[test]
fn current_schema_startup_does_not_contend_with_publisher_preparation() {
    let (_dir, workspace) = fixture();
    let _publisher = workspace.update_owner().unwrap();
    let mut validated = false;
    workspace
        .migrate(&[FIRST], |connection| {
            validated = true;
            assert_eq!(
                connection.query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))?,
                "retained"
            );
            assert!(
                connection
                    .execute("UPDATE saved SET title='changed'", [])
                    .is_err()
            );
            Ok(())
        })
        .unwrap();
    assert!(validated);
    assert_eq!(workspace.read(1).unwrap().reference_revision().unwrap(), 0);
    assert!(!workspace.upgrade_backup_path().exists());
    assert!(matches!(
        workspace.migrate(&[FIRST], |_| Err(StorageError::Migration(
            "validator failed".into()
        ))),
        Err(StorageError::Migration(_))
    ));
}

#[test]
fn startup_fast_path_authenticates_history_and_rejects_newer_schema() {
    let (_dir, workspace) = fixture();
    let _publisher = workspace.update_owner().unwrap();
    let changed = Migration {
        backfill: None,
        sql: "SELECT 1",
        ..FIRST
    };
    assert!(matches!(
        workspace.migrate(&[changed], |_| Ok(())),
        Err(StorageError::Migration(_))
    ));
    let connection = rusqlite::Connection::open(workspace.path()).unwrap();
    connection
        .execute("UPDATE schema_migrations SET checksum='tampered'", [])
        .unwrap();
    assert!(matches!(
        workspace.migrate(&[FIRST], |_| Ok(())),
        Err(StorageError::Migration(_))
    ));
    connection.pragma_update(None, "user_version", 2).unwrap();
    assert!(matches!(
        workspace.migrate(&[FIRST], |_| Ok(())),
        Err(StorageError::Schema { actual: 2, .. })
    ));
}

#[test]
fn pending_migration_still_requires_publisher_ownership() {
    let (_dir, workspace) = fixture();
    let publisher = workspace.update_owner().unwrap();
    let second = Migration {
        backfill: None,
        version: 2,
        name: "extra",
        sql: "ALTER TABLE saved ADD COLUMN extra INTEGER;",
    };
    assert!(matches!(
        workspace.migrate(&[FIRST, second], |_| Ok(())),
        Err(StorageError::Busy)
    ));
    assert_eq!(workspace.read(1).unwrap().reference_revision().unwrap(), 0);
    drop(publisher);
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    assert!(workspace.read(2).is_ok());
}

#[test]
fn current_schema_fast_path_never_reopens_abandoned_maintenance() {
    let (_dir, workspace) = fixture();
    let owner = workspace.update_owner().unwrap();
    drop(owner.maintenance(1, "interrupted").unwrap());
    workspace
        .migrate(&[FIRST], |_| panic!("recovery requires domain coordinator"))
        .unwrap();
    assert!(matches!(
        workspace.read(1),
        Err(StorageError::Maintenance(_))
    ));
}

#[test]
fn snapshots_include_live_wal_and_seeds_are_independent() {
    let (dir, workspace) = fixture();
    let writer = rusqlite::Connection::open(workspace.path()).unwrap();
    writer
        .execute_batch("PRAGMA journal_mode=WAL; INSERT INTO saved VALUES(2,'wal row');")
        .unwrap();
    let snapshot = dir.path().join("seed.sqlite");
    workspace.snapshot_to(&snapshot).unwrap();
    let seed = Workspace::at(&snapshot).unwrap();
    assert_eq!(
        seed.read(1)
            .unwrap()
            .connection()
            .query_row("SELECT count(*) FROM saved", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    let target = Workspace::at(dir.path().join("copy.sqlite")).unwrap();
    let admission = target.begin_initialization().unwrap().unwrap();
    admission.install(&snapshot, |_| Ok(())).unwrap();
    drop(admission);
    target
        .write_action(1, |s| -> StorageResult<()> {
            s.connection().execute("DELETE FROM saved", [])?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        seed.read(1)
            .unwrap()
            .connection()
            .query_row("SELECT count(*) FROM saved", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert!(target.begin_initialization().unwrap().is_none());
}

#[test]
fn snapshot_and_seed_admission_preserve_existing_state() {
    let (dir, workspace) = fixture();
    let snapshot = dir.path().join("seed.sqlite");
    let owner = workspace.update_owner().unwrap();
    assert!(matches!(
        workspace.snapshot_to(&snapshot),
        Err(StorageError::Busy)
    ));
    drop(owner);
    workspace.snapshot_to(&snapshot).unwrap();
    let before = std::fs::read(&snapshot).unwrap();
    workspace
        .write_action(1, |session| -> StorageResult<()> {
            session
                .connection()
                .execute("UPDATE saved SET title='new state'", [])?;
            Ok(())
        })
        .unwrap();
    assert!(workspace.snapshot_to(&snapshot).is_err());
    assert_eq!(std::fs::read(&snapshot).unwrap(), before);
    let target = Workspace::at(dir.path().join("copy.sqlite")).unwrap();
    let admission = target.begin_initialization().unwrap().unwrap();
    assert!(
        admission
            .install(&snapshot, |_| Err(StorageError::Unsupported(
                "rejected hash".into()
            )))
            .is_err()
    );
    assert!(!target.path().exists());
    drop(admission);
    std::fs::write(locking::sidecar(target.path(), "-wal"), b"orphan").unwrap();
    assert!(target.begin_initialization().is_err());
}

#[test]
fn typed_backfill_runs_atomically_and_its_identity_is_authenticated() {
    let (_dir, workspace) = fixture();
    let second = Migration {
        version: 2,
        name: "typed-backfill",
        sql: "ALTER TABLE saved ADD COLUMN digest BLOB;",
        backfill: Some(DataBackfill {
            id: "saved-digest/v1",
            apply: |c| {
                c.execute("UPDATE saved SET digest=?", [&[1u8, 2, 3][..]])?;
                Err(StorageError::Migration("injected backfill failure".into()))
            },
        }),
    };
    assert!(workspace.migrate(&[FIRST, second], |_| Ok(())).is_err());
    let read = workspace.read(1).unwrap();
    assert!(
        read.connection()
            .prepare("SELECT digest FROM saved")
            .is_err()
    );
    drop(read);
    let second = Migration {
        backfill: Some(DataBackfill {
            id: "saved-digest/v1",
            apply: |c| {
                c.execute("UPDATE saved SET digest=?", [&[1u8, 2, 3][..]])?;
                Ok(())
            },
        }),
        ..second
    };
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    let changed = Migration {
        backfill: Some(DataBackfill {
            id: "saved-digest/v2",
            ..second.backfill.unwrap()
        }),
        ..second
    };
    assert!(matches!(
        workspace.migrate(&[FIRST, changed], |_| Ok(())),
        Err(StorageError::Migration(_))
    ));
    let read = workspace.read(2).unwrap();
    assert_eq!(
        read.connection()
            .query_row("SELECT digest FROM saved", [], |r| r.get::<_, Vec<u8>>(0))
            .unwrap(),
        [1, 2, 3]
    );
    drop(read);
    workspace
        .write_action(2, |session| -> StorageResult<()> {
            session
                .connection()
                .execute("UPDATE saved SET digest=?", [&[9u8][..]])?;
            Ok(())
        })
        .unwrap();
    workspace.migrate(&[FIRST, second], |_| Ok(())).unwrap();
    assert_eq!(
        workspace
            .read(2)
            .unwrap()
            .connection()
            .query_row("SELECT digest FROM saved", [], |r| r.get::<_, Vec<u8>>(0))
            .unwrap(),
        [9]
    );
}

#[test]
fn physical_reclamation_preserves_saved_data_and_logical_revision() {
    let (_dir, workspace) = fixture();
    let owner = workspace.update_owner().unwrap();
    assert!(!owner.reclaim_space(1).unwrap());
    let write = workspace.write(1).unwrap();
    write.connection().execute_batch("CREATE TABLE obsolete(value BLOB); INSERT INTO obsolete VALUES(zeroblob(40*1024*1024));").unwrap();
    write.commit().unwrap();
    let write = workspace.write(1).unwrap();
    write
        .connection()
        .execute_batch("DROP TABLE obsolete")
        .unwrap();
    write.commit().unwrap();
    // Materialize the committed free pages in the main file before measuring.
    let connection = rusqlite::Connection::open(workspace.path()).unwrap();
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    drop(connection);
    let before = std::fs::metadata(workspace.path()).unwrap().len();
    assert!(before >= 40 * 1024 * 1024);
    assert!(owner.reclaim_space(1).unwrap());
    let after = std::fs::metadata(workspace.path()).unwrap().len();
    assert!(
        before - after >= 32 * 1024 * 1024,
        "before={before}, after={after}"
    );
    assert!(!owner.reclaim_space(1).unwrap());
    drop(owner);
    let read = workspace.read(1).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 0);
    assert_eq!(
        read.connection()
            .query_row("SELECT title FROM saved", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retained"
    );
}

#[test]
fn mapping_is_connection_local_and_writes_remain_unmapped() {
    use rusqlite::OptionalExtension;
    let (_dir, workspace) = fixture();
    let read = workspace.read(1).unwrap();
    let effective: Option<u64> = read
        .connection()
        .query_row("PRAGMA main.mmap_size", [], |r| r.get(0))
        .optional()
        .unwrap();
    if let Some(effective) = effective {
        assert!(effective <= std::fs::metadata(workspace.path()).unwrap().len());
        if read._mapping.is_none() {
            assert_eq!(effective, 0);
        }
    }
    let write = workspace.write(1).unwrap();
    assert_eq!(
        write
            .connection()
            .query_row("PRAGMA main.mmap_size", [], |r| r.get::<_, u64>(0))
            .optional()
            .unwrap()
            .unwrap_or(0),
        0
    );
    drop((read, write));
    let raw = rusqlite::Connection::open(workspace.path()).unwrap();
    assert_eq!(
        raw.query_row("PRAGMA main.mmap_size", [], |r| r.get::<_, u64>(0))
            .optional()
            .unwrap()
            .unwrap_or(0),
        0
    );
}
