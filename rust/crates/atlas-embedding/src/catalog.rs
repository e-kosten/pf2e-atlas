use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingModelId {
    MiniLmL12V2,
    BgeSmallEnV15,
    BgeBaseEnV15,
    NomicEmbedTextV15,
    JinaEmbeddingsV2SmallEn,
    JinaEmbeddingsV3,
    GteModernbertBase,
    Qwen3Embedding06b,
    SnowflakeArcticEmbedXs,
    SnowflakeArcticEmbedMV15,
    MxbaiEmbedLargeV1,
    EmbeddingGemma300m,
    E5SmallV2,
    MpnetBaseV2,
}

impl EmbeddingModelId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MiniLmL12V2 => "minilm-l12-v2",
            Self::BgeSmallEnV15 => "bge-small-en-v1.5",
            Self::BgeBaseEnV15 => "bge-base-en-v1.5",
            Self::NomicEmbedTextV15 => "nomic-embed-text-v1.5",
            Self::JinaEmbeddingsV2SmallEn => "jina-embeddings-v2-small-en",
            Self::JinaEmbeddingsV3 => "jina-embeddings-v3",
            Self::GteModernbertBase => "gte-modernbert-base",
            Self::Qwen3Embedding06b => "qwen3-embedding-0.6b",
            Self::SnowflakeArcticEmbedXs => "snowflake-arctic-embed-xs",
            Self::SnowflakeArcticEmbedMV15 => "snowflake-arctic-embed-m-v1.5",
            Self::MxbaiEmbedLargeV1 => "mxbai-embed-large-v1",
            Self::EmbeddingGemma300m => "embeddinggemma-300m",
            Self::E5SmallV2 => "e5-small-v2",
            Self::MpnetBaseV2 => "all-mpnet-base-v2",
        }
    }
}

impl fmt::Display for EmbeddingModelId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EmbeddingModelId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "default" | "minilm" | "minilm-l12-v2" | "Xenova/all-MiniLM-L12-v2" => {
                Ok(Self::MiniLmL12V2)
            }
            "bge-small" | "bge-small-en-v1.5" | "BAAI/bge-small-en-v1.5" => Ok(Self::BgeSmallEnV15),
            "bge-base" | "bge-base-en-v1.5" | "BAAI/bge-base-en-v1.5" => Ok(Self::BgeBaseEnV15),
            "nomic" | "nomic-embed-text-v1.5" | "nomic-ai/nomic-embed-text-v1.5" => {
                Ok(Self::NomicEmbedTextV15)
            }
            "jina-v2-small"
            | "jina-embeddings-v2-small-en"
            | "jinaai/jina-embeddings-v2-small-en" => Ok(Self::JinaEmbeddingsV2SmallEn),
            "jina-v3" | "jina-embeddings-v3" | "jinaai/jina-embeddings-v3" => {
                Ok(Self::JinaEmbeddingsV3)
            }
            "gte-modernbert" | "gte-modernbert-base" | "Alibaba-NLP/gte-modernbert-base" => {
                Ok(Self::GteModernbertBase)
            }
            "qwen3-0.6b" | "qwen3-embedding-0.6b" | "Qwen/Qwen3-Embedding-0.6B" => {
                Ok(Self::Qwen3Embedding06b)
            }
            "snowflake-xs"
            | "snowflake-arctic-embed-xs"
            | "Snowflake/snowflake-arctic-embed-xs" => Ok(Self::SnowflakeArcticEmbedXs),
            "snowflake-m-v1.5"
            | "snowflake-arctic-embed-m-v1.5"
            | "Snowflake/snowflake-arctic-embed-m-v1.5" => Ok(Self::SnowflakeArcticEmbedMV15),
            "mxbai-large" | "mxbai-embed-large-v1" | "mixedbread-ai/mxbai-embed-large-v1" => {
                Ok(Self::MxbaiEmbedLargeV1)
            }
            "embeddinggemma" | "embeddinggemma-300m" | "google/embeddinggemma-300m" => {
                Ok(Self::EmbeddingGemma300m)
            }
            "e5-small-v2" | "intfloat/e5-small-v2" => Ok(Self::E5SmallV2),
            "mpnet-base-v2" | "all-mpnet-base-v2" | "sentence-transformers/all-mpnet-base-v2" => {
                Ok(Self::MpnetBaseV2)
            }
            _ => Err(format!(
                "unsupported embedding model `{value}`; supported values: {}",
                supported_embedding_model_ids().join(", ")
            )),
        }
    }
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
pub const ALL_EMBEDDING_MODELS: &[EmbeddingModelId] = &[
    EmbeddingModelId::MiniLmL12V2,
    EmbeddingModelId::BgeSmallEnV15,
    EmbeddingModelId::BgeBaseEnV15,
    EmbeddingModelId::NomicEmbedTextV15,
    EmbeddingModelId::JinaEmbeddingsV2SmallEn,
    EmbeddingModelId::JinaEmbeddingsV3,
    EmbeddingModelId::GteModernbertBase,
    EmbeddingModelId::Qwen3Embedding06b,
    EmbeddingModelId::SnowflakeArcticEmbedXs,
    EmbeddingModelId::SnowflakeArcticEmbedMV15,
    EmbeddingModelId::MxbaiEmbedLargeV1,
    EmbeddingModelId::EmbeddingGemma300m,
    EmbeddingModelId::E5SmallV2,
    EmbeddingModelId::MpnetBaseV2,
];

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
        EmbeddingModelId::BgeSmallEnV15 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "BAAI/bge-small-en-v1.5",
            model_revision: "main",
            tokenizer_id: "BAAI/bge-small-en-v1.5",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 384,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "Represent this sentence for searching relevant passages: ",
        },
        EmbeddingModelId::BgeBaseEnV15 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "BAAI/bge-base-en-v1.5",
            model_revision: "main",
            tokenizer_id: "BAAI/bge-base-en-v1.5",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "Represent this sentence for searching relevant passages: ",
        },
        EmbeddingModelId::NomicEmbedTextV15 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "nomic-ai/nomic-embed-text-v1.5",
            model_revision: "main",
            tokenizer_id: "nomic-ai/nomic-embed-text-v1.5",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "search_document: ",
            query_prefix: "search_query: ",
        },
        EmbeddingModelId::JinaEmbeddingsV2SmallEn => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "jinaai/jina-embeddings-v2-small-en",
            model_revision: "main",
            tokenizer_id: "jinaai/jina-embeddings-v2-small-en",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 512,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::JinaEmbeddingsV3 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "jinaai/jina-embeddings-v3",
            model_revision: "main",
            tokenizer_id: "jinaai/jina-embeddings-v3",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 1024,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::GteModernbertBase => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "Alibaba-NLP/gte-modernbert-base",
            model_revision: "main",
            tokenizer_id: "Alibaba-NLP/gte-modernbert-base",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::Qwen3Embedding06b => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "Qwen/Qwen3-Embedding-0.6B",
            model_revision: "main",
            tokenizer_id: "Qwen/Qwen3-Embedding-0.6B",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 1024,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::SnowflakeArcticEmbedXs => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "Snowflake/snowflake-arctic-embed-xs",
            model_revision: "main",
            tokenizer_id: "Snowflake/snowflake-arctic-embed-xs",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 384,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::SnowflakeArcticEmbedMV15 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "Snowflake/snowflake-arctic-embed-m-v1.5",
            model_revision: "main",
            tokenizer_id: "Snowflake/snowflake-arctic-embed-m-v1.5",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "",
        },
        EmbeddingModelId::MxbaiEmbedLargeV1 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "mixedbread-ai/mxbai-embed-large-v1",
            model_revision: "main",
            tokenizer_id: "mixedbread-ai/mxbai-embed-large-v1",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 1024,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "",
            query_prefix: "Represent this sentence for searching relevant passages: ",
        },
        EmbeddingModelId::EmbeddingGemma300m => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "google/embeddinggemma-300m",
            model_revision: "main",
            tokenizer_id: "google/embeddinggemma-300m",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "title: none | text: ",
            query_prefix: "task: search result | query: ",
        },
        EmbeddingModelId::E5SmallV2 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "intfloat/e5-small-v2",
            model_revision: "main",
            tokenizer_id: "intfloat/e5-small-v2",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 384,
            dtype: VectorDType::F32,
            distance_metric: DistanceMetric::Cosine,
            document_prefix: "passage: ",
            query_prefix: "query: ",
        },
        EmbeddingModelId::MpnetBaseV2 => EmbeddingModelSpec {
            provider_family: "onnx-mean-pooling",
            model_id: "sentence-transformers/all-mpnet-base-v2",
            model_revision: "main",
            tokenizer_id: "sentence-transformers/all-mpnet-base-v2",
            pooling: PoolingStrategy::Mean,
            normalization: Normalization::L2,
            dimensions: 768,
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

pub fn supported_embedding_model_ids() -> Vec<&'static str> {
    ALL_EMBEDDING_MODELS
        .iter()
        .map(|model| model.as_str())
        .collect()
}

pub fn embedding_model_for_model_id(model_id: &str) -> Option<EmbeddingModelId> {
    ALL_EMBEDDING_MODELS
        .iter()
        .copied()
        .find(|model| embedding_model_spec(*model).model_id == model_id)
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
