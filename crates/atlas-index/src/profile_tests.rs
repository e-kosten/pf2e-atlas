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
        .vector_candidates(crate::SourceVectorCandidateRequest {
            profile: &base,
            vector: &needle,
            query: &query,
            keys: None,
            exclude: None,
            content_scope: atlas_domain::SearchContentScope::All,
            limit: 1,
        })
        .unwrap();
    let g = reader
        .vector_candidates(crate::SourceVectorCandidateRequest {
            profile: &gemma,
            vector: &needle,
            query: &query,
            keys: None,
            exclude: None,
            content_scope: atlas_domain::SearchContentScope::All,
            limit: 1,
        })
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
            .vector_candidates(crate::SourceVectorCandidateRequest {
                profile: &SearchProfileId::new("missing"),
                vector: &needle,
                query: &query,
                keys: None,
                exclude: None,
                content_scope: atlas_domain::SearchContentScope::All,
                limit: 1
            })
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
    connection
        .execute(
            "UPDATE embedding_models SET identity_json=? WHERE identity_json=?",
            rusqlite::params![
                serde_json::to_string(new).unwrap(),
                serde_json::to_string(old).unwrap()
            ],
        )
        .unwrap();
    let old_table = crate::schema::vector_table(old).unwrap();
    let new_table = crate::schema::vector_table(new).unwrap();
    connection
        .execute_batch(&crate::schema::vector_ddl(new).unwrap())
        .unwrap();
    connection.execute_batch(&format!("INSERT INTO {new_table}(unit_id,embedding) SELECT unit_id,embedding FROM {old_table}; DROP TABLE {old_table};")).unwrap();
}

#[test]
fn adding_and_removing_profile_preserves_existing_vector_rows() {
    let profile_fixture = |dual: bool| {
        let mut input = crate::foundation_tests::fixture_with_description(
            true,
            "<h2>Flame</h2><p>@UUID[Compendium.pf2e.actors.Actor.aaaaaaaaaaaaaaaa] @Localize[PF2E.TraitDescriptionDragon] Dragon breath.</p>",
        );
        if dual {
            input
                .context
                .search_profiles
                .push(SourceSearchProfileIdentity::from_spec(
                    atlas_embedding::SEARCH_PROFILES[1],
                ));
            let alternate = input.records[1].record.key();
            let mut units = input.semantic_units.clone();
            for unit in &mut units {
                unit.profile_id = SearchProfileId::new("embeddinggemma");
                unit.vector.fill(0.0);
                unit.vector[if &unit.record == alternate { 0 } else { 2 }] = 1.0;
            }
            input.semantic_units.extend(units);
        }
        let key = atlas_domain::TraitKey::new("dragon").unwrap();
        input.traits.push(TraitDefinition {
            key: key.clone(),
            label: "Dragon".into(),
            label_key: None,
            description_key: None,
            description_html: None,
            authored_markup: None,
            prepared: None,
            catalogs: vec!["creatureTraits".into()],
            source_locations: vec![],
            source_revision: input.context.source_revision.clone(),
            indexing_locale: input.context.indexing_locale.clone(),
        });
        let text = "Dragon\n";
        let hash = crate::codec::sha256(text.as_bytes());
        for model in &input.context.search_profiles {
            let vector = input
                .semantic_units
                .iter()
                .find(|unit| unit.profile_id == model.profile_id)
                .unwrap()
                .vector
                .clone();
            input.trait_vectors.push(TraitVectorInput {
                key: key.clone(),
                profile_id: model.profile_id.clone(),
                input_hash: crate::codec::sha256(
                    format!("trait-definition-centroid/v1:{hash}").as_bytes(),
                ),
                vector: vector.clone(),
                chunks: vec![TraitSeedChunk {
                    input: text.into(),
                    input_hash: hash.clone(),
                    token_count: 2,
                    address: atlas_domain::SourcePassageAddress::PlainSection {
                        source_text_sha256: hash.clone(),
                        selection_version: "trait-definition-centroid/v1".into(),
                        section_ordinal: 0,
                        label: Some("Dragon".into()),
                        chunk_bytes: atlas_domain::SourceByteRange {
                            start: 0,
                            end: text.len(),
                        },
                    },
                    vector,
                }],
            });
        }
        input
    };
    let single = profile_fixture(false);
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
    let c = rusqlite::Connection::open(&path).unwrap();
    let before = snapshot(&c);
    let protected = [
        "records",
        "record_traits",
        "actor_projection",
        "physical_projection",
        "weapon_projection",
        "spell_projection",
        "spell_traditions",
        "query_field_catalog",
        "projection_allocations",
        "record_bodies",
        "prepared_content",
        "relationship_occurrences",
        "selected_sections",
        "lexical_units",
        "trait_definitions",
        "trait_memberships",
        "trait_references",
        "reference_component_digests",
    ];
    c.execute_batch(
        "CREATE TABLE profile_unrelated_write_audit(component TEXT NOT NULL,event TEXT NOT NULL)",
    )
    .unwrap();
    for table in protected {
        let count: usize = c
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(
            count > 0,
            "protected component {table} must be populated before write auditing"
        );
        for event in ["INSERT", "UPDATE", "DELETE"] {
            c.execute_batch(&format!("CREATE TRIGGER profile_audit_{table}_{event} AFTER {event} ON {table} BEGIN INSERT INTO profile_unrelated_write_audit VALUES('{table}','{event}'); END;")).unwrap();
        }
    }
    let assert_untouched = |c: &rusqlite::Connection, phase: &str| {
        let writes = c.prepare("SELECT component,event,count(*) FROM profile_unrelated_write_audit GROUP BY component,event ORDER BY component,event").unwrap().query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,usize>(2)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        assert!(
            writes.is_empty(),
            "{phase} rewrote unrelated components: {writes:?}"
        );
    };
    drop(c);
    let dual = profile_fixture(true);
    crate::test_support::publish_input(&path, &dual).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(snapshot(&c), before);
    assert_untouched(&c, "adding a profile");
    drop(c);
    let removed_table = crate::schema::vector_table(&dual.context.search_profiles[1]).unwrap();
    crate::test_support::publish_input(&path, &single).unwrap();
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(snapshot(&c), before);
    assert_untouched(&c, "removing a profile");
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
