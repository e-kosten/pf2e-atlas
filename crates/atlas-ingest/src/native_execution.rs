//! Native validation and execution use bounded windows over immutable task streams.
use crate::{
    BuildArtifactOptions, IngestError,
    product_preparation::{PreparedSourceBuild, RawOccurrence},
};
use atlas_embedding::{
    NativeInferenceRunner, NativeInputDescriptor, PreparedEmbeddingBatch, PreparedNativeInput,
    TextEmbeddingTokenizer,
};
use atlas_index::{
    BuildFrameStream, SealedBuildFrameStream, SemanticOccurrence, SourceSearchProfileIdentity,
    TraitChunkOccurrence, TraitSeedOccurrence,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn index(error: atlas_index::IndexError) -> IngestError {
    IngestError::ArtifactWriteFailed(error.to_string())
}
fn embedding(error: atlas_embedding::EmbeddingError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(error.to_string())
}
fn invalid(message: &str) -> IngestError {
    IngestError::DocumentEmbeddingFailed(message.into())
}
fn window() -> usize {
    atlas_index::NATIVE_VALUE_BATCH_LIMIT.min(atlas_embedding::NATIVE_INFERENCE_BATCH_LIMIT)
}

#[derive(Serialize, Deserialize)]
struct NativeFrame {
    ordinal: usize,
    input: NativeInputDescriptor,
}
pub(crate) struct NativeProfilePlan {
    model: SourceSearchProfileIdentity,
    descriptors: SealedBuildFrameStream<NativeFrame>,
}

fn counted_progress(
    model: atlas_embedding::SearchProfileSpec,
    stage: &str,
    current: usize,
    total: usize,
) {
    let label = model.label;
    let action = match stage {
        "native_validation" => format!("Validating {label} inputs"),
        _ => format!("Checking saved {label} embeddings"),
    };
    tracing::info!(target:"atlas_progress",phase=%format!("{}:{stage}",model.id),
        current=current as u64,total=total as u64,unit="passages",profile=model.id,"{action}");
}

/// Validate every requested native profile before the complete target is registered.
/// Raw text remains in its single existing spool; the new stream contains no text.
pub(crate) fn prepare_profiles(
    prepared: &mut PreparedSourceBuild,
    options: &BuildArtifactOptions,
) -> Result<Vec<NativeProfilePlan>, IngestError> {
    prepared.check_execution_streams()?;
    let mut plans = Vec::new();
    for config in &options.search_profiles {
        let spec = atlas_embedding::profile_for_model(config.model);
        let profile = spec.profile_id();
        let model = prepared
            .spool
            .context
            .search_profile(&profile)
            .map_err(index)?
            .clone();
        let tokenizer = TextEmbeddingTokenizer::load(config).map_err(embedding)?;
        let mut raw = prepared.native_inputs.cursor().map_err(index)?;
        let mut occurrences = prepared.occurrences.cursor().map_err(index)?;
        let mut descriptors = BuildFrameStream::new().map_err(index)?;
        let mut ordinal = 0;
        let mut seed = None;
        let mut max_validated_batch = 0;
        counted_progress(spec, "native_validation", 0, prepared.native_inputs.len());
        loop {
            let mut inputs = Vec::new();
            while inputs.len() < window() {
                match raw.next_frame().map_err(index)? {
                    Some(input) => inputs.push(input),
                    None => break,
                }
            }
            if inputs.is_empty() {
                break;
            }
            max_validated_batch = max_validated_batch.max(inputs.len());
            let batch = PreparedEmbeddingBatch::prepare(&tokenizer, inputs).map_err(embedding)?;
            for input in batch.inputs() {
                let occurrence = occurrences
                    .next_frame()
                    .map_err(index)?
                    .ok_or_else(|| invalid("native occurrence is absent"))?;
                let (RawOccurrence::Document {
                    input: position, ..
                }
                | RawOccurrence::Trait {
                    input: position, ..
                }) = &occurrence;
                if *position != ordinal {
                    return Err(invalid("native occurrence ordinal differs"));
                }
                descriptors
                    .push(&NativeFrame {
                        ordinal,
                        input: input.descriptor(),
                    })
                    .map_err(index)?;
                match occurrence {
                    RawOccurrence::Document { attribution, .. } => {
                        flush_seed(&mut prepared.spool, &mut seed).map_err(index)?;
                        prepared
                            .spool
                            .semantic
                            .push(&SemanticOccurrence {
                                profile_id: profile.clone(),
                                record: attribution.record,
                                owners: attribution.owners,
                                field: attribution.field,
                                address: attribution.address,
                                chunk_ordinal: attribution.chunk_ordinal,
                                input: input.descriptor(),
                            })
                            .map_err(index)?;
                    }
                    RawOccurrence::Trait { key, address, .. } => {
                        if seed
                            .as_ref()
                            .is_some_and(|old: &TraitSeedOccurrence| old.key != key)
                        {
                            flush_seed(&mut prepared.spool, &mut seed).map_err(index)?;
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
                ordinal += 1;
            }
            counted_progress(
                spec,
                "native_validation",
                ordinal,
                prepared.native_inputs.len(),
            );
        }
        if occurrences.next_frame().map_err(index)?.is_some()
            || ordinal != prepared.native_inputs.len()
        {
            return Err(invalid("native stream coverage differs"));
        }
        flush_seed(&mut prepared.spool, &mut seed).map_err(index)?;
        tracing::info!(target:"atlas_build_diagnostic",profile=%profile,max_native_prevalidation_batch=max_validated_batch,native_prevalidation_inputs=ordinal,"Validated native descriptor stream");
        let binding = (
            &prepared.spool.context,
            prepared.spool.expected.workspace_key(),
            prepared.spool.expected.reference_revision(),
            &model,
            ordinal,
        );
        plans.push(NativeProfilePlan {
            model: model.clone(),
            descriptors: descriptors.seal(&binding).map_err(index)?,
        });
    }
    Ok(plans)
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

/// Selection commits before model loading; checkpoints commit before releasing outputs.
pub(crate) struct NativeExecution<'a> {
    pub workspace: &'a atlas_storage::Workspace,
    pub schema: u32,
    pub job: &'a str,
    pub source: &'a dyn atlas_index::PublicationSource,
    pub raw_inputs: &'a SealedBuildFrameStream<PreparedNativeInput>,
    pub options: &'a BuildArtifactOptions,
    pub semantic_units: usize,
}
impl NativeExecution<'_> {
    pub(crate) fn execute_profile(
        &self,
        plan: &NativeProfilePlan,
        config: &atlas_embedding::EmbeddingRuntimeConfig,
    ) -> Result<crate::ProfileBuildReport, IngestError> {
        let Self {
            workspace,
            schema,
            job,
            source,
            raw_inputs,
            options,
            semantic_units,
        } = *self;
        let expected = source
            .expected_instance()
            .ok_or_else(|| invalid("native execution workspace binding absent"))?;
        plan.descriptors
            .check_binding(&(
                source.context(),
                expected.workspace_key(),
                expected.reference_revision(),
                &plan.model,
                raw_inputs.len(),
            ))
            .map_err(index)?;
        let mut raw = raw_inputs.cursor().map_err(index)?;
        let mut descriptors = plan.descriptors.cursor().map_err(index)?;
        let mut runner =
            NativeInferenceRunner::new(config, options.embedding_batch_size).map_err(embedding)?;
        let mut tokenizer = None;
        let mut ordinal = 0;
        let mut reused = 0;
        let mut max_batch = 0;
        let mut max_raw_batch = 0;
        let mut max_missing_batch = 0;
        let profile = atlas_embedding::profile_for_model(config.model);
        counted_progress(profile, "reuse_selection", 0, raw_inputs.len());
        loop {
            // Deduplicate inside this bounded window before selection. Across windows,
            // the active target owns selected membership, including forced execution.
            let mut unique =
                BTreeMap::<(String, usize), (NativeInputDescriptor, PreparedNativeInput)>::new();
            let mut consumed = 0;
            while consumed < window() {
                match (
                    raw.next_frame().map_err(index)?,
                    descriptors.next_frame().map_err(index)?,
                ) {
                    (None, None) => break,
                    (Some(input), Some(frame))
                        if frame.ordinal == ordinal
                            && input.input_sha256 == frame.input.input_sha256 =>
                    {
                        let key = (frame.input.input_sha256.clone(), frame.input.token_count);
                        if let Some((_, previous)) = unique.get(&key) {
                            if previous.input != input.input {
                                return Err(invalid("duplicate raw native inputs disagree"));
                            }
                        } else {
                            unique.insert(key, (frame.input, input));
                        }
                    }
                    _ => return Err(invalid("raw/native descriptor coverage differs")),
                }
                ordinal += 1;
                consumed += 1;
            }
            if consumed == 0 {
                break;
            }
            max_batch = max_batch.max(unique.len());
            max_raw_batch = max_raw_batch.max(consumed);
            let entries = unique.into_values().collect::<Vec<_>>();
            let requested = entries
                .iter()
                .map(|(descriptor, _)| descriptor.clone())
                .collect::<Vec<_>>();
            let selected = {
                let write = workspace.write(schema)?;
                let selected = atlas_index::select_native_values(
                    &write,
                    job,
                    &plan.model,
                    &requested,
                    options.reuse_embeddings,
                )
                .map_err(index)?;
                write.commit()?;
                selected
            };
            counted_progress(profile, "reuse_selection", ordinal, raw_inputs.len());
            if selected.len() != entries.len() {
                return Err(invalid("native selection coverage differs"));
            }
            let mut missing = Vec::new();
            let mut expected = Vec::new();
            for ((descriptor, input), status) in entries.into_iter().zip(selected) {
                match status {
                    atlas_index::NativeValueSelection::Missing => {
                        missing.push(input);
                        expected.push(descriptor);
                    }
                    atlas_index::NativeValueSelection::Reused => reused += 1,
                    atlas_index::NativeValueSelection::AlreadySelected => {}
                }
            }
            max_missing_batch = max_missing_batch.max(missing.len());
            if missing.is_empty() {
                continue;
            }
            if tokenizer.is_none() {
                tokenizer = Some(TextEmbeddingTokenizer::load(config).map_err(embedding)?);
            }
            // Re-run the normal bounded native guard, rather than restoring trusted
            // prepared batches from serialized counts. Cache hits require no runner.
            let batch = PreparedEmbeddingBatch::prepare(
                tokenizer
                    .as_ref()
                    .ok_or_else(|| invalid("native tokenizer absent"))?,
                missing,
            )
            .map_err(embedding)?;
            if batch
                .inputs()
                .iter()
                .map(PreparedNativeInput::descriptor)
                .ne(expected)
            {
                return Err(invalid("revalidated native descriptors differ"));
            }
            runner
                .infer(&batch, |values| {
                    let checkpoint = || -> Result<(), IngestError> {
                        let descriptors = values
                            .iter()
                            .map(|(input, _)| input.descriptor())
                            .collect::<Vec<_>>();
                        let values = descriptors
                            .iter()
                            .zip(values)
                            .map(|(descriptor, (_, value))| (descriptor, *value))
                            .collect::<Vec<_>>();
                        let write = workspace.write(schema)?;
                        atlas_index::stage_native_values(&write, job, &plan.model, &values)
                            .map_err(index)?;
                        write.commit()?;
                        Ok(())
                    };
                    checkpoint().map_err(|e| {
                        atlas_embedding::EmbeddingError::ModelRunFailed(format!(
                            "could not checkpoint inferred batch: {e}"
                        ))
                    })
                })
                .map_err(embedding)?;
        }
        if ordinal != raw_inputs.len() {
            return Err(invalid("native descriptor count differs"));
        }
        tracing::info!(target:"atlas_build_diagnostic",profile=%plan.model.profile_id,max_distinct_native_batch=max_batch,max_raw_native_batch=max_raw_batch,
        max_missing_native_batch=max_missing_batch,max_native_output_vectors=runner.peak_output_vectors(),
        native_prevalidation_inputs=ordinal,native_inference_revalidation_inputs=runner.inferred_inputs(),"Completed bounded native execution");
        Ok(crate::ProfileBuildReport {
            profile_id: plan.model.profile_id.clone(),
            semantic_units,
            inferred_inputs: runner.inferred_inputs(),
            reused_inputs: reused,
        })
    }
}
