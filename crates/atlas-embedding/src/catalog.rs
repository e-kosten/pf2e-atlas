use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModelId {
    BgeSmallEnV15,
}
impl EmbeddingModelId {
    pub const fn as_str(self) -> &'static str {
        "bge-small-en-v1.5"
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
            "default" | "bge-small-en-v1.5" => Ok(Self::BgeSmallEnV15),
            _ => Err(format!(
                "unsupported embedding model `{value}`; supported: bge-small-en-v1.5"
            )),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolingStrategy {
    Cls,
}
impl PoolingStrategy {
    pub const fn as_str(self) -> &'static str {
        "cls"
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
pub const DEFAULT_EMBEDDING_MODEL: EmbeddingModelId = EmbeddingModelId::BgeSmallEnV15;
pub const ALL_EMBEDDING_MODELS: &[EmbeddingModelId] = &[DEFAULT_EMBEDDING_MODEL];
pub const fn embedding_model_spec(_: EmbeddingModelId) -> EmbeddingModelSpec {
    EmbeddingModelSpec {
        provider_family: "fastembed-local-bge-cls",
        model_id: "BAAI/bge-small-en-v1.5",
        model_revision: "5c38ec7c405ec4b44b94cc5a9bb96e735b38267a",
        model_sha256: BGE_MODEL_SHA256,
        tokenizer_id: "BAAI/bge-small-en-v1.5",
        tokenizer_sha256: BGE_TOKENIZER_SHA256,
        max_input_tokens: Some(512),
        pooling: PoolingStrategy::Cls,
        normalization: Normalization::L2,
        dimensions: 384,
        dtype: VectorDType::F32,
        distance_metric: DistanceMetric::Cosine,
        document_prefix: "",
        query_prefix: "Represent this sentence for searching relevant passages: ",
    }
}
pub(crate) const BGE_MODEL_SHA256: &str =
    "828e1496d7fabb79cfa4dcd84fa38625c0d3d21da474a00f08db0f559940cf35";
pub(crate) const BGE_TOKENIZER_SHA256: &str =
    "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66";
pub const fn default_embedding_model_spec() -> EmbeddingModelSpec {
    embedding_model_spec(DEFAULT_EMBEDDING_MODEL)
}
pub fn supported_embedding_model_ids() -> Vec<&'static str> {
    vec![DEFAULT_EMBEDDING_MODEL.as_str()]
}
pub fn embedding_model_for_model_id(id: &str) -> Option<EmbeddingModelId> {
    (id == default_embedding_model_spec().model_id).then_some(DEFAULT_EMBEDDING_MODEL)
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
