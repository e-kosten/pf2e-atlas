use atlas_domain::SearchFilterNode;
use serde::{Deserialize, Serialize};

use crate::SearchError;
use crate::fusion::{FusionMethod, FusionOptions};
use crate::page::SearchPage;

pub const DEFAULT_RANKED_CANDIDATE_WINDOW: u32 = 200;
pub const MAX_RANKED_CANDIDATE_WINDOW: u32 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetrievalMode {
    Fts,
    Vector,
    Hybrid,
}

impl RetrievalMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fts => "fts",
            Self::Vector => "vector",
            Self::Hybrid => "hybrid",
        }
    }

    pub(crate) const fn uses_fts(self) -> bool {
        matches!(self, Self::Fts | Self::Hybrid)
    }

    pub(crate) const fn uses_vector(self) -> bool {
        matches!(self, Self::Vector | Self::Hybrid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextSearchTuning {
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub fts_top_k: u32,
    pub vector_top_k: u32,
}

impl TextSearchTuning {
    pub fn default_for_page(page: SearchPage) -> Result<Self, SearchError> {
        Self::resolve_for_page(None, page)
    }

    pub fn resolve_for_page(tuning: Option<Self>, page: SearchPage) -> Result<Self, SearchError> {
        let required_window = page.required_window()?;
        let tuning = tuning.unwrap_or_else(|| Self {
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions::default(),
            fts_top_k: DEFAULT_RANKED_CANDIDATE_WINDOW,
            vector_top_k: DEFAULT_RANKED_CANDIDATE_WINDOW,
        });
        let tuning = Self {
            fts_top_k: tuning.fts_top_k.max(required_window),
            vector_top_k: tuning.vector_top_k.max(required_window),
            ..tuning
        };
        validate_text_search_tuning(&tuning, page)?;
        Ok(tuning)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRequest<'a> {
    pub query: &'a str,
    pub exclude: Option<&'a str>,
    pub filter: Option<&'a SearchFilterNode>,
    pub page: SearchPage,
    pub tuning: Option<TextSearchTuning>,
    pub explain: bool,
}

pub(super) fn validate_search_text_request(
    request: &TextSearchRequest<'_>,
) -> Result<TextSearchTuning, SearchError> {
    if let Some(filter) = request.filter {
        filter
            .validate()
            .map_err(|error| SearchError::invalid_search_options(error.to_string()))?;
    }
    TextSearchTuning::resolve_for_page(request.tuning, request.page)
}

fn validate_text_search_tuning(
    tuning: &TextSearchTuning,
    page: SearchPage,
) -> Result<(), SearchError> {
    if tuning.fusion.method == FusionMethod::Rrf
        && ((tuning.fusion.fts_weight - 1.0).abs() > f64::EPSILON
            || (tuning.fusion.vector_weight - 1.0).abs() > f64::EPSILON)
    {
        return Err(SearchError::invalid_search_options(
            "unweighted rrf does not accept lane weights; use weighted-rrf".to_string(),
        ));
    }
    if tuning.fusion.rank_constant <= 0.0
        || tuning.fusion.fts_weight < 0.0
        || tuning.fusion.vector_weight < 0.0
    {
        return Err(SearchError::invalid_search_options(
            "fusion weights must be non-negative and rank constant must be greater than zero"
                .to_string(),
        ));
    }
    let ranked_window = page.required_window()?;
    if ranked_window > MAX_RANKED_CANDIDATE_WINDOW
        || tuning.fts_top_k > MAX_RANKED_CANDIDATE_WINDOW
        || tuning.vector_top_k > MAX_RANKED_CANDIDATE_WINDOW
    {
        return Err(SearchError::invalid_search_options(format!(
            "ranked search candidate windows must be at most {MAX_RANKED_CANDIDATE_WINDOW}; got page {}, page size {}, fts_top_k {}, vector_top_k {}",
            page.number(),
            page.size(),
            tuning.fts_top_k,
            tuning.vector_top_k
        )));
    }
    Ok(())
}
