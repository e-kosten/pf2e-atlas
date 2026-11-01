//! Build orchestration: source admission, preparation, inference, atomic publication.
use crate::source::localization::LocalizationCatalog;
use crate::{
    BuildArtifactOptions, BuildArtifactReport, IngestError, compute_source_fingerprint,
    source_git_commit_if_clean,
};
#[cfg(test)]
use atlas_index::PublicationSource;
use atlas_index::{
    ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, LEXICAL_SELECTION_VERSION,
    LOCALIZATION_POLICY_VERSION, RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION,
    SOURCE_PROJECTION_VERSION, SourceArtifactBuildContext, SourceSearchProfileIdentity,
    SqliteIndexWriter,
};
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentVisibilityRule},
    source_record::SOURCE_CONTENT_SELECTION_VERSION,
};
use std::{collections::BTreeMap, time::Instant};

pub(crate) fn default_audience() -> ContentAudience {
    ContentAudience {
        include_gm: true,
        include_owner: true,
        implicit_check_dc: ContentVisibilityRule::Gm,
    }
}
fn progress(phase: &str, message: &str) {
    tracing::info!(target:"atlas_progress",phase,"{message}");
}
pub(crate) fn build_context(
    options: &BuildArtifactOptions,
    catalog: &LocalizationCatalog,
    fingerprint: &crate::SourceFingerprint,
) -> Result<SourceArtifactBuildContext, IngestError> {
    let mut search_profiles = options
        .search_profiles
        .iter()
        .map(|config| {
            SourceSearchProfileIdentity::from_spec(atlas_embedding::profile_for_model(config.model))
        })
        .collect::<Vec<_>>();
    search_profiles.sort_by(|a, b| a.profile_id.cmp(&b.profile_id));
    if search_profiles
        .windows(2)
        .any(|pair| pair[0].profile_id == pair[1].profile_id)
    {
        return Err(IngestError::DocumentEmbeddingFailed(
            "duplicate search profile".into(),
        ));
    }
    let context = SourceArtifactBuildContext {
        selected_sections_recipe: atlas_index::SELECTED_SECTIONS_RECIPE.into(),
        admission: None,
        core_sections_recipe: atlas_domain::CORE_SECTIONS_RECIPE.into(),
        trait_policy: "trait-definition-centroid/v1".into(),
        used_trait_labels: BTreeMap::new(),
        format_version: ARTIFACT_FORMAT_VERSION,
        snapshot_version: atlas_foundry_model::SNAPSHOT_VERSION,
        source_contract: atlas_foundry_model::SOURCE_CONTRACT_ID.into(),
        source_revision: source_git_commit_if_clean(&options.source_root).ok(),
        source_fingerprint: fingerprint.value.clone(),
        source_file_count: fingerprint.file_count,
        indexing_locale: catalog.locale.clone(),
        locale_catalog_sha256: catalog.locale_sha256.clone(),
        english_catalog_sha256: catalog.english_sha256.clone(),
        localization_policy: LOCALIZATION_POLICY_VERSION.into(),
        audience: default_audience(),
        content_interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
        content_selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
        asset_policy: ASSET_POLICY_VERSION.into(),
        relationship_policy: RELATIONSHIP_POLICY_VERSION.into(),
        query_projection_version: SOURCE_PROJECTION_VERSION.into(),
        query_catalog_version: atlas_index::query::QUERY_CAPABILITY_VERSION,
        lookup_version: SOURCE_LOOKUP_VERSION.into(),
        lexical_selection_version: LEXICAL_SELECTION_VERSION.into(),
        fts_tokenizer: "unicode61 remove_diacritics 2".into(),
        search_profiles,
    };
    Ok(context)
}
pub(crate) fn build_artifact(
    workspace: &atlas_storage::Workspace,
    schema: u32,
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let started = Instant::now();
    if atlas_storage::Workspace::at(&options.output_path)?.path() != workspace.path() {
        return Err(IngestError::ArtifactWriteFailed(
            "build destination differs from the admitted workspace".into(),
        ));
    }
    let update_owner = workspace.update_owner()?;
    atlas_index::recover_reference_publication(&update_owner, schema)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let requested = atlas_index::RequestedReferenceConfig {
        source_root: options.source_root.clone(),
        manifest_path: options.manifest_path.clone(),
        indexing_locale: options.locale.clone(),
        search_profiles: options
            .search_profiles
            .iter()
            .map(|config| atlas_embedding::profile_for_model(config.model).profile_id())
            .collect(),
    };
    let previous = atlas_index::read_requested_reference_config(&workspace.read(schema)?)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    if previous
        .as_ref()
        .is_some_and(|previous| previous != &requested)
    {
        return Err(IngestError::ArtifactWriteFailed("requested source configuration changed before the build started; configure the intended update again before retrying".into()));
    }
    let write = workspace.write(schema)?;
    atlas_index::write_requested_reference_config(&write, &requested)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    write.commit()?;
    let reuse = atlas_index::ReferenceReuseReader::new(workspace, schema).map_err(index_error)?;
    let phase_started = Instant::now();
    let mut prepared = crate::product_preparation::prepare(&options, &reuse)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="source_preparation",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    tracing::info!(target:"atlas_build_diagnostic",reused_sections=prepared.reused_sections,selection_calls=prepared.work.selection_calls,"Prepared shared passage layouts");
    prepared.check_execution_streams()?;
    let target_hash = prepared.spool.context.sha256().map_err(index_error)?;
    let phase_started = Instant::now();
    let plans = crate::native_execution::prepare_profiles(&mut prepared, &options)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="all_profile_prevalidation",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    let record_count = prepared.spool.records.len();
    let pack_count = prepared.spool.packs.len();
    let mut product_record_count = 0usize;
    prepared
        .spool
        .records
        .visit(&mut |frame| {
            if prepared
                .spool
                .packs
                .iter()
                .find(|p| p.pack_id == frame.pack)
                .is_some_and(|p| p.document_kind != "Macro")
            {
                product_record_count += 1;
            }
            Ok(())
        })
        .map_err(index_error)?;
    let semantic_unit_count = prepared.spool.semantic.len();
    let admission = prepared.spool.context.admission.clone().ok_or_else(|| {
        IngestError::ArtifactWriteFailed("publication admission summary is absent".into())
    })?;
    let sealed = prepared.spool.seal().map_err(index_error)?;
    let phase_started = Instant::now();
    let job = atlas_index::register_native_target(&update_owner, workspace, schema, &sealed)
        .map_err(index_error)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="native_target_registration_and_pruning",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    let mut profiles = Vec::new();
    for (plan, config) in plans.iter().zip(&options.search_profiles) {
        profiles.push(
            crate::native_execution::NativeExecution {
                workspace,
                schema,
                job: &job,
                source: &sealed,
                raw_inputs: &prepared.native_inputs,
                options: &options,
                semantic_units: prepared.document_units,
            }
            .execute_profile(plan, config)?,
        );
    }
    let inferred_inputs = profiles.iter().map(|p| p.inferred_inputs).sum();
    let reused_inputs = profiles.iter().map(|p| p.reused_inputs).sum();
    let phase_started = Instant::now();
    let values = atlas_index::freeze_native_values(&update_owner, workspace, schema, &job, &sealed)
        .map_err(index_error)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="native_value_freeze",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    let mut usage = sealed
        .file_usage()
        .map_err(index_error)?
        .combine(prepared.native_inputs.file_usage().map_err(index_error)?)
        .map_err(index_error)?
        .combine(prepared.occurrences.file_usage().map_err(index_error)?)
        .map_err(index_error)?
        .combine(values.temporary_file_usage().map_err(index_error)?)
        .map_err(index_error)?;
    for plan in &plans {
        usage = usage
            .combine(plan.file_usage().map_err(index_error)?)
            .map_err(index_error)?;
    }
    tracing::info!(target:"atlas_build_diagnostic",stage="sealed_metadata_and_frozen_values",file_bytes=usage.file_bytes,allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed live metadata and numerical spool files");
    tracing::info!(target:"atlas_build_diagnostic",logical_native_bytes=values.logical_bytes(),
        temporary_native_bytes=values.temporary_file_bytes().map_err(index_error)?,"Froze selected native values");
    drop(prepared.native_inputs);
    drop(prepared.occurrences);
    let phase_started = Instant::now();
    progress("recheck_source", "Checking source for changes");
    let final_fingerprint =
        compute_source_fingerprint(&options.source_root, options.manifest_path.as_deref())?;
    let final_catalog = LocalizationCatalog::load(
        &options.source_root,
        &options.locale,
        options.manifest_path.as_deref(),
    )?;
    if prepared.fingerprint != final_fingerprint
        || prepared.catalog.locale_sha256 != final_catalog.locale_sha256
        || prepared.catalog.english_sha256 != final_catalog.english_sha256
    {
        return Err(IngestError::SourceUnavailable(
            "source changed during artifact preparation; retry build".into(),
        ));
    }
    tracing::info!(target:"atlas_build_diagnostic",stage="final_source_freshness",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    let phase_started = Instant::now();
    progress("write_artifact", "Saving and checking search index");
    let maintenance = update_owner.maintenance(schema, "publishing reference update")?;
    SqliteIndexWriter::new(&maintenance)
        .write_job(&sealed, &values, &job, &target_hash)
        .map_err(index_error)?;
    drop(maintenance);
    tracing::info!(target:"atlas_build_diagnostic",stage="exclusive_publication",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    drop(values);
    drop(sealed);
    let phase_started = Instant::now();
    let reclaimed = update_owner
        .reclaim_space(schema)
        .map_err(|e| IngestError::SpaceReclamationFailed(e.to_string()))?;
    tracing::info!(target:"atlas_build_diagnostic",stage="physical_reclamation",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,reclaimed,"Completed ingest phase");
    Ok(BuildArtifactReport {
        admission,
        profiles,
        output_path: options.output_path,
        pack_count,
        record_count,
        product_record_count,
        semantic_unit_count,
        inferred_inputs,
        reused_inputs,
        context_shortened_sections: prepared.context_shortened,
        source_fingerprint: prepared.fingerprint.value,
        build_duration_ms: started.elapsed().as_millis(),
        source_report: prepared.source_report,
    })
}

fn index_error(error: atlas_index::IndexError) -> IngestError {
    IngestError::ArtifactWriteFailed(error.to_string())
}

/// Tokenizer-only fixtures publish the real metadata stream and deterministic
/// normalized numerical values. No occurrence aggregate or model inference.
#[cfg(test)]
pub(crate) fn publish_fixture_without_inference(
    options: &BuildArtifactOptions,
) -> Result<(), IngestError> {
    let workspace = atlas_index::test_support::initialize_workspace(&options.output_path)
        .map_err(index_error)?;
    let schema = atlas_index::test_support::TEST_WORKSPACE_SCHEMA_VERSION;
    let owner = workspace.update_owner()?;
    let reuse = atlas_index::ReferenceReuseReader::new(&workspace, schema).map_err(index_error)?;
    let mut prepared = crate::product_preparation::prepare(options, &reuse)?;
    let _plans = crate::native_execution::prepare_profiles(&mut prepared, options)?;
    let target = prepared.spool.context.sha256().map_err(index_error)?;
    let sealed = prepared.spool.seal().map_err(index_error)?;
    let job = atlas_index::register_native_target(&owner, &workspace, schema, &sealed)
        .map_err(index_error)?;
    struct FixtureValues(atlas_index::SourceArtifactBuildContext);
    impl atlas_index::NativeValueSource for FixtureValues {
        fn value<'a>(
            &'a self,
            model: &SourceSearchProfileIdentity,
            _: &atlas_embedding::NativeInputDescriptor,
        ) -> Result<std::borrow::Cow<'a, [f32]>, atlas_index::IndexError> {
            if self.0.search_profile(&model.profile_id)? != model {
                return Err(atlas_index::IndexError::Invalid(
                    "fixture model differs".into(),
                ));
            }
            let mut vector = vec![0.; model.dimensions];
            vector[0] = 1.;
            Ok(std::borrow::Cow::Owned(vector))
        }
    }
    let values = atlas_index::freeze_fixture_native_values(
        &sealed,
        &FixtureValues(sealed.context().clone()),
        Some(&job),
        sealed
            .expected_instance()
            .ok_or_else(|| IngestError::ArtifactWriteFailed("fixture origin absent".into()))?,
    )
    .map_err(index_error)?;
    let maintenance = owner.maintenance(schema, "publishing tokenizer-only fixture")?;
    SqliteIndexWriter::new(&maintenance)
        .write_job(&sealed, &values, &job, &target)
        .map_err(index_error)?;
    Ok(())
}
