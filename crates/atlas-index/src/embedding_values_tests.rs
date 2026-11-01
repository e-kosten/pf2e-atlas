use crate::{test_support::*, *};
use atlas_embedding::PreparedNativeInput;

fn prepared(unit: &SourceSemanticUnitInput) -> PreparedNativeInput {
    PreparedNativeInput {
        input: String::new(),
        input_sha256: unit.input_hash.clone(),
        reuse_key: format!("{}:{}", unit.profile_id, unit.input_hash),
        token_count: unit.input_token_count,
    }
}
fn reused_fixture_values(
    session: &atlas_storage::ReadSession,
    model: &SourceSearchProfileIdentity,
    inputs: &[PreparedNativeInput],
) -> Result<std::collections::BTreeMap<String, Vec<f32>>, IndexError> {
    let mut output = std::collections::BTreeMap::new();
    for input in inputs {
        if let Some(vector) = native_fixture_value(session, model, &input.descriptor())? {
            output.insert(input.reuse_key.clone(), vector);
        }
    }
    Ok(output)
}
fn stage_fixture_values(
    session: &atlas_storage::WriteSession,
    job: &str,
    model: &SourceSearchProfileIdentity,
    batch: &[(&PreparedNativeInput, &[f32])],
) -> Result<(), IndexError> {
    for (input, vector) in batch {
        stage_pending_native_fixture(session, job, model, &input.descriptor(), vector)?;
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
            assert_eq!(reused[&prepared.reuse_key], unit.vector);
        }
        let mut missing = inputs[0].clone();
        missing.input_sha256 = crate::codec::sha256(b"changed input");
        missing.reuse_key = "new".into();
        inputs.push(missing);
        assert!(
            !reused_fixture_values(&read, model, &inputs)
                .unwrap()
                .contains_key("new")
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
    assert_eq!(count(read.connection(), "trait_native_values"), 0);
    assert_eq!(count(read.connection(), "embedding_models"), 2);
    assert_eq!(
        count(read.connection(), "semantic_locations") * 2,
        count(read.connection(), "semantic_units")
    );
}
#[test]
fn forced_computation_survives_reopen_without_mutating_published_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let model = input.context.search_profiles[0].clone();
    let unit = input.semantic_units[0].clone();
    let prepared = prepared(&unit);
    let mut replacement = vec![0.; model.dimensions];
    replacement[17] = 1.;
    let workspace = initialize_workspace(&path).unwrap();
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let target = input.context.sha256().unwrap();
    let job = start_reference_job(&write, &target).unwrap();
    stage_fixture_values(&write, &job, &model, &[(&prepared, &replacement)]).unwrap();
    write.commit().unwrap();
    drop(workspace);
    let workspace = initialize_workspace(&path).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        reused_fixture_values(&read, &model, std::slice::from_ref(&prepared)).unwrap()
            [&prepared.reuse_key],
        replacement
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
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    let maintenance = owner
        .maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "forced publication")
        .unwrap();
    input.semantic_units[0].vector = replacement.clone();
    let frozen = freeze_fixture_native_values(
        &input,
        &input,
        Some(&job),
        &crate::frozen_native_values::origin(maintenance.connection()).unwrap(),
    )
    .unwrap();
    assert!(
        SqliteIndexWriter::new(&maintenance)
            .write_job(&input, &frozen, "wrong-job", &target)
            .is_err()
    );
    assert_eq!(
        count(maintenance.connection(), "pending_embedding_values"),
        1
    );
    let table = crate::schema::vector_table(&model).unwrap();
    let bytes:Vec<u8>=maintenance.connection().query_row(&format!("SELECT embedding FROM {table} WHERE unit_id=(SELECT unit_id FROM semantic_units WHERE unit_kind='identity' AND record_id=(SELECT record_id FROM records WHERE key=?))"),[unit.record.to_string()],|r|r.get(0)).unwrap();
    assert_eq!(
        crate::read_units::decode_vector(&bytes).unwrap(),
        unit.vector
    );
    SqliteIndexWriter::new(&maintenance)
        .write_job(&input, &frozen, &job, &target)
        .unwrap();
    drop(maintenance);
    drop(owner);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(count(read.connection(), "pending_embedding_values"), 0);
    assert_eq!(
        reused_fixture_values(&read, &model, std::slice::from_ref(&prepared)).unwrap()
            [&prepared.reuse_key],
        replacement
    );
}
#[test]
fn reuse_rejects_corrupt_values_and_native_count_mismatches() {
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
    stage_fixture_values(&write, "job", model, &[(&prepared, &unit.vector)]).unwrap();
    write
        .connection()
        .execute(
            "UPDATE pending_embedding_values SET vector_hash=zeroblob(32)",
            [],
        )
        .unwrap();
    write.commit().unwrap();
    assert!(
        reused_fixture_values(
            &workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap(),
            model,
            &[prepared]
        )
        .is_err()
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
        assert_eq!(reuse[&prepared(unit).reuse_key], unit.vector);
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
fn publication_updates_slots_in_place_and_retains_attribution_per_occurrence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let all = crate::validate_query(&atlas_domain::QueryPredicate::boolean(true)).unwrap();
    let model = &input.context.search_profiles[0];
    let reader = open_reader(&path).unwrap();
    let old_passage = reader
        .vector_candidates(crate::SourceVectorCandidateRequest {
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
    let table = crate::schema::vector_table(model).unwrap();
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

#[test]
fn migration_preserves_published_values_and_interrupted_forced_progress() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..4], |_| Ok(()))
        .unwrap();
    let mut input = crate::foundation_tests::fixture(true);
    input.context.query_projection_version = "authored-query/v4".into();
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(4, "legacy fixture").unwrap();
    publish_historical_record_fixture(&maintenance, &input).unwrap();
    drop(maintenance);
    drop(owner);
    // Historical trait migration is still SQL-only; the new conversion is last.
    workspace
        .migrate(&TEST_MIGRATIONS[..8], |_| Ok(()))
        .unwrap();
    let write = workspace.write(8).unwrap();
    write
        .connection()
        .execute_batch(
            "CREATE TABLE saved_probe(value TEXT); INSERT INTO saved_probe VALUES('retain me');",
        )
        .unwrap();
    // This fixture manually publishes the schema-8 trait component below. Its
    // current policy also requires the authored memberships; schema creation
    // alone cannot certify an empty derived membership table.
    for source in &input.records {
        for membership in atlas_record::source_record::trait_record_memberships(
            &source.record,
            input.context.audience,
        ) {
            write
                .connection()
                .execute(
                    "INSERT INTO trait_memberships VALUES(?,?,?,?)",
                    rusqlite::params![
                        membership.trait_key.to_string(),
                        source.record.key().to_string(),
                        serde_json::to_string(&membership.owners).unwrap(),
                        if membership.owners.is_empty() {
                            None
                        } else {
                            membership.source_label
                        }
                    ],
                )
                .unwrap();
        }
    }
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let prepared = prepared(unit);
    let mut replacement = vec![0.; model.dimensions];
    replacement[27] = 1.;
    let job = start_reference_job(&write, "forced target").unwrap();
    stage_historical_embedding_fixture(&write, &job, model, &prepared, &replacement).unwrap();
    let mut traits = crate::foundation_tests::fixture(true);
    add_label_only_trait(&mut traits);
    let definition = &traits.traits[0];
    let seed = &traits.trait_vectors[0];
    let json = serde_json::to_string(definition).unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO trait_definitions VALUES(?,?,?)",
            rusqlite::params![
                definition.key.to_string(),
                json,
                crate::codec::sha256(json.as_bytes())
            ],
        )
        .unwrap();
    let json = serde_json::to_string(&seed.vector).unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO trait_vectors VALUES(?,?,?,?,?,?)",
            rusqlite::params![
                seed.key.to_string(),
                seed.profile_id.as_str(),
                seed.input_hash,
                json,
                serde_json::to_string(&seed.chunks).unwrap(),
                crate::codec::sha256(json.as_bytes())
            ],
        )
        .unwrap();
    write.commit().unwrap();
    let upgraded = initialize_workspace(&path).unwrap();
    let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(
        read.connection()
            .query_row("SELECT value FROM saved_probe", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "retain me"
    );
    assert_eq!(count(read.connection(), "pending_embedding_values"), 1);
    assert_eq!(
        reused_fixture_values(&read, model, std::slice::from_ref(&prepared)).unwrap()
            [&prepared.reuse_key],
        replacement
    );
    assert_eq!(count(read.connection(), "trait_native_values"), 1);
    drop(read);
    assert!(ensure_reference_projections(&upgraded, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    let read = upgraded.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 2);
    assert_eq!(count(read.connection(), "pending_embedding_values"), 1);
    let reader = SqliteIndexReader::from_session(read).unwrap();
    assert_eq!(
        reader
            .trait_vector(&seed.key, &seed.profile_id)
            .unwrap()
            .unwrap(),
        seed.vector
    );
    assert_eq!(
        reader
            .seed_identity_vector(&unit.profile_id, &unit.record)
            .unwrap()
            .unwrap()
            .vector,
        unit.vector
    );
    assert!(
        reader
            .native()
            .unwrap()
            .prepare("SELECT * FROM embedding_checkpoints")
            .is_err()
    );
}

#[test]
fn migration_corruption_rolls_back_new_schema_and_preserves_retryable_progress() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let workspace = atlas_storage::Workspace::at(&path).unwrap();
    workspace
        .migrate(&TEST_MIGRATIONS[..4], |_| Ok(()))
        .unwrap();
    let input = crate::foundation_tests::fixture(true);
    let owner = workspace.update_owner().unwrap();
    let maintenance = owner.maintenance(4, "legacy fixture").unwrap();
    publish_historical_record_fixture(&maintenance, &input).unwrap();
    drop(maintenance);
    drop(owner);
    let write = workspace.write(4).unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let job = start_reference_job(&write, "target").unwrap();
    stage_historical_embedding_fixture(&write, &job, model, &prepared(unit), &unit.vector).unwrap();
    write
        .connection()
        .execute("UPDATE embedding_checkpoints SET vector_hash='corrupt'", [])
        .unwrap();
    write.commit().unwrap();
    assert!(initialize_workspace(&path).is_err());
    let read = workspace.read(4).unwrap();
    assert_eq!(count(read.connection(), "embedding_checkpoints"), 1);
    assert_eq!(
        count(read.connection(), "semantic_units"),
        input.semantic_units.len()
    );
    assert!(
        read.connection()
            .prepare("SELECT * FROM embedding_inputs")
            .is_err()
    );
    assert_eq!(
        read.connection()
            .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        4
    );
}

pub(super) fn add_label_only_trait(input: &mut IndexBuildInput) {
    let text = "agile\n";
    let hash = crate::codec::sha256(text.as_bytes());
    let vector = input.semantic_units[0].vector.clone();
    input.traits.push(TraitDefinition {
        key: atlas_domain::TraitKey::new("agile").unwrap(),
        label: "agile".into(),
        label_key: None,
        description_key: None,
        description_html: None,
        authored_markup: None,
        prepared: None,
        catalogs: vec![],
        source_locations: vec![],
        source_revision: input.context.source_revision.clone(),
        indexing_locale: input.context.indexing_locale.clone(),
    });
    input.trait_vectors.push(TraitVectorInput {
        key: input.traits[0].key.clone(),
        profile_id: input.context.search_profiles[0].profile_id.clone(),
        input_hash: crate::codec::sha256(format!("trait-definition-centroid/v1:{hash}").as_bytes()),
        vector: vector.clone(),
        chunks: vec![TraitSeedChunk {
            input: text.into(),
            input_hash: hash.clone(),
            token_count: 2,
            address: atlas_domain::SourcePassageAddress::PlainSection {
                source_text_sha256: hash,
                selection_version: "trait-definition-centroid/v1".into(),
                section_ordinal: 0,
                label: Some("agile".into()),
                chunk_bytes: atlas_domain::SourceByteRange {
                    start: 0,
                    end: text.len(),
                },
            },
            vector,
        }],
    });
}
#[test]
fn trait_native_values_move_between_document_and_trait_only_storage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlas.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    add_label_only_trait(&mut input);
    let chunk = input.trait_vectors[0].chunks[0].clone();
    let original = input.semantic_units[0].input_hash.clone();
    let tokens = input.semantic_units[0].input_token_count;
    input.semantic_units[0].input_hash = chunk.input_hash.clone();
    input.semantic_units[0].input_token_count = chunk.token_count;
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    assert_eq!(
        count(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
            "trait_native_values"
        ),
        0
    );
    // The last document moves to a new input, but the retained trait still needs
    // the old native value to verify its centroid and support future reuse.
    input.semantic_units[0].input_hash = original;
    input.semantic_units[0].input_token_count = tokens;
    publish_input(&path, &input).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(count(read.connection(), "trait_native_values"), 1);
    drop(read);
    input.semantic_units[0].input_hash = chunk.input_hash;
    input.semantic_units[0].input_token_count = chunk.token_count;
    publish_input(&path, &input).unwrap();
    assert_eq!(
        count(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
            "trait_native_values"
        ),
        0
    );
    input.traits.clear();
    input.trait_vectors.clear();
    publish_input(&path, &input).unwrap();
    validate_artifact(&path).unwrap();
}
