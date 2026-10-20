use crate::{
    foundation_tests::{build, fixture},
    *,
};
use atlas_domain::{QueryPredicate, SearchProfileId};

pub(super) fn both() -> IndexBuildInput {
    let mut input = fixture(true);
    input
        .context
        .search_profiles
        .push(SourceSearchProfileIdentity::from_spec(
            atlas_embedding::SEARCH_PROFILES[1],
        ));
    let alternate = input.records[1].record.key().clone();
    let mut units = input.semantic_units.clone();
    for unit in &mut units {
        unit.profile_id = SearchProfileId::new("embeddinggemma");
        unit.vector = vec![0.0; 768];
        unit.vector[if unit.record == alternate { 0 } else { 2 }] = 1.0;
    }
    input.semantic_units.extend(units);
    input
}
#[test]
fn manifest_order_is_canonical_and_profile_changes_do_not_rehash_shared_preparation() {
    let input = both();
    let generation = input.context.sha256().unwrap();
    let preparation = input.context.preparation_sha256().unwrap();
    let mut context = input.context.clone();
    context.search_profiles.reverse();
    assert_eq!(context.sha256().unwrap(), generation);
    assert_eq!(context.preparation_sha256().unwrap(), preparation);
    let (_dir, path) = build(&input);
    assert_eq!(
        crate::test_support::open_reader(path)
            .unwrap()
            .context()
            .search_profiles,
        input.context.search_profiles
    );
    context.search_profiles[0].input_recipe.push_str("/changed");
    assert_ne!(context.sha256().unwrap(), generation);
    assert_eq!(context.preparation_sha256().unwrap(), preparation);
    context
        .search_profiles
        .push(context.search_profiles[0].clone());
    assert!(context.sha256().is_err());
}
#[test]
fn equal_dimension_profiles_isolate_knn_seeds_and_witnesses_before_top_k() {
    let input = both();
    let (_dir, path) = build(&input);
    let reader = crate::test_support::open_reader(&path).unwrap();
    let query = validate_query(&QueryPredicate::boolean(true)).unwrap();
    let base = SearchProfileId::new("bge-base");
    let gemma = SearchProfileId::new("embeddinggemma");
    let mut needle = vec![0.0; 768];
    needle[0] = 1.0;
    let b = reader
        .vector_candidates(&base, &needle, &query, None, 1)
        .unwrap();
    let g = reader
        .vector_candidates(&gemma, &needle, &query, None, 1)
        .unwrap();
    assert_eq!(b[0].location.record, *input.records[0].record.key());
    assert_eq!(g[0].location.record, *input.records[1].record.key());
    assert_ne!(b[0].location.unit_id, g[0].location.unit_id);
    let root = input.records[0].record.key();
    let base_seed = reader.seed_identity_vector(&base, root).unwrap().unwrap();
    let gemma_seed = reader.seed_identity_vector(&gemma, root).unwrap().unwrap();
    assert_ne!(base_seed.vector, gemma_seed.vector);
    assert_eq!(base_seed.model.profile_id, base);
    assert_eq!(gemma_seed.model.profile_id, gemma);
    assert!(
        reader
            .vector_candidates(&SearchProfileId::new("missing"), &needle, &query, None, 1)
            .is_err()
    );
}
#[test]
fn every_profile_requires_complete_coverage_and_native_limits_before_atomic_publication() {
    let input = both();
    let (_dir, path) = build(&input);
    let before = validate_artifact(&path).unwrap();
    let mut missing = input;
    let gemma = SearchProfileId::new("embeddinggemma");
    let index = missing
        .semantic_units
        .iter()
        .position(|u| u.profile_id == gemma && u.field.is_some())
        .unwrap();
    let removed = missing.semantic_units.remove(index);
    assert!(crate::test_support::publish_input(&path, &missing).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), before);
    missing.semantic_units.insert(index, removed);
    missing.semantic_units[0].input_token_count = 513;
    assert!(crate::test_support::publish_input(&path, &missing).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), before);
    missing.semantic_units[0].input_token_count = 8;
    missing.semantic_units[index].input_token_count = 2049;
    assert!(crate::test_support::publish_input(&path, &missing).is_err());
    assert_eq!(validate_artifact(&path).unwrap(), before);
}
#[test]
fn historical_same_format_gemma_recipe_preserves_profile_identity_and_shared_content() {
    let input = both();
    let (_dir, path) = build(&input);
    let mut context = input.context.clone();
    context.search_profiles[1].input_recipe.push_str("/prior");
    let json = serde_json::to_string(&context).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute(
        "UPDATE artifact_context SET context_json=?1,context_hash=?2",
        rusqlite::params![json, context.sha256().unwrap()],
    )
    .unwrap();
    c.execute(
        "UPDATE search_profiles SET identity_json=? WHERE profile_id='embeddinggemma'",
        [serde_json::to_string(&context.search_profiles[1]).unwrap()],
    )
    .unwrap();
    move_fixture_vectors(
        &c,
        &input.context.search_profiles[1],
        &context.search_profiles[1],
    );
    drop(c);
    validate_artifact(&path).unwrap();
    let reader = crate::test_support::open_reader(&path).unwrap();
    assert_eq!(
        reader.context().search_profiles[0],
        input.context.search_profiles[0]
    );
    assert_ne!(
        reader.context().search_profiles[1],
        input.context.search_profiles[1]
    );
    assert_eq!(
        reader
            .read_source_record(input.records[0].record.key())
            .unwrap()
            .unwrap()
            .key(),
        input.records[0].record.key()
    );
    assert!(
        reader
            .seed_identity_vector(
                &SearchProfileId::new("embeddinggemma"),
                input.records[0].record.key()
            )
            .unwrap()
            .is_some()
    );
    drop(reader);
    assert!(
        crate::test_support::publish_input(&path, &IndexBuildInput { context, ..input }).is_err()
    );
}

#[test]
fn full_segmentation_digest_changes_generation_and_invalidates_each_affected_profile() {
    let input = both();
    let shared = input.context.preparation_sha256().unwrap();
    let generation = input.context.sha256().unwrap();
    let digest = atlas_embedding::BASELINE_SEGMENTATION_TOKENIZER_SHA256;
    let changed_digest = format!("{}{}", &digest[..8], "0".repeat(56));
    for affected in 0..input.context.search_profiles.len() {
        let (_dir, path) = build(&input);
        let mut changed = input.context.clone();
        assert_eq!(
            changed.search_profiles[affected].segmentation_tokenizer_sha256,
            digest
        );
        changed.search_profiles[affected].segmentation_tokenizer_sha256 = changed_digest.clone();
        assert_ne!(
            changed.search_profiles[affected],
            input.context.search_profiles[affected]
        );
        assert_ne!(changed.sha256().unwrap(), generation);
        assert_eq!(changed.preparation_sha256().unwrap(), shared);
        let profile = &changed.search_profiles[affected];
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute(
                "UPDATE artifact_context SET context_json=?1,context_hash=?2",
                rusqlite::params![
                    serde_json::to_string(&changed).unwrap(),
                    changed.sha256().unwrap()
                ],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE search_profiles SET identity_json=?1 WHERE profile_id=?2",
                rusqlite::params![
                    serde_json::to_string(profile).unwrap(),
                    profile.profile_id.as_str()
                ],
            )
            .unwrap();
        move_fixture_vectors(
            &connection,
            &input.context.search_profiles[affected],
            profile,
        );
        drop(connection);
        validate_artifact(&path).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        for (ordinal, expected) in input.context.search_profiles.iter().enumerate() {
            let seed = reader
                .seed_identity_vector(&expected.profile_id, input.records[0].record.key())
                .unwrap()
                .unwrap();
            assert_eq!(seed.model == *expected, ordinal != affected);
        }
        drop(reader);
        let mut publication = both();
        publication.context = changed.clone();
        assert!(crate::test_support::publish_input(&path, &publication).is_err());
        changed.search_profiles[affected].segmentation_tokenizer_sha256 = digest[..8].into();
        assert!(crate::writer::validate_context(&changed).is_err());
    }
}

fn move_fixture_vectors(
    connection: &rusqlite::Connection,
    old: &SourceSearchProfileIdentity,
    new: &SourceSearchProfileIdentity,
) {
    let old_table = crate::schema::vector_table(old).unwrap();
    let new_table = crate::schema::vector_table(new).unwrap();
    connection
        .execute_batch(&crate::schema::vector_ddl(new).unwrap())
        .unwrap();
    connection.execute_batch(&format!("INSERT INTO {new_table}(unit_id,embedding) SELECT unit_id,embedding FROM {old_table}; DROP TABLE {old_table};")).unwrap();
}

#[test]
fn adding_and_removing_profile_preserves_existing_vector_rows() {
    let single = fixture(true);
    let (_directory, path) = build(&single);
    let table = crate::schema::vector_table(&single.context.search_profiles[0]).unwrap();
    let snapshot = |c: &rusqlite::Connection| {
        c.prepare(&format!(
            "SELECT unit_id,embedding FROM {table} ORDER BY unit_id"
        ))
        .unwrap()
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    };
    let before = snapshot(&rusqlite::Connection::open(&path).unwrap());
    let dual = both();
    crate::test_support::publish_input(&path, &dual).unwrap();
    assert_eq!(
        snapshot(&rusqlite::Connection::open(&path).unwrap()),
        before
    );
    let removed_table = crate::schema::vector_table(&dual.context.search_profiles[1]).unwrap();
    crate::test_support::publish_input(&path, &single).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(snapshot(&c), before);
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?)",
            [&removed_table],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!exists);
}

#[test]
fn vector_storage_dimensions_belong_to_each_model_identity() {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().unwrap();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    for dimensions in [3, 7] {
        let mut model = SourceSearchProfileIdentity::from_spec(atlas_embedding::SEARCH_PROFILES[0]);
        model.dimensions = dimensions;
        let table = crate::schema::vector_table(&model).unwrap();
        connection
            .execute_batch(&crate::schema::vector_ddl(&model).unwrap())
            .unwrap();
        let mut vector = vec![0.0; dimensions];
        vector[0] = 1.0;
        let bytes = crate::read_units::vector_bytes(&vector);
        connection
            .execute(
                &format!("INSERT INTO {table}(unit_id,embedding) VALUES(1,?)"),
                [&bytes],
            )
            .unwrap();
        let distance: f64 = connection
            .query_row(
                &format!("SELECT distance FROM {table} WHERE embedding MATCH ? AND k=1"),
                [&bytes],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(distance, 0.0);
    }
}
