use super::*;

#[test]
fn fresh_workspace_has_fourteen_encounter_tables() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("commands-fresh.sqlite")).unwrap();
    let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        read.connection()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE type='table' AND name GLOB 'encounter*'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
        14,
    );
}
