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
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &next,
        true,
    )
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
        false,
    )
    .unwrap();
    assert_eq!(
        publication
            .select_native(&model, std::slice::from_ref(&desc), false)
            .unwrap(),
        [NativeValueSelection::Missing]
    );
    for unit in &input.semantic_units {
        assert_eq!(
            publication
                .select_native(&model, &[descriptor(unit)], true)
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
        false,
    )
    .unwrap();
    assert_eq!(
        incompatible
            .select_native(&input.context.search_profiles[0], &[desc], true)
            .unwrap(),
        [NativeValueSelection::Missing]
    );
}

#[test]
fn forced_cache_precedes_published_and_cross_batch_selection_survives_eviction() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("forced.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let mut forced = vec![0.; model.dimensions];
    forced[9] = 1.;
    {
        let scratch = BuildScratch::new(&owner).unwrap();
        let metadata = publication_spool_in(
            scratch.path(),
            &input,
            ArtifactInstance::new(
                workspace
                    .read(TEST_WORKSPACE_SCHEMA_VERSION)
                    .unwrap()
                    .workspace_key()
                    .unwrap(),
                1,
            ),
        )
        .unwrap();
        let mut publication = PreparedPublication::new_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            metadata,
        )
        .unwrap();
        assert_eq!(
            publication
                .select_native(model, std::slice::from_ref(&desc), false)
                .unwrap(),
            [NativeValueSelection::Missing]
        );
        publication
            .stage_inferred(model, &[(&desc, &forced)])
            .unwrap();
        assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
        publication.cache_limit = 0;
        publication.evict_cache().unwrap();
        assert_eq!(count(&workspace, "pending_embedding_values"), 0);
        assert_eq!(
            publication
                .select_native(model, std::slice::from_ref(&desc), false)
                .unwrap(),
            [NativeValueSelection::AlreadySelected]
        );
        assert!(
            publication
                .stage_inferred(model, &[(&desc, &unit.vector)])
                .is_err()
        );
        // Restore the expensive result durably before abandoning this attempt.
        publication.cache_limit = CACHE_PAYLOAD_LIMIT;
        publication
            .stage_inferred(model, &[(&desc, &forced)])
            .unwrap();
    }
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
        false,
    )
    .unwrap();
    assert!(publication.logical_native_bytes().unwrap() > 0);
    publication.publish().unwrap();
    assert_eq!(raw_value(&workspace, model, &desc), forced);
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
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
            false,
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
        write.connection().execute_batch("INSERT INTO saved_lists(list_key,slug,name,created_at,updated_at) VALUES('keep','keep','Saved name','before','before');
            CREATE TRIGGER reject_after_activation AFTER DELETE ON pending_embedding_values BEGIN
            SELECT CASE WHEN (SELECT published_hash FROM embedding_inputs WHERE input_id=OLD.input_id) IS NOT OLD.vector_hash THEN RAISE(ABORT,'selected value was not installed before failure injection') END;
            SELECT RAISE(ABORT,'injected publication failure'); END;").unwrap();
        if damaged {
            write
                .connection()
                .execute("UPDATE records SET content_hash='already-damaged'", [])
                .unwrap();
        }
        let mut before = reference_preservation_rows(write.connection()).unwrap();
        before.remove("pending_embedding_values");
        write.commit().unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let mut next = crate::foundation_tests::fixture(true);
        next.context.source_revision = Some("changed-context".into());
        for unit in &mut next.semantic_units {
            unit.vector = vec![0.; 768];
            unit.vector[11] = 1.;
        }
        let publication = prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &next,
            false,
        )
        .unwrap();
        let pending = count(&workspace, "pending_embedding_values");
        assert!(pending > 0);
        let error = publication.publish().unwrap_err();
        assert!(
            error.to_string().contains("injected publication failure"),
            "{error}"
        );
        let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(read.reference_revision().unwrap(), 1);
        let mut after = reference_preservation_rows(read.connection()).unwrap();
        after.remove("pending_embedding_values");
        assert_eq!(after, before);
        assert_eq!(
            read.connection()
                .query_row("SELECT count(*) FROM pending_embedding_values", [], |row| {
                    row.get::<_, usize>(0)
                })
                .unwrap(),
            pending
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
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
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
        true,
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
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    {
        let scratch = BuildScratch::new(&owner).unwrap();
        let publication = prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
            false,
        )
        .unwrap();
        assert!(count(&workspace, "pending_embedding_values") > 0);
        drop(publication);
    }
    let before = count(&workspace, "pending_embedding_values");
    let next = crate::foundation_tests::fixture(false);
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &next,
        true,
    )
    .unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), before);
    assert!(
        !raw_value(
            &workspace,
            &input.context.search_profiles[0],
            &descriptor(&input.semantic_units[0])
        )
        .is_empty()
    );
    publication.publish().unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    assert_eq!(count(&workspace, "embedding_inputs"), 0);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(read.connection().query_row("SELECT count(*) FROM sqlite_schema WHERE sql LIKE 'CREATE VIRTUAL TABLE%USING vec0%'",[],|r|r.get::<_,usize>(0)).unwrap(),0);
    crate::validate_reference(read.connection()).unwrap();
}

#[test]
fn active_cache_corruption_never_falls_back_and_staged_oversize_rolls_back_publication() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("corrupt-selection.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(&input.semantic_units[0]);
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let id = crate::native_values::lookup_input(write.connection(), model, &desc)
        .unwrap()
        .unwrap();
    write
        .connection()
        .execute(
            "INSERT INTO pending_embedding_values VALUES(?,zeroblob(8388608),zeroblob(32),0)",
            [id],
        )
        .unwrap();
    write.commit().unwrap();
    {
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
        assert!(
            publication
                .select_native(model, std::slice::from_ref(&desc), true)
                .unwrap_err()
                .to_string()
                .contains("cached native length mismatch")
        );
        assert_eq!(publication.logical_native_bytes().unwrap(), 0);
    }
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    write
        .connection()
        .execute("DELETE FROM pending_embedding_values", [])
        .unwrap();
    write.commit().unwrap();
    let scratch = BuildScratch::new(&owner).unwrap();
    let publication = prepare_publication_in(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        &input,
        false,
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
    assert!(
        publication
            .select_native(model, &[], true)
            .unwrap()
            .is_empty()
    );
    publication.stage_inferred(model, &[]).unwrap();
    assert!(
        publication
            .select_native(model, &vec![desc.clone(); 129], true)
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
            .select_native(&wrong_model, std::slice::from_ref(&desc), true)
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
        let input = crate::foundation_tests::fixture(true);
        publish_input(&path, &input).unwrap();
        let workspace = initialize_workspace(&path).unwrap();
        let owner = workspace.update_owner().unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        let publication = prepare_publication_in(
            &owner,
            TEST_WORKSPACE_SCHEMA_VERSION,
            scratch.path(),
            &input,
            false,
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
        let id = crate::native_values::lookup_input(maintenance.connection(), model, &desc)
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
            "UPDATE embedding_inputs SET published_hash=NULL WHERE input_id=?",
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
            .select_native(model, std::slice::from_ref(&desc), true)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("published native checksum missing"),
            "{consumer}: {error}"
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
            .select_native(model, std::slice::from_ref(&desc), true)
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
            .select_native(model, std::slice::from_ref(&desc), true)
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
                .select_native(value.model, std::slice::from_ref(&value.input), false)
                .unwrap(),
            [NativeValueSelection::AlreadySelected]
        );
    }
    publication.publish().unwrap();
    crate::validate_artifact(workspace.path()).unwrap();
}

#[test]
fn current_fixture_publication_installs_explicit_values_over_pending_cache() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("explicit-fixture-values.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let model = &input.context.search_profiles[0];
    let unit = &input.semantic_units[0];
    let desc = descriptor(unit);
    let mut pending = vec![0.; model.dimensions];
    pending[23] = 1.;
    assert_ne!(pending, unit.vector);
    {
        let owner = workspace.update_owner().unwrap();
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
        publication
            .stage_inferred(model, &[(&desc, &pending)])
            .unwrap();
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
    publish_input(&path, &input).unwrap();
    assert_eq!(raw_value(&workspace, model, &desc), unit.vector);
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
         UPDATE embedding_models SET identity_json='{}';",
        )
        .unwrap();
    write.commit().unwrap();

    let scratch = BuildScratch::new(&owner).unwrap();
    let expected = ReferenceReuseReader::fresh(&workspace, TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let spool = publication_spool_in(scratch.path(), &input, expected.expected().clone()).unwrap();
    let mut publication = PreparedPublication::new_rebuild(
        &owner,
        TEST_WORKSPACE_SCHEMA_VERSION,
        scratch.path(),
        spool,
    )
    .unwrap();
    for unit in &input.semantic_units {
        let desc = descriptor(unit);
        if publication
            .select_native(model, std::slice::from_ref(&desc), true)
            .unwrap()
            == [NativeValueSelection::Missing]
        {
            publication
                .stage_inferred(model, &[(&desc, &unit.vector)])
                .unwrap();
        }
    }
    assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    publication.publish().unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    for unit in &input.semantic_units {
        assert_eq!(raw_value(&workspace, model, &descriptor(unit)), unit.vector);
    }
}
