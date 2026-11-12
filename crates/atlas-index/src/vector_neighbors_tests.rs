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
        discovery_scope: Default::default(),
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
    let table = crate::schema::native_vector_table(model).unwrap();
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
        discovery_scope: Default::default(),
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
            discovery_scope: Default::default(),
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
            discovery_scope: Default::default(),
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
    let history = crate::test_support::RELEASE_HISTORY;
    let mut migrations = history.migrations.to_vec();
    migrations.push(atlas_storage::Migration {
        version: crate::test_support::TEST_WORKSPACE_SCHEMA_VERSION + 1,
        name: "future-schema-proof",
        sql: "CREATE TABLE future_reference_policy(id INTEGER PRIMARY KEY);",
        backfill: None,
    });
    workspace
        .migrate_history(
            atlas_storage::MigrationHistory {
                migrations: &migrations,
                ..history
            },
            |_| Ok(()),
        )
        .unwrap();
    assert!(crate::test_support::open_reader(&path).is_err());
    assert!(neighbors.allocated_bytes() > 0);
}

#[test]
fn shared_inputs_expand_to_eligible_occurrences_and_keep_the_occurrence_budget() {
    let records = (0..3)
        .map(|i| {
            (
                "items",
                "Item",
                serde_json::json!({"_id":format!("{i:016}"),"name":format!("Shared {i}"),"type":"equipment","system":{"description":{"value":"<p>Shared body.</p>"}}}),
            )
        })
        .collect();
    let mut input = crate::core_tests::fixture(records, false);
    let mut needle = vec![0.; 768];
    needle[0] = 1.;
    let shared_hash = crate::codec::sha256(b"Shared body.");
    let mut shared_occurrences = 0;
    for unit in &mut input.semantic_units {
        if unit.field.is_some() {
            unit.input_hash = shared_hash.clone();
            unit.vector.clone_from(&needle);
            shared_occurrences += 1;
        }
    }
    assert_eq!(shared_occurrences, 3);
    let (_directory, path) = crate::foundation_tests::build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let model = &input.context.search_profiles[0];
    let table = crate::schema::native_vector_table(model).unwrap();
    let native_count: usize = reader
        .workspace_session()
        .connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        native_count, 4,
        "three identities and one shared passage input"
    );
    let query = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let request = SourceVectorCandidateRequest {
        discovery_scope: Default::default(),
        profile: &model.profile_id,
        vector: &needle,
        query: &query,
        keys: None,
        exclude: None,
        content_scope: atlas_domain::SearchContentScope::All,
        limit: 2,
    };
    let (hits, neighbors) = reader.vector_candidates_with_neighbors(request).unwrap();
    assert_eq!(hits.len(), 2);
    assert!(
        hits.iter()
            .all(|hit| hit.distance == 0. && hit.location.field.is_some())
    );
    assert_ne!(hits[0].location.record, hits[1].location.record);
    assert_eq!(
        reader
            .hydrate_vector_neighbors(request, &neighbors)
            .unwrap(),
        hits
    );
    let configured = SourceSimilarityCandidateRequest {
        discovery_scope: Default::default(),
        profile: request.profile,
        vector: request.vector,
        query: request.query,
        keys: request.keys,
        exclude: request.exclude,
        comparison_key: None,
        content_scope: request.content_scope,
        include_identity: false,
        limit: request.limit,
    };
    assert_eq!(reader.similarity_candidates(configured).unwrap(), hits);
    let mut operation = reader.configured_similarity_candidates();
    for _ in 0..2 {
        assert_eq!(
            operation
                .candidates::<IndexError>(configured, || Ok(()))
                .unwrap(),
            hits
        );
    }
    let keys = [
        input.records[0].record.key().clone(),
        input.records[1].record.key().clone(),
    ];
    let eligible = SourceVectorCandidateRequest {
        keys: Some(&keys),
        exclude: Some(&keys[0]),
        limit: 1,
        ..request
    };
    let (hits, neighbors) = reader.vector_candidates_with_neighbors(eligible).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].location.record, keys[1]);
    assert_eq!(hits[0].distance, 0.);
    assert!(hits[0].location.field.is_some());
    assert_eq!(
        reader
            .hydrate_vector_neighbors(eligible, &neighbors)
            .unwrap(),
        hits
    );
    let configured = SourceSimilarityCandidateRequest {
        discovery_scope: Default::default(),
        profile: eligible.profile,
        vector: eligible.vector,
        query: eligible.query,
        keys: eligible.keys,
        exclude: eligible.exclude,
        comparison_key: None,
        content_scope: eligible.content_scope,
        include_identity: false,
        limit: eligible.limit,
    };
    assert_eq!(reader.similarity_candidates(configured).unwrap(), hits);
    assert_eq!(
        operation
            .candidates::<IndexError>(configured, || Ok(()))
            .unwrap(),
        hits
    );
}
