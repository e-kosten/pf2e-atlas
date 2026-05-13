#![deny(unsafe_code)]

use std::fmt::Write;
use std::path::{Path, PathBuf};

use ort::{inputs, session::Session, value::Tensor};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokenizers::{EncodeInput, PaddingDirection, PaddingParams, PaddingStrategy, Tokenizer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModelId {
    MiniLmL12V2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolingStrategy {
    Mean,
}

impl PoolingStrategy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mean => "mean",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalization {
    L2,
}

impl Normalization {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L2 => "l2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorDType {
    F32,
}

impl VectorDType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::F32 => "f32",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    Cosine,
}

impl DistanceMetric {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cosine => "cosine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingModelSpec {
    pub provider_family: &'static str,
    pub model_id: &'static str,
    pub model_revision: &'static str,
    pub tokenizer_id: &'static str,
    pub pooling: PoolingStrategy,
    pub normalization: Normalization,
    pub dimensions: usize,
    pub dtype: VectorDType,
    pub distance_metric: DistanceMetric,
    pub document_prefix: &'static str,
    pub query_prefix: &'static str,
}

impl EmbeddingModelSpec {
    pub fn dimensions_string(self) -> String {
        self.dimensions.to_string()
    }

    pub fn model_cache_path(self, cache_root: impl AsRef<Path>) -> PathBuf {
        cache_root.as_ref().join(self.model_id)
    }
}

pub const DEFAULT_EMBEDDING_MODEL: EmbeddingModelId = EmbeddingModelId::MiniLmL12V2;

pub const fn embedding_model_spec(model: EmbeddingModelId) -> EmbeddingModelSpec {
    match model {
        EmbeddingModelId::MiniLmL12V2 => EmbeddingModelSpec {
            provider_family: "transformers-js-minilm",
            model_id: "Xenova/all-MiniLM-L12-v2",
            model_revision: "main",
            tokenizer_id: "Xenova/all-MiniLM-L12-v2",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 384,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
    }
}

pub const fn default_embedding_model_spec() -> EmbeddingModelSpec {
    embedding_model_spec(DEFAULT_EMBEDDING_MODEL)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentEmbeddingInputParts<'a> {
    pub name: &'a str,
    pub traits: &'a [String],
    pub taxonomy_families: &'a [String],
    pub description_text: Option<&'a str>,
    pub aliases: &'a [String],
}

pub fn build_document_embedding_input(parts: DocumentEmbeddingInputParts<'_>) -> String {
    let mut chunks = Vec::new();
    let mut seen = Vec::new();

    append_unique_text_chunk(&mut chunks, &mut seen, parts.name);
    for trait_value in parts.traits {
        append_unique_text_chunk(&mut chunks, &mut seen, trait_value);
    }
    for family in parts.taxonomy_families {
        append_unique_text_chunk(&mut chunks, &mut seen, family);
    }
    if let Some(description_text) = parts.description_text {
        append_unique_text_chunk(&mut chunks, &mut seen, description_text);
    }
    for alias in parts.aliases {
        append_unique_text_chunk(&mut chunks, &mut seen, alias);
    }

    chunks.join("\n")
}

pub fn hash_document_embedding_input(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn append_unique_text_chunk(chunks: &mut Vec<String>, seen: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    let normalized = normalize_embedding_text(trimmed);
    if normalized.is_empty() || seen.contains(&normalized) {
        return;
    }
    seen.push(normalized);
    chunks.push(trimmed.to_string());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingRuntimeConfig {
    pub model: EmbeddingModelId,
    pub cache_root: PathBuf,
}

impl EmbeddingRuntimeConfig {
    pub fn new(model: EmbeddingModelId, cache_root: impl Into<PathBuf>) -> Self {
        Self {
            model,
            cache_root: cache_root.into(),
        }
    }

    pub fn default_model(cache_root: impl Into<PathBuf>) -> Self {
        Self::new(DEFAULT_EMBEDDING_MODEL, cache_root)
    }

    pub fn model_spec(&self) -> EmbeddingModelSpec {
        embedding_model_spec(self.model)
    }

    pub fn model_dir(&self) -> PathBuf {
        self.model_spec().model_cache_path(&self.cache_root)
    }
}

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("failed to load tokenizer `{path}`: {message}")]
    TokenizerLoadFailed { path: String, message: String },
    #[error("failed to tokenize query: {0}")]
    TokenizationFailed(String),
    #[error("failed to load ONNX model `{path}`: {message}")]
    ModelLoadFailed { path: String, message: String },
    #[error("failed to prepare ONNX tensor: {0}")]
    TensorPrepareFailed(String),
    #[error("failed to run ONNX model: {0}")]
    ModelRunFailed(String),
    #[error("model did not return a hidden-state tensor")]
    MissingHiddenState,
    #[error("expected hidden-state shape [batch, tokens, dims], got {0:?}")]
    UnexpectedHiddenStateShape(Vec<usize>),
    #[error("model returned {actual} dimensions, but embedding catalog expects {expected}")]
    DimensionMismatch { expected: usize, actual: usize },
}

pub struct QueryEmbedder {
    spec: EmbeddingModelSpec,
    tokenizer: Tokenizer,
    session: Session,
}

impl QueryEmbedder {
    pub fn load(config: &EmbeddingRuntimeConfig) -> Result<Self, EmbeddingError> {
        Self::load_from_model_dir(config.model_spec(), config.model_dir())
    }

    pub fn load_from_model_dir(
        spec: EmbeddingModelSpec,
        model_dir: impl AsRef<Path>,
    ) -> Result<Self, EmbeddingError> {
        let model_dir = model_dir.as_ref();
        let tokenizer_path = model_dir.join("tokenizer.json");
        let onnx_path = model_dir.join("onnx").join("model.onnx");

        let mut tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|error| {
            EmbeddingError::TokenizerLoadFailed {
                path: tokenizer_path.display().to_string(),
                message: error.to_string(),
            }
        })?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            direction: PaddingDirection::Right,
            pad_to_multiple_of: None,
            pad_id: 0,
            pad_type_id: 0,
            pad_token: "[PAD]".to_string(),
        }));

        let session = Session::builder()
            .map_err(|error| EmbeddingError::ModelLoadFailed {
                path: onnx_path.display().to_string(),
                message: error.to_string(),
            })?
            .commit_from_file(&onnx_path)
            .map_err(|error| EmbeddingError::ModelLoadFailed {
                path: onnx_path.display().to_string(),
                message: error.to_string(),
            })?;

        Ok(Self {
            spec,
            tokenizer,
            session,
        })
    }

    pub fn spec(&self) -> EmbeddingModelSpec {
        self.spec
    }

    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let normalized = normalize_embedding_text(&format!("{}{}", self.spec.query_prefix, text));
        if normalized.is_empty() {
            return Ok(vec![0.0; self.spec.dimensions]);
        }

        let encoding = self
            .tokenizer
            .encode(EncodeInput::Single(normalized.into()), true)
            .map_err(|error| EmbeddingError::TokenizationFailed(error.to_string()))?;

        let token_count = encoding.get_ids().len();
        let shape = [1usize, token_count];
        let input_ids = Tensor::from_array((shape, to_i64_vec(encoding.get_ids())))
            .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;
        let attention_mask = Tensor::from_array((shape, to_i64_vec(encoding.get_attention_mask())))
            .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;
        let token_type_ids = Tensor::from_array((shape, to_i64_vec(encoding.get_type_ids())))
            .map_err(|error| EmbeddingError::TensorPrepareFailed(error.to_string()))?;

        let outputs = self
            .session
            .run(inputs![
                "input_ids" => input_ids,
                "attention_mask" => attention_mask,
                "token_type_ids" => token_type_ids,
            ])
            .map_err(|error| EmbeddingError::ModelRunFailed(error.to_string()))?;

        let output = if let Some(output) = outputs
            .get("last_hidden_state")
            .or_else(|| outputs.get("output_0"))
        {
            output
        } else if outputs.len() > 0 {
            &outputs[0]
        } else {
            return Err(EmbeddingError::MissingHiddenState);
        };
        let (shape, data) = output
            .try_extract_tensor::<f32>()
            .map_err(|error| EmbeddingError::ModelRunFailed(error.to_string()))?;
        let shape = shape
            .iter()
            .map(|dimension| *dimension as usize)
            .collect::<Vec<_>>();
        if shape.len() != 3 {
            return Err(EmbeddingError::UnexpectedHiddenStateShape(shape));
        }
        let tokens = shape[1];
        let dimensions = shape[2];
        if dimensions != self.spec.dimensions {
            return Err(EmbeddingError::DimensionMismatch {
                expected: self.spec.dimensions,
                actual: dimensions,
            });
        }
        mean_pool_normalized(data, tokens, dimensions, encoding.get_attention_mask())
    }
}

pub fn normalize_embedding_text(value: &str) -> String {
    let lower = value.to_lowercase().replace("&nbsp;", " ");
    let mut normalized = String::with_capacity(lower.len());
    let mut previous_space = true;
    for character in lower.chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            normalized.push(character);
            previous_space = false;
        } else if !previous_space {
            normalized.push(' ');
            previous_space = true;
        }
    }
    normalized.trim().to_string()
}

fn to_i64_vec(values: &[u32]) -> Vec<i64> {
    values.iter().map(|value| i64::from(*value)).collect()
}

fn mean_pool_normalized(
    data: &[f32],
    tokens: usize,
    dimensions: usize,
    attention_mask: &[u32],
) -> Result<Vec<f32>, EmbeddingError> {
    let mut vector = vec![0.0; dimensions];
    let mut token_count = 0.0f32;

    for token_index in 0..tokens {
        if attention_mask.get(token_index).copied().unwrap_or(0) == 0 {
            continue;
        }
        token_count += 1.0;
        let offset = token_index * dimensions;
        for dimension in 0..dimensions {
            vector[dimension] += data[offset + dimension];
        }
    }

    if token_count == 0.0 {
        return Ok(vector);
    }

    for value in &mut vector {
        *value /= token_count;
    }
    normalize_vector(&mut vector);
    Ok(vector)
}

fn normalize_vector(vector: &mut [f32]) {
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude == 0.0 {
        return;
    }
    for value in vector {
        *value /= magnitude;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN_REPO_MINILM_CACHE: &str =
        "/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/.cache/hf-models";
    const VECTOR_TOLERANCE: f32 = 0.00001;

    #[test]
    fn default_model_spec_matches_minilm_contract() {
        let spec = default_embedding_model_spec();

        assert_eq!(spec.provider_family, "transformers-js-minilm");
        assert_eq!(spec.model_id, "Xenova/all-MiniLM-L12-v2");
        assert_eq!(spec.model_revision, "main");
        assert_eq!(spec.tokenizer_id, "Xenova/all-MiniLM-L12-v2");
        assert_eq!(spec.pooling.as_str(), "mean");
        assert_eq!(spec.normalization.as_str(), "l2");
        assert_eq!(spec.dimensions, 384);
        assert_eq!(spec.dtype.as_str(), "f32");
        assert_eq!(spec.distance_metric.as_str(), "cosine");
        assert_eq!(spec.document_prefix, "");
        assert_eq!(spec.query_prefix, "");
    }

    #[test]
    fn normalizes_queries_like_typescript_provider() {
        assert_eq!(
            normalize_embedding_text("Remove&nbsp;Frightened Condition!"),
            "remove frightened condition"
        );
    }

    #[test]
    fn builds_document_embedding_input_from_stable_chunks() {
        let input = build_document_embedding_input(DocumentEmbeddingInputParts {
            name: "Heal",
            traits: &["healing".to_string(), "vitality".to_string()],
            taxonomy_families: &["spell healing".to_string()],
            description_text: Some("Restore Hit Points."),
            aliases: &["Restore".to_string(), "heal".to_string()],
        });

        assert_eq!(
            input,
            "Heal\nhealing\nvitality\nspell healing\nRestore Hit Points.\nRestore"
        );
        assert_eq!(
            hash_document_embedding_input(&input),
            "b378ff4932396a900910defcba972f84722eebb067c8b518c5619d6132d44c85"
        );
    }

    #[test]
    fn minilm_query_vectors_match_typescript_fixture_when_model_cache_exists() {
        let config = EmbeddingRuntimeConfig::default_model(MAIN_REPO_MINILM_CACHE);
        if !config.model_dir().join("onnx").join("model.onnx").exists() {
            return;
        }

        let mut embedder =
            QueryEmbedder::load(&config).expect("local MiniLM cache should load from main repo");
        for fixture in ts_vector_fixtures() {
            let vector = embedder
                .embed_query(fixture.query)
                .expect("query embedding should succeed");

            assert_eq!(vector.len(), 384, "query `{}`", fixture.query);
            for (index, expected) in fixture.first8.iter().enumerate() {
                let actual = vector[index];
                assert!(
                    (actual - expected).abs() <= VECTOR_TOLERANCE,
                    "query `{}` vector[{index}] expected {expected}, got {actual}",
                    fixture.query
                );
            }
        }
    }

    struct VectorFixture {
        query: &'static str,
        first8: [f32; 8],
    }

    fn ts_vector_fixtures() -> [VectorFixture; 5] {
        [
            VectorFixture {
                query: "low level healing spell",
                first8: [
                    -0.068_654_3,
                    -0.002_315_331_7,
                    0.043_613_01,
                    0.033_749_383,
                    -0.063_630_88,
                    -0.027_416_993,
                    -0.015_849_806,
                    -0.007_909_846,
                ],
            },
            VectorFixture {
                query: "reaction to raise a shield",
                first8: [
                    -0.026_845_824,
                    0.122_861_5,
                    -0.011_812_944,
                    0.035_734_233,
                    -0.010_173_997,
                    -0.065_088_47,
                    0.048_870_15,
                    0.004_172_891_4,
                ],
            },
            VectorFixture {
                query: "monster with grab and swim speed",
                first8: [
                    -0.011_481_782,
                    -0.000_345_111_77,
                    -0.025_876_341,
                    0.017_008_279,
                    -0.015_930_57,
                    -0.036_722_105,
                    0.080_977_455,
                    0.007_360_012_3,
                ],
            },
            VectorFixture {
                query: "fireball",
                first8: [
                    -0.001_271_67,
                    0.019_497_56,
                    -0.011_835_683,
                    -0.017_335_506,
                    0.039_618_62,
                    0.031_982_79,
                    0.180_924_61,
                    0.056_330_826,
                ],
            },
            VectorFixture {
                query: "remove frightened condition",
                first8: [
                    0.093_608_31,
                    -0.002_027_062,
                    -0.004_421_522_4,
                    0.088_750_735,
                    0.101_151_95,
                    -0.123_675_734,
                    0.119_872_94,
                    -0.047_043_37,
                ],
            },
        ]
    }
}
