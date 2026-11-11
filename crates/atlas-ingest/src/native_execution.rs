//! Bounded native preparation, reuse and inference over attributed inputs.
use crate::{BuildArtifactOptions, IngestError, product_preparation::AttributedNativeInput};
use atlas_embedding::{
    NativeInferenceRunner, PreparedEmbeddingBatch, PreparedNativeInput, TextEmbeddingTokenizer,
};
use atlas_index::{
    BuildFrameStream, PreparedPublication, SemanticOccurrence, SourceSearchProfileIdentity,
};
use std::{collections::BTreeMap, path::Path};

pub type EmbeddingSeedLoader<'a> = dyn FnMut(&SourceSearchProfileIdentity, &Path) -> Result<Option<atlas_index::EmbeddingSeed>, String>
    + 'a;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SeedBuildPolicy {
    Strict,
    Refresh,
    Audit,
}

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
        self.execute(config, true, |publication, model, batch| {
            let count = runner
                .infer(&batch, |values| stage_values(publication, model, values))
                .map_err(embedding)?;
            Ok((
                count,
                0,
                runner.peak_output_vectors(),
                std::time::Duration::ZERO,
            ))
        })
    }
    pub(crate) fn execute_seeded(
        self,
        config: &atlas_embedding::EmbeddingRuntimeConfig,
        scratch: &Path,
        load: &mut EmbeddingSeedLoader<'_>,
        policy: SeedBuildPolicy,
    ) -> Result<crate::ProfileBuildReport, IngestError> {
        use crate::EmbeddingCoverageGapKind as Gap;
        let mut seed = None;
        let mut first_gap = None;
        let mut runner = None;
        let batch_size = self.options.embedding_batch_size;
        let report = self.execute(config, true, |publication, model, batch| {
            let seed_started = std::time::Instant::now();
            let seed_error = |reason| IngestError::EmbeddingSeedUnavailable {
                profile: model.profile_id.to_string(),
                reason,
            };
            if seed.is_none() {
                let loaded = load(model, scratch).map_err(seed_error)?;
                seed = Some(match loaded {
                    Some(mut asset) if model.vector_compatible_with(asset.model()) => {
                        asset
                            .bind(model)
                            .map_err(|error| seed_error(error.to_string()))?;
                        Some(asset)
                    }
                    Some(asset)
                        if policy == SeedBuildPolicy::Audit
                            && asset.model().profile_id == model.profile_id =>
                    {
                        first_gap = Some(Gap::IncompatibleModel {
                            previous: Box::new(asset.model().clone()),
                            expected: Box::new(model.clone()),
                        });
                        None
                    }
                    Some(_) => {
                        return Err(seed_error(
                            "model execution identity differs from the requested profile".into(),
                        ));
                    }
                    None if policy == SeedBuildPolicy::Refresh => None,
                    None => return Err(seed_error("seed reader is unavailable".into())),
                });
            }

            let inputs = batch
                .inputs()
                .iter()
                .map(PreparedNativeInput::descriptor)
                .collect::<Vec<_>>();
            let (reused, missing) = if let Some(Some(asset)) = &mut seed {
                if policy == SeedBuildPolicy::Strict {
                    let selection = asset
                        .select(&inputs)
                        .map_err(|error| seed_error(error.to_string()))?;
                    publication.stage_seeded(model, selection).map_err(index)?;
                    return Ok((0, inputs.len(), 0, seed_started.elapsed()));
                }
                let (selection, missing) = asset
                    .partition(&inputs)
                    .map_err(|error| seed_error(error.to_string()))?;
                publication.stage_seeded(model, selection).map_err(index)?;
                (inputs.len() - missing.len(), missing)
            } else {
                (0, (0..inputs.len()).collect())
            };
            let seed_elapsed = seed_started.elapsed();
            if missing.is_empty() {
                return Ok((0, reused, 0, seed_elapsed));
            }
            if policy == SeedBuildPolicy::Audit {
                first_gap.get_or_insert_with(|| Gap::MissingInput {
                    first_hash: inputs[missing[0]].input_sha256.clone(),
                });
                return Ok((0, reused, 0, seed_elapsed));
            }
            let batch = batch.into_subset(&missing).map_err(embedding)?;
            if runner.is_none() {
                runner = Some(NativeInferenceRunner::new(config, batch_size).map_err(embedding)?);
            }
            let runner = runner
                .as_mut()
                .ok_or_else(|| invalid("inference runner unavailable"))?;
            let inferred = runner
                .infer(&batch, |values| stage_values(publication, model, values))
                .map_err(embedding)?;
            Ok((inferred, reused, runner.peak_output_vectors(), seed_elapsed))
        })?;
        if let Some(kind) = first_gap {
            return Err(IngestError::EmbeddingCoverageGap(Box::new(
                crate::EmbeddingCoverageGap {
                    profile: report.profile_id,
                    kind,
                },
            )));
        }
        Ok(report)
    }

    fn execute(
        self,
        config: &atlas_embedding::EmbeddingRuntimeConfig,
        verify_counts: bool,
        mut infer: impl FnMut(
            &mut PreparedPublication<'_>,
            &SourceSearchProfileIdentity,
            PreparedEmbeddingBatch,
        )
            -> Result<(usize, usize, usize, std::time::Duration), IngestError>,
    ) -> Result<crate::ProfileBuildReport, IngestError> {
        let started = std::time::Instant::now();
        let Self {
            publication,
            raw_inputs,
            semantic_units,
            ..
        } = self;
        let profile = atlas_embedding::profile_for_model(config.model);
        let model = publication
            .context()
            .search_profile(&profile.profile_id())
            .map_err(index)?
            .clone();
        let tokenizer = TextEmbeddingTokenizer::load(config).map_err(embedding)?;
        let counts = if verify_counts {
            publication
                .verified_native_token_counts(&model)
                .map_err(index)?
        } else {
            atlas_index::VerifiedNativeTokenCounts::default()
        };
        let mut witnessed_occurrences = 0;
        let mut raw = raw_inputs.cursor().map_err(index)?;
        let mut consumed = 0;
        let mut reused = 0;
        let mut inferred = 0;
        let mut preparation_elapsed = std::time::Duration::ZERO;
        let mut selection_elapsed = std::time::Duration::ZERO;
        let mut inference_elapsed = std::time::Duration::ZERO;
        let mut seed_elapsed = std::time::Duration::ZERO;
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
                occurrences.push(frame.attribution);
            }
            if inputs.is_empty() {
                break;
            }
            max_raw_batch = max_raw_batch.max(inputs.len());
            consumed += inputs.len();
            let preparation_started = std::time::Instant::now();
            let fresh_batch = if counts.is_empty() {
                Some(
                    PreparedEmbeddingBatch::prepare(&tokenizer, std::mem::take(&mut inputs))
                        .map_err(embedding)?,
                )
            } else {
                for input in &mut inputs {
                    if input.input_sha256
                        != atlas_embedding::hash_document_embedding_input(&input.input)
                    {
                        return Err(embedding(
                            atlas_embedding::EmbeddingError::PreparedInputMismatch,
                        ));
                    }
                    input.token_count = if let Some(count) = counts.count_for(&input.input_sha256) {
                        witnessed_occurrences += 1;
                        count
                    } else {
                        tokenizer
                            .validate_document_input(&input.input)
                            .map_err(embedding)?
                    };
                }
                None
            };
            let prepared = fresh_batch
                .as_ref()
                .map_or(inputs.as_slice(), PreparedEmbeddingBatch::inputs);
            preparation_elapsed += preparation_started.elapsed();
            let mut unique = BTreeMap::<(String, usize), usize>::new();
            for (position, (input, occurrence)) in prepared.iter().zip(occurrences).enumerate() {
                publication
                    .append_semantic(SemanticOccurrence {
                        profile_id: model.profile_id.clone(),
                        record: occurrence.record,
                        owners: occurrence.owners,
                        field: occurrence.field,
                        address: occurrence.address,
                        chunk_ordinal: occurrence.chunk_ordinal,
                        input: input.descriptor(),
                    })
                    .map_err(index)?;
                let key = (input.input_sha256.clone(), input.token_count);
                if let Some(previous) = unique.get(&key) {
                    if prepared[*previous].input != input.input {
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
                .map(|position| prepared[*position].descriptor())
                .collect::<Vec<_>>();
            let selection_started = std::time::Instant::now();
            let selected = publication
                .select_native(&model, &requested)
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
            // Report reuse-only windows. A window that needs inference reports
            // its generation progress without briefly replacing it with reuse.
            if missing.is_empty() {
                counted_progress(profile, consumed, raw_inputs.len());
            }
            max_missing_batch = max_missing_batch.max(missing.len());
            if !missing.is_empty() {
                missing.sort_unstable();
                let batch = if let Some(batch) = fresh_batch {
                    batch.into_subset(&missing).map_err(embedding)?
                } else {
                    let mut positions = missing.into_iter().peekable();
                    let inference_inputs = inputs
                        .into_iter()
                        .enumerate()
                        .filter_map(|(position, input)| {
                            if positions.peek() == Some(&position) {
                                positions.next();
                                Some(input)
                            } else {
                                None
                            }
                        })
                        .collect();
                    PreparedEmbeddingBatch::prepare(&tokenizer, inference_inputs)
                        .map_err(embedding)?
                };
                let inference_started = std::time::Instant::now();
                let (count, seed_hits, outputs, seed_time) = infer(publication, &model, batch)?;
                inferred += count;
                reused += seed_hits;
                max_outputs = max_outputs.max(outputs);
                seed_elapsed += seed_time;
                inference_elapsed += inference_started.elapsed().saturating_sub(seed_time);
            }
        }
        if verify_counts {
            publication
                .finish_verified_native_profile(&model)
                .map_err(index)?;
        }
        tracing::info!(target:"atlas_build_diagnostic",profile=%model.profile_id,elapsed_ms=started.elapsed().as_secs_f64()*1000.,native_preparation_ms=preparation_elapsed.as_secs_f64()*1000.,selection_ms=selection_elapsed.as_secs_f64()*1000.,inference_with_checkpoint_ms=inference_elapsed.as_secs_f64()*1000.,seed_load_and_selection_ms=seed_elapsed.as_secs_f64()*1000.,max_distinct_native_batch=max_unique_batch,max_raw_native_batch=max_raw_batch,max_missing_native_batch=max_missing_batch,max_native_output_vectors=max_outputs,native_preparation_inputs=consumed,witnessed_occurrences,"Completed bounded native execution");
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
        .execute(config, false, |publication, model, batch| {
            let mut vector = vec![0.; model.dimensions];
            vector[0] = 1.;
            let values = batch
                .inputs()
                .iter()
                .map(|input| (input, vector.as_slice()))
                .collect::<Vec<_>>();
            stage_values(publication, model, &values).map_err(embedding)?;
            Ok((values.len(), 0, 1, std::time::Duration::ZERO))
        })?;
    }
    Ok(())
}
