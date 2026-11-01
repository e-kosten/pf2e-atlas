use crate::{test_support::*, *};
use atlas_embedding::NativeInputDescriptor;
use atlas_storage::{UpdateOwner, Workspace};
use std::collections::BTreeMap;

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
pub(crate) fn staged(
    owner: &UpdateOwner,
    workspace: &Workspace,
    input: &IndexBuildInput,
    reuse: bool,
) -> String {
    staged_requiring_reuse(owner, workspace, input, reuse, &[])
}
fn staged_requiring_reuse(
    owner: &UpdateOwner,
    workspace: &Workspace,
    input: &IndexBuildInput,
    reuse: bool,
    required: &[NativeInputDescriptor],
) -> String {
    let job =
        register_native_target(owner, workspace, TEST_WORKSPACE_SCHEMA_VERSION, input).unwrap();
    crate::native_manifest::visit_native_manifest(input, &mut |model, desc| {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION)?;
        let selected =
            select_native_values(&write, &job, model, std::slice::from_ref(desc), reuse)?;
        if required.contains(desc){
            assert!(matches!(selected.as_slice(),[NativeValueSelection::Reused|NativeValueSelection::AlreadySelected]),"required cache hit became {selected:?}: {desc:?}");
            let pending:bool=write.connection().query_row("SELECT EXISTS(SELECT 1 FROM pending_embedding_values p JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) WHERE m.identity_json=? AND i.input_hash=?)",rusqlite::params![serde_json::to_string(model)?,crate::embedding_values::digest(&desc.input_sha256)?],|r|r.get(0))?;
            assert!(!pending,"the lifetime fixture must not stage replacement bytes for an existing selected value");
        }
        if selected == [NativeValueSelection::Missing] {
            let vector = input.value(model, desc)?;
            stage_native_values(&write, &job, model, &[(desc, &vector)])?;
        }
        write.commit()?;
        Ok(())
    })
    .unwrap();
    job
}
fn publish_selected(
    owner: &UpdateOwner,
    workspace: &Workspace,
    input: &IndexBuildInput,
    reuse: bool,
) {
    publish_selected_requiring_reuse(owner, workspace, input, reuse, &[])
}
fn publish_selected_requiring_reuse(
    owner: &UpdateOwner,
    workspace: &Workspace,
    input: &IndexBuildInput,
    reuse: bool,
    required: &[NativeInputDescriptor],
) {
    let job = staged_requiring_reuse(owner, workspace, input, reuse, required);
    let frozen =
        freeze_native_values(owner, workspace, TEST_WORKSPACE_SCHEMA_VERSION, &job, input).unwrap();
    let maintenance = owner
        .maintenance(
            TEST_WORKSPACE_SCHEMA_VERSION,
            "native target test publication",
        )
        .unwrap();
    SqliteIndexWriter::new(&maintenance)
        .write_job(input, &frozen, &job, &input.context.sha256().unwrap())
        .unwrap();
}

#[test]
fn complete_target_deduplicates_forced_batches_and_preserves_pending_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::profile_tests::both();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(
        input
            .semantic_units
            .iter()
            .find(|u| u.profile_id == model.profile_id)
            .unwrap(),
    );
    let mut replacement = vec![0.; model.dimensions];
    replacement[41] = 1.;
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        select_native_values(&write, &job, model, &[desc.clone(), desc.clone()], false).unwrap(),
        [NativeValueSelection::Missing, NativeValueSelection::Missing]
    );
    stage_native_values(&write, &job, model, &[(&desc, &replacement)]).unwrap();
    write.commit().unwrap();
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        select_native_values(&write, &job, model, std::slice::from_ref(&desc), false).unwrap(),
        [NativeValueSelection::AlreadySelected]
    );
    write.commit().unwrap();
    // Registration of the next exact target retains matching first/second-profile
    // computations, selecting pending bytes instead of old published bytes.
    let next = staged(&owner, &workspace, &input, true);
    let frozen = freeze_native_values(
        &owner,
        &workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        &next,
        &input,
    )
    .unwrap();
    assert_eq!(frozen.value(model, &desc).unwrap().as_ref(), replacement);
    assert_eq!(count(&workspace, "native_update_targets"), 1);
    assert_eq!(count(&workspace, "reference_update_jobs"), 1);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        crate::native_selection::load_value(read.connection(), model, &desc)
            .unwrap()
            .unwrap()
            .vector,
        replacement
    );
    assert_eq!(read.reference_revision().unwrap(), 1);
}

struct FailedRegistration<'a> {
    input: &'a IndexBuildInput,
}
impl PublicationSource for FailedRegistration<'_> {
    fn context(&self) -> &SourceArtifactBuildContext {
        &self.input.context
    }
    fn packs(&self) -> &[IndexBuildPack] {
        &self.input.packs
    }
    fn inventory(&self) -> &CompleteSourceInventory {
        &self.input.inventory
    }
    fn record_count(&self) -> usize {
        self.input.records.len()
    }
    fn semantic_count(&self) -> usize {
        self.input.semantic_units.len()
    }
    fn check_seal(&self) -> Result<(), IndexError> {
        Ok(())
    }
    fn records(
        &self,
        v: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.records(v)
    }
    fn lexical(
        &self,
        v: &mut dyn FnMut(&SourceLexicalUnitInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.lexical(v)
    }
    fn aliases(
        &self,
        v: &mut dyn FnMut(&SourceAliasInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.aliases(v)
    }
    fn remaster_pairs(
        &self,
        v: &mut dyn FnMut(&SourceRemasterPairInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.remaster_pairs(v)
    }
    fn traits(
        &self,
        v: &mut dyn FnMut(&TraitDefinition) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.traits(v)
    }
    fn trait_seeds(
        &self,
        v: &mut dyn FnMut(&TraitSeedOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.input.trait_seeds(v)
    }
    fn semantic(
        &self,
        v: &mut dyn FnMut(&SemanticOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        let mut count = 0;
        self.input.semantic(&mut |unit| {
            if count == NATIVE_VALUE_BATCH_LIMIT {
                if std::env::var_os("ATLAS_NATIVE_REGISTRATION_CHILD").is_some() {
                    std::process::exit(73);
                }
                return Err(IndexError::Operation(
                    "injected registration interruption".into(),
                ));
            }
            count += 1;
            v(unit)
        })
    }
}
fn failing_input(mut input: IndexBuildInput, attempt: usize) -> IndexBuildInput {
    let unit = input.semantic_units[0].clone();
    input.semantic_units = (0..=NATIVE_VALUE_BATCH_LIMIT)
        .map(|i| {
            let mut unit = unit.clone();
            unit.input_hash =
                crate::codec::sha256(format!("attempt-{attempt}-input-{i}").as_bytes());
            unit
        })
        .collect();
    input
}
#[test]
fn repeated_incomplete_registration_bounds_inert_metadata_without_any_promotion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let originals = count(&workspace, "embedding_inputs");
    // Historical unknown membership must survive pre-registration cleanup.
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let unit = &input.semantic_units[0];
        let model = &input.context.search_profiles[0];
        let id = crate::embedding_values::input_id(
            write.connection(),
            model,
            &unit.input_hash,
            unit.input_token_count,
        )
        .unwrap();
        let bytes = crate::read_units::vector_bytes(&unit.vector);
        write
            .connection()
            .execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,'legacy-unknown')",
                rusqlite::params![id, bytes, crate::embedding_values::hash_bytes(&bytes)],
            )
            .unwrap();
        write.commit().unwrap();
    }
    for attempt in 0..12 {
        let failed = failing_input(crate::foundation_tests::fixture(true), attempt);
        assert!(
            register_native_target(
                &owner,
                &workspace,
                TEST_WORKSPACE_SCHEMA_VERSION,
                &FailedRegistration { input: &failed }
            )
            .unwrap_err()
            .to_string()
            .contains("injected registration interruption")
        );
        assert_eq!(count(&workspace, "native_update_targets"), 1);
        assert_eq!(count(&workspace, "reference_update_jobs"), 1);
        assert_eq!(
            count(&workspace, "embedding_inputs"),
            originals + NATIVE_VALUE_BATCH_LIMIT
        );
        assert_eq!(count(&workspace, "embedding_models"), 1);
        assert_eq!(count(&workspace, "pending_embedding_values"), 1);
    }
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
}
#[test]
fn incomplete_candidate_preserves_second_profile_checkpoint_and_prior_active_target() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::profile_tests::both();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    let model = &input.context.search_profiles[1];
    let unit = input
        .semantic_units
        .iter()
        .find(|u| u.profile_id == model.profile_id)
        .unwrap();
    let desc = descriptor(unit);
    let mut vector = vec![0.; model.dimensions];
    vector[55] = 1.;
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        stage_native_values(&write, &job, model, &[(&desc, &vector)]).unwrap();
        write.commit().unwrap();
    }
    let failed = failing_input(crate::profile_tests::both(), 33);
    assert!(
        register_native_target(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &FailedRegistration { input: &failed }
        )
        .is_err()
    );
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        select_native_values(&write, &job, model, std::slice::from_ref(&desc), true).unwrap(),
        [NativeValueSelection::AlreadySelected]
    );
    write.commit().unwrap();
    let next = staged(&owner, &workspace, &input, true);
    let frozen = freeze_native_values(
        &owner,
        &workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        &next,
        &input,
    )
    .unwrap();
    assert_eq!(frozen.value(model, &desc).unwrap().as_ref(), vector);
    assert_eq!(count(&workspace, "native_update_targets"), 1);
}
#[test]
fn complete_zero_profile_target_prunes_obsolete_pending_but_preserves_published_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let _job = staged(&owner, &workspace, &input, false);
    assert!(count(&workspace, "pending_embedding_values") > 0);
    let before = reference_preservation_rows(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .connection(),
    )
    .unwrap();
    let mut empty = crate::foundation_tests::fixture(true);
    empty.context.search_profiles.clear();
    empty.semantic_units.clear();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &empty).unwrap();
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    let after = reference_preservation_rows(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .connection(),
    )
    .unwrap();
    for (table, rows) in before {
        if table.starts_with("semantic_vectors_")
            || table == "semantic_units"
            || table == "trait_native_values"
        {
            assert_eq!(after[&table], rows, "{table}");
        }
    }
    let frozen = freeze_native_values(
        &owner,
        &workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        &job,
        &empty,
    )
    .unwrap();
    assert_eq!(frozen.logical_bytes(), 0);
}
#[test]
fn frozen_values_survive_slot_overwrite_before_later_occurrence_and_final_validation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let mut old = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>East</h2><p>Amber lantern.</p><h2>West</h2><p>Blue lantern.</p>",
    );
    let mut vectors = BTreeMap::new();
    for unit in &mut old.semantic_units {
        if unit.field.is_some() {
            let ordinal = match &unit.address {
                atlas_domain::SourcePassageAddress::HtmlSection {
                    section_ordinal, ..
                } => *section_ordinal,
                _ => panic!("HTML section"),
            };
            unit.vector = vec![0.; 768];
            unit.vector[20 + ordinal] = 1.;
        }
        vectors.insert(unit.input_hash.clone(), unit.vector.clone());
    }
    publish_input(&path, &old).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let mut next = crate::foundation_tests::fixture_with_description(
        true,
        "<h2>West</h2><p>Blue lantern.</p><h2>East</h2><p>Amber lantern.</p>",
    );
    for unit in &mut next.semantic_units {
        unit.vector = vectors[&unit.input_hash].clone();
    }
    let required = next
        .semantic_units
        .iter()
        .map(descriptor)
        .collect::<Vec<_>>();
    publish_selected_requiring_reuse(&owner, &workspace, &next, true, &required);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    for unit in &next.semantic_units {
        let model = next.context.search_profile(&unit.profile_id).unwrap();
        assert_eq!(
            crate::native_selection::load_value(read.connection(), model, &descriptor(unit))
                .unwrap()
                .unwrap()
                .vector,
            unit.vector
        );
    }
    assert_eq!(read.reference_revision().unwrap(), 2);
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
}
#[test]
fn freeze_rejects_wrong_owner_and_missing_or_changed_selected_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let other = initialize_workspace(dir.path().join("other.sqlite")).unwrap();
    let other_owner = other.update_owner().unwrap();
    assert!(
        register_native_target(
            &other_owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &input
        )
        .is_err()
    );
    drop(other_owner);
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    assert!(
        freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input
        )
        .is_err()
    );
    let job = staged(&owner, &workspace, &input, true);
    let unit = &input.semantic_units[0];
    let model = &input.context.search_profiles[0];
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let id = crate::embedding_values::input_id(
            write.connection(),
            model,
            &unit.input_hash,
            unit.input_token_count,
        )
        .unwrap();
        let mut value = vec![0.; 768];
        value[66] = 1.;
        let bytes = crate::read_units::vector_bytes(&value);
        write
            .connection()
            .execute(
                "INSERT INTO pending_embedding_values VALUES(?,?,?,'changed')",
                rusqlite::params![id, bytes, crate::embedding_values::hash_bytes(&bytes)],
            )
            .unwrap();
        write.commit().unwrap();
    }
    assert!(
        freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input
        )
        .is_err()
    );
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
}

#[test]
fn fresh_empty_workspace_publishes_from_origin_zero_and_retires_target_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = initialize_workspace(dir.path().join("fresh.sqlite")).unwrap();
    let input = crate::foundation_tests::fixture(true);
    let owner = workspace.update_owner().unwrap();
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        0
    );
    publish_selected(&owner, &workspace, &input, false);
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
    assert_eq!(count(&workspace, "pending_embedding_values"), 0);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let phase: String = read
        .connection()
        .query_row("SELECT phase FROM native_update_targets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(phase, "retired");
    assert!(crate::validate_reference(read.connection()).is_ok());
}
#[test]
fn removing_last_document_keeps_selected_trait_native_value_and_reverse_transition() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let mut input = crate::foundation_tests::fixture(true);
    crate::embedding_values_tests::add_label_only_trait(&mut input);
    let original = descriptor(&input.semantic_units[0]);
    let chunk = input.trait_vectors[0].chunks[0].clone();
    input.semantic_units[0].input_hash = chunk.input_hash.clone();
    input.semantic_units[0].input_token_count = chunk.token_count;
    input.semantic_units[0].vector = chunk.vector.clone();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    assert_eq!(count(&workspace, "trait_native_values"), 0);
    let mut trait_only = crate::foundation_tests::fixture(true);
    crate::embedding_values_tests::add_label_only_trait(&mut trait_only);
    trait_only.semantic_units[0].input_hash = original.input_sha256;
    trait_only.semantic_units[0].input_token_count = original.token_count;
    let desc = NativeInputDescriptor {
        input_sha256: chunk.input_hash.clone(),
        token_count: chunk.token_count,
    };
    publish_selected_requiring_reuse(
        &owner,
        &workspace,
        &trait_only,
        true,
        std::slice::from_ref(&desc),
    );
    assert_eq!(count(&workspace, "trait_native_values"), 1);
    let model = &input.context.search_profiles[0];
    assert_eq!(
        crate::native_selection::load_value(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
            model,
            &desc
        )
        .unwrap()
        .unwrap()
        .vector,
        chunk.vector
    );
    publish_selected_requiring_reuse(
        &owner,
        &workspace,
        &input,
        true,
        std::slice::from_ref(&desc),
    );
    assert_eq!(count(&workspace, "trait_native_values"), 0);
    assert_eq!(
        crate::native_selection::load_value(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
            model,
            &desc
        )
        .unwrap()
        .unwrap()
        .vector,
        chunk.vector
    );
}
#[test]
fn replacing_one_profile_table_preserves_other_profile_and_frozen_new_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::profile_tests::both();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let mut retired = input.context.clone();
    retired.search_profiles[0].input_recipe.push_str("/retired");
    let old_table = crate::schema::vector_table(&retired.search_profiles[0]).unwrap();
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        let c = write.connection();
        c.execute(
            "UPDATE artifact_context SET context_json=?,context_hash=?",
            rusqlite::params![
                serde_json::to_string(&retired).unwrap(),
                retired.sha256().unwrap()
            ],
        )
        .unwrap();
        c.execute(
            "UPDATE search_profiles SET identity_json=? WHERE profile_id=?",
            rusqlite::params![
                serde_json::to_string(&retired.search_profiles[0]).unwrap(),
                retired.search_profiles[0].profile_id.as_str()
            ],
        )
        .unwrap();
        crate::profile_tests::move_fixture_vectors(
            c,
            &input.context.search_profiles[0],
            &retired.search_profiles[0],
        );
        write.commit().unwrap();
    }
    let owner = workspace.update_owner().unwrap();
    publish_selected(&owner, &workspace, &input, true);
    let read = workspace.read(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert!(
        !read
            .connection()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?)",
                [old_table],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
    );
    for unit in &input.semantic_units {
        let model = input.context.search_profile(&unit.profile_id).unwrap();
        assert_eq!(
            crate::native_selection::load_value(read.connection(), model, &descriptor(unit))
                .unwrap()
                .unwrap()
                .vector,
            unit.vector
        );
    }
}
#[test]
fn repeated_complete_abandoned_targets_bound_pending_and_metadata_without_evicting_publication() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let baseline = count(&workspace, "embedding_inputs");
    let model = &input.context.search_profiles[0];
    let original = &input.semantic_units[0];
    let published = crate::native_selection::load_value(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .connection(),
        model,
        &descriptor(original),
    )
    .unwrap()
    .unwrap()
    .vector;
    for attempt in 0..25 {
        let mut next = crate::foundation_tests::fixture(true);
        next.semantic_units[0].input_hash =
            crate::codec::sha256(format!("abandoned-{attempt}").as_bytes());
        let job = register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &next)
            .unwrap();
        let desc = descriptor(&next.semantic_units[0]);
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        stage_native_values(&write, &job, model, &[(&desc, &original.vector)]).unwrap();
        write.commit().unwrap();
        assert_eq!(count(&workspace, "pending_embedding_values"), 1);
        assert!(count(&workspace, "embedding_inputs") <= baseline + 1);
        assert_eq!(count(&workspace, "native_update_targets"), 1);
        assert_eq!(count(&workspace, "reference_update_jobs"), 1);
    }
    assert_eq!(
        crate::native_selection::load_value(
            workspace
                .read(TEST_WORKSPACE_SCHEMA_VERSION)
                .unwrap()
                .connection(),
            model,
            &descriptor(original)
        )
        .unwrap()
        .unwrap()
        .vector,
        published
    );
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
}
#[test]
fn oversized_pending_is_rejected_before_payload_materialization_without_published_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let job = staged(&owner, &workspace, &input, false);
    let model = &input.context.search_profiles[0];
    let desc = descriptor(&input.semantic_units[0]);
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        write.connection().execute("UPDATE pending_embedding_values SET vector=zeroblob(8388608) WHERE input_id=(SELECT min(input_id) FROM pending_embedding_values)",[]).unwrap();
        write.commit().unwrap();
    }
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    let error = select_native_values(&write, &job, model, &[desc], true).unwrap_err();
    assert!(error.to_string().contains("length mismatch"), "{error}");
    drop(write);
    assert!(
        freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input
        )
        .err()
        .unwrap()
        .to_string()
        .contains("length mismatch")
    );
    assert_eq!(
        workspace
            .read(TEST_WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap(),
        1
    );
}

#[test]
fn registration_interruption_process_child() {
    let Some(path) = std::env::var_os("ATLAS_NATIVE_REGISTRATION_CHILD") else {
        return;
    };
    let workspace = initialize_workspace(path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let input = failing_input(crate::foundation_tests::fixture(true), 900);
    let _ = register_native_target(
        &owner,
        &workspace,
        TEST_WORKSPACE_SCHEMA_VERSION,
        &FailedRegistration { input: &input },
    );
    panic!("interruption did not exit");
}
#[test]
fn abrupt_process_exit_during_registration_preserves_active_retry_and_retires_candidate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("native.sqlite");
    let input = crate::foundation_tests::fixture(true);
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    {
        let owner = workspace.update_owner().unwrap();
        let _ = staged(&owner, &workspace, &input, false);
    }
    let pending = count(&workspace, "pending_embedding_values");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "native_targets_tests::registration_interruption_process_child",
            "--nocapture",
        ])
        .env("ATLAS_NATIVE_REGISTRATION_CHILD", &path)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(73));
    assert_eq!(count(&workspace, "pending_embedding_values"), pending);
    let owner = workspace.update_owner().unwrap();
    let job = staged(&owner, &workspace, &input, true);
    assert!(
        freeze_native_values(
            &owner,
            &workspace,
            TEST_WORKSPACE_SCHEMA_VERSION,
            &job,
            &input
        )
        .is_ok()
    );
    assert_eq!(count(&workspace, "native_update_targets"), 1);
}

#[test]
fn full_execution_identity_is_rechecked_after_external_model_row_change_between_sessions() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native-identity.sqlite");
    let input = crate::profile_tests::both();
    publish_input(&path, &input).unwrap();
    let workspace = initialize_workspace(&path).unwrap();
    let owner = workspace.update_owner().unwrap();
    let job =
        register_native_target(&owner, &workspace, TEST_WORKSPACE_SCHEMA_VERSION, &input).unwrap();
    let model = &input.context.search_profiles[0];
    let desc = descriptor(
        input
            .semantic_units
            .iter()
            .find(|unit| unit.profile_id == model.profile_id)
            .unwrap(),
    );
    let mut changed = model.clone();
    changed.input_recipe.push_str("/different-execution");
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            select_native_values(&write, &job, &changed, std::slice::from_ref(&desc), true)
                .is_err()
        );
        assert_eq!(
            select_native_values(&write, &job, model, std::slice::from_ref(&desc), true).unwrap(),
            [NativeValueSelection::Reused]
        );
        assert_eq!(
            select_native_values(&write, &job, model, std::slice::from_ref(&desc), false).unwrap(),
            [NativeValueSelection::AlreadySelected]
        );
        write.commit().unwrap();
    }
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(
            write
                .connection()
                .execute(
                    "UPDATE embedding_models SET identity_json=? WHERE identity_json=?",
                    rusqlite::params![
                        serde_json::to_string(&changed).unwrap(),
                        serde_json::to_string(model).unwrap()
                    ]
                )
                .unwrap(),
            1
        );
        write.commit().unwrap();
    }
    {
        let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(
            select_native_values(&write, &job, model, std::slice::from_ref(&desc), true).is_err()
        );
        assert!(
            crate::native_selection::load_value(write.connection(), model, &desc)
                .unwrap()
                .is_none()
        );
        write
            .connection()
            .execute(
                "UPDATE embedding_models SET identity_json=? WHERE identity_json=?",
                rusqlite::params![
                    serde_json::to_string(model).unwrap(),
                    serde_json::to_string(&changed).unwrap()
                ],
            )
            .unwrap();
        write.commit().unwrap();
    }
    let write = workspace.write(TEST_WORKSPACE_SCHEMA_VERSION).unwrap();
    assert_eq!(
        select_native_values(&write, &job, model, std::slice::from_ref(&desc), false).unwrap(),
        [NativeValueSelection::AlreadySelected]
    );
    assert_eq!(
        crate::native_selection::load_value(write.connection(), model, &desc)
            .unwrap()
            .unwrap()
            .vector,
        input.value(model, &desc).unwrap().as_ref()
    );
}
