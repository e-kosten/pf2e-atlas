use crate::{test_support::*, *};
use atlas_embedding::PreparedNativeInput;

fn prepared(unit: &SourceSemanticUnitInput) -> PreparedNativeInput {
    PreparedNativeInput {
        input: String::new(),
        input_sha256: unit.input_hash.clone(),
        token_count: unit.input_token_count,
    }
}
fn reused_fixture_values(
    session: &atlas_storage::ReadSession,
    model: &SourceSearchProfileIdentity,
    inputs: &[PreparedNativeInput],
) -> Result<std::collections::BTreeMap<String, Vec<f32>>, IndexError> {
    let keys = crate::native_values::NativeModelKey::all(std::slice::from_ref(model))?;
    let binding = crate::native_values::NativeModel::bind(session.connection(), &keys[0], false)?;
    let mut output = std::collections::BTreeMap::new();
    for input in inputs {
        let descriptor = crate::native_values::NativeInput::new(model, &input.descriptor())?;
        let Some(selected) = binding.select(&descriptor)? else {
            continue;
        };
        let vector = match selected.staged {
            Some(bytes) => crate::read_units::decode_vector(&bytes)?,
            None => binding.read(binding.selected_input(&descriptor, &selected.checksum)?)?,
        };
        output.insert(input.input_sha256.clone(), vector);
    }
    Ok(output)
}
fn stage_fixture_values(
    session: &atlas_storage::WriteSession,
    model: &SourceSearchProfileIdentity,
    batch: &[(&PreparedNativeInput, &[f32])],
) -> Result<(), IndexError> {
    for (input, vector) in batch {
        crate::validation::check_vector(vector, model.dimensions)?;
        let id = crate::embedding_values::input_id(
            session.connection(),
            model,
            &input.input_sha256,
            input.token_count,
        )?;
        let bytes = crate::read_units::vector_bytes(vector);
        session.connection().execute("INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at) VALUES(?,?,?,unixepoch()) ON CONFLICT(input_id) DO UPDATE SET vector=excluded.vector,vector_hash=excluded.vector_hash,cached_at=excluded.cached_at",rusqlite::params![id,bytes,crate::embedding_values::hash_bytes(&bytes)])?;
    }
    Ok(())
}
fn count(c: &rusqlite::Connection, table: &str) -> usize {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[test]
fn published_values_supply_exact_reuse_without_a_checkpoint_corpus() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let input = crate::profile_tests::both();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    for model in &input.context.search_profiles {
        let units = input
            .semantic_units
            .iter()
            .filter(|u| u.profile_id == model.profile_id)
            .collect::<Vec<_>>();
        let mut inputs = units.iter().map(|u| prepared(u)).collect::<Vec<_>>();
        let reused = reused_fixture_values(&read, model, &inputs).unwrap();
        for (unit, prepared) in units.iter().zip(&inputs) {
            assert_eq!(reused[&prepared.input_sha256], unit.vector);
        }
        let mut missing = inputs[0].clone();
        missing.input_sha256 = crate::codec::sha256(b"changed input");
        inputs.push(missing);
        assert!(
            !reused_fixture_values(&read, model, &inputs)
                .unwrap()
                .contains_key(&crate::codec::sha256(b"changed input"))
        );
        let mut changed = model.clone();
        changed.input_recipe.push_str("/different");
        assert!(
            reused_fixture_values(&read, &changed, &inputs)
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(count(read.connection(), "pending_embedding_values"), 0);
    assert!(
        read.connection()
            .prepare("SELECT * FROM trait_native_values")
            .is_err()
    );
    assert_eq!(count(read.connection(), "embedding_models"), 2);
    assert_eq!(
        count(read.connection(), "semantic_locations") * 2,
        count(read.connection(), "semantic_units")
    );
}
#[test]
fn pending_vector_survives_reopen_without_mutating_published_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let model = input.context.search_profiles[0].clone();
    let unit = input.semantic_units[0].clone();
    input.semantic_units[0].input_hash = crate::codec::sha256(b"new native input");
    let descriptor = prepared(&input.semantic_units[0]);
    let mut replacement = vec![0.; model.dimensions];
    replacement[17] = 1.;
    {
        let workspace = initialize_workspace(&path).unwrap();
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        stage_fixture_values(&write, &model, &[(&descriptor, &replacement)]).unwrap();
        write.commit().unwrap();
    }
    let workspace = initialize_workspace(&path).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        reused_fixture_values(&read, &model, std::slice::from_ref(&descriptor)).unwrap()
            [&descriptor.input_sha256],
        replacement
    );
    assert_eq!(count(read.connection(), "pending_embedding_values"), 1);
    let reader = SqliteIndexReader::from_session(read).unwrap();
    assert_eq!(
        reader
            .seed_identity_vector(&unit.profile_id, &unit.record)
            .unwrap()
            .unwrap()
            .vector,
        unit.vector
    );
    drop(reader);
    input.semantic_units[0].vector = replacement.clone();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let mut publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    assert!(
        publication
            .stage_inferred(
                &model,
                &[(&descriptor.descriptor(), unit.vector.as_slice())]
            )
            .is_err(),
        "selected cached bytes cannot change within an attempt"
    );
    {
        let maintenance = owner
            .maintenance(
                TEST_WORKSPACE_SCHEMA_VERSION,
                "advance generation for stale input fixture",
            )
            .unwrap();
        maintenance.publish(|_| Ok::<_, IndexError>(())).unwrap();
    }
    assert!(
        publication.publish().is_err(),
        "a prepared publication cannot activate over a different generation"
    );
    assert!(!recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert!(
        !reused_fixture_values(&read, &model, std::slice::from_ref(&descriptor))
            .unwrap()
            .is_empty()
    );
    let reader = SqliteIndexReader::from_session(read).unwrap();
    assert_eq!(
        reader
            .seed_identity_vector(&unit.profile_id, &unit.record)
            .unwrap()
            .unwrap()
            .vector,
        unit.vector
    );
    drop(reader);
    prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap()
    .publish()
    .unwrap();
    drop(scratch);
    drop(owner);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(count(read.connection(), "pending_embedding_values"), 0);
    assert_eq!(
        reused_fixture_values(&read, &model, std::slice::from_ref(&descriptor)).unwrap()
            [&descriptor.input_sha256],
        replacement
    );
}
#[test]
fn published_reuse_rejects_token_mismatches_and_ignores_malformed_pending() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let prepared = prepared(unit);
    let workspace = initialize_workspace(&path).unwrap();
    let mut wrong = prepared.clone();
    wrong.token_count += 1;
    assert!(
        reused_fixture_values(
            &workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap(),
            model,
            &[wrong]
        )
        .is_err()
    );
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    stage_fixture_values(&write, model, &[(&prepared, &unit.vector)]).unwrap();
    write
        .connection()
        .execute(
            "UPDATE pending_embedding_values SET vector_hash=zeroblob(32)",
            [],
        )
        .unwrap();
    write.commit().unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        reused_fixture_values(&read, model, std::slice::from_ref(&prepared)).unwrap()
            [&prepared.input_sha256],
        unit.vector
    );
    // The corrupt pending computation does not invalidate the intact publication.
    validate_artifact(&path).unwrap();
}
#[test]
fn new_record_reuses_native_inputs_and_keeps_distinct_filtered_witnesses() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let original = input.records[1].record.key().clone();
    let mut added = crate::foundation_tests::record(
        "items",
        "Item",
        serde_json::json!({"_id":"eeeeeeeeeeeeeeee","name":"Dragon Staff","type":"weapon","system":{"level":{"value":0},"traits":{"value":[],"rarity":"common"},"price":{"value":{"gp":2},"per":1},"bulk":{"value":1},"description":{"value":"<h2>Flame</h2><p>@Check[reflex|dc:20] Dragon breath.</p>"}}}),
    );
    added.preparation_context_hash = input.context.preparation_sha256().unwrap();
    let key = added.record.key().clone();
    let new_units = input
        .semantic_units
        .iter()
        .filter(|unit| unit.record == original)
        .cloned()
        .map(|mut unit| {
            unit.record = key.clone();
            unit
        })
        .collect::<Vec<_>>();
    let model = &input.context.search_profiles[0];
    let workspace = initialize_workspace(&path).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let reuse = reused_fixture_values(
        &read,
        model,
        &new_units.iter().map(prepared).collect::<Vec<_>>(),
    )
    .unwrap();
    for unit in &new_units {
        assert_eq!(reuse[&unit.input_hash], unit.vector);
    }
    drop(read);
    let lexical = input
        .lexical_units
        .iter()
        .filter(|unit| unit.record == original)
        .cloned()
        .map(|mut unit| {
            unit.record = key.clone();
            unit
        })
        .collect::<Vec<_>>();
    input.records.push(added);
    input.context.source_file_count += 1;
    input.lexical_units.extend(lexical);
    input.semantic_units.extend(new_units.clone());
    publish_input(&path, &input).unwrap();
    let reader = open_reader(&path).unwrap();
    let all = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let model = &input.context.search_profiles[0];
    let mut occurrence_ids = std::collections::BTreeSet::new();
    for record in [original, key] {
        let hits = reader
            .vector_candidates(crate::SourceVectorCandidateRequest {
                discovery_scope: Default::default(),
                profile: &model.profile_id,
                vector: &new_units[0].vector,
                query: &all,
                keys: Some(std::slice::from_ref(&record)),
                exclude: None,
                content_scope: atlas_domain::SearchContentScope::All,
                limit: 100,
            })
            .unwrap();
        assert_eq!(hits.len(), new_units.len());
        for hit in hits {
            assert_eq!(hit.location.record, record);
            assert!(occurrence_ids.insert(hit.location.unit_id));
            if hit.location.field.is_some() {
                assert!(reader.recover_passage(&hit.location).unwrap().is_some());
            }
        }
    }
    assert_eq!(occurrence_ids.len(), new_units.len() * 2);
}

#[test]
fn publication_updates_native_values_and_retains_attribution_per_occurrence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let all = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let model = &input.context.search_profiles[0];
    let reader = open_reader(&path).unwrap();
    let old_passage = reader
        .vector_candidates(crate::SourceVectorCandidateRequest {
            discovery_scope: Default::default(),
            profile: &model.profile_id,
            vector: &input.semantic_units[0].vector,
            query: &all,
            keys: None,
            exclude: None,
            content_scope: atlas_domain::SearchContentScope::All,
            limit: 100,
        })
        .unwrap()
        .into_iter()
        .find(|hit| {
            reader
                .recover_passage(&hit.location)
                .unwrap()
                .is_some_and(|text| text.contains("Dragon breath"))
        })
        .unwrap()
        .location;
    drop(reader);
    let c = rusqlite::Connection::open(&path).unwrap();
    let ids = c
        .prepare("SELECT unit_id,location_id FROM semantic_units ORDER BY unit_id")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let model = &input.context.search_profiles[0];
    let table = crate::schema::native_vector_table(model).unwrap();
    let blocks: usize = c
        .query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |r| {
            r.get(0)
        })
        .unwrap();
    drop(c);
    input = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>Flame</h2><p>@Check[reflex|dc:20] Changed dragon breath.</p>",
    );
    for unit in &mut input.semantic_units {
        unit.vector.fill(0.);
        unit.vector[23] = 1.;
    }
    publish_input(&path, &input).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    let after = c
        .prepare("SELECT unit_id,location_id FROM semantic_units ORDER BY unit_id")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        ids.iter().map(|(id, _)| id).collect::<Vec<_>>(),
        after.iter().map(|(id, _)| id).collect::<Vec<_>>()
    );
    assert_ne!(
        ids, after,
        "changed source addresses must replace their locations"
    );
    assert_eq!(
        c.query_row(&format!("SELECT count(*) FROM {table}_chunks"), [], |r| {
            r.get::<_, usize>(0)
        })
        .unwrap(),
        blocks
    );
    assert_eq!(count(&c, "pending_embedding_values"), 0);
    assert_eq!(count(&c, "semantic_units"), input.semantic_units.len());
    drop(c);
    let reader = open_reader(&path).unwrap();
    let model = &input.context.search_profiles[0];
    let new_passage = reader
        .vector_candidates(crate::SourceVectorCandidateRequest {
            discovery_scope: Default::default(),
            profile: &model.profile_id,
            vector: &input.semantic_units[0].vector,
            query: &all,
            keys: None,
            exclude: None,
            content_scope: atlas_domain::SearchContentScope::All,
            limit: 100,
        })
        .unwrap()
        .into_iter()
        .find(|hit| hit.location.unit_id == old_passage.unit_id)
        .unwrap()
        .location;
    assert_eq!(new_passage.record, old_passage.record);
    assert_eq!(new_passage.owners, old_passage.owners);
    assert_eq!(new_passage.field, old_passage.field);
    assert_ne!(new_passage.address, old_passage.address);
    assert!(
        reader
            .recover_passage(&new_passage)
            .unwrap()
            .unwrap()
            .contains("Changed dragon breath")
    );
}
