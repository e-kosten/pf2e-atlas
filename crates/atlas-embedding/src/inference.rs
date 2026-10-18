use crate::model_cache::read_verified_asset;
use crate::{
    EmbeddingError, EmbeddingModelSpec, EmbeddingRuntimeConfig, PoolingStrategy,
    PreparedEmbeddingInput, TextEmbeddingTokenizer, hash_document_embedding_input,
};
use fastembed::{
    InitOptionsUserDefined, OutputKey, Pooling, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use std::{collections::BTreeMap, path::Path};

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

#[derive(Debug)]
pub struct GeneratedEmbeddings {
    /// One vector per supplied source occurrence, in input order.
    pub vectors: Vec<Vec<f32>>,
    /// Distinct exact execution/input identities which required inference.
    pub inferred_inputs: usize,
    /// Distinct exact execution/input identities restored from the cache.
    pub reused_inputs: usize,
}

/// Reuse and inference deduplicate exact prepared inputs, while the returned
/// vector sequence retains one entry for every independent source attribution.
pub fn generate_prepared_embeddings(
    embedder: &mut TextEmbedder,
    inputs: &[PreparedEmbeddingInput],
    reusable: &BTreeMap<String, Vec<f32>>,
    batch_size: usize,
    mut checkpoint: impl FnMut(&[(String, Vec<f32>)]) -> Result<(), EmbeddingError>,
) -> Result<GeneratedEmbeddings, EmbeddingError> {
    if batch_size == 0 {
        return Err(EmbeddingError::ModelRunFailed(
            "batch size must be positive".into(),
        ));
    }
    let mut unique = BTreeMap::<String, &str>::new();
    embedding_progress(
        embedder.spec,
        "native_validation",
        0,
        inputs.len(),
        "occurrences",
        None,
        inputs.len(),
    );
    let mut last_reported = 0;
    for (ordinal, input) in inputs.iter().enumerate() {
        if input.input_sha256 != hash_document_embedding_input(&input.input)
            || input.reuse_key != embedder.spec.document_reuse_key(&input.input)
            || input.token_count != embedder.tokenizer.validate_document_input(&input.input)?
        {
            return Err(EmbeddingError::PreparedInputMismatch);
        }
        unique.insert(input.reuse_key.clone(), &input.input);
        if report_embedding_progress(ordinal + 1, inputs.len(), last_reported) {
            embedding_progress(
                embedder.spec,
                "native_validation",
                ordinal + 1,
                inputs.len(),
                "occurrences",
                None,
                inputs.len(),
            );
            last_reported = ordinal + 1;
        }
    }
    let mut available = BTreeMap::new();
    let mut pending = Vec::new();
    let mut reused_inputs = 0;
    last_reported = 0;
    embedding_progress(
        embedder.spec,
        "reuse_selection",
        0,
        unique.len(),
        "unique inputs",
        Some(0),
        inputs.len(),
    );
    for (ordinal, (key, text)) in unique.iter().enumerate() {
        if let Some(vector) = reusable.get(key) {
            validate_embedding_vector(vector, embedder.spec.dimensions)?;
            available.insert(key.clone(), vector.clone());
            reused_inputs += 1;
        } else {
            pending.push((key.clone(), *text));
        }
        if report_embedding_progress(ordinal + 1, unique.len(), last_reported) {
            embedding_progress(
                embedder.spec,
                "reuse_selection",
                ordinal + 1,
                unique.len(),
                "unique inputs",
                Some(reused_inputs),
                inputs.len(),
            );
            last_reported = ordinal + 1;
        }
    }
    let inferred_inputs = pending.len();
    embedding_progress(
        embedder.spec,
        "inference",
        0,
        inferred_inputs,
        "unique inputs",
        Some(reused_inputs),
        inputs.len(),
    );
    let mut completed = 0;
    last_reported = 0;
    for batch in pending.chunks(batch_size) {
        let texts = batch.iter().map(|(_, text)| *text).collect::<Vec<_>>();
        let completed_batch = batch
            .iter()
            .zip(embedder.embed_documents(&texts)?)
            .map(|((key, _), vector)| (key.clone(), vector))
            .collect::<Vec<_>>();
        checkpoint(&completed_batch)?;
        for (key, vector) in completed_batch {
            available.insert(key, vector);
        }
        completed += batch.len();
        if report_embedding_progress(completed, inferred_inputs, last_reported) {
            embedding_progress(
                embedder.spec,
                "inference",
                completed,
                inferred_inputs,
                "unique inputs",
                Some(reused_inputs),
                inputs.len(),
            );
            last_reported = completed;
        }
    }
    let vectors = inputs
        .iter()
        .map(|input| {
            available
                .get(&input.reuse_key)
                .cloned()
                .ok_or(EmbeddingError::PreparedInputMismatch)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GeneratedEmbeddings {
        vectors,
        inferred_inputs,
        reused_inputs,
    })
}

fn report_embedding_progress(current: usize, total: usize, last_reported: usize) -> bool {
    let interval = (total / 100).clamp(25, 500);
    current == total || current.saturating_sub(last_reported) >= interval
}
fn embedding_progress(
    spec: EmbeddingModelSpec,
    stage: &str,
    current: usize,
    total: usize,
    unit: &str,
    reused: Option<usize>,
    occurrences: usize,
) {
    let profile = crate::embedding_model_for_model_id(spec.model_id).map(crate::profile_for_model);
    let label = profile.map_or(spec.model_id, |p| p.label);
    let id = profile.map_or(spec.model_id, |p| p.id);
    let phase = format!("{id}:{stage}");
    let action = match stage {
        "native_validation" => "Validating native inputs",
        "reuse_selection" => "Checking reusable inputs",
        "inference" => "Inferring vectors",
        _ => stage,
    };
    let reused = reused.map_or_else(|| "unknown".to_owned(), |count| count.to_string());
    tracing::info!(target: "atlas_progress", phase = %phase, current = current as u64, total = total as u64,
        "{label} [{id}] {action}: {current}/{total} {unit}; {reused} reused unique inputs; {occurrences} occurrences");
}
