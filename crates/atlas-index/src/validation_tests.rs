use crate::foundation_tests::{build, hidden_preparation_fixture};
use rusqlite::{Connection, params};

#[test]
fn unreferenced_metadata_status_remains_intrinsically_allowed_but_attested_changes_fail() {
    let input = crate::foundation_tests::fixture(false);
    let (_directory, path) = build(&input);
    let before = crate::validate_artifact(&path).unwrap();
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM relationship_occurrences WHERE origin='content'",
                [],
                |r| r.get::<_, usize>(0),
            )
            .unwrap(),
        0
    );
    assert_eq!(connection.execute(
        r#"UPDATE prepared_content SET metadata_json=json_set(metadata_json,'$.status',json('{"PreparationFailed":"fixture"}')) WHERE content_id=(SELECT content_id FROM prepared_content WHERE outcome='prepared' LIMIT 1)"#,
        [],
    ).unwrap(), 1);
    let error = crate::validate_artifact(&path).unwrap_err();
    assert!(
        error.to_string().contains("component attestation"),
        "{error}"
    );
    // Schema26 explicitly permits historical roots without attestations. Keep
    // the intrinsic status policy separate from the new attestation invariant.
    assert_eq!(connection.execute("DELETE FROM reference_component_digests WHERE record_id=(SELECT record_id FROM prepared_content WHERE content_id=(SELECT content_id FROM prepared_content WHERE outcome='prepared' LIMIT 1))",[]).unwrap(),1);
    assert_eq!(crate::validate_artifact(&path).unwrap(), before);
}

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
    let mut input = hidden_preparation_fixture();
    input.records[0]
        .diagnostics
        .push(crate::SourceArtifactDiagnostic {
            source_diagnostic: None,
            owners: Some(vec![]),
            field: None,
            stage: "admission".into(),
            code: "fixture".into(),
            message: "fixture".into(),
        });
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    let diagnostic: i64 = connection
        .query_row(
            "SELECT diagnostic_id FROM developer_diagnostics WHERE stage='admission'",
            [],
            |r| r.get(0),
        )
        .unwrap();
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

#[test]
fn persisted_admission_summary_rejects_corrupt_counts_paths_and_root_coverage() {
    let mut input = crate::foundation_tests::fixture(false);
    input.context.source_file_count = 5;
    input.context.admission = Some(crate::SourceAdmissionSummary {
        discovered_files: 5,
        admitted_records: 4,
        partial_records: 0,
        quarantined_files: 1,
        excluded_documents: 0,
        exclusions: vec![crate::SourceExcludedInput {
            pack: "items".into(),
            path: "packs/items/bad.json".into(),
            reason: "invalid JSON".into(),
        }],
    });
    let (_directory, path) = build(&input);
    let connection = Connection::open(&path).unwrap();
    let before = crate::validate_reference(&connection).unwrap();
    for case in [
        "total",
        "partial",
        "exclusions",
        "relative path",
        "absolute path",
        "reason",
        "pack",
        "root coverage",
    ] {
        let mut context = input.context.clone();
        let admission = context.admission.as_mut().unwrap();
        match case {
            "total" => admission.discovered_files += 1,
            "partial" => admission.partial_records = 5,
            "exclusions" => admission.exclusions.clear(),
            "relative path" => admission.exclusions[0].path = "../bad.json".into(),
            "absolute path" => admission.exclusions[0].path = "/bad.json".into(),
            "reason" => admission.exclusions[0].reason.clear(),
            "pack" => admission.exclusions[0].pack.clear(),
            "root coverage" => {
                admission.admitted_records = 3;
                admission.discovered_files = 4;
            }
            _ => unreachable!(),
        }
        let json = serde_json::to_string(&context).unwrap();
        connection
            .execute_batch("SAVEPOINT corrupt_admission")
            .unwrap();
        connection
            .execute(
                "UPDATE artifact_context SET context_json=?,context_hash=?",
                params![json, context.sha256().unwrap()],
            )
            .unwrap();
        let error = crate::validate_reference(&connection)
            .unwrap_err()
            .to_string();
        assert!(error.contains("source admission"), "{case}: {error}");
        connection
            .execute_batch("ROLLBACK TO corrupt_admission; RELEASE corrupt_admission")
            .unwrap();
    }
    assert_eq!(crate::validate_artifact(&path).unwrap(), before);
}
