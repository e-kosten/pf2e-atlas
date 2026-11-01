//! Work and parity protection for the actual production membership-page query.
use super::*;
use crate::test_support::{TEST_WORKSPACE_SCHEMA_VERSION, initialize_workspace};
use rusqlite::StatementStatus;

type PageRow = (i64, String, String, usize, Option<Vec<u8>>);

fn fixture(
    rows: i64,
    selected_limit: i64,
) -> (tempfile::TempDir, atlas_storage::Workspace, Vec<PageRow>) {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("page.sqlite")).unwrap();
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let c = write.connection();
    for job in ["selected", "unrelated"] {
        c.execute("INSERT INTO reference_update_jobs(job_key,expected_revision,target_hash,state) VALUES(?,0,?,'preparing')", params![job, "a".repeat(64)]).unwrap();
        c.execute(
            "INSERT INTO native_update_targets(job_key,phase) VALUES(?,'retired')",
            [job],
        )
        .unwrap();
    }
    let models = crate::profile_tests::both().context.search_profiles;
    let identities = models
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect::<Vec<_>>();
    for (index, identity) in identities.iter().enumerate() {
        c.execute(
            "INSERT INTO embedding_models(model_id,identity_json) VALUES(?,?)",
            params![index + 1, identity],
        )
        .unwrap();
    }
    let mut expected = Vec::new();
    for ordinal in 0..rows {
        let id = ordinal * 3 + 7;
        let model = ordinal as usize % identities.len();
        let hash = crate::embedding_values::hash_bytes(&ordinal.to_le_bytes());
        let tokens = ordinal as usize % 100 + 1;
        c.execute("INSERT INTO embedding_inputs(input_id,model_id,input_hash,token_count) VALUES(?,?,?,?)", params![id,model+1,hash,tokens]).unwrap();
        let selected =
            (ordinal % 7 != 0).then(|| crate::embedding_values::hash_bytes(&id.to_le_bytes()));
        let job = if ordinal % 3 == 0 || ordinal >= selected_limit {
            "unrelated"
        } else {
            "selected"
        };
        c.execute(
            "INSERT INTO native_target_inputs VALUES(?,?,?)",
            params![job, id, selected],
        )
        .unwrap();
        if job == "selected" {
            expected.push((
                id,
                identities[model].clone(),
                crate::codec::sha256(&ordinal.to_le_bytes()),
                tokens,
                selected,
            ));
        }
    }
    write.commit().unwrap();
    (directory, workspace, expected)
}

#[test]
fn selected_native_pages_preserve_sparse_metadata_with_bounded_work_as_corpus_grows() {
    let mut excessive_work = Vec::new();
    for (size, selected_limit) in [(1024, 1024), (8192, 8192), (8192, 1024)] {
        let (_directory, workspace, expected) = fixture(size, selected_limit);
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let c = read.connection();
        for start in [0, expected.len() / 2, expected.len() - 19, expected.len()] {
            let after = if start == 0 { 0 } else { expected[start - 1].0 };
            let wanted = expected
                .iter()
                .skip(start)
                .take(NATIVE_VALUE_BATCH_LIMIT)
                .cloned()
                .collect::<Vec<_>>();
            let mut statement = c.prepare(SELECTED_NATIVE_PAGE_SQL).unwrap();
            let actual = statement
                .query_map(params!["selected", after], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .unwrap()
                .collect::<Result<Vec<PageRow>, _>>()
                .unwrap();
            assert_eq!(actual, wanted, "size={size} start={start}");
            let vm = statement.get_status(StatementStatus::VmStep);
            let scans = statement.get_status(StatementStatus::FullscanStep);
            let sorts = statement.get_status(StatementStatus::Sort);
            let plan = c
                .prepare(&format!("EXPLAIN QUERY PLAN {SELECTED_NATIVE_PAGE_SQL}"))
                .unwrap()
                .query_map(params!["selected", after], |row| row.get::<_, String>(3))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            eprintln!(
                "native_page size={size} selected_limit={selected_limit} start={start} rows={} vm={vm} fullscan={scans} sort={sorts} plan={plan:?}",
                actual.len()
            );
            if vm > actual.len() as i32 * 100 + 1000 {
                excessive_work.push(format!("size={size} selected_limit={selected_limit} start={start} rows={} vm={vm} plan={plan:?}",actual.len()));
            }
        }
    }
    assert!(
        excessive_work.is_empty(),
        "membership page must perform page-sized metadata work: {excessive_work:?}"
    );
}

#[test]
fn selected_native_page_rejects_missing_input_or_model_instead_of_skipping_members() {
    for missing in ["input", "model"] {
        let (_directory, workspace, expected) = fixture(300, 300);
        let c = Connection::open(workspace.path()).unwrap();
        // Deliberately construct corrupt historical storage outside domain writers.
        c.pragma_update(None, "foreign_keys", false).unwrap();
        let id = expected[0].0;
        if missing == "input" {
            c.execute("DELETE FROM embedding_inputs WHERE input_id=?", [id])
                .unwrap();
        } else {
            c.execute("DELETE FROM embedding_models WHERE model_id=(SELECT model_id FROM embedding_inputs WHERE input_id=?)",[id]).unwrap();
        }
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let mut statement = read.connection().prepare(SELECTED_NATIVE_PAGE_SQL).unwrap();
        let result = statement
            .query_map(params!["selected", 0], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, usize>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<PageRow>, _>>();
        assert!(
            result.is_err(),
            "missing {missing} must not silently shorten the page"
        );
    }
}

#[test]
fn selected_native_page_bounds_corrupt_selected_checksum_before_fetch() {
    let (_directory, workspace, expected) = fixture(300, 300);
    let c = Connection::open(workspace.path()).unwrap();
    c.execute_batch("PRAGMA ignore_check_constraints=ON")
        .unwrap();
    c.execute("UPDATE native_target_inputs SET selected_hash=zeroblob(8388608) WHERE job_key='selected' AND input_id=?",[expected[0].0]).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let mut statement = read.connection().prepare(SELECTED_NATIVE_PAGE_SQL).unwrap();
    let selected: Vec<u8> = statement
        .query_row(params!["selected", 0], |row| row.get(4))
        .unwrap();
    assert!(
        selected.is_empty(),
        "a damaged checksum must be represented by the guarded invalid sentinel"
    );
}
