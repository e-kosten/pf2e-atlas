#![deny(unsafe_code)]

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
