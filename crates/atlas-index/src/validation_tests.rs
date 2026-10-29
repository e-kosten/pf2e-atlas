use crate::foundation_tests::{build, hidden_preparation_fixture};
use rusqlite::{Connection, params};

#[test]
fn unpublished_cache_metadata_is_checked_without_changing_selected_field_counts() {
    let input = hidden_preparation_fixture();
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    let before = crate::validate_artifact(&path).unwrap();
    let hidden: i64 = connection
        .query_row(
            "SELECT content_id FROM prepared_content WHERE published=0 AND outcome<>'prepared' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let published: usize = connection
        .query_row(
            "SELECT count(*) FROM prepared_content WHERE published=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(before.prepared_fields, published);
    for (column, value) in [
        ("owners_json", "[{}]".to_owned()),
        ("field_path", "/system/absent".to_owned()),
        ("authored_markup_sha256", "0".repeat(64)),
        ("preparation_context_hash", "0".repeat(64)),
        ("role", "invented".to_owned()),
        ("visibility", "invented".to_owned()),
        ("metadata_json", "{}".to_owned()),
        ("metadata_json", r#"{"visibility_availability":{},"status":{"PreparationFailed":"fixture"}}"#.to_owned()),
        ("metadata_json", r#"{"visibility_availability":"Value","status":{"Prepared":{"hidden_references":[{}],"diagnostics":[]}}}"#.to_owned()),
        ("metadata_json", r#"{"visibility_availability":"Value","status":{"Prepared":{"hidden_references":[],"diagnostics":[{}]}}}"#.to_owned()),
    ] {
        connection.execute_batch("SAVEPOINT corrupt_cache").unwrap();
        assert_eq!(
            connection.execute(
                &format!("UPDATE prepared_content SET {column}=? WHERE content_id=?"),
                params![value, hidden],
            ).unwrap(),
            1,
        );
        assert!(crate::validate_reference(&connection).is_err(), "{column}: {value}");
        connection.execute_batch("ROLLBACK TO corrupt_cache; RELEASE corrupt_cache").unwrap();
    }
    assert_eq!(crate::validate_reference(&connection).unwrap(), before);
}

#[test]
fn admission_diagnostic_shapes_are_checked_without_materializing_build_inputs() {
    let input = hidden_preparation_fixture();
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    connection.execute(
        "INSERT INTO developer_diagnostics(record_id,owners_json,stage,details_json) VALUES((SELECT min(record_id) FROM records),'[]','admission',?)",
        [r#"{"code":"fixture","message":"fixture"}"#],
    ).unwrap();
    let diagnostic = connection.last_insert_rowid();
    let before = crate::validate_artifact(&path).unwrap();
    for (column, value) in [
        ("owners_json", "{}"),
        ("owners_json", "[{}]"),
        ("details_json", "{}"),
        ("details_json", r#"{"code":1,"message":"fixture"}"#),
        (
            "details_json",
            r#"{"code":"fixture","message":"fixture","source_diagnostic":{}}"#,
        ),
    ] {
        connection
            .execute_batch("SAVEPOINT corrupt_diagnostic")
            .unwrap();
        connection
            .execute(
                &format!("UPDATE developer_diagnostics SET {column}=? WHERE diagnostic_id=?"),
                params![value, diagnostic],
            )
            .unwrap();
        assert!(
            crate::validate_reference(&connection).is_err(),
            "{column}: {value}"
        );
        connection
            .execute_batch("ROLLBACK TO corrupt_diagnostic; RELEASE corrupt_diagnostic")
            .unwrap();
    }
    assert_eq!(crate::validate_reference(&connection).unwrap(), before);
}

#[test]
fn trait_membership_comparison_keeps_exact_owner_serialization() {
    let input = crate::foundation_tests::fixture(false);
    let (_directory, path) = build(&input);
    let connection = Connection::open(path).unwrap();
    let before = crate::validate_reference(&connection).unwrap();
    assert!(
        connection
            .execute(
                "UPDATE trait_memberships SET owners_json='[ ]' WHERE owners_json='[]'",
                [],
            )
            .unwrap()
            > 0
    );
    let error = crate::validate_reference(&connection).unwrap_err();
    assert!(
        error.to_string().contains("trait memberships differ"),
        "{error}"
    );
    assert!(before.roots > 0);
}
