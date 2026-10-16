use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModelId {
    BgeBaseEnV15,
    EmbeddingGemma300M,
}
impl EmbeddingModelId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BgeBaseEnV15 => "bge-base-en-v1.5",
            Self::EmbeddingGemma300M => "embeddinggemma-300m",
        }
    }
}
impl fmt::Display for EmbeddingModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl FromStr for EmbeddingModelId {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "bge-base-en-v1.5" => Ok(Self::BgeBaseEnV15),
            "embeddinggemma-300m" => Ok(Self::EmbeddingGemma300M),
            _ => Err(format!(
                "unsupported embedding model `{value}`; supported: {}",
                supported_embedding_model_ids().join(", ")
            )),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolingStrategy {
    Cls,
    ExportedSentenceEmbedding,
}
impl PoolingStrategy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cls => "cls",
            Self::ExportedSentenceEmbedding => "exported-rank2-trained-projection",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalization {
    L2,
}
impl Normalization {
    pub const fn as_str(self) -> &'static str {
        "l2"
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorDType {
    F32,
}
impl VectorDType {
    pub const fn as_str(self) -> &'static str {
        "f32"
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    Cosine,
}
impl DistanceMetric {
    pub const fn as_str(self) -> &'static str {
        "cosine"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingModelSpec {
    pub provider_family: &'static str,
    pub model_id: &'static str,
    pub model_revision: &'static str,
    pub model_sha256: &'static str,
    pub tokenizer_id: &'static str,
    pub tokenizer_sha256: &'static str,
    pub max_input_tokens: Option<usize>,
    pub pooling: PoolingStrategy,
    pub output_name: &'static str,
    pub normalization: Normalization,
    pub dimensions: usize,
    pub dtype: VectorDType,
    pub distance_metric: DistanceMetric,
    pub document_prefix: &'static str,
    pub query_prefix: &'static str,
    /// Required package-relative asset paths and their SHA-256 digests.
    pub required_assets: &'static [(&'static str, &'static str)],
}
impl EmbeddingModelSpec {
    /// Exact model input from the unprefixed selected document string.
    pub fn document_input(self, text: &str) -> String {
        format!("{}{text}", self.document_prefix)
    }
    /// Exact model input from the unprefixed query string.
    pub fn query_input(self, text: &str) -> String {
        format!("{}{text}", self.query_prefix)
    }
    pub fn dimensions_string(self) -> String {
        self.dimensions.to_string()
    }
    pub fn model_cache_path(self, cache_root: impl AsRef<Path>) -> PathBuf {
        cache_root
            .as_ref()
            .join(self.model_id)
            .join(self.model_revision)
    }
}
pub const ALL_EMBEDDING_MODELS: &[EmbeddingModelId] = &[
    EmbeddingModelId::BgeBaseEnV15,
    EmbeddingModelId::EmbeddingGemma300M,
];
pub const fn embedding_model_spec(model: EmbeddingModelId) -> EmbeddingModelSpec {
    let base = EmbeddingModelSpec {
        provider_family: "fastembed-local-bge-cls",
        model_id: "BAAI/bge-base-en-v1.5",
        model_revision: "a5beb1e3e68b9ab74eb54cfd186867f64f240e1a",
        model_sha256: "9bc579acdba21c253c62a9bf866891355a63ffa3442b52c8a37d75b2ccb91848",
        tokenizer_id: "BAAI/bge-base-en-v1.5",
        tokenizer_sha256: BGE_TOKENIZER_SHA256,
        max_input_tokens: Some(512),
        pooling: PoolingStrategy::Cls,
        output_name: "last_hidden_state",
        normalization: Normalization::L2,
        dimensions: 768,
        dtype: VectorDType::F32,
        distance_metric: DistanceMetric::Cosine,
        document_prefix: "",
        query_prefix: "Represent this sentence for searching relevant passages: ",
        required_assets: crate::model_cache::BASE_ASSETS,
    };
    match model {
        EmbeddingModelId::BgeBaseEnV15 => base,
        EmbeddingModelId::EmbeddingGemma300M => EmbeddingModelSpec {
            provider_family: "fastembed-local-embeddinggemma-projected",
            model_id: "onnx-community/embeddinggemma-300m-ONNX",
            model_revision: "5090578d9565bb06545b4552f76e6bc2c93e4a66",
            model_sha256: "ea91fd315a7c152d427d231746f0f811a1ac93beaba656abfdf2b24e091265e4",
            tokenizer_id: "onnx-community/embeddinggemma-300m-ONNX",
            tokenizer_sha256: "4dda02faaf32bc91031dc8c88457ac272b00c1016cc679757d1c441b248b9c47",
            max_input_tokens: Some(2048),
            pooling: PoolingStrategy::ExportedSentenceEmbedding,
            output_name: "sentence_embedding",
            dimensions: 768,
            document_prefix: "title: none | text: ",
            query_prefix: "task: search result | query: ",
            required_assets: crate::model_cache::GEMMA_ASSETS,
            ..base
        },
    }
}
pub(crate) const BGE_TOKENIZER_SHA256: &str =
    "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66";
pub fn supported_embedding_model_ids() -> Vec<&'static str> {
    ALL_EMBEDDING_MODELS
        .iter()
        .map(|model| model.as_str())
        .collect()
}
pub fn embedding_model_for_model_id(id: &str) -> Option<EmbeddingModelId> {
    ALL_EMBEDDING_MODELS
        .iter()
        .copied()
        .find(|model| embedding_model_spec(*model).model_id == id)
}

pub(crate) fn validate_model_spec(
    spec: EmbeddingModelSpec,
) -> Result<EmbeddingModelId, crate::EmbeddingError> {
    embedding_model_for_model_id(spec.model_id)
        .filter(|model| embedding_model_spec(*model) == spec)
        .ok_or(crate::EmbeddingError::UnsupportedModelContract)
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
    pub fn model_spec(&self) -> EmbeddingModelSpec {
        embedding_model_spec(self.model)
    }
    pub fn model_dir(&self) -> PathBuf {
        self.model_spec().model_cache_path(&self.cache_root)
    }
}
