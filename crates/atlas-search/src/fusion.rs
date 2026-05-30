use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_index::{FtsSearchHit, FtsSearchLane};
use atlas_record::PersistedRecord;
use serde::{Deserialize, Serialize};

use crate::semantic::SemanticSearchHit;
use crate::text::RetrievalMode;

mod fts;
#[cfg(test)]
mod tests;

pub(crate) use fts::DEFAULT_FTS_FUSION_POLICY;
pub use fts::FtsMatchConfidence;
use fts::{
    DEFAULT_FTS_FUSION_POLICY_LABEL, FtsFusionPolicy, classify_fts_hit, confidence_score,
    effective_fts_rank, fts_confidence_weight, fts_lane_weight,
};

pub const DEFAULT_FTS_FUSION_POLICY_NAME: &str = DEFAULT_FTS_FUSION_POLICY_LABEL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FusionMethod {
    Rrf,
    WeightedRrf,
}

impl FusionMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rrf => "rrf",
            Self::WeightedRrf => "weighted-rrf",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionOptions {
    pub method: FusionMethod,
    pub fts_weight: f64,
    pub vector_weight: f64,
    pub rank_constant: f64,
}

impl Default for FusionOptions {
    fn default() -> Self {
        Self {
            method: FusionMethod::WeightedRrf,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchExplain {
    pub rank: u32,
    pub fused_score: Option<f64>,
    pub fts_rank: Option<u32>,
    pub fts_score: Option<f64>,
    pub fts_lane: Option<FtsSearchLane>,
    pub fts_confidence: Option<FtsMatchConfidence>,
    pub vector_rank: Option<u32>,
    pub vector_distance: Option<f64>,
    pub vector_rank_distance: Option<f64>,
    pub vector_unit_kind: Option<String>,
    pub vector_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FusedRankedHit {
    pub record_key: RecordKey,
    pub explain: Option<TextSearchExplain>,
}

#[derive(Debug, Clone, PartialEq)]
struct FusionAccumulator {
    record_key: RecordKey,
    fused_score: f64,
    fts_rank: Option<u32>,
    fts_score: Option<f64>,
    fts_lane: Option<FtsSearchLane>,
    fts_confidence: Option<FtsMatchConfidence>,
    vector_rank: Option<u32>,
    vector_distance: Option<f64>,
    vector_rank_distance: Option<f64>,
    vector_unit_kind: Option<String>,
    vector_label: Option<String>,
}

pub(crate) struct FusionInput<'a> {
    pub fts_hits: &'a [FtsSearchHit],
    pub vector_hits: &'a [SemanticSearchHit],
    pub records_by_key: &'a BTreeMap<RecordKey, PersistedRecord>,
    pub fts_tokens: &'a [String],
    pub identity_keys: &'a BTreeSet<RecordKey>,
    pub excluded_keys: &'a BTreeSet<RecordKey>,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub fts_policy: FtsFusionPolicy,
    pub explain: bool,
    pub identity_count: usize,
}

pub(crate) fn fuse_ranked_hits(input: FusionInput<'_>) -> Vec<FusedRankedHit> {
    let mut by_key = BTreeMap::<RecordKey, FusionAccumulator>::new();
    if input.retrieval.uses_fts() {
        for hit in input.fts_hits {
            if input.identity_keys.contains(&hit.record_key)
                || input.excluded_keys.contains(&hit.record_key)
            {
                continue;
            }
            let entry = by_key
                .entry(hit.record_key.clone())
                .or_insert_with(|| FusionAccumulator::new(hit.record_key.clone()));
            let confidence = input
                .records_by_key
                .get(&hit.record_key)
                .map(|record| classify_fts_hit(hit, record, input.fts_tokens))
                .unwrap_or(FtsMatchConfidence::WeakLexical);
            let rank = effective_fts_rank(hit.lane_rank.max(1), hit.lane, confidence);
            entry.record_best_fts_explain(rank, hit.rank, hit.lane, confidence);
            let score = lane_rrf_score(
                rank,
                input.fusion.rank_constant,
                input.fusion.fts_weight
                    * fts_lane_weight(hit.lane)
                    * fts_confidence_weight(hit.lane, confidence),
            );
            entry.fused_score += input.fts_policy.apply(confidence, score);
        }
    }
    if input.retrieval.uses_vector() {
        for (index, hit) in input.vector_hits.iter().enumerate() {
            let record_key = match RecordKey::parse(&hit.record_key) {
                Ok(record_key) => record_key,
                Err(_) => continue,
            };
            if input.identity_keys.contains(&record_key)
                || input.excluded_keys.contains(&record_key)
            {
                continue;
            }
            let rank = (index + 1) as u32;
            let entry = by_key
                .entry(record_key.clone())
                .or_insert_with(|| FusionAccumulator::new(record_key));
            entry.vector_rank = Some(rank);
            entry.vector_distance = Some(hit.distance);
            entry.vector_rank_distance = Some(hit.rank_distance);
            entry.vector_unit_kind = Some(hit.unit_kind.clone());
            entry.vector_label = hit.label.clone();
            entry.fused_score +=
                lane_rrf_score(rank, input.fusion.rank_constant, input.fusion.vector_weight);
        }
    }

    let mut fused = by_key
        .into_values()
        .filter(retains_fused_hit)
        .collect::<Vec<_>>();
    fused.sort_by(compare_fused_hits);
    fused
        .into_iter()
        .enumerate()
        .map(|(index, hit)| FusedRankedHit {
            record_key: hit.record_key.clone(),
            explain: input.explain.then(|| TextSearchExplain {
                rank: (input.identity_count + index + 1) as u32,
                fused_score: Some(hit.fused_score),
                fts_rank: hit.fts_rank,
                fts_score: hit.fts_score,
                fts_lane: hit.fts_lane,
                fts_confidence: hit.fts_confidence,
                vector_rank: hit.vector_rank,
                vector_distance: hit.vector_distance,
                vector_rank_distance: hit.vector_rank_distance,
                vector_unit_kind: hit.vector_unit_kind,
                vector_label: hit.vector_label,
            }),
        })
        .collect()
}

pub(crate) fn identity_explain(index: usize) -> TextSearchExplain {
    TextSearchExplain {
        rank: (index + 1) as u32,
        fused_score: None,
        fts_rank: None,
        fts_score: None,
        fts_lane: None,
        fts_confidence: None,
        vector_rank: None,
        vector_distance: None,
        vector_rank_distance: None,
        vector_unit_kind: None,
        vector_label: None,
    }
}

impl FusionAccumulator {
    fn new(record_key: RecordKey) -> Self {
        Self {
            record_key,
            fused_score: 0.0,
            fts_rank: None,
            fts_score: None,
            fts_lane: None,
            fts_confidence: None,
            vector_rank: None,
            vector_distance: None,
            vector_rank_distance: None,
            vector_unit_kind: None,
            vector_label: None,
        }
    }

    fn record_best_fts_explain(
        &mut self,
        rank: u32,
        score: f64,
        lane: FtsSearchLane,
        confidence: FtsMatchConfidence,
    ) {
        let should_replace = match (self.fts_confidence, self.fts_rank) {
            (None, _) => true,
            (Some(current_confidence), Some(current_rank)) => {
                confidence_score(confidence) > confidence_score(current_confidence)
                    || (confidence_score(confidence) == confidence_score(current_confidence)
                        && rank < current_rank)
            }
            (Some(_), None) => true,
        };
        if should_replace {
            self.fts_rank = Some(rank);
            self.fts_score = Some(score);
            self.fts_lane = Some(lane);
            self.fts_confidence = Some(confidence);
        }
    }
}

fn retains_fused_hit(hit: &FusionAccumulator) -> bool {
    hit.fused_score > 0.0 || hit.vector_rank.is_some()
}

fn lane_rrf_score(rank: u32, rank_constant: f64, weight: f64) -> f64 {
    weight / (rank_constant + f64::from(rank))
}

fn compare_fused_hits(left: &FusionAccumulator, right: &FusionAccumulator) -> std::cmp::Ordering {
    right
        .fused_score
        .total_cmp(&left.fused_score)
        .then_with(|| compare_optional_rank(left.fts_rank, right.fts_rank))
        .then_with(|| compare_optional_rank(left.vector_rank, right.vector_rank))
        .then_with(|| left.record_key.cmp(&right.record_key))
}

fn compare_optional_rank(left: Option<u32>, right: Option<u32>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
