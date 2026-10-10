use crate::model_cache::read_verified_asset;
use crate::{
    EmbeddingError, EmbeddingModelSpec, EmbeddingRuntimeConfig, PreparedEmbeddingInput,
    TextEmbeddingTokenizer, default_embedding_model_spec, embedding_reuse_key,
    hash_document_embedding_input,
};
use fastembed::{
    InitOptionsUserDefined, OutputKey, Pooling, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde_json::Value;
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
        if spec != default_embedding_model_spec() {
            return Err(EmbeddingError::UnsupportedModelContract);
        }
        let directory = directory.as_ref();
        let onnx_file = read_verified_asset(directory, "onnx/model.onnx")?;
        let tokenizer_file = read_verified_asset(directory, "tokenizer.json")?;
        let config_file = read_verified_asset(directory, "config.json")?;
        let tokenizer_config_file = read_verified_asset(directory, "tokenizer_config.json")?;
        let config: Value =
            serde_json::from_slice(&tokenizer_config_file).map_err(|e| load_error(directory, e))?;
        // Supplemental metadata is derived exclusively from verified tokenizer
        // configuration, rather than downloaded or invented token declarations.
        let mut special_tokens = serde_json::Map::new();
        for key in [
            "cls_token",
            "mask_token",
            "pad_token",
            "sep_token",
            "unk_token",
        ] {
            special_tokens.insert(
                key.into(),
                config
                    .get(key)
                    .ok_or_else(|| load_error(directory, format!("missing {key}")))?
                    .clone(),
            );
        }
        let special_tokens_map_file =
            serde_json::to_vec(&special_tokens).map_err(|e| load_error(directory, e))?;
        let mut model = UserDefinedEmbeddingModel::new(
            onnx_file,
            TokenizerFiles {
                tokenizer_file,
                config_file,
                special_tokens_map_file,
                tokenizer_config_file,
            },
        )
        .with_pooling(Pooling::Cls);
        model.output_key = Some(OutputKey::ByName("last_hidden_state"));
        let mut library = TextEmbedding::try_new_from_user_defined(
            model,
            InitOptionsUserDefined::new()
                .with_max_length(512)
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
        let tokenizer = TextEmbeddingTokenizer::from_tokenizer(spec, library.tokenizer.clone())?;
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
        self.tokenizer.validate_query_input(text)?;
        let prefixed = format!("{}{text}", self.spec.query_prefix);
        let mut vectors = self.infer(&[prefixed.as_str()], 1)?;
        vectors
            .pop()
            .ok_or(EmbeddingError::UnexpectedEmbeddingOutputCount {
                expected: 1,
                actual: 0,
            })
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
    pub fn embed_documents(&mut self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        for text in texts {
            self.tokenizer.validate_document_input(text)?;
        }
        self.infer(texts, 16)
    }
    fn infer(
        &mut self,
        texts: &[&str],
        batch_size: usize,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(Vec::new());
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
) -> Result<GeneratedEmbeddings, EmbeddingError> {
    if batch_size == 0 {
        return Err(EmbeddingError::ModelRunFailed(
            "batch size must be positive".into(),
        ));
    }
    let mut unique = BTreeMap::<String, &str>::new();
    for input in inputs {
        if input.input_sha256 != hash_document_embedding_input(&input.input)
            || input.reuse_key != embedding_reuse_key(&input.input)
            || input.token_count != embedder.tokenizer.validate_document_input(&input.input)?
        {
            return Err(EmbeddingError::PreparedInputMismatch);
        }
        unique.insert(input.reuse_key.clone(), &input.input);
    }
    let mut available = BTreeMap::new();
    let mut pending = Vec::new();
    let mut reused_inputs = 0;
    for (key, text) in &unique {
        if let Some(vector) = reusable.get(key) {
            validate_embedding_vector(vector, embedder.spec.dimensions)?;
            available.insert(key.clone(), vector.clone());
            reused_inputs += 1;
        } else {
            pending.push((key.clone(), *text));
        }
    }
    let inferred_inputs = pending.len();
    for batch in pending.chunks(batch_size) {
        let texts = batch.iter().map(|(_, text)| *text).collect::<Vec<_>>();
        for ((key, _), vector) in batch.iter().zip(embedder.infer(&texts, batch_size)?) {
            available.insert(key.clone(), vector);
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
