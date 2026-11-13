use super::*;
use crate::{test_support::*, *};
use atlas_storage::Workspace;

fn descriptor(unit: &SourceSemanticUnitInput) -> NativeInputDescriptor {
    NativeInputDescriptor {
        input_sha256: unit.input_hash.clone(),
        token_count: unit.input_token_count,
    }
}
fn count(workspace: &Workspace, table: &str) -> usize {
    workspace
        .read(TEST_WORKSPACE_SCHEMA_VERSION)
        .unwrap()
        .connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn raw_value(
    workspace: &Workspace,
    model: &SourceSearchProfileIdentity,
    input: &NativeInputDescriptor,
) -> Vec<f32> {
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let id = crate::native_values::lookup_input(read.connection(), model, input)
        .unwrap()
        .unwrap();
    crate::native_values::read(read.connection(), id, model).unwrap()
}

#[test]
fn published_reuse_has_no_staged_vector_copy_and_semantic_swaps_keep_exact_values() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("swap.sqlite");
    let mut old = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>East</h2><p>Amber lantern.</p><h2>West</h2><p>Blue lantern.</p>",
    );
    let mut vectors = std::collections::BTreeMap::new();
    for unit in &mut old.semantic_units {
        if unit.field.is_some() {
            let atlas_domain::SourcePassageAddress::HtmlSection {
                section_ordinal, ..
            } = unit.address
            else {
                panic!("HTML section")
            };
            unit.vector = vec![0.; 768];
            unit.vector[20 + section_ordinal] = 1.;
        }
        vectors.insert(unit.input_hash.clone(), unit.vector.clone());
    }
    publish_input(&path, &old).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let mut next = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>West</h2><p>Blue lantern.</p><h2>East</h2><p>Amber lantern.</p>",
    );
    for unit in &mut next.semantic_units {
        unit.vector = vectors[&unit.input_hash].clone();
    }
    let publication =
        prepare_publication_in(&owner, TEST_WORKSPACE_SCHEMA_VERSION, scratch.path(), &next)
            .unwrap();
    assert_eq!(publication.logical_native_bytes().unwrap(), 0);
    publication.publish().unwrap();
    for unit in &next.semantic_units {
        assert_eq!(
            raw_value(
                &workspace,
                next.context.search_profile(&unit.profile_id).unwrap(),
                &descriptor(unit)
            ),
            unit.vector
        );
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        2
    );
}

#[test]
fn edited_input_stages_only_its_new_vector_and_a_noop_stages_none() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("edit.sqlite");
    let old = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>East</h2><p>Amber lantern.</p><h2>West</h2><p>Blue lantern.</p>",
    );
    let previous: std::collections::BTreeMap<_, _> = old
        .semantic_units
        .iter()
        .map(|unit| (unit.input_hash.clone(), unit.vector.clone()))
        .collect();
    publish_input(&path, &old).unwrap();

    let mut edited = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>East</h2><p>Green lantern.</p><h2>West</h2><p>Blue lantern.</p>",
    );
    let mut changed = 0;
    for unit in &mut edited.semantic_units {
        if let Some(vector) = previous.get(&unit.input_hash) {
            unit.vector.clone_from(vector);
        } else {
            unit.vector = vec![0.; 768];
            unit.vector[42] = 1.;
            changed += 1;
        }
    }
    assert!(changed > 0 && changed < edited.semantic_units.len());

    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    for expected_values in [changed, 0] {
        let scratch = BuildScratch::new(&owner).unwrap();
        let publication = prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &edited,
        )
        .unwrap();
        assert_eq!(
            publication.logical_native_bytes().unwrap(),
            (expected_values * 768 * size_of::<f32>()) as u64
        );
        publication.publish().unwrap();
        for unit in &edited.semantic_units {
            let model = edited.context.search_profile(&unit.profile_id).unwrap();
            let actual = raw_value(&workspace, model, &descriptor(unit));
            assert_eq!(
                crate::read_units::vector_bytes(&actual),
                crate::read_units::vector_bytes(&unit.vector)
            );
        }
    }
}

#[test]
fn a_layout_upgrade_reuses_exact_published_inputs_under_the_same_execution_contract() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("layout.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(
        read.workspace_key().unwrap(),
        read.reference_revision().unwrap(),
    );
    drop(read);

    input.context.search_profiles[0].unit_policy_version = "updated-layout".into();
    let model = input.context.search_profiles[0].clone();
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    // Simulate the next executable's accepted layout through the owned staging path.
    let mut publication = PreparedPublication::stage(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
        None,
        None,
    )
    .unwrap();
    for unit in &input.semantic_units {
        assert_eq!(
            publication
                .select_native(&model, &[descriptor(unit)])
                .unwrap(),
            [NativeValueSelection::Reused]
        );
    }
    publication.publish().unwrap();
    assert_eq!(raw_value(&workspace, &model, &desc), unit.vector);
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);

    input.context.search_profiles[0].execution_policy = "incompatible-execution".into();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(
        read.workspace_key().unwrap(),
        read.reference_revision().unwrap(),
    );
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut incompatible = PreparedPublication::stage(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        incompatible
            .select_native(&input.context.search_profiles[0], &[desc])
            .unwrap(),
        [NativeValueSelection::Missing]
    );
}

#[test]
fn ordinary_preparation_clears_legacy_overlaps_and_keeps_unpublished_cache() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy-overlap.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let unpublished = NativeInputDescriptor {
        input_sha256: crate::codec::sha256(b"unpublished input"),
        ..desc.clone()
    };
    let scratch = BuildScratch::new(&owner).unwrap();
    {
        let mut publication = fixture_publication(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        let mut alternative = vec![0.; model.dimensions];
        alternative[9] = 1.;
        publication
            .stage_inferred(
                model,
                &[(&desc, &alternative), (&unpublished, &alternative)],
            )
            .unwrap();
        assert_eq!(count(&workspace, "pending_embedding_values"), 1);
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let published_id = crate::native_values::lookup_input(write.connection(), model, &desc)
            .unwrap()
            .unwrap();
        let bytes = crate::read_units::vector_bytes(&alternative);
        let checksum = crate::embedding_values::hash_bytes(&bytes);
        write
            .connection()
            .execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,0)",
                params![published_id, bytes, checksum],
            )
            .unwrap();
        write.commit().unwrap();
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 2);
    let mut retry = fixture_publication(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    assert_eq!(
        retry
            .select_native(model, std::slice::from_ref(&desc))
            .unwrap(),
        [NativeValueSelection::Reused]
    );
    assert_eq!(retry.logical_native_bytes().unwrap(), 0);
    assert_eq!(
        retry.select_native(model, &[unpublished]).unwrap(),
        [NativeValueSelection::Reused]
    );
    assert_eq!(
        retry.logical_native_bytes().unwrap(),
        (model.dimensions * 4) as u64
    );
    assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
}

#[test]
fn cache_eviction_collects_unpublished_metadata_without_invalidating_owned_staging() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("eviction.sqlite")).unwrap();
    let owner = workspace.update_owner().unwrap();
    let input = crate::foundation_tests::fixture(true);
    for attempt in 0..2 {
        let scratch = BuildScratch::new(&owner).unwrap();
        let mut publication = prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        publication.cache_limit = 0;
        publication.evict_cache().unwrap();
        assert_eq!(count(&workspace, "pending_embedding_values"), 0);
        assert_eq!(count(&workspace, "embedding_inputs"), 0);
        assert_eq!(count(&workspace, "embedding_models"), 0);
        if attempt == 1 {
            publication.publish().unwrap();
        }
    }
    assert!(count(&workspace, "embedding_inputs") > 0);
    crate::validate_artifact(workspace.path()).unwrap();
}

#[test]
fn failed_publication_reopens_exact_prior_state_without_repairing_old_damage() {
    for damaged in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("rollback.sqlite");
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        write.connection().execute_batch("INSERT INTO saved_lists(owner_id,list_key,slug,name,created_at,updated_at) VALUES(1,'keep','keep','Saved name','before','before');
            CREATE TRIGGER reject_after_activation AFTER DELETE ON pending_embedding_values BEGIN
            SELECT CASE WHEN (SELECT published_hash FROM embedding_inputs WHERE input_id=OLD.input_id) IS NOT OLD.vector_hash THEN RAISE(ABORT,'selected value was not installed before failure injection') END;
            SELECT RAISE(ABORT,'injected publication failure'); END;").unwrap();
        if damaged {
            write
                .connection()
                .execute("UPDATE records SET content_hash='already-damaged'", [])
                .unwrap();
        }
        write.commit().unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let mut next = crate::foundation_tests::fixture(true);
        next.context.source_revision = Some("changed-context".into());
        for unit in &mut next.semantic_units {
            unit.input_hash = crate::codec::sha256(unit.input_hash.as_bytes());
            unit.vector = vec![0.; 768];
            unit.vector[11] = 1.;
        }
        let publication = prepare_publication_with_inferred_values(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &next,
        )
        .unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let before = reference_preservation_rows(read.connection()).unwrap();
        assert!(!before["pending_embedding_values"].is_empty());
        drop(read);
        let error = publication.publish().unwrap_err();
        assert!(
            error.to_string().contains("injected publication failure"),
            "{error}"
        );
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), 1);
        assert_eq!(
            reference_preservation_rows(read.connection()).unwrap(),
            before
        );
        assert_eq!(
            crate::validate_reference(read.connection()).is_err(),
            damaged
        );
    }
}

#[test]
fn publication_rejects_context_presence_inconsistent_with_the_reference_revision() {
    let input = crate::foundation_tests::fixture(false);
    for (corruption, revision) in [
        ("DELETE FROM artifact_context", 1),
        ("UPDATE workspace SET reference_revision=0 WHERE id=1", 0),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("workspace.sqlite");
        publish_input(&path, &input).unwrap();
        let connection = Connection::open(&path).unwrap();
        connection.execute(corruption, []).unwrap();
        drop(connection);

        let error = publish_input(&path, &input).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("invalid workspace/revision/context identity")
        );
        let workspace = Workspace::at(&path).unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), revision);
    }
}

#[test]
fn conflicting_batch_tokens_or_values_fail_before_any_cache_write() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("duplicate.sqlite")).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let input = crate::foundation_tests::fixture(true);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(read.workspace_key().unwrap(), 0);
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut publication = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(&input.semantic_units[0]);
    let mut different_tokens = desc.clone();
    different_tokens.token_count += 1;
    let vector = &input.semantic_units[0].vector;
    assert!(
        publication
            .stage_inferred(model, &[(&desc, vector), (&different_tokens, vector)])
            .is_err()
    );
    let mut other = vec![0.; model.dimensions];
    other[8] = 1.;
    assert!(
        publication
            .stage_inferred(model, &[(&desc, vector), (&desc, &other)])
            .is_err()
    );
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    assert_eq!(count(&workspace, "embedding_inputs"), 0);
    assert_eq!(publication.logical_native_bytes().unwrap(), 0);
}

#[test]
fn real_scratch_sqlite_full_keeps_committed_cache_and_old_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("full.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    for unit in &mut input.semantic_units {
        unit.input_hash = crate::codec::sha256(unit.input_hash.as_bytes());
    }
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(read.workspace_key().unwrap(), 1);
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut publication = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    let pages: u32 = publication
        .selected
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap();
    publication
        .selected
        .pragma_update(None, "max_page_count", pages)
        .unwrap();
    let native = crate::test_support::fixture_native_values(&input).unwrap();
    let model = native[0].model;
    let batch = native
        .iter()
        .filter(|value| value.model.profile_id == model.profile_id)
        .map(|value| (&value.input, value.vector))
        .collect::<Vec<_>>();
    let error = publication.stage_inferred(model, &batch).unwrap_err();
    assert!(error.to_string().contains("full"), "{error}");
    assert_eq!(count(&workspace, "pending_embedding_values"), batch.len());
    assert_eq!(
        publication
            .selected
            .query_row("SELECT count(*) FROM selected", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
    crate::validate_artifact(&path).unwrap();
}

#[test]
fn fixture_conflicting_late_duplicate_is_rejected_before_reference_writes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fixture-duplicate.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let mut duplicate = input.semantic_units[0].clone();
    duplicate.vector = vec![0.; 768];
    duplicate.vector[15] = 1.;
    input.semantic_units.push(duplicate);
    assert!(
        publish_input(&path, &input)
            .unwrap_err()
            .to_string()
            .contains("duplicate native input values disagree")
    );
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    crate::recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
    crate::validate_artifact(&path).unwrap();
}

#[test]
fn empty_requested_profile_has_one_empty_native_table_and_no_target_history() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("empty.sqlite")).unwrap();
    let mut input = crate::foundation_tests::fixture(true);
    input.records.clear();
    input.semantic_units.clear();
    input.lexical_units.clear();
    input.aliases.clear();
    input.remaster_pairs.clear();
    input.context.source_file_count = 0;
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap()
    .publish()
    .unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let table = crate::schema::native_vector_table(&input.context.search_profiles[0]).unwrap();
    assert_eq!(
        read.connection()
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    assert_eq!(read.connection().query_row("SELECT count(*) FROM sqlite_schema WHERE name IN ('native_update_targets','native_target_inputs','reference_update_jobs','trait_native_values')",[],|r|r.get::<_,usize>(0)).unwrap(),0);
    crate::validate_reference(read.connection()).unwrap();
}

#[test]
fn zero_profile_preparation_keeps_retry_cache_until_successful_complete_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("zero-profile.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let published = descriptor(&input.semantic_units[0]);
    input.semantic_units[0].input_hash = crate::codec::sha256(b"unpublished input");
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    {
        let scratch = BuildScratch::new(&owner).unwrap();
        let publication = prepare_publication_with_inferred_values(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        assert!(count(&workspace, "pending_embedding_values") > 0);
        drop(publication);
    }
    let before = count(&workspace, "pending_embedding_values");
    let next = crate::foundation_tests::fixture(false);
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication =
        prepare_publication_in(&owner, TEST_WORKSPACE_SCHEMA_VERSION, scratch.path(), &next)
            .unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), before);
    assert!(!raw_value(&workspace, &input.context.search_profiles[0], &published).is_empty());
    publication.publish().unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    assert_eq!(count(&workspace, "embedding_inputs"), 0);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.connection().query_row("SELECT count(*) FROM sqlite_schema WHERE sql LIKE 'CREATE VIRTUAL TABLE%USING vec0%'",[],|r|r.get::<_,usize>(0)).unwrap(),0);
    crate::validate_reference(read.connection()).unwrap();
}

#[test]
fn malformed_pending_payloads_are_recomputed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("corrupt-cache.sqlite");
    let input = crate::foundation_tests::fixture(true);
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let scratch = BuildScratch::new(&owner).unwrap();
    let mut first = fixture_publication(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    first
        .stage_inferred(model, &[(&desc, &unit.vector)])
        .unwrap();
    drop(first);

    let good = crate::read_units::vector_bytes(&unit.vector);
    let malformed = [
        (vec![0; 8 * 1024 * 1024], vec![0; 32], desc.token_count),
        (good.clone(), vec![0; 32], desc.token_count),
        {
            let bytes = crate::read_units::vector_bytes(&vec![f32::NAN; model.dimensions]);
            let hash = crate::embedding_values::hash_bytes(&bytes);
            (bytes, hash, desc.token_count)
        },
        (
            good.clone(),
            crate::embedding_values::hash_bytes(&good),
            desc.token_count + 1,
        ),
    ];
    for (bytes, hash, tokens) in malformed {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        write
            .connection()
            .execute(
                "UPDATE pending_embedding_values SET vector=?,vector_hash=?",
                params![bytes, hash],
            )
            .unwrap();
        write
            .connection()
            .execute("UPDATE embedding_inputs SET token_count=?", [tokens])
            .unwrap();
        write.commit().unwrap();

        let mut retry = fixture_publication(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        assert_eq!(
            retry
                .select_native(model, std::slice::from_ref(&desc))
                .unwrap(),
            [NativeValueSelection::Missing]
        );
        retry
            .stage_inferred(model, &[(&desc, &unit.vector)])
            .unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let (count, vector): (usize, Vec<u8>) = read.connection().query_row(
            "SELECT token_count,vector FROM embedding_inputs JOIN pending_embedding_values USING(input_id)",
            [], |row| Ok((row.get(0)?,row.get(1)?)),
        ).unwrap();
        assert_eq!(count, desc.token_count);
        assert_eq!(vector, good);
    }
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    publication.publish().unwrap();
    assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
}

#[test]
fn oversized_staged_vector_rolls_back_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("corrupt-selection.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    input.semantic_units[0].input_hash = crate::codec::sha256(b"unpublished input");
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_with_inferred_values(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    publication.selected.execute("UPDATE selected SET vector=zeroblob(8388608) WHERE input_hash=(SELECT min(input_hash) FROM selected)",[]).unwrap();
    let pending = count(&workspace, "pending_embedding_values");
    assert!(
        publication
            .publish()
            .unwrap_err()
            .to_string()
            .contains("staged vector length mismatch")
    );
    let maintenance = owner
        .maintenance(TEST_WORKSPACE_SCHEMA_VERSION, "inspect rejected staging")
        .unwrap();
    assert_eq!(
        maintenance
            .connection()
            .query_row("SELECT reference_revision FROM workspace", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        maintenance
            .connection()
            .query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
                r.get::<_, usize>(0)
            })
            .unwrap(),
        pending
    );
    crate::validate_reference(maintenance.connection()).unwrap();
    drop(maintenance);
    crate::recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
}

#[test]
fn empty_and_oversized_batches_do_not_open_the_workspace() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("bounded.sqlite")).unwrap();
    let input = crate::foundation_tests::fixture(true);
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(read.workspace_key().unwrap(), 0);
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut publication = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    let missing = directory.path().join("must-not-open.sqlite");
    publication.workspace = Workspace::at(&missing).unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(&input.semantic_units[0]);
    assert!(publication.select_native(model, &[]).unwrap().is_empty());
    publication.stage_inferred(model, &[]).unwrap();
    assert!(
        publication
            .select_native(model, &vec![desc.clone(); 129])
            .unwrap_err()
            .to_string()
            .contains("batch exceeds its bound")
    );
    let batch = vec![(&desc, input.semantic_units[0].vector.as_slice()); 129];
    assert!(
        publication
            .stage_inferred(model, &batch)
            .unwrap_err()
            .to_string()
            .contains("batch exceeds its bound")
    );
    let mut wrong_model = model.clone();
    wrong_model.revision.push_str("-different");
    assert!(
        publication
            .select_native(&wrong_model, std::slice::from_ref(&desc))
            .unwrap_err()
            .to_string()
            .contains("model differs from publication")
    );
    assert!(!missing.exists());
}

#[test]
fn oversized_selected_hash_or_key_is_rejected_on_consumption_without_losing_rows() {
    for field in ["selected_hash", "input_hash"] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("metadata-bounds.sqlite");
        let mut input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let published = descriptor(&input.semantic_units[0]);
        input.semantic_units[0].input_hash = crate::codec::sha256(b"unpublished input");
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let publication = prepare_publication_with_inferred_values(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        let model = &input.context.search_profiles[0];
        let desc = descriptor(&input.semantic_units[0]);
        publication
            .selected
            .pragma_update(None, "ignore_check_constraints", true)
            .unwrap();
        publication
            .selected
            .execute(
                &format!(
                    "UPDATE selected SET {field}=zeroblob(8388608) WHERE model=0 AND input_hash=?"
                ),
                [crate::embedding_values::digest(&desc.input_sha256).unwrap()],
            )
            .unwrap();
        if field == "selected_hash" {
            assert!(
                selected_checksum(
                    &publication.selected,
                    publication.model_key(model).unwrap(),
                    &crate::native_values::NativeInput::new(model, &desc).unwrap(),
                )
                .unwrap_err()
                .to_string()
                .contains("selected native identity mismatch")
            );
            let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
            let bindings = crate::native_values::NativeBindings::new(
                read.connection(),
                &publication.models,
                false,
            )
            .unwrap();
            let values = PublicationValues {
                selected: &publication.selected,
                bindings: &bindings,
            };
            assert!(
                values
                    .value(model, &desc)
                    .unwrap_err()
                    .to_string()
                    .contains("selected native identity mismatch")
            );
        }
        let pending = count(&workspace, "pending_embedding_values");
        assert!(pending > 0);
        let error = publication.publish().unwrap_err();
        let expected = if field == "input_hash" {
            "native input hash length mismatch"
        } else {
            "selected native checksum length mismatch"
        };
        assert!(error.to_string().contains(expected), "{field}: {error}");
        let maintenance = owner
            .maintenance(
                TEST_WORKSPACE_SCHEMA_VERSION,
                "inspect bounded metadata failure",
            )
            .unwrap();
        assert_eq!(
            maintenance
                .connection()
                .query_row("SELECT reference_revision FROM workspace", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            maintenance
                .connection()
                .query_row("SELECT count(*) FROM pending_embedding_values", [], |r| {
                    r.get::<_, usize>(0)
                })
                .unwrap(),
            pending
        );
        let id = crate::native_values::lookup_input(maintenance.connection(), model, &published)
            .unwrap()
            .unwrap();
        assert_eq!(
            crate::native_values::read(maintenance.connection(), id, model).unwrap(),
            input.semantic_units[0].vector
        );
        crate::validate_reference(maintenance.connection()).unwrap();
        drop(maintenance);
        crate::recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    }
}

#[test]
fn selection_rejects_missing_published_checksum_for_native_or_document_consumers() {
    for consumer in ["native", "document"] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("null-published.sqlite");
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let model = &input.context.search_profiles[0];
        let desc = descriptor(&input.semantic_units[0]);
        let table = crate::schema::native_vector_table(model).unwrap();
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let c = write.connection();
        let id = crate::native_values::lookup_input(c, model, &desc)
            .unwrap()
            .unwrap();
        c.execute(
            "UPDATE embedding_inputs SET published_hash=NULL,token_count=token_count+1 WHERE input_id=?",
            [id],
        )
        .unwrap();
        if consumer == "native" {
            c.execute("DELETE FROM semantic_units WHERE input_id=?", [id])
                .unwrap();
        } else {
            c.execute(&format!("DELETE FROM {table} WHERE input_id=?"), [id])
                .unwrap();
        }
        write.commit().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let expected = ArtifactInstance::new(
            read.workspace_key().unwrap(),
            read.reference_revision().unwrap(),
        );
        drop(read);
        let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
        let mut publication = PreparedPublication::new_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            metadata,
        )
        .unwrap();
        let error = publication
            .select_native(model, std::slice::from_ref(&desc))
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("published native checksum missing"),
            "{consumer}: {error}"
        );
        assert!(
            publication
                .stage_inferred(model, &[(&desc, &input.semantic_units[0].vector)])
                .is_err()
        );
        assert_eq!(count(&workspace, "pending_embedding_values"), 0);
        assert_eq!(publication.logical_native_bytes().unwrap(), 0);
        assert_eq!(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .reference_revision()
                .unwrap(),
            1
        );
    }
}

#[test]
fn unpublished_input_without_consumers_is_missing_and_cache_only_input_is_reused() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("unpublished.sqlite");
    let input = crate::foundation_tests::fixture(true);
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(&input.semantic_units[0]);
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    crate::embedding_values::input_id(
        write.connection(),
        model,
        &desc.input_sha256,
        desc.token_count,
    )
    .unwrap();
    write.commit().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(read.workspace_key().unwrap(), 0);
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut publication = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    assert_eq!(
        publication
            .select_native(model, std::slice::from_ref(&desc))
            .unwrap(),
        [NativeValueSelection::Missing]
    );
    publication
        .stage_inferred(model, &[(&desc, &input.semantic_units[0].vector)])
        .unwrap();
    drop(publication);
    let metadata = publication_spool_in(
        scratch.path(),
        &input,
        ArtifactInstance::new(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .workspace_key()
                .unwrap(),
            0,
        ),
    )
    .unwrap();
    let mut retry = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    assert_eq!(
        retry
            .select_native(model, std::slice::from_ref(&desc))
            .unwrap(),
        [NativeValueSelection::Reused]
    );
    assert_eq!(
        retry.logical_native_bytes().unwrap(),
        (model.dimensions * 4) as u64
    );
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
}

#[test]
fn inferred_batches_rebind_after_every_cache_entry_and_model_are_evicted() {
    let directory = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(directory.path().join("rebind.sqlite")).unwrap();
    let input = crate::foundation_tests::fixture(true);
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let expected = ArtifactInstance::new(read.workspace_key().unwrap(), 0);
    drop(read);
    let metadata = publication_spool_in(scratch.path(), &input, expected).unwrap();
    let mut publication = PreparedPublication::new_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        metadata,
    )
    .unwrap();
    publication.cache_limit = 0;
    let native = crate::test_support::fixture_native_values(&input).unwrap();
    for value in native {
        publication
            .stage_inferred(value.model, &[(&value.input, value.vector)])
            .unwrap();
        assert_eq!(count(&workspace, "pending_embedding_values"), 0);
        assert_eq!(count(&workspace, "embedding_inputs"), 0);
        assert_eq!(count(&workspace, "embedding_models"), 0);
        assert_eq!(
            publication
                .select_native(value.model, std::slice::from_ref(&value.input))
                .unwrap(),
            [NativeValueSelection::AlreadySelected]
        );
    }
    publication.publish().unwrap();
    crate::validate_artifact(workspace.path()).unwrap();
}

#[test]
fn explicit_fixture_publication_installs_its_value_over_unpublished_cache() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("explicit-fixture-values.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    input.semantic_units[0].input_hash = crate::codec::sha256(b"unpublished input");
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let mut pending = vec![0.; model.dimensions];
    pending[23] = 1.;
    assert_ne!(pending, unit.vector);
    let workspace = initialize_workspace(&path).unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let mut publication = fixture_publication(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
        )
        .unwrap();
        publication
            .stage_inferred(model, &[(&desc, &pending)])
            .unwrap();
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    publish_input(&path, &input).unwrap();
    assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
}

#[test]
fn rebuild_ignores_corrupt_native_registry_and_pending_values() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rebuild.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute_batch(
            "INSERT INTO pending_embedding_values(input_id,vector,vector_hash,cached_at)
         SELECT input_id,zeroblob(3072),published_hash,0 FROM embedding_inputs LIMIT 1;
         UPDATE embedding_models SET identity_json='{}',published_token_count_digest=X'00';",
        )
        .unwrap();
    write.commit().unwrap();

    let scratch = BuildScratch::new(&owner).unwrap();
    let repair = crate::test_support::reference_repair(&owner).unwrap();
    let expected = ReferenceReuseReader::fresh(&repair);
    let spool = publication_spool_in(scratch.path(), &input, expected.expected().clone()).unwrap();
    let mut publication = PreparedPublication::new_rebuild(
        repair,
        scratch.path(),
        spool,
        crate::RequestedReferenceConfig {
            source_root: directory.path().join("source"),
            manifest_path: None,
            indexing_locale: "en".into(),
            search_profiles: vec![model.profile_id.clone()],
        },
    )
    .unwrap();
    for unit in &input.semantic_units {
        let desc = descriptor(unit);
        if publication
            .select_native(model, std::slice::from_ref(&desc))
            .unwrap()
            == [NativeValueSelection::Missing]
        {
            publication
                .stage_inferred(model, &[(&desc, &unit.vector)])
                .unwrap();
        }
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    assert!(
        publication
            .verified_native_token_counts(model)
            .unwrap()
            .is_empty()
    );
    publication.finish_verified_native_profile(model).unwrap();
    publication.publish().unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let digest_bytes: usize = read
        .connection()
        .query_row(
            "SELECT length(published_token_count_digest) FROM embedding_models",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(digest_bytes, 33);
    crate::validate_reference_with_token_count_digests(read.connection()).unwrap();
    drop(read);
    for unit in &input.semantic_units {
        assert_eq!(raw_value(&workspace, model, &descriptor(unit)), unit.vector);
    }
}

#[test]
fn published_token_count_digest_rejects_corruption_without_blocking_maintenance_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("descriptor-proof.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let model = &input.context.search_profiles[0];
    let descriptor = descriptor(&input.semantic_units[0]);

    let mut bootstrap = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    assert!(
        bootstrap
            .verified_native_token_counts(model)
            .unwrap()
            .is_empty()
    );
    bootstrap.finish_verified_native_profile(model).unwrap();
    bootstrap.publish().unwrap();

    let retry = fixture_publication(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    assert_eq!(
        retry
            .verified_native_token_counts(model)
            .unwrap()
            .count_for(&descriptor.input_sha256),
        Some(descriptor.token_count)
    );
    drop(retry);
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute(
            "UPDATE embedding_inputs SET token_count=token_count+1 WHERE input_hash=?",
            [crate::embedding_values::digest(&descriptor.input_sha256).unwrap()],
        )
        .unwrap();
    write.commit().unwrap();

    let retry = fixture_publication(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    let error = retry.verified_native_token_counts(model).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("published token-count digest mismatch")
    );
    drop(retry);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 2);
    assert!(
        crate::validate_reference_with_token_count_digests(read.connection())
            .unwrap_err()
            .to_string()
            .contains("published token-count digest mismatch")
    );
    drop(read);

    drop(
        owner
            .maintenance(
                TEST_WORKSPACE_SCHEMA_VERSION,
                "test interrupted maintenance",
            )
            .unwrap(),
    );
    assert!(crate::recover_reference_publication(&owner, TEST_WORKSPACE_SCHEMA_VERSION).unwrap());
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        2
    );
}

#[test]
fn token_count_digest_failure_rolls_back_reference_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("digest-rollback.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute_batch(
            "CREATE TRIGGER reject_descriptor_digest BEFORE UPDATE OF published_token_count_digest
         ON embedding_models BEGIN SELECT RAISE(ABORT,'injected digest write failure'); END;",
        )
        .unwrap();
    write.commit().unwrap();
    input.semantic_units[0].input_hash = crate::codec::sha256(b"new source input for rollback");
    let mut publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
    )
    .unwrap();
    publication
        .finish_verified_native_profile(&input.context.search_profiles[0])
        .unwrap();
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let before = reference_preservation_rows(read.connection()).unwrap();
    drop(read);
    assert!(
        publication
            .publish()
            .unwrap_err()
            .to_string()
            .contains("injected digest write failure")
    );
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.reference_revision().unwrap(), 1);
    assert_eq!(
        reference_preservation_rows(read.connection()).unwrap(),
        before
    );
}

#[test]
fn first_vector_publication_keeps_a_keyword_snapshot_readable() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("staged.sqlite");
    let lexical = crate::foundation_tests::fixture(false);
    publish_input(&path, &lexical).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let semantic = crate::foundation_tests::fixture(true);
    let publication = prepare_publication_with_inferred_values(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &semantic,
    )
    .unwrap();
    let keyword = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let keyword_revision = keyword.reference_revision().unwrap();

    publication.publish_first_profiles().unwrap();

    assert_eq!(keyword.reference_revision().unwrap(), keyword_revision);
    assert!(
        crate::read_stored_reference_context(keyword.connection())
            .unwrap()
            .unwrap()
            .search_profiles
            .is_empty()
    );
    crate::validate_reference(keyword.connection()).unwrap();
    let committed = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        committed.reference_revision().unwrap(),
        keyword_revision + 1
    );
    assert_eq!(
        crate::read_stored_reference_context(committed.connection())
            .unwrap()
            .unwrap()
            .search_profiles,
        semantic.context.search_profiles
    );
    crate::validate_reference(committed.connection()).unwrap();
}

#[test]
fn exact_model_publication_commits_configuration_and_preference_with_vectors() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("models.sqlite");
    let indexed = crate::foundation_tests::fixture(true);
    publish_input(&path, &indexed).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let base = atlas_domain::SearchProfileId::new("bge-base");
    let original = RequestedReferenceConfig {
        source_root: directory.path().join("source"),
        manifest_path: None,
        indexing_locale: "en".into(),
        search_profiles: vec![base.clone()],
    };
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write_requested_reference_config(&write, &original).unwrap();
    set_preferred_search_profile(write.connection(), Some(&base)).unwrap();
    write.commit().unwrap();

    let keyword = crate::foundation_tests::fixture(false);
    let mut change = WorkspaceModelChange {
        previous_configuration: original.clone(),
        previous_preference: None, // Deliberately stale.
        configuration: RequestedReferenceConfig {
            search_profiles: vec![],
            ..original.clone()
        },
        preference: None,
    };
    let owner = workspace.update_owner().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let prepare = || {
        prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &keyword,
        )
        .unwrap()
    };
    assert!(prepare().publish_model_change(&change).is_err());
    let before = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let revision = before.reference_revision().unwrap();
    assert_eq!(
        read_requested_reference_config(&before).unwrap(),
        Some(original.clone())
    );
    assert_eq!(
        preferred_search_profile(before.connection()).unwrap(),
        Some(base.clone())
    );
    let reader = SqliteIndexReader::from_session(before).unwrap();
    assert_eq!(reader.context().search_profiles.len(), 1);
    drop(reader);

    change.previous_preference = Some(base.clone());
    change.preference = Some(base); // Disallowed after removing the model.
    assert!(prepare().publish_model_change(&change).is_err());
    let after_failure = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(after_failure.reference_revision().unwrap(), revision);
    assert_eq!(
        read_requested_reference_config(&after_failure).unwrap(),
        Some(original)
    );
    drop(after_failure);

    change.preference = None;
    prepare().publish_model_change(&change).unwrap();
    let after = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(after.reference_revision().unwrap(), revision + 1);
    assert_eq!(
        read_requested_reference_config(&after).unwrap(),
        Some(change.configuration)
    );
    assert_eq!(preferred_search_profile(after.connection()).unwrap(), None);
    let reader = SqliteIndexReader::from_session(after).unwrap();
    assert!(reader.context().search_profiles.is_empty());
}
