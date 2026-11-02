//! Bounded native preparation, reuse and inference over attributed inputs.
use crate::{
    BuildArtifactOptions, IngestError,
    product_preparation::{AttributedNativeInput, RawOccurrence},
};
use atlas_embedding::{
    NativeInferenceRunner, PreparedEmbeddingBatch, PreparedNativeInput, TextEmbeddingTokenizer,
};
use atlas_index::{
    BuildFrameStream, PreparedPublication, SemanticOccurrence, SourceSearchProfileIdentity,
    TraitChunkOccurrence, TraitSeedOccurrence,
};
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
fn counted_progress(model: atlas_embedding::SearchProfileSpec, current: usize, total: usize) {
    tracing::info!(target:"atlas_progress",phase=%format!("{}:reuse_selection",model.id),
        current=current as u64,total=total as u64,unit="passages",profile=model.id,
        "Checking saved {} embeddings",model.label);
}
fn flush_seed(
    publication: &mut PreparedPublication<'_>,
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
        publication.append_trait_seed(value)?;
    }
    Ok(())
}
fn append_occurrence(
    publication: &mut PreparedPublication<'_>,
    model: &SourceSearchProfileIdentity,
    input: &PreparedNativeInput,
    occurrence: RawOccurrence,
    seed: &mut Option<TraitSeedOccurrence>,
) -> Result<(), atlas_index::IndexError> {
    match occurrence {
        RawOccurrence::Document { attribution } => {
            flush_seed(publication, seed)?;
            publication.append_semantic(SemanticOccurrence {
                profile_id: model.profile_id.clone(),
                record: attribution.record,
                owners: attribution.owners,
                field: attribution.field,
                address: attribution.address,
                chunk_ordinal: attribution.chunk_ordinal,
                input: input.descriptor(),
            })?;
        }
        RawOccurrence::Trait { key, address } => {
            if seed.as_ref().is_some_and(|old| old.key != key) {
                flush_seed(publication, seed)?;
            }
            let seed = seed.get_or_insert_with(|| TraitSeedOccurrence {
                key,
                profile_id: model.profile_id.clone(),
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
}
fn stage_values(
    publication: &mut PreparedPublication<'_>,
    model: &SourceSearchProfileIdentity,
    values: &[(&PreparedNativeInput, &[f32])],
) -> Result<(), atlas_embedding::EmbeddingError> {
    let descriptors = values
        .iter()
        .map(|(input, _)| input.descriptor())
        .collect::<Vec<_>>();
    let values = descriptors
        .iter()
        .zip(values)
        .map(|(descriptor, (_, value))| (descriptor, *value))
        .collect::<Vec<_>>();
    publication.stage_inferred(model, &values).map_err(|error| {
        atlas_embedding::EmbeddingError::ModelRunFailed(format!(
            "could not checkpoint inferred batch: {error}"
        ))
    })
}

pub(crate) struct NativeExecution<'a, 'owner> {
    pub publication: &'a mut PreparedPublication<'owner>,
    pub raw_inputs: &'a BuildFrameStream<AttributedNativeInput>,
    pub options: &'a BuildArtifactOptions,
    pub semantic_units: usize,
}
impl NativeExecution<'_, '_> {
    pub(crate) fn execute_profile(
        self,
        config: &atlas_embedding::EmbeddingRuntimeConfig,
    ) -> Result<crate::ProfileBuildReport, IngestError> {
        let mut runner = NativeInferenceRunner::new(config, self.options.embedding_batch_size)
            .map_err(embedding)?;
        self.execute(config, |publication, model, batch| {
            let count = runner
                .infer(batch, |values| stage_values(publication, model, values))
                .map_err(embedding)?;
            Ok((count, runner.peak_output_vectors()))
        })
    }
    fn execute(
        self,
        config: &atlas_embedding::EmbeddingRuntimeConfig,
        mut infer: impl FnMut(
            &mut PreparedPublication<'_>,
            &SourceSearchProfileIdentity,
            &PreparedEmbeddingBatch,
        ) -> Result<(usize, usize), IngestError>,
    ) -> Result<crate::ProfileBuildReport, IngestError> {
        let started = std::time::Instant::now();
        let Self {
            publication,
            raw_inputs,
            options,
            semantic_units,
        } = self;
        let profile = atlas_embedding::profile_for_model(config.model);
        let model = publication
            .context()
            .search_profile(&profile.profile_id())
            .map_err(index)?
            .clone();
        let tokenizer = TextEmbeddingTokenizer::load(config).map_err(embedding)?;
        let mut raw = raw_inputs.cursor().map_err(index)?;
        let mut consumed = 0;
        let mut reused = 0;
        let mut inferred = 0;
        let mut seed = None;
        let mut preparation_elapsed = std::time::Duration::ZERO;
        let mut selection_elapsed = std::time::Duration::ZERO;
        let mut inference_elapsed = std::time::Duration::ZERO;
        let mut max_raw_batch = 0;
        let mut max_unique_batch = 0;
        let mut max_missing_batch = 0;
        let mut max_outputs = 0;
        counted_progress(profile, 0, raw_inputs.len());
        loop {
            let mut inputs = Vec::with_capacity(window());
            let mut occurrences = Vec::with_capacity(window());
            while inputs.len() < window() {
                let Some(frame) = raw.next_frame().map_err(index)? else {
                    break;
                };
                inputs.push(frame.input);
                occurrences.push(frame.occurrence);
            }
            if inputs.is_empty() {
                break;
            }
            max_raw_batch = max_raw_batch.max(inputs.len());
            consumed += inputs.len();
            let preparation_started = std::time::Instant::now();
            let batch = PreparedEmbeddingBatch::prepare(&tokenizer, inputs).map_err(embedding)?;
            preparation_elapsed += preparation_started.elapsed();
            let mut unique = BTreeMap::<(String, usize), usize>::new();
            for (position, (input, occurrence)) in
                batch.inputs().iter().zip(occurrences).enumerate()
            {
                append_occurrence(publication, &model, input, occurrence, &mut seed)
                    .map_err(index)?;
                let key = (input.input_sha256.clone(), input.token_count);
                if let Some(previous) = unique.get(&key) {
                    if batch.inputs()[*previous].input != input.input {
                        return Err(invalid("duplicate raw native inputs disagree"));
                    }
                } else {
                    unique.insert(key, position);
                }
            }
            max_unique_batch = max_unique_batch.max(unique.len());
            let positions = unique.into_values().collect::<Vec<_>>();
            let requested = positions
                .iter()
                .map(|position| batch.inputs()[*position].descriptor())
                .collect::<Vec<_>>();
            let selection_started = std::time::Instant::now();
            let selected = publication
                .select_native(&model, &requested, options.reuse_embeddings)
                .map_err(index)?;
            selection_elapsed += selection_started.elapsed();
            if selected.len() != positions.len() {
                return Err(invalid("native selection coverage differs"));
            }
            let mut missing = Vec::new();
            for (position, status) in positions.into_iter().zip(selected) {
                match status {
                    atlas_index::NativeValueSelection::Missing => missing.push(position),
                    atlas_index::NativeValueSelection::Reused => reused += 1,
                    atlas_index::NativeValueSelection::AlreadySelected => {}
                }
            }
            counted_progress(profile, consumed, raw_inputs.len());
            max_missing_batch = max_missing_batch.max(missing.len());
            if !missing.is_empty() {
                missing.sort_unstable();
                let batch = batch.into_subset(&missing).map_err(embedding)?;
                let inference_started = std::time::Instant::now();
                let (count, outputs) = infer(publication, &model, &batch)?;
                inferred += count;
                max_outputs = max_outputs.max(outputs);
                inference_elapsed += inference_started.elapsed();
            }
        }
        flush_seed(publication, &mut seed).map_err(index)?;
        tracing::info!(target:"atlas_build_diagnostic",profile=%model.profile_id,elapsed_ms=started.elapsed().as_secs_f64()*1000.,native_preparation_ms=preparation_elapsed.as_secs_f64()*1000.,selection_ms=selection_elapsed.as_secs_f64()*1000.,inference_with_checkpoint_ms=inference_elapsed.as_secs_f64()*1000.,max_distinct_native_batch=max_unique_batch,max_raw_native_batch=max_raw_batch,max_missing_native_batch=max_missing_batch,max_native_output_vectors=max_outputs,native_preparation_inputs=consumed,"Completed bounded native execution");
        Ok(crate::ProfileBuildReport {
            profile_id: model.profile_id,
            semantic_units,
            inferred_inputs: inferred,
            reused_inputs: reused,
        })
    }
}

#[cfg(test)]
pub(crate) fn stage_fixture_values(
    publication: &mut PreparedPublication<'_>,
    raw_inputs: &BuildFrameStream<AttributedNativeInput>,
    options: &BuildArtifactOptions,
    semantic_units: usize,
) -> Result<(), IngestError> {
    for config in &options.search_profiles {
        NativeExecution {
            publication,
            raw_inputs,
            options,
            semantic_units,
        }
        .execute(config, |publication, model, batch| {
            let mut vector = vec![0.; model.dimensions];
            vector[0] = 1.;
            let values = batch
                .inputs()
                .iter()
                .map(|input| (input, vector.as_slice()))
                .collect::<Vec<_>>();
            stage_values(publication, model, &values).map_err(embedding)?;
            Ok((values.len(), 1))
        })?;
    }
    Ok(())
}
