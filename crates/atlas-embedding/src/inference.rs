use crate::model_cache::read_verified_asset;
use crate::{
    EmbeddingError, EmbeddingModelSpec, EmbeddingRuntimeConfig, PoolingStrategy,
    PreparedEmbeddingBatch, TextEmbeddingTokenizer,
};
use fastembed::{
    InitOptionsUserDefined, OutputKey, Pooling, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use std::path::Path;

pub struct TextEmbedder {
    spec: EmbeddingModelSpec,
    tokenizer: TextEmbeddingTokenizer,
    library: TextEmbedding,
}
impl TextEmbedder {
    pub fn load(config: &EmbeddingRuntimeConfig) -> Result<Self, EmbeddingError> {
        Self::load_from_model_dir(config.model_spec(), config.model_dir())
    }
    pub fn load_from_model_dir(
        spec: EmbeddingModelSpec,
        directory: impl AsRef<Path>,
    ) -> Result<Self, EmbeddingError> {
        crate::catalog::validate_model_spec(spec)?;
        let directory = directory.as_ref();
        let onnx_file = read_verified_asset(spec, directory, "onnx/model.onnx")?;
        let tokenizer_file = read_verified_asset(spec, directory, "tokenizer.json")?;
        let config_file = read_verified_asset(spec, directory, "config.json")?;
        let tokenizer_config_file = read_verified_asset(spec, directory, "tokenizer_config.json")?;
        let special_tokens_map_file =
            read_verified_asset(spec, directory, "special_tokens_map.json")?;
        let native_tokenizer = tokenizers::Tokenizer::from_bytes(&tokenizer_file)
            .map_err(|e| load_error(directory, e))?;
        let mut model = UserDefinedEmbeddingModel::new(
            onnx_file,
            TokenizerFiles {
                tokenizer_file,
                config_file,
                special_tokens_map_file,
                tokenizer_config_file,
            },
        )
        .with_pooling(match spec.pooling {
            PoolingStrategy::Cls => Pooling::Cls,
            // FastEmbed preserves a rank-2 export directly; it does not pool
            // the projected sentence output over a token axis.
            PoolingStrategy::ExportedSentenceEmbedding => Pooling::Mean,
        });
        model.output_key = Some(OutputKey::ByName(spec.output_name));
        for &(path, _) in crate::model_cache::model_assets(spec)? {
            if let Some(file_name) = path
                .strip_prefix("onnx/")
                .filter(|name| *name != "model.onnx")
            {
                model = model.with_external_initializer(
                    file_name.into(),
                    read_verified_asset(spec, directory, path)?,
                );
            }
        }
        let mut library = TextEmbedding::try_new_from_user_defined(
            model,
            InitOptionsUserDefined::new()
                .with_max_length(
                    spec.max_input_tokens
                        .ok_or(EmbeddingError::UnsupportedModelContract)?,
                )
                .with_intra_threads(4),
        )
        .map_err(|e| load_error(directory, e))?;
        // Atlas has already checked every actual input with the exact tokenizer.
        // Disable truncation even inside the library so oversized input cannot
        // become an apparently successful clipped vector.
        library
            .tokenizer
            .with_truncation(None)
            .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))?;
        let tokenizer = TextEmbeddingTokenizer::from_tokenizer(spec, native_tokenizer)?;
        Ok(Self {
            spec,
            tokenizer,
            library,
        })
    }
    pub fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }
    pub fn tokenizer(&self) -> &TextEmbeddingTokenizer {
        &self.tokenizer
    }
    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut vectors = self.embed_queries(&[text])?;
        vectors
            .pop()
            .ok_or(EmbeddingError::UnexpectedEmbeddingOutputCount {
                expected: 1,
                actual: 0,
            })
    }
    /// Embed raw queries. The pinned query prefix is applied exactly once here.
    pub fn embed_queries(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        for text in texts {
            self.tokenizer.validate_query_input(text)?;
        }
        let inputs = texts
            .iter()
            .map(|text| self.spec.query_input(text))
            .collect::<Vec<_>>();
        let inputs = inputs.iter().map(String::as_str).collect::<Vec<_>>();
        self.infer(&inputs, 16)
    }
    pub fn embed_document(&mut self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut vectors = self.embed_documents(&[text])?;
        vectors
            .pop()
            .ok_or(EmbeddingError::UnexpectedEmbeddingOutputCount {
                expected: 1,
                actual: 0,
            })
    }
    /// Embed raw selected documents. Callers must not supply an official prefix.
    pub fn embed_documents(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        for text in texts {
            self.tokenizer.validate_document_input(text)?;
        }
        let inputs = texts
            .iter()
            .map(|text| self.spec.document_input(text))
            .collect::<Vec<_>>();
        let inputs = inputs.iter().map(String::as_str).collect::<Vec<_>>();
        self.infer(&inputs, 16)
    }
    fn infer(
        &mut self,
        texts: &[&str],
        batch_size: usize,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        // FastEmbed configures padding and special-token declarations at load.
        // Check its actual input IDs against the pinned native tokenizer rather
        // than assuming this configuration preserves the research contract.
        for text in texts {
            let native = self
                .tokenizer
                .tokenizer()
                .encode(*text, true)
                .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))?;
            let configured = self
                .library
                .tokenizer
                .encode(*text, true)
                .map_err(|e| EmbeddingError::TokenizationFailed(e.to_string()))?;
            if native.get_ids() != configured.get_ids()
                || native.get_attention_mask() != configured.get_attention_mask()
                || native.get_type_ids() != configured.get_type_ids()
            {
                return Err(EmbeddingError::TokenizationFailed(
                    "native and inference tokenizer inputs differ".into(),
                ));
            }
        }
        let vectors = self
            .library
            .embed(texts, Some(batch_size))
            .map_err(|e| EmbeddingError::ModelRunFailed(e.to_string()))?;
        if vectors.len() != texts.len() {
            return Err(EmbeddingError::UnexpectedEmbeddingOutputCount {
                expected: texts.len(),
                actual: vectors.len(),
            });
        }
        for vector in &vectors {
            validate_embedding_vector(vector, self.spec.dimensions)?;
        }
        Ok(vectors)
    }
}
fn load_error(directory: &Path, error: impl std::fmt::Display) -> EmbeddingError {
    EmbeddingError::ModelLoadFailed {
        path: directory.display().to_string(),
        message: error.to_string(),
    }
}

pub fn validate_embedding_vector(vector: &[f32], dimensions: usize) -> Result<(), EmbeddingError> {
    if vector.len() != dimensions {
        return Err(EmbeddingError::DimensionMismatch {
            expected: dimensions,
            actual: vector.len(),
        });
    }
    let norm = vector
        .iter()
        .map(|v| f64::from(*v).powi(2))
        .sum::<f64>()
        .sqrt();
    if vector.iter().any(|v| !v.is_finite()) || (norm - 1.0).abs() > 1e-4 {
        return Err(EmbeddingError::InvalidVector);
    }
    Ok(())
}

/// Maximum resident numerical input/output window, independent of corpus size.
pub const NATIVE_INFERENCE_BATCH_LIMIT: usize = 128;

struct NativeInferenceState<M> {
    config: EmbeddingRuntimeConfig,
    batch_size: usize,
    model: Option<M>,
    inferred_inputs: usize,
    peak_vectors: usize,
}
impl<M> NativeInferenceState<M> {
    fn infer(
        &mut self,
        batch: &PreparedEmbeddingBatch,
        load: impl FnOnce(&EmbeddingRuntimeConfig) -> Result<M, EmbeddingError>,
        mut infer: impl FnMut(&mut M, &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>,
        mut checkpoint: impl FnMut(
            &[(&crate::PreparedNativeInput, &[f32])],
        ) -> Result<(), EmbeddingError>,
    ) -> Result<usize, EmbeddingError> {
        if batch.spec() != self.config.model_spec()
            || batch.inputs().len() > NATIVE_INFERENCE_BATCH_LIMIT
        {
            return Err(EmbeddingError::PreparedInputMismatch);
        }
        if batch.inputs().is_empty() {
            return Ok(0);
        }
        generation_progress(batch.spec(), self.inferred_inputs);
        if self.model.is_none() {
            self.model = Some(load(&self.config)?);
        }
        let mut completed = 0;
        for inputs in batch.inputs().chunks(self.batch_size) {
            let texts = inputs
                .iter()
                .map(|input| input.input.as_str())
                .collect::<Vec<_>>();
            let vectors = infer(
                self.model
                    .as_mut()
                    .ok_or(EmbeddingError::PreparedInputMismatch)?,
                &texts,
            )?;
            if vectors.len() != inputs.len() {
                return Err(EmbeddingError::UnexpectedEmbeddingOutputCount {
                    expected: inputs.len(),
                    actual: vectors.len(),
                });
            }
            self.peak_vectors = self.peak_vectors.max(vectors.len());
            for vector in &vectors {
                validate_embedding_vector(vector, batch.spec().dimensions)?;
            }
            let values = inputs
                .iter()
                .zip(&vectors)
                .map(|(input, vector)| (input, vector.as_slice()))
                .collect::<Vec<_>>();
            checkpoint(&values)?;
            completed += inputs.len();
            self.inferred_inputs += inputs.len();
            generation_progress(batch.spec(), self.inferred_inputs);
        }
        Ok(completed)
    }
}
fn generation_progress(spec: EmbeddingModelSpec, durable_inputs: usize) {
    let profile = crate::embedding_model_for_model_id(spec.model_id).map(crate::profile_for_model);
    let label = profile.map_or(spec.model_id, |profile| profile.label);
    let id = profile.map_or(spec.model_id, |profile| profile.id);
    // Distinct future work is unknown while streaming selection. No total is
    // supplied: the existing CLI renders an indeterminate stage, without ETA.
    tracing::info!(target:"atlas_progress",phase=%format!("{id}:inference"),
        current=durable_inputs as u64,unit="unique passages",profile=id,
        "Generating {label} embeddings");
}
/// One model owner per profile. Construction and empty batches never load weights.
/// Outputs live only until the synchronous durable checkpoint returns.
pub struct NativeInferenceRunner {
    state: NativeInferenceState<TextEmbedder>,
}
impl NativeInferenceRunner {
    pub fn new(config: &EmbeddingRuntimeConfig, batch_size: usize) -> Result<Self, EmbeddingError> {
        if batch_size == 0 {
            return Err(EmbeddingError::ModelRunFailed(
                "batch size must be positive".into(),
            ));
        }
        Ok(Self {
            state: NativeInferenceState {
                config: config.clone(),
                batch_size: batch_size.min(NATIVE_INFERENCE_BATCH_LIMIT),
                model: None,
                inferred_inputs: 0,
                peak_vectors: 0,
            },
        })
    }
    pub fn inferred_inputs(&self) -> usize {
        self.state.inferred_inputs
    }
    pub fn peak_output_vectors(&self) -> usize {
        self.state.peak_vectors
    }
    pub fn is_loaded(&self) -> bool {
        self.state.model.is_some()
    }
    pub fn infer(
        &mut self,
        batch: &PreparedEmbeddingBatch,
        checkpoint: impl FnMut(&[(&crate::PreparedNativeInput, &[f32])]) -> Result<(), EmbeddingError>,
    ) -> Result<usize, EmbeddingError> {
        self.state.infer(batch, |config| {
            let profile = crate::profile_for_model(config.model);
            tracing::info!(target:"atlas_progress",phase="load_embeddings",model=config.model_spec().model_id,
                "Loading {} model", profile.label);
            TextEmbedder::load(config)
        }, TextEmbedder::embed_documents, checkpoint)
    }
}

#[cfg(test)]
mod bounded_runner_tests {
    use super::*;
    #[test]
    fn lazy_batches_load_once_and_count_only_durable_checkpoints() {
        let (_, events) = crate::tests::observed_progress(|| {
            let tokenizer = crate::tests::tokenizer();
            let input =
                crate::prepare_embedding_section(&tokenizer, crate::tests::plain("one", "one"))
                    .unwrap()
                    .remove(0);
            let batch = PreparedEmbeddingBatch::prepare(&tokenizer, vec![input.into(); 5]).unwrap();
            let config =
                EmbeddingRuntimeConfig::new(crate::EmbeddingModelId::BgeBaseEnV15, "/unused");
            let mut state = NativeInferenceState {
                config,
                batch_size: 2,
                model: None,
                inferred_inputs: 0,
                peak_vectors: 0,
            };
            let loads = std::cell::Cell::new(0);
            let calls = std::cell::RefCell::new(Vec::new());
            let load = |_: &EmbeddingRuntimeConfig| {
                loads.set(loads.get() + 1);
                Ok(())
            };
            let infer = |_: &mut (), texts: &[&str]| {
                calls.borrow_mut().push(texts.len());
                Ok(texts
                    .iter()
                    .map(|_| {
                        let mut v = vec![0.; 768];
                        v[0] = 1.;
                        v
                    })
                    .collect())
            };
            let mut checkpoints = 0;
            assert!(
                state
                    .infer(&batch, load, infer, |values| {
                        checkpoints += 1;
                        assert!(values.len() <= 2);
                        if checkpoints == 2 {
                            return Err(EmbeddingError::ModelRunFailed(
                                "durable checkpoint failed".into(),
                            ));
                        }
                        Ok(())
                    })
                    .is_err()
            );
            assert_eq!(state.inferred_inputs, 2);
            assert_eq!(loads.get(), 1);
            assert_eq!(state.infer(&batch, load, infer, |_| Ok(())).unwrap(), 5);
            assert_eq!(loads.get(), 1);
            assert_eq!(state.inferred_inputs, 7);
            assert_eq!(state.peak_vectors, 2);
            assert_eq!(*calls.borrow(), [2, 2, 2, 2, 1]);
        });
        let progress = events
            .iter()
            .filter(|event| {
                event
                    .get("phase")
                    .is_some_and(|phase| phase == "bge-base:inference")
            })
            .collect::<Vec<_>>();
        let counts = progress
            .iter()
            .map(|event| event["current"].parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(counts, [0, 2, 2, 4, 6, 7]);
        assert!(progress.iter().all(|event| !event.contains_key("total")
            && event["unit"] == "unique passages"
            && event["message"] == "Generating BGE Base embeddings"));
    }
}
