use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_index::FtsSearchHit;
use serde::{Deserialize, Serialize};

use crate::semantic::SemanticSearchHit;
use crate::text::RetrievalMode;

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
    pub vector_rank: Option<u32>,
    pub vector_distance: Option<f64>,
    pub vector_rank_distance: Option<f64>,
    pub vector_unit_kind: Option<String>,
    pub vector_label: Option<String>,
    pub vector_embedding_unit_key: Option<String>,
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
    vector_rank: Option<u32>,
    vector_distance: Option<f64>,
    vector_rank_distance: Option<f64>,
    vector_unit_kind: Option<String>,
    vector_label: Option<String>,
    vector_embedding_unit_key: Option<String>,
}

pub(crate) struct FusionInput<'a> {
    pub fts_hits: &'a [FtsSearchHit],
    pub vector_hits: &'a [SemanticSearchHit],
    pub identity_keys: &'a BTreeSet<RecordKey>,
    pub excluded_keys: &'a BTreeSet<RecordKey>,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub explain: bool,
    pub identity_count: usize,
}

pub(crate) fn fuse_ranked_hits(input: FusionInput<'_>) -> Vec<FusedRankedHit> {
    let mut by_key = BTreeMap::<RecordKey, FusionAccumulator>::new();
    if input.retrieval.uses_fts() {
        for (index, hit) in input.fts_hits.iter().enumerate() {
            if input.identity_keys.contains(&hit.record_key)
                || input.excluded_keys.contains(&hit.record_key)
            {
                continue;
            }
            let rank = (index + 1) as u32;
            let entry = by_key
                .entry(hit.record_key.clone())
                .or_insert_with(|| FusionAccumulator::new(hit.record_key.clone()));
            entry.fts_rank = Some(rank);
            entry.fts_score = Some(hit.rank);
            entry.fused_score +=
                lane_rrf_score(rank, input.fusion.rank_constant, input.fusion.fts_weight);
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
            entry.vector_embedding_unit_key = Some(hit.embedding_unit_key.clone());
            entry.fused_score +=
                lane_rrf_score(rank, input.fusion.rank_constant, input.fusion.vector_weight);
        }
    }

    let mut fused = by_key.into_values().collect::<Vec<_>>();
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
                vector_rank: hit.vector_rank,
                vector_distance: hit.vector_distance,
                vector_rank_distance: hit.vector_rank_distance,
                vector_unit_kind: hit.vector_unit_kind,
                vector_label: hit.vector_label,
                vector_embedding_unit_key: hit.vector_embedding_unit_key,
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
        vector_rank: None,
        vector_distance: None,
        vector_rank_distance: None,
        vector_unit_kind: None,
        vector_label: None,
        vector_embedding_unit_key: None,
    }
}

impl FusionAccumulator {
    fn new(record_key: RecordKey) -> Self {
        Self {
            record_key,
            fused_score: 0.0,
            fts_rank: None,
            fts_score: None,
            vector_rank: None,
            vector_distance: None,
            vector_rank_distance: None,
            vector_unit_kind: None,
            vector_label: None,
            vector_embedding_unit_key: None,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_rrf_combines_lanes_and_excludes_identity_matches() {
        let fts_hits = vec![
            FtsSearchHit {
                record_key: RecordKey::parse("records:a").unwrap(),
                rank: -2.0,
            },
            FtsSearchHit {
                record_key: RecordKey::parse("records:b").unwrap(),
                rank: -1.0,
            },
        ];
        let vector_hits = vec![
            SemanticSearchHit {
                record_key: "records:b".to_string(),
                embedding_unit_key: "records:b#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.1,
                rank_distance: 0.1,
            },
            SemanticSearchHit {
                record_key: "records:c".to_string(),
                embedding_unit_key: "records:c#parent".to_string(),
                unit_kind: "parent".to_string(),
                label: None,
                distance: 0.2,
                rank_distance: 0.2,
            },
        ];
        let identity_keys = [RecordKey::parse("records:a").unwrap()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let excluded_keys = BTreeSet::new();

        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: RetrievalMode::Hybrid,
            fusion: FusionOptions::default(),
            explain: true,
            identity_count: 1,
        });

        assert_eq!(
            fused
                .iter()
                .map(|hit| hit.record_key.to_string())
                .collect::<Vec<_>>(),
            vec!["records:b", "records:c"]
        );
        assert_eq!(fused[0].explain.as_ref().unwrap().rank, 2);
        assert_eq!(fused[0].explain.as_ref().unwrap().fts_rank, Some(2));
        assert_eq!(fused[0].explain.as_ref().unwrap().vector_rank, Some(1));
    }
}
