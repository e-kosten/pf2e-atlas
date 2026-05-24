use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::RecordKey;
use atlas_index::{FtsSearchHit, FtsSearchLane};
use atlas_record::PersistedRecord;
use serde::{Deserialize, Serialize};

use crate::query::tokenize_fts_query;
use crate::{RetrievalMode, SemanticSearchHit};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FusionMethod {
    Rrf,
    WeightedRrf,
    MinMaxScore,
}

impl FusionMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rrf => "rrf",
            Self::WeightedRrf => "weighted-rrf",
            Self::MinMaxScore => "min-max-score",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionOptions {
    pub method: FusionMethod,
    pub fts_weight: f64,
    pub vector_weight: f64,
    pub rank_constant: f64,
    pub fts_policy: FtsFusionPolicy,
}

impl Default for FusionOptions {
    fn default() -> Self {
        Self {
            method: FusionMethod::WeightedRrf,
            fts_weight: 1.0,
            vector_weight: 1.0,
            rank_constant: 60.0,
            fts_policy: FtsFusionPolicy::DemoteWeak,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FtsFusionPolicy {
    All,
    DemoteWeak,
    StrongOnly,
}

impl FtsFusionPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::DemoteWeak => "demote-weak",
            Self::StrongOnly => "strong-only",
        }
    }

    fn apply(self, confidence: FtsMatchConfidence, score: f64) -> f64 {
        match self {
            Self::All => score,
            Self::DemoteWeak => match confidence {
                FtsMatchConfidence::DirectTitle | FtsMatchConfidence::StrongLexical => score,
                FtsMatchConfidence::MediumLexical => score * 0.5,
                FtsMatchConfidence::WeakLexical => score * 0.1,
            },
            Self::StrongOnly => match confidence {
                FtsMatchConfidence::DirectTitle | FtsMatchConfidence::StrongLexical => score,
                FtsMatchConfidence::MediumLexical | FtsMatchConfidence::WeakLexical => 0.0,
            },
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
    pub vector_embedding_unit_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FtsMatchConfidence {
    DirectTitle,
    StrongLexical,
    MediumLexical,
    WeakLexical,
}

impl FtsMatchConfidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DirectTitle => "direct-title",
            Self::StrongLexical => "strong-lexical",
            Self::MediumLexical => "medium-lexical",
            Self::WeakLexical => "weak-lexical",
        }
    }
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
    vector_embedding_unit_key: Option<String>,
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
    pub explain: bool,
    pub identity_count: usize,
}

pub(crate) fn fuse_ranked_hits(input: FusionInput<'_>) -> Vec<FusedRankedHit> {
    let mut by_key = BTreeMap::<RecordKey, FusionAccumulator>::new();
    if input.retrieval.uses_fts() {
        let fts_score_range = lane_score_range(input.fts_hits.iter().map(|hit| hit.rank));
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
            entry.fused_score += input.fusion.fts_policy.apply(
                confidence,
                lane_fusion_score(
                    rank,
                    hit.rank,
                    fts_score_range,
                    input.fusion,
                    input.fusion.fts_weight
                        * fts_lane_weight(hit.lane)
                        * fts_confidence_weight(hit.lane, confidence),
                ),
            );
        }
    }
    if input.retrieval.uses_vector() {
        let vector_score_range =
            lane_score_range(input.vector_hits.iter().map(|hit| hit.rank_distance));
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
            entry.fused_score += lane_fusion_score(
                rank,
                hit.rank_distance,
                vector_score_range,
                input.fusion,
                input.fusion.vector_weight,
            );
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
                vector_embedding_unit_key: hit.vector_embedding_unit_key,
            }),
        })
        .collect()
}

fn retains_fused_hit(hit: &FusionAccumulator) -> bool {
    if hit.fused_score > 0.0 || hit.vector_rank.is_some() {
        return true;
    }
    matches!(
        hit.fts_confidence,
        Some(FtsMatchConfidence::DirectTitle | FtsMatchConfidence::StrongLexical)
    )
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
            fts_lane: None,
            fts_confidence: None,
            vector_rank: None,
            vector_distance: None,
            vector_rank_distance: None,
            vector_unit_kind: None,
            vector_label: None,
            vector_embedding_unit_key: None,
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

fn confidence_score(confidence: FtsMatchConfidence) -> u8 {
    match confidence {
        FtsMatchConfidence::DirectTitle => 4,
        FtsMatchConfidence::StrongLexical => 3,
        FtsMatchConfidence::MediumLexical => 2,
        FtsMatchConfidence::WeakLexical => 1,
    }
}

fn fts_lane_weight(lane: FtsSearchLane) -> f64 {
    match lane {
        FtsSearchLane::Mixed | FtsSearchLane::TitleAlias => 1.0,
        FtsSearchLane::Facet => 0.35,
    }
}

fn fts_confidence_weight(lane: FtsSearchLane, confidence: FtsMatchConfidence) -> f64 {
    if lane != FtsSearchLane::TitleAlias {
        return 1.0;
    }
    match confidence {
        FtsMatchConfidence::DirectTitle => 2.0,
        FtsMatchConfidence::StrongLexical => 1.5,
        FtsMatchConfidence::MediumLexical | FtsMatchConfidence::WeakLexical => 1.0,
    }
}

fn lane_rrf_score(rank: u32, rank_constant: f64, weight: f64) -> f64 {
    weight / (rank_constant + f64::from(rank))
}

#[derive(Debug, Clone, Copy)]
struct LaneScoreRange {
    best: f64,
    worst: f64,
    count: usize,
}

fn lane_score_range(scores: impl Iterator<Item = f64>) -> Option<LaneScoreRange> {
    let mut scores = scores.filter(|score| score.is_finite());
    let first = scores.next()?;
    let mut best = first;
    let mut worst = first;
    let mut count = 1;
    for score in scores {
        worst = score;
        count += 1;
    }
    if best == worst {
        best = first;
    }
    Some(LaneScoreRange { best, worst, count })
}

fn lane_fusion_score(
    rank: u32,
    raw_score: f64,
    score_range: Option<LaneScoreRange>,
    fusion: FusionOptions,
    weight: f64,
) -> f64 {
    match fusion.method {
        FusionMethod::Rrf | FusionMethod::WeightedRrf => {
            lane_rrf_score(rank, fusion.rank_constant, weight)
        }
        FusionMethod::MinMaxScore => {
            weight
                * lane_min_max_score(raw_score, score_range)
                    .unwrap_or_else(|| lane_rrf_score(rank, fusion.rank_constant, 1.0))
        }
    }
}

fn lane_min_max_score(raw_score: f64, score_range: Option<LaneScoreRange>) -> Option<f64> {
    if !raw_score.is_finite() {
        return None;
    }
    let LaneScoreRange { best, worst, count } = score_range?;
    if !best.is_finite() || !worst.is_finite() {
        return None;
    }
    let span = (best - worst).abs();
    if span <= f64::EPSILON {
        return Some(if count == 1 { 1.0 } else { 0.0 });
    }
    let normalized = if best > worst {
        (raw_score - worst) / span
    } else {
        (worst - raw_score) / span
    };
    Some(normalized.clamp(0.0, 1.0))
}

fn classify_fts_hit(
    hit: &FtsSearchHit,
    record: &PersistedRecord,
    query_tokens: &[String],
) -> FtsMatchConfidence {
    classify_fts_texts(
        hit.lane,
        record,
        query_tokens,
        hit.title_alias_texts.iter().map(String::as_str),
    )
}

fn classify_fts_texts<'a>(
    lane: FtsSearchLane,
    record: &'a PersistedRecord,
    query_tokens: &[String],
    title_alias_texts: impl Iterator<Item = &'a str>,
) -> FtsMatchConfidence {
    let query_tokens = query_tokens
        .iter()
        .filter(|token| !token.is_empty())
        .map(String::as_str)
        .collect::<Vec<_>>();
    if query_tokens.is_empty() {
        return FtsMatchConfidence::WeakLexical;
    }

    let mut candidate_texts = vec![record.name.as_str()];
    candidate_texts.extend(title_alias_texts);
    let title_confidence = classify_title_alias_match(&query_tokens, &candidate_texts);
    if matches!(
        title_confidence,
        FtsMatchConfidence::DirectTitle | FtsMatchConfidence::StrongLexical
    ) {
        return title_confidence;
    }
    if lane == FtsSearchLane::TitleAlias {
        return title_confidence;
    }

    let high_value_tokens = high_value_record_tokens(record);
    let high_value_refs = high_value_tokens
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let high_value_coverage = token_coverage(&query_tokens, &high_value_refs);
    if high_value_coverage >= 1.0 {
        return FtsMatchConfidence::StrongLexical;
    }
    if query_tokens.len() >= 3 && high_value_coverage >= 0.67 {
        return FtsMatchConfidence::StrongLexical;
    }
    if high_value_coverage >= 0.5 {
        return FtsMatchConfidence::MediumLexical;
    }

    FtsMatchConfidence::WeakLexical
}

fn effective_fts_rank(rank: u32, lane: FtsSearchLane, confidence: FtsMatchConfidence) -> u32 {
    if lane != FtsSearchLane::TitleAlias {
        return rank;
    }
    match confidence {
        FtsMatchConfidence::DirectTitle => 1,
        FtsMatchConfidence::StrongLexical => rank.min(3),
        FtsMatchConfidence::MediumLexical | FtsMatchConfidence::WeakLexical => rank,
    }
}

fn classify_title_alias_match(
    query_tokens: &[&str],
    candidate_texts: &[&str],
) -> FtsMatchConfidence {
    let significant_query_tokens = significant_tokens(query_tokens.iter().copied());
    if significant_query_tokens.is_empty() {
        return FtsMatchConfidence::WeakLexical;
    }

    let mut best = FtsMatchConfidence::WeakLexical;
    for candidate in candidate_texts {
        let candidate_tokens = tokenize_fts_query(candidate);
        let significant_candidate_tokens =
            significant_tokens(candidate_tokens.iter().map(String::as_str));
        if significant_candidate_tokens.is_empty() {
            continue;
        }
        let score = title_alias_score(&significant_query_tokens, &significant_candidate_tokens);
        let confidence = title_alias_confidence(score);
        if confidence_score(confidence) > confidence_score(best) {
            best = confidence;
        }
    }
    best
}

#[derive(Debug, Clone, Copy)]
struct TitleAliasScore {
    matched_query_tokens: usize,
    query_coverage: f64,
    candidate_coverage: f64,
    exact_token_match: bool,
}

fn title_alias_score(query_tokens: &[&str], candidate_tokens: &[&str]) -> TitleAliasScore {
    let matched_query_tokens = query_tokens
        .iter()
        .filter(|query| {
            candidate_tokens
                .iter()
                .any(|candidate| title_alias_token_matches(query, candidate))
        })
        .count();
    let matched_candidate_tokens = candidate_tokens
        .iter()
        .filter(|candidate| {
            query_tokens
                .iter()
                .any(|query| title_alias_token_matches(query, candidate))
        })
        .count();
    TitleAliasScore {
        matched_query_tokens,
        query_coverage: matched_query_tokens as f64 / query_tokens.len() as f64,
        candidate_coverage: matched_candidate_tokens as f64 / candidate_tokens.len() as f64,
        exact_token_match: query_tokens == candidate_tokens,
    }
}

fn title_alias_confidence(score: TitleAliasScore) -> FtsMatchConfidence {
    if score.exact_token_match
        || (score.query_coverage >= 1.0
            && score.candidate_coverage >= 1.0
            && score.matched_query_tokens >= 2)
    {
        return FtsMatchConfidence::DirectTitle;
    }
    if score.query_coverage >= 1.0 && score.matched_query_tokens >= 2 {
        return FtsMatchConfidence::StrongLexical;
    }
    if score.query_coverage >= 0.75
        && score.candidate_coverage >= 0.5
        && score.matched_query_tokens >= 2
    {
        return FtsMatchConfidence::StrongLexical;
    }
    if score.query_coverage >= 0.5
        && score.candidate_coverage >= 0.5
        && score.matched_query_tokens >= 2
    {
        return FtsMatchConfidence::MediumLexical;
    }
    FtsMatchConfidence::WeakLexical
}

fn significant_tokens<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    tokens
        .filter(|token| !is_title_stopword(token))
        .collect::<Vec<_>>()
}

fn is_title_stopword(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "at"
            | "by"
            | "for"
            | "from"
            | "in"
            | "into"
            | "of"
            | "on"
            | "or"
            | "the"
            | "to"
            | "with"
    )
}

fn high_value_record_tokens(record: &PersistedRecord) -> Vec<String> {
    let mut tokens = tokenize_fts_query(&record.name);
    tokens.extend(
        record
            .traits
            .iter()
            .flat_map(|value| tokenize_fts_query(value)),
    );
    tokens.extend(
        record
            .taxonomy_families
            .iter()
            .flat_map(|value| tokenize_fts_query(value)),
    );
    tokens.extend(
        record
            .prerequisites
            .iter()
            .flat_map(|value| tokenize_fts_query(value)),
    );
    if let Some(value) = &record.system_category {
        tokens.extend(tokenize_fts_query(value));
    }
    if let Some(value) = &record.system_group {
        tokens.extend(tokenize_fts_query(value));
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

fn token_coverage(needles: &[&str], haystack: &[&str]) -> f64 {
    if needles.is_empty() {
        return 0.0;
    }
    let matched = needles
        .iter()
        .filter(|needle| haystack.iter().any(|token| token_matches(needle, token)))
        .count();
    matched as f64 / needles.len() as f64
}

fn token_matches(left: &str, right: &str) -> bool {
    left == right
        || (left.len() >= 4 && right.starts_with(left))
        || (right.len() >= 4 && left.starts_with(right))
}

fn title_alias_token_matches(query: &str, candidate: &str) -> bool {
    query == candidate || (query.len() >= 3 && candidate.starts_with(query))
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
