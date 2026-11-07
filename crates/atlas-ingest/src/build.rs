//! Build orchestration: source admission, preparation, inference, atomic publication.
use crate::source::localization::LocalizationCatalog;
use crate::{
    BuildArtifactOptions, BuildArtifactReport, IngestError, SourceFingerprint,
    source_fingerprint::SourceCapture, source_git_commit_if_clean,
};
use atlas_index::{
    ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, LEXICAL_SELECTION_VERSION,
    LOCALIZATION_POLICY_VERSION, RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION,
    SOURCE_PROJECTION_VERSION, SourceArtifactBuildContext, SourceSearchProfileIdentity,
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
    inputs: &CapturedBuildInputs,
    catalog: &LocalizationCatalog,
) -> Result<SourceArtifactBuildContext, IngestError> {
    let fingerprint = inputs.fingerprint();
    let mut search_profiles = inputs
        .options
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
    let mut reference_keys = atlas_foundry_model::generated::trait_definitions()
        .iter()
        .map(|entry| {
            atlas_domain::TraitKey::new(entry.identifier)
                .map(atlas_domain::DefinitionKey::from)
                .map_err(|error| IngestError::SourceSelectionFailed(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for entry in atlas_foundry_model::generated::reference_definitions()
        .iter()
        .filter(|entry| !entry.excluded)
    {
        reference_keys.push(
            format!("{}:{}", entry.namespace, entry.identifier)
                .parse::<atlas_domain::DefinitionKey>()
                .map_err(|error| IngestError::SourceSelectionFailed(error.to_string()))?,
        );
    }
    let (expected_entry_count, entry_keys_sha256) =
        atlas_index::reference_inventory_identity(reference_keys).map_err(index_error)?;
    let context = SourceArtifactBuildContext {
        selected_sections_recipe: atlas_index::SELECTED_SECTIONS_RECIPE.into(),
        admission: None,
        reference_preparation: Some(atlas_index::ReferencePreparation {
            recipe: crate::REFERENCE_PREPARATION_RECIPE.into(),
            input_hash: inputs.reference_inputs_identity().into(),
            coverage: atlas_index::ReferenceCoverage::Complete,
            expected_entry_count,
            entry_keys_sha256,
            source_package: inputs
                .capture
                .manifest()
                .manifest
                .metadata
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            pack_systems: inputs.reference_pack_systems(),
        }),
        core_sections_recipe: atlas_domain::CORE_SECTIONS_RECIPE.into(),
        trait_policy: "trait-definition-centroid/v1".into(),
        used_trait_labels: BTreeMap::new(),
        format_version: ARTIFACT_FORMAT_VERSION,
        snapshot_version: atlas_foundry_model::SNAPSHOT_VERSION,
        source_contract: atlas_foundry_model::SOURCE_CONTRACT_ID.into(),
        source_revision: inputs.source_revision.clone(),
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
        remaster_inputs_identity: atlas_record::remaster::remaster_inputs_identity(),
        query_projection_version: SOURCE_PROJECTION_VERSION.into(),
        query_catalog_version: atlas_index::query::QUERY_CAPABILITY_VERSION,
        lookup_version: SOURCE_LOOKUP_VERSION.into(),
        lexical_selection_version: LEXICAL_SELECTION_VERSION.into(),
        fts_tokenizer: "unicode61 remove_diacritics 2".into(),
        search_profiles,
    };
    Ok(context)
}
pub struct CapturedBuildInputs {
    options: BuildArtifactOptions,
    capture: SourceCapture,
    source_revision: Option<String>,
    reference_supplement: crate::reference_supplement::ReferenceSupplementCapture,
    reference_inputs_identity: String,
    reference_notices: crate::reference_notices::CapturedReferenceNotices,
}
fn captured_pack_systems(capture: &SourceCapture) -> BTreeMap<String, Option<String>> {
    let mut systems = BTreeMap::new();
    for pack in &capture.manifest().manifest.packs {
        if let Some(value) = pack.metadata.get("system") {
            let declared = value.as_str().map(str::to_owned);
            systems
                .entry(pack.name.clone())
                .and_modify(|previous| {
                    if *previous != declared {
                        *previous = None;
                    }
                })
                .or_insert(declared);
        }
    }
    systems
}
impl CapturedBuildInputs {
    pub fn capture(options: BuildArtifactOptions) -> Result<Self, IngestError> {
        let started = Instant::now();
        let source_revision = source_git_commit_if_clean(&options.source_root).ok();
        let capture =
            SourceCapture::capture(&options.source_root, options.manifest_path.as_deref())?;
        let source_revision = source_revision.filter(|revision| {
            source_git_commit_if_clean(&options.source_root).is_ok_and(|after| after == *revision)
        });
        crate::source_compatibility::validate_parsed_manifest(
            &capture.manifest_path,
            &capture.manifest().manifest,
        )?;
        let package = capture
            .manifest()
            .manifest
            .metadata
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let version = capture
            .manifest()
            .manifest
            .metadata
            .get("version")
            .and_then(serde_json::Value::as_str);
        let reference_supplement =
            crate::reference_supplement::ReferenceSupplementCapture::capture(
                package,
                version,
                &options.locale,
            )?;
        let reference_notices = crate::reference_notices::CapturedReferenceNotices::capture(
            &options.source_root,
            package,
            source_revision.as_deref(),
        )?;
        let reference_inputs_identity = {
            use sha2::{Digest, Sha256};
            let mut hash = Sha256::new();
            let pack_systems = serde_json::to_string(&captured_pack_systems(&capture))
                .map_err(|error| IngestError::SourceSelectionFailed(error.to_string()))?;
            for value in [
                crate::REFERENCE_PREPARATION_RECIPE,
                atlas_foundry_model::generated::REFERENCE_CATALOG_SHA256,
                reference_supplement.identity.as_str(),
                reference_notices.identity.as_str(),
                options.locale.as_str(),
                package,
                pack_systems.as_str(),
            ] {
                hash.update((value.len() as u64).to_be_bytes());
                hash.update(value.as_bytes());
            }
            format!("{:x}", hash.finalize())
        };
        tracing::info!(target:"atlas_build_diagnostic",stage="source_capture",captured_files=capture.fingerprint.file_count,elapsed_ms=started.elapsed().as_secs_f64()*1000.,"Captured source inventory and manifest");
        Ok(Self {
            options,
            capture,
            source_revision,
            reference_supplement,
            reference_inputs_identity,
            reference_notices,
        })
    }
    pub(crate) fn reference_pack_systems(&self) -> BTreeMap<String, Option<String>> {
        captured_pack_systems(&self.capture)
    }
    pub(crate) fn source_revision(&self) -> Option<&str> {
        self.source_revision.as_deref()
    }
    pub(crate) fn reference_notices(&self) -> &crate::reference_notices::CapturedReferenceNotices {
        &self.reference_notices
    }
    pub fn reference_inputs_identity(&self) -> &str {
        &self.reference_inputs_identity
    }
    pub(crate) fn reference_supplement(
        &self,
    ) -> &crate::reference_supplement::ReferenceSupplementCapture {
        &self.reference_supplement
    }
    pub fn fingerprint(&self) -> &SourceFingerprint {
        &self.capture.fingerprint
    }
    pub fn build(
        self,
        workspace: &atlas_storage::Workspace,
        schema: u32,
    ) -> Result<BuildArtifactReport, IngestError> {
        let started = Instant::now();
        let (owner, scratch) = begin_build(workspace, schema, &self.options)?;
        finish_build(workspace, schema, owner, scratch, self, started)
    }
    pub(crate) fn options(&self) -> &BuildArtifactOptions {
        &self.options
    }
    pub(crate) fn source(&self) -> &SourceCapture {
        &self.capture
    }
}

pub(crate) fn build_artifact(
    workspace: &atlas_storage::Workspace,
    schema: u32,
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let started = Instant::now();
    let (owner, scratch) = begin_build(workspace, schema, &options)?;
    let inputs = CapturedBuildInputs::capture(options)?;
    finish_build(workspace, schema, owner, scratch, inputs, started)
}

fn begin_build(
    workspace: &atlas_storage::Workspace,
    schema: u32,
    options: &BuildArtifactOptions,
) -> Result<(atlas_storage::UpdateOwner, atlas_index::BuildScratch), IngestError> {
    if atlas_storage::Workspace::at(&options.output_path)?.path() != workspace.path() {
        return Err(IngestError::ArtifactWriteFailed(
            "build destination differs from the admitted workspace".into(),
        ));
    }
    let update_owner = workspace.update_owner()?;
    let scratch = atlas_index::BuildScratch::new(&update_owner).map_err(index_error)?;
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
    Ok((update_owner, scratch))
}

fn finish_build(
    workspace: &atlas_storage::Workspace,
    schema: u32,
    update_owner: atlas_storage::UpdateOwner,
    scratch: atlas_index::BuildScratch,
    inputs: CapturedBuildInputs,
    started: Instant,
) -> Result<BuildArtifactReport, IngestError> {
    let reuse = atlas_index::ReferenceReuseReader::new(workspace, schema).map_err(index_error)?;
    let phase_started = Instant::now();
    let prepared = crate::product_preparation::prepare(&inputs, &reuse, scratch.path())?;
    let CapturedBuildInputs {
        options, capture, ..
    } = inputs;
    drop(capture);
    tracing::info!(target:"atlas_build_diagnostic",stage="source_preparation",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
    tracing::info!(target:"atlas_build_diagnostic",reused_sections=prepared.reused_sections,selection_calls=prepared.work.selection_calls,"Prepared shared passage layouts");
    let record_count = prepared.spool.record_count();
    let pack_count = prepared.spool.pack_count();
    let admission = prepared.spool.context().admission.clone().ok_or_else(|| {
        IngestError::ArtifactWriteFailed("publication admission summary is absent".into())
    })?;
    let mut publication = atlas_index::PreparedPublication::new_in(
        &update_owner,
        schema,
        scratch.path(),
        prepared.spool,
    )
    .map_err(index_error)?;
    let product_record_count = publication.product_record_count();
    let mut profiles = Vec::new();
    for config in &options.search_profiles {
        profiles.push(
            crate::native_execution::NativeExecution {
                publication: &mut publication,
                raw_inputs: &prepared.native_inputs,
                options: &options,
                semantic_units: prepared.document_units,
            }
            .execute_profile(config)?,
        );
    }
    let inferred_inputs = profiles.iter().map(|p| p.inferred_inputs).sum();
    let reused_inputs = profiles.iter().map(|p| p.reused_inputs).sum();
    let semantic_unit_count = publication.semantic_unit_count();
    let usage = publication
        .file_usage()
        .map_err(index_error)?
        .combine(prepared.native_inputs.file_usage().map_err(index_error)?)
        .map_err(index_error)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="owned_metadata_and_staged_values",file_bytes=usage.file_bytes,allocated_bytes=usage.allocated_bytes.unwrap_or(0),allocated_bytes_known=usage.allocated_bytes.is_some(),"Observed live metadata and numerical spool files");
    tracing::info!(target:"atlas_build_diagnostic",logical_native_bytes=publication.logical_native_bytes().map_err(index_error)?,
        temporary_native_bytes=publication.temporary_native_bytes().map_err(index_error)?,"Observed staged native values");
    drop(prepared.native_inputs);
    let phase_started = Instant::now();
    progress("write_artifact", "Saving and checking search index");
    publication.publish().map_err(index_error)?;
    tracing::info!(target:"atlas_build_diagnostic",stage="exclusive_publication",elapsed_ms=phase_started.elapsed().as_secs_f64()*1000.,"Completed ingest phase");
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
    let scratch = atlas_index::BuildScratch::new(&owner).map_err(index_error)?;
    let inputs = CapturedBuildInputs::capture(options.clone())?;
    let prepared = crate::product_preparation::prepare(&inputs, &reuse, scratch.path())?;
    let mut publication =
        atlas_index::PreparedPublication::new_in(&owner, schema, scratch.path(), prepared.spool)
            .map_err(index_error)?;
    crate::native_execution::stage_fixture_values(
        &mut publication,
        &prepared.native_inputs,
        options,
        prepared.document_units,
    )?;
    publication.publish().map_err(index_error)?;
    Ok(())
}
