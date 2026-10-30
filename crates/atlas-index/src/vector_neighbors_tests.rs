use crate::*;

#[test]
fn admitted_neighbor_hydration_does_not_reopen_native_vector_data() {
    let input = crate::foundation_tests::fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let model = &input.context.search_profiles[0];
    let query = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let request = SourceVectorCandidateRequest {
        profile: &model.profile_id,
        vector: &vector,
        query: &query,
        keys: None,
        exclude: None,
        content_scope: atlas_domain::SearchContentScope::All,
        limit: 1024,
    };
    let (expected, neighbors) = reader.vector_candidates_with_neighbors(request).unwrap();
    // Deny subsequent native SQL access in this connection, after real admission
    // and KNN. TEMP shadowing changes no persisted vector data or generation.
    // The cold path must still touch native storage; hydration must not.
    let table = crate::schema::vector_table(model).unwrap();
    reader
        .workspace_session()
        .connection()
        .execute_batch(&format!(
            "CREATE TEMP TABLE {table}(native_access_denied INTEGER)"
        ))
        .unwrap();
    assert!(reader.vector_candidates(request).is_err());
    assert_eq!(
        reader
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap(),
        expected
    );
}

#[test]
fn native_and_rehydrated_batches_preserve_macro_empty_scope_and_request_errors() {
    let input = crate::foundation_tests::fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let profile = &input.context.search_profiles[0].profile_id;
    let query = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let request = SourceVectorCandidateRequest {
        profile,
        vector: &vector,
        query: &query,
        keys: None,
        exclude: None,
        content_scope: atlas_domain::SearchContentScope::All,
        limit: 1024,
    };
    let (cold, neighbors) = reader.vector_candidates_with_neighbors(request).unwrap();
    assert!(!cold.is_empty());
    assert!(
        cold.iter()
            .all(|h| h.location.record.pack().as_str() != "macros")
    );
    assert_eq!(
        cold,
        reader
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap()
    );
    let macro_keys = ["macros:dddddddddddddddd".parse().unwrap()];
    for keys in [&[][..], &macro_keys[..]] {
        let request = SourceVectorCandidateRequest {
            keys: Some(keys),
            ..request
        };
        let (cold, neighbors) = reader.vector_candidates_with_neighbors(request).unwrap();
        assert!(cold.is_empty());
        assert!(
            reader
                .hydrate_vector_neighbors(request, &neighbors)
                .unwrap()
                .is_empty()
        );
    }
    let unavailable = atlas_domain::SearchProfileId::new("not-built");
    for request in [
        SourceVectorCandidateRequest {
            limit: 0,
            ..request
        },
        SourceVectorCandidateRequest {
            limit: 4097,
            ..request
        },
        SourceVectorCandidateRequest {
            profile: &unavailable,
            ..request
        },
    ] {
        assert_eq!(
            reader.vector_candidates(request).unwrap_err().to_string(),
            reader
                .hydrate_vector_neighbors(request, &neighbors)
                .unwrap_err()
                .to_string()
        );
    }
}

#[test]
fn retained_neighbors_do_not_bypass_incompatible_schema_admission() {
    let input = crate::foundation_tests::fixture(true);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let query = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let mut vector = vec![0.; 768];
    vector[0] = 1.;
    let (_, neighbors) = reader
        .vector_candidates_with_neighbors(SourceVectorCandidateRequest {
            profile: &input.context.search_profiles[0].profile_id,
            vector: &vector,
            query: &query,
            keys: None,
            exclude: None,
            content_scope: atlas_domain::SearchContentScope::All,
            limit: 1024,
        })
        .unwrap();
    drop(reader);
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    let mut migrations = crate::test_support::TEST_MIGRATIONS.to_vec();
    migrations.push(atlas_storage::Migration {
        version: crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION + 1,
        name: "future-schema-proof",
        sql: "CREATE TABLE future_reference_policy(id INTEGER PRIMARY KEY);",
        backfill: None,
    });
    workspace.migrate(&migrations, |_| Ok(())).unwrap();
    assert!(crate::test_support::open_reader(&path).is_err());
    assert!(neighbors.allocated_bytes() > 0);
}
