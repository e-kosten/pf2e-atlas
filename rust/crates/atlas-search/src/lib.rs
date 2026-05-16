#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use atlas_domain::SearchFilterNode;
use atlas_embedding::{EmbeddingModelId, EmbeddingRuntimeConfig, EmbeddingUnitKind, TextEmbedder};
use atlas_index::{AtlasIndex, IndexValidationError, VectorQueryError, VectorSearchHit};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct SemanticSearchService {
    index: AtlasIndex,
    embedder: TextEmbedder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticSearchMode {
    ParentOnly,
    Chunks,
    WeightedChunks,
}

impl SemanticSearchMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParentOnly => "parent-only",
            Self::Chunks => "chunks",
            Self::WeightedChunks => "weighted-chunks",
        }
    }

    const fn includes_child_units(self) -> bool {
        !matches!(self, Self::ParentOnly)
    }

    const fn uses_rank_weights(self) -> bool {
        matches!(self, Self::WeightedChunks)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchHit {
    pub record_key: String,
    pub embedding_unit_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
    pub rank_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchTiming {
    pub query_embedding_duration_ms: u128,
    pub vector_search_duration_ms: u128,
    pub total_duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub hits: Vec<SemanticSearchHit>,
    pub timing: SemanticSearchTiming,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Index(#[from] IndexValidationError),
    #[error("invalid embedding model `{model}`: {message}")]
    InvalidEmbeddingModel { model: String, message: String },
    #[error("embedding operation failed: {0}")]
    Embedding(String),
    #[error(transparent)]
    Vector(#[from] VectorQueryError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEmbeddingConfig {
    pub model_id: String,
    pub cache_root: PathBuf,
}

impl SemanticSearchService {
    pub fn open(
        index_path: impl AsRef<Path>,
        embedding_config: &SearchEmbeddingConfig,
    ) -> Result<Self, SearchError> {
        let model = EmbeddingModelId::from_str(&embedding_config.model_id).map_err(|error| {
            SearchError::InvalidEmbeddingModel {
                model: embedding_config.model_id.clone(),
                message: error.to_string(),
            }
        })?;
        let embedding_config = EmbeddingRuntimeConfig::new(model, &embedding_config.cache_root);
        Ok(Self {
            index: AtlasIndex::open_read_only(index_path)?,
            embedder: TextEmbedder::load(&embedding_config)
                .map_err(|error| SearchError::Embedding(error.to_string()))?,
        })
    }

    pub fn semantic(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    ) -> Result<Vec<SemanticSearchHit>, SearchError> {
        Ok(self.semantic_with_timing(query, filter, limit, mode)?.hits)
    }

    pub fn semantic_with_timing(
        &mut self,
        query: &str,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    ) -> Result<SemanticSearchResult, SearchError> {
        let total_started_at = Instant::now();
        let embedding_started_at = Instant::now();
        let query_vector = self
            .embedder
            .embed_query(query)
            .map_err(|error| SearchError::Embedding(error.to_string()))?;
        let query_embedding_duration_ms = embedding_started_at.elapsed().as_millis();
        let vector_started_at = Instant::now();
        let raw_limit = semantic_unit_limit(limit, mode);
        let hits = self.index.query_vector_index(
            &query_vector,
            filter,
            raw_limit,
            mode.includes_child_units(),
        )?;
        let vector_search_duration_ms = vector_started_at.elapsed().as_millis();
        Ok(SemanticSearchResult {
            hits: collapse_vector_hits(hits, limit as usize, mode),
            timing: SemanticSearchTiming {
                query_embedding_duration_ms,
                vector_search_duration_ms,
                total_duration_ms: total_started_at.elapsed().as_millis(),
            },
        })
    }
}

fn semantic_unit_limit(limit: u32, mode: SemanticSearchMode) -> u32 {
    if mode.includes_child_units() {
        limit.saturating_mul(20).max(limit).min(1000)
    } else {
        limit
    }
}

fn collapse_vector_hits(
    rows: Vec<VectorSearchHit>,
    limit: usize,
    mode: SemanticSearchMode,
) -> Vec<SemanticSearchHit> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<VectorSearchHit>>::new();
    for hit in rows {
        grouped.entry(hit.record_key.clone()).or_default().push(hit);
    }
    let mut collapsed = grouped
        .into_values()
        .filter_map(|hits| best_record_hit(hits, mode))
        .collect::<Vec<_>>();
    collapsed.sort_by(compare_semantic_hits_for_rank);
    collapsed.truncate(limit);
    collapsed
}

fn best_record_hit(
    hits: Vec<VectorSearchHit>,
    mode: SemanticSearchMode,
) -> Option<SemanticSearchHit> {
    let has_parent = hits
        .iter()
        .any(|hit| parsed_unit_kind(hit) == Some(EmbeddingUnitKind::Parent));
    hits.into_iter()
        .map(|hit| {
            let rank_distance = rank_distance(&hit, has_parent, mode);
            semantic_hit_from_vector_hit(hit, rank_distance)
        })
        .min_by(compare_semantic_hits_for_rank)
}

fn semantic_hit_from_vector_hit(hit: VectorSearchHit, rank_distance: f64) -> SemanticSearchHit {
    SemanticSearchHit {
        record_key: hit.record_key,
        embedding_unit_key: hit.embedding_unit_key,
        unit_kind: hit.unit_kind,
        label: hit.label,
        distance: hit.distance,
        rank_distance,
    }
}

fn rank_distance(hit: &VectorSearchHit, has_parent: bool, mode: SemanticSearchMode) -> f64 {
    if !mode.uses_rank_weights() {
        return hit.distance;
    }
    let unit_penalty = match parsed_unit_kind(hit) {
        Some(EmbeddingUnitKind::Parent) => 0.0,
        Some(EmbeddingUnitKind::HeadingSection) => 0.025,
        Some(EmbeddingUnitKind::TitledOption) => 0.040,
        _ => 0.050,
    };
    let missing_parent_penalty = if has_parent { 0.0 } else { 0.025 };
    hit.distance + unit_penalty + missing_parent_penalty
}

fn parsed_unit_kind(hit: &VectorSearchHit) -> Option<EmbeddingUnitKind> {
    hit.unit_kind.parse().ok()
}

fn compare_semantic_hits_for_rank(
    left: &SemanticSearchHit,
    right: &SemanticSearchHit,
) -> std::cmp::Ordering {
    left.rank_distance
        .total_cmp(&right.rank_distance)
        .then_with(|| left.distance.total_cmp(&right.distance))
        .then_with(|| left.record_key.cmp(&right.record_key))
        .then_with(|| left.embedding_unit_key.cmp(&right.embedding_unit_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(unit: &str, record: &str, unit_kind: &str, distance: f64) -> VectorSearchHit {
        VectorSearchHit {
            embedding_unit_key: unit.to_string(),
            record_key: record.to_string(),
            unit_kind: unit_kind.to_string(),
            label: None,
            distance,
        }
    }

    #[test]
    fn collapse_vector_hits_keeps_one_unit_per_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit("records:a#parent", "records:a", "parent", 0.1),
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.2,
                ),
                hit("records:b#parent", "records:b", "parent", 0.3),
                hit("records:c#parent", "records:c", "parent", 0.4),
            ],
            2,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed
                .iter()
                .map(|hit| (hit.embedding_unit_key.as_str(), hit.record_key.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("records:a#parent", "records:a"),
                ("records:b#parent", "records:b"),
            ]
        );
    }

    #[test]
    fn collapse_vector_hits_allows_much_closer_child_to_recover_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.200),
            ],
            10,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[0].distance, 0.100);
        assert_eq!(collapsed[0].rank_distance, 0.125);
    }

    #[test]
    fn collapse_vector_hits_penalizes_records_without_parent_hit() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:b#parent", "records:b", "parent", 0.145),
            ],
            10,
            SemanticSearchMode::WeightedChunks,
        );

        assert_eq!(collapsed[0].embedding_unit_key, "records:b#parent");
        assert_eq!(collapsed[0].rank_distance, 0.145);
        assert_eq!(
            collapsed[1].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[1].rank_distance, 0.150);
    }

    #[test]
    fn collapse_vector_hits_can_rank_chunks_without_unit_weights() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.120),
            ],
            10,
            SemanticSearchMode::Chunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[0].rank_distance, 0.100);
    }
}
