//! Build orchestration: source admission, preparation, inference, atomic publication.
use crate::product_preparation::RawOccurrence;
use crate::source::localization::LocalizationCatalog;
use crate::{
    BuildArtifactOptions, BuildArtifactReport, IngestError, compute_source_fingerprint,
    source_git_commit_if_clean,
};
use atlas_embedding::{
    PreparedEmbeddingBatch, TextEmbeddingTokenizer, generate_prepared_embeddings,
};
use atlas_index::{
    ARTIFACT_FORMAT_VERSION, ASSET_POLICY_VERSION, LEXICAL_SELECTION_VERSION,
    LOCALIZATION_POLICY_VERSION, RELATIONSHIP_POLICY_VERSION, SOURCE_LOOKUP_VERSION,
    SOURCE_PROJECTION_VERSION, SourceArtifactBuildContext, SourceSearchProfileIdentity,
    SqliteIndexWriter,
};
use atlas_index::{
    NativeValueSource, SemanticOccurrence, TraitChunkOccurrence, TraitSeedOccurrence,
};
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentVisibilityRule},
    source_record::SOURCE_CONTENT_SELECTION_VERSION,
};
use std::{borrow::Cow, collections::BTreeMap, time::Instant};

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
    let mut prepared = crate::product_preparation::prepare(&options, &reuse)?;
    tracing::info!(target:"atlas_build_diagnostic",reused_sections=prepared.reused_sections,selection_calls=prepared.work.selection_calls,"Prepared shared passage layouts");
    prepared.check_execution_streams()?;
    let target_hash = prepared.spool.context.sha256().map_err(index_error)?;
    let job = {
        let write = workspace.write(schema)?;
        let job = atlas_index::start_reference_job(&write, &target_hash).map_err(index_error)?;
        write.commit()?;
        job
    };
    let mut values = NativeOutputs::default();
    let mut inferred_inputs = 0usize;
    let mut reused_inputs = 0usize;
    let mut profiles = Vec::new();
    for config in &options.search_profiles {
        let profile = atlas_embedding::profile_for_model(config.model);
        let native = TextEmbeddingTokenizer::load(config).map_err(embedding_error)?;
        let mut inputs = Vec::new();
        prepared
            .native_inputs
            .visit(&mut |input| {
                inputs.push(input);
                Ok(())
            })
            .map_err(index_error)?;
        let batch = PreparedEmbeddingBatch::prepare(&native, inputs).map_err(embedding_error)?;
        let inputs = batch.inputs();
        let model = prepared
            .spool
            .context
            .search_profile(&profile.profile_id())
            .map_err(index_error)?
            .clone();
        let reusable = if options.reuse_embeddings {
            atlas_index::read_reusable_embeddings(&workspace.read(schema)?, &model, inputs)
                .map_err(index_error)?
        } else {
            BTreeMap::new()
        };
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
                                        "checkpoint input is absent".into(),
                                    )
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    atlas_index::stage_embedding_values(&write, &job, &model, &values)
                        .map_err(index_error)?;
                    write.commit()?;
                    Ok(())
                };
                persist().map_err(|error| {
                    atlas_embedding::EmbeddingError::ModelRunFailed(format!(
                        "could not checkpoint inferred batch: {error}"
                    ))
                })
            },
        )
        .map_err(embedding_error)?;
        inferred_inputs += generated.inferred_inputs;
        reused_inputs += generated.reused_inputs;
        profiles.push(crate::ProfileBuildReport {
            profile_id: profile.profile_id(),
            semantic_units: prepared.document_units,
            inferred_inputs: generated.inferred_inputs,
            reused_inputs: generated.reused_inputs,
        });
        append_occurrences(
            &mut prepared.spool,
            &prepared.occurrences,
            &profile.profile_id(),
            inputs,
        )
        .map_err(index_error)?;
        values
            .insert(model.clone(), inputs, generated.vectors)
            .map_err(index_error)?;
    }
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
    drop(prepared.native_inputs);
    drop(prepared.occurrences);
    progress("write_artifact", "Saving and checking search index");
    let maintenance = update_owner.maintenance(schema, "publishing reference update")?;
    SqliteIndexWriter::new(&maintenance)
        .write_job(&sealed, &values, &job, &target_hash)
        .map_err(index_error)?;
    drop(maintenance);
    drop(values);
    drop(sealed);
    update_owner
        .reclaim_space(schema)
        .map_err(|e| IngestError::SpaceReclamationFailed(e.to_string()))?;
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

fn append_occurrences(
    spool: &mut atlas_index::PublicationSpool,
    occurrences: &atlas_index::SealedBuildFrameStream<RawOccurrence>,
    profile: &atlas_domain::SearchProfileId,
    inputs: &[atlas_embedding::PreparedNativeInput],
) -> Result<(), atlas_index::IndexError> {
    let mut seed: Option<TraitSeedOccurrence> = None;
    occurrences.visit(&mut |occurrence| {
        match occurrence {
            RawOccurrence::Document { input, attribution } => {
                let input = inputs.get(input).ok_or_else(|| {
                    atlas_index::IndexError::Invalid(
                        "native occurrence input ordinal is absent".into(),
                    )
                })?;
                spool.semantic.push(&SemanticOccurrence {
                    profile_id: profile.clone(),
                    record: attribution.record,
                    owners: attribution.owners,
                    field: attribution.field,
                    address: attribution.address,
                    chunk_ordinal: attribution.chunk_ordinal,
                    input: input.descriptor(),
                })?;
            }
            RawOccurrence::Trait {
                input,
                key,
                address,
            } => {
                let input = inputs.get(input).ok_or_else(|| {
                    atlas_index::IndexError::Invalid("trait native input ordinal is absent".into())
                })?;
                if seed.as_ref().is_some_and(|old| old.key != key) {
                    flush_seed(spool, &mut seed)?;
                }
                let seed = seed.get_or_insert_with(|| TraitSeedOccurrence {
                    key,
                    profile_id: profile.clone(),
                    input_hash: String::new(),
                    chunks: Vec::new(),
                });
                seed.chunks.push(TraitChunkOccurrence {
                    input: input.input.clone(),
                    descriptor: input.descriptor(),
                    address,
                });
            }
        }
        Ok(())
    })?;
    flush_seed(spool, &mut seed)?;
    Ok(())
}

fn flush_seed(
    spool: &mut atlas_index::PublicationSpool,
    seed: &mut Option<TraitSeedOccurrence>,
) -> Result<(), atlas_index::IndexError> {
    if let Some(mut value) = seed.take() {
        use sha2::{Digest, Sha256};
        let hashes = value
            .chunks
            .iter()
            .map(|c| c.descriptor.input_sha256.as_str())
            .collect::<Vec<_>>();
        value.input_hash = format!(
            "{:x}",
            Sha256::digest(format!(
                "{}:{}",
                crate::traits::TRAIT_SEED_RECIPE,
                hashes.join(":")
            ))
        );
        spool.trait_seeds.push(&value)?;
    }
    Ok(())
}

/// Phase 5 deliberately retains selected numerical outputs through publication.
/// Occurrence attribution and prepared rich records are never retained here.
#[derive(Default)]
struct NativeOutputs {
    profiles: BTreeMap<atlas_domain::SearchProfileId, NativeProfile>,
}
struct NativeProfile {
    model: SourceSearchProfileIdentity,
    values: Vec<Vec<f32>>,
    lookup: BTreeMap<(String, usize), usize>,
}
impl NativeOutputs {
    fn insert(
        &mut self,
        model: SourceSearchProfileIdentity,
        inputs: &[atlas_embedding::PreparedNativeInput],
        values: Vec<Vec<f32>>,
    ) -> Result<(), atlas_index::IndexError> {
        if inputs.len() != values.len() {
            return Err(atlas_index::IndexError::Invalid(
                "native output coverage differs".into(),
            ));
        }
        let mut lookup = BTreeMap::new();
        for (ordinal, input) in inputs.iter().enumerate() {
            if let Some(previous) =
                lookup.insert((input.input_sha256.clone(), input.token_count), ordinal)
            {
                let before: &Vec<f32> = &values[previous];
                let after = &values[ordinal];
                if before.len() != after.len()
                    || before
                        .iter()
                        .zip(after)
                        .any(|(a, b)| a.to_bits() != b.to_bits())
                {
                    return Err(atlas_index::IndexError::Invalid(
                        "duplicate selected native values disagree".into(),
                    ));
                }
            }
        }
        self.profiles.insert(
            model.profile_id.clone(),
            NativeProfile {
                model,
                values,
                lookup,
            },
        );
        Ok(())
    }
}
impl NativeValueSource for NativeOutputs {
    fn value<'a>(
        &'a self,
        model: &SourceSearchProfileIdentity,
        input: &atlas_embedding::NativeInputDescriptor,
    ) -> Result<Cow<'a, [f32]>, atlas_index::IndexError> {
        let profile = self.profiles.get(&model.profile_id).ok_or_else(|| {
            atlas_index::IndexError::Invalid("selected native profile is absent".into())
        })?;
        if &profile.model != model {
            return Err(atlas_index::IndexError::Invalid(
                "selected native execution identity differs".into(),
            ));
        }
        let index = profile
            .lookup
            .get(&(input.input_sha256.clone(), input.token_count))
            .ok_or_else(|| {
                atlas_index::IndexError::Invalid(
                    "selected native input or token count differs".into(),
                )
            })?;
        Ok(Cow::Borrowed(&profile.values[*index]))
    }
}
fn index_error(error: atlas_index::IndexError) -> IngestError {
    IngestError::ArtifactWriteFailed(error.to_string())
}
fn embedding_error(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
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
    let mut values = NativeOutputs::default();
    for config in &options.search_profiles {
        let profile = atlas_embedding::profile_for_model(config.model).profile_id();
        let tokenizer = TextEmbeddingTokenizer::load(config).map_err(embedding_error)?;
        let mut raw = Vec::new();
        prepared
            .native_inputs
            .visit(&mut |input| {
                raw.push(input);
                Ok(())
            })
            .map_err(index_error)?;
        let batch = PreparedEmbeddingBatch::prepare(&tokenizer, raw).map_err(embedding_error)?;
        let model = prepared
            .spool
            .context
            .search_profile(&profile)
            .map_err(index_error)?
            .clone();
        let vectors = batch
            .inputs()
            .iter()
            .map(|_| {
                let mut v = vec![0.; model.dimensions];
                v[0] = 1.;
                v
            })
            .collect();
        append_occurrences(
            &mut prepared.spool,
            &prepared.occurrences,
            &profile,
            batch.inputs(),
        )
        .map_err(index_error)?;
        values
            .insert(model, batch.inputs(), vectors)
            .map_err(index_error)?;
    }
    let target = prepared.spool.context.sha256().map_err(index_error)?;
    let write = workspace.write(schema)?;
    let job = atlas_index::start_reference_job(&write, &target).map_err(index_error)?;
    write.commit()?;
    let sealed = prepared.spool.seal().map_err(index_error)?;
    let maintenance = owner.maintenance(schema, "publishing tokenizer-only fixture")?;
    SqliteIndexWriter::new(&maintenance)
        .write_job(&sealed, &values, &job, &target)
        .map_err(index_error)?;
    Ok(())
}
