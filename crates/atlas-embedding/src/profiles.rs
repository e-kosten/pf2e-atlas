//! The two supported model/input contracts. Profile selection is request local.
use crate::{EmbeddingError, EmbeddingModelId, EmbeddingModelSpec, embedding_model_spec};
use atlas_domain::SearchProfileId;

pub const DEFAULT_SEARCH_PROFILE: &str = "bge-base";
/// Exact shared segmentation dependency, independent of the selected native tokenizer.
pub const BASELINE_SEGMENTATION_TOKENIZER_SHA256: &str = crate::catalog::BGE_TOKENIZER_SHA256;
pub const BASELINE_INPUT_RECIPE: &str = "baseline/v2/bge-tokenizer-d241a60d/body256-context64-overlap32-identity480/owned-header-no-pack/root-unchanged/no-parent";

#[derive(Debug, Clone, Copy)]
pub struct SearchProfileSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub model: EmbeddingModelId,
}
pub const SEARCH_PROFILES: &[SearchProfileSpec] = &[
    SearchProfileSpec {
        id: "bge-base",
        label: "BGE Base",
        model: EmbeddingModelId::BgeBaseEnV15,
    },
    SearchProfileSpec {
        id: "embeddinggemma",
        label: "EmbeddingGemma",
        model: EmbeddingModelId::EmbeddingGemma300M,
    },
];
impl SearchProfileSpec {
    pub fn profile_id(self) -> SearchProfileId {
        SearchProfileId::new(self.id)
    }
    pub fn model_spec(self) -> EmbeddingModelSpec {
        embedding_model_spec(self.model)
    }
}
pub fn search_profile(id: &SearchProfileId) -> Result<SearchProfileSpec, EmbeddingError> {
    SEARCH_PROFILES
        .iter()
        .copied()
        .find(|p| p.id == id.as_str())
        .ok_or(EmbeddingError::UnsupportedModelContract)
}
pub fn profile_for_model(model: EmbeddingModelId) -> SearchProfileSpec {
    match model {
        EmbeddingModelId::BgeBaseEnV15 => SEARCH_PROFILES[0],
        EmbeddingModelId::EmbeddingGemma300M => SEARCH_PROFILES[1],
    }
}
