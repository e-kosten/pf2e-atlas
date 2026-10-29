//! Build orchestration: source admission, preparation, inference, atomic publication.
use crate::{
    BuildArtifactOptions, BuildArtifactReport, IngestError, SourceEnrichmentContext,
    compute_source_fingerprint, enrich_loaded_source, source_git_commit_if_clean,
};
use crate::{index_build_input::index_build_input, source::localization::LocalizationCatalog};
use atlas_embedding::{
    PreparedEmbeddingBatch, TextEmbeddingTokenizer, generate_prepared_embeddings,
};
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
pub(crate) struct PreparedSourceBuild {
    pub(crate) prepared: crate::index_build_input::PreparedBuildInput,
    pub(crate) source_report: crate::SourceEnrichmentReport,
    pub(crate) fingerprint: crate::SourceFingerprint,
    pub(crate) catalog: LocalizationCatalog,
}
pub(crate) fn prepare_build(
    options: &BuildArtifactOptions,
    reusable: Vec<atlas_index::SourceArtifactRecordInput>,
    layouts: BTreeMap<atlas_domain::RecordKey, Vec<atlas_index::SourcePassageLayout>>,
) -> Result<PreparedSourceBuild, IngestError> {
    crate::validate_source_compatibility(&options.source_root, options.manifest_path.as_deref())?;
    progress("load_source", "Reading Foundry source");
    let capture = crate::source_fingerprint::SourceCapture::capture(
        &options.source_root,
        options.manifest_path.as_deref(),
    )?;
    crate::source_compatibility::validate_manifest(
        &capture.manifest_path,
        &capture.read(&capture.manifest_path)?,
    )?;
    let fingerprint = capture.fingerprint.clone();
    progress("load_catalog", "Loading translations");
    let catalog = LocalizationCatalog::load_captured(
        &options.source_root,
        &options.locale,
        options.manifest_path.as_deref(),
        Some(&capture),
    )?;
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
    let mut context = SourceArtifactBuildContext {
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
    let preparation_hash = context
        .preparation_sha256()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    progress("admit_source", "Parsing source records");
    let loaded = crate::source_loading::load_captured_source(
        &options.source_root,
        Some(&capture.manifest_path),
        reusable,
        Some(&capture),
    )?;
    progress("enrich_source", "Preparing changed records and references");
    let source = enrich_loaded_source(
        loaded,
        SourceEnrichmentContext {
            audience: default_audience(),
            localization: Some(&catalog),
            preparation_context_hash: Some(&preparation_hash),
        },
    );
    let source_report = source.report();
    if source_report.addressed_documents == 0 {
        return Err(IngestError::NoRecordsLoaded);
    }
    progress("trait_labels", "Resolving trait names");
    context.used_trait_labels = crate::source::localization::used_trait_labels(&source, &catalog)?;
    let tokenizer = options
        .search_profiles
        .first()
        .map(|config| {
            progress("load_tokenizer", "Loading search tokenizer");
            TextEmbeddingTokenizer::load_baseline(&config.cache_root)
        })
        .transpose()
        .map_err(embedding_error)?;
    let mut prepared = index_build_input(source, context, tokenizer.as_ref(), &capture, layouts)?;
    let (traits, trait_pending) = crate::traits::prepare_traits(
        &catalog,
        &prepared.index.context,
        tokenizer.as_ref(),
        &prepared.index.records,
    )?;
    prepared.index.traits = traits;
    prepared.trait_pending = trait_pending;
    tracing::info!(
        records = prepared.index.records.len(),
        semantic_units = prepared.pending.len(),
        "Search preparation complete"
    );
    Ok(PreparedSourceBuild {
        prepared,
        source_report,
        fingerprint,
        catalog,
    })
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
    let (reusable, layouts) = {
        let read = workspace.read(schema)?;
        match atlas_index::read_stored_reference_context(read.connection())
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
        {
            Some(context)
                if context.snapshot_version == atlas_foundry_model::SNAPSHOT_VERSION
                    && context.source_contract == atlas_foundry_model::SOURCE_CONTRACT_ID =>
            {
                progress(
                    "read_previous",
                    "Reading indexed reference records for reuse",
                );
                let records = atlas_index::read_persisted_inputs(read.connection(), &context)
                    .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
                let layouts = if options.reuse_embeddings && !options.search_profiles.is_empty() {
                    atlas_index::read_reusable_passage_layouts(&read)
                        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
                } else {
                    BTreeMap::new()
                };
                (records, layouts)
            }
            _ => (Vec::new(), BTreeMap::new()),
        }
    };
    let PreparedSourceBuild {
        mut prepared,
        source_report,
        fingerprint,
        catalog,
    } = prepare_build(&options, reusable, layouts)?;
    let job = {
        let write = workspace.write(schema)?;
        let target = prepared
            .index
            .context
            .sha256()
            .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
        let job = atlas_index::start_reference_job(&write, &target)
            .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
        write.commit()?;
        job
    };
    let mut inferred_inputs = 0;
    let mut reused_inputs = 0;
    let mut profiles = Vec::new();
    for config in &options.search_profiles {
        let profile = atlas_embedding::profile_for_model(config.model);
        progress(
            &format!("{}:load_tokenizer", profile.id),
            &format!("Loading {} tokenizer", profile.label),
        );
        let native = TextEmbeddingTokenizer::load(config).map_err(embedding_error)?;
        let batch = PreparedEmbeddingBatch::prepare(
            &native,
            prepared
                .pending
                .iter()
                .map(|pending| pending.input.clone())
                .chain(
                    prepared
                        .trait_pending
                        .iter()
                        .map(|pending| pending.input.clone()),
                )
                .collect(),
        )
        .map_err(embedding_error)?;
        let inputs = batch.inputs();
        let mut reusable = BTreeMap::new();
        let model = prepared
            .index
            .context
            .search_profile(&profile.profile_id())
            .map_err(|e| IngestError::DocumentEmbeddingFailed(e.to_string()))?;
        if options.reuse_embeddings {
            tracing::info!(profile = profile.id, "Reading saved embeddings");
            reusable =
                atlas_index::read_reusable_embeddings(&workspace.read(schema)?, model, inputs)
                    .map_err(|e| IngestError::DocumentEmbeddingFailed(e.to_string()))?;
        }
        let by_key = inputs
            .iter()
            .map(|input| (input.reuse_key.as_str(), input))
            .collect::<BTreeMap<_, _>>();
        let generated = generate_prepared_embeddings(
            config,
            &batch,
            &reusable,
            options.embedding_batch_size,
            |batch| {
                let persist = || -> Result<(), IngestError> {
                    let write = workspace.write(schema)?;
                    let values = batch
                        .iter()
                        .map(|(key, vector)| {
                            by_key
                                .get(key.as_str())
                                .map(|input| (*input, vector.as_slice()))
                                .ok_or_else(|| {
                                    IngestError::DocumentEmbeddingFailed(
                                        "checkpoint input missing".into(),
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    atlas_index::stage_embedding_values(&write, &job, model, &values)
                        .map_err(|e| IngestError::ArtifactWriteFailed(e.to_string()))?;
                    write.commit()?;
                    Ok(())
                };
                persist().map_err(|e| {
                    atlas_embedding::EmbeddingError::ModelRunFailed(format!(
                        "could not checkpoint inferred batch: {e}"
                    ))
                })
            },
        )
        .map_err(embedding_error)?;
        inferred_inputs += generated.inferred_inputs;
        reused_inputs += generated.reused_inputs;
        profiles.push(crate::ProfileBuildReport {
            profile_id: atlas_embedding::profile_for_model(config.model).profile_id(),
            semantic_units: prepared.pending.len(),
            inferred_inputs: generated.inferred_inputs,
            reused_inputs: generated.reused_inputs,
        });
        let mut trait_chunks: BTreeMap<atlas_domain::TraitKey, Vec<atlas_index::TraitSeedChunk>> =
            BTreeMap::new();
        for ((pending, input), vector) in prepared
            .trait_pending
            .iter()
            .zip(inputs.iter().skip(prepared.pending.len()))
            .zip(generated.vectors.iter().skip(prepared.pending.len()))
        {
            trait_chunks.entry(pending.key.clone()).or_default().push(
                atlas_index::TraitSeedChunk {
                    input: input.input.clone(),
                    input_hash: input.input_sha256.clone(),
                    token_count: input.token_count,
                    address: input.address.clone(),
                    vector: vector.clone(),
                },
            );
        }
        for (key, chunks) in trait_chunks {
            use sha2::{Digest, Sha256};
            let hashes = chunks
                .iter()
                .map(|c| c.input_hash.as_str())
                .collect::<Vec<_>>();
            let input_hash = format!(
                "{:x}",
                Sha256::digest(format!(
                    "{}:{}",
                    crate::traits::TRAIT_SEED_RECIPE,
                    hashes.join(":")
                ))
            );
            let vector = atlas_embedding::normalized_centroid(
                &chunks
                    .iter()
                    .map(|c| c.vector.as_slice())
                    .collect::<Vec<_>>(),
            )
            .map_err(embedding_error)?;
            prepared
                .index
                .trait_vectors
                .push(atlas_index::TraitVectorInput {
                    key,
                    profile_id: profile.profile_id(),
                    input_hash,
                    vector,
                    chunks,
                });
        }
        for ((pending, input), vector) in prepared.pending.iter().zip(inputs).zip(generated.vectors)
        {
            let mut unit = pending.unit.clone();
            unit.profile_id = profile.profile_id();
            unit.input_token_count = input.token_count;
            unit.vector = vector;
            prepared.index.semantic_units.push(unit);
        }
    }
    // Fail before touching the output if source changes while preparation runs.
    progress("recheck_source", "Checking source for changes");
    let final_fingerprint =
        compute_source_fingerprint(&options.source_root, options.manifest_path.as_deref())?;
    let final_catalog = LocalizationCatalog::load(
        &options.source_root,
        &options.locale,
        options.manifest_path.as_deref(),
    )?;
    if fingerprint != final_fingerprint
        || catalog.locale_sha256 != final_catalog.locale_sha256
        || catalog.english_sha256 != final_catalog.english_sha256
    {
        return Err(IngestError::SourceUnavailable(
            "source changed during artifact preparation; retry build".into(),
        ));
    }
    progress("write_artifact", "Saving and checking search index");
    let maintenance = update_owner.maintenance(schema, "publishing reference update")?;
    let target_hash = prepared
        .index
        .context
        .sha256()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    SqliteIndexWriter::new(&maintenance)
        .write_job(&prepared.index, &job, &target_hash)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    drop(maintenance);
    // Publication has committed and validated; physical reclamation can be
    // retried without inference and does not advance reference revision.
    update_owner
        .reclaim_space(schema)
        .map_err(|error| IngestError::SpaceReclamationFailed(error.to_string()))?;
    let report = BuildArtifactReport {
        profiles,
        output_path: options.output_path,
        pack_count: prepared.index.packs.len(),
        record_count: prepared.index.records.len(),
        product_record_count: prepared
            .index
            .records
            .iter()
            .filter(|record| {
                !matches!(
                    record.record.source(),
                    atlas_foundry_model::FoundryDocumentSource::Macro(_)
                )
            })
            .count(),
        semantic_unit_count: prepared.index.semantic_units.len(),
        inferred_inputs,
        reused_inputs,
        context_shortened_sections: prepared.context_shortened,
        source_fingerprint: fingerprint.value,
        build_duration_ms: started.elapsed().as_millis(),
        source_report,
    };
    tracing::info!("Artifact build complete");
    Ok(report)
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
