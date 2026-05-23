use atlas_embedding::EmbeddingUnitKind;
use atlas_index::VectorSearchHit;
use serde::{Deserialize, Serialize};

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

    pub(crate) const fn includes_child_units(self) -> bool {
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

pub(crate) fn semantic_unit_limit(limit: u32, mode: SemanticSearchMode) -> u32 {
    if mode.includes_child_units() {
        limit.saturating_mul(20).max(limit).min(1000)
    } else {
        limit
    }
}

pub(crate) fn collapse_vector_hits(
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
