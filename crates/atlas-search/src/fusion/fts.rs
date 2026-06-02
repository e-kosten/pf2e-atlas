use atlas_index::{FtsSearchHit, FtsSearchLane, SearchCandidateRecord};
use serde::{Deserialize, Serialize};

use crate::query::tokenize_fts_query;

pub(crate) const DEFAULT_FTS_FUSION_POLICY: FtsFusionPolicy = FtsFusionPolicy::DemoteWeak;
pub(crate) const DEFAULT_FTS_FUSION_POLICY_LABEL: &str = DEFAULT_FTS_FUSION_POLICY.as_str();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FtsFusionPolicy {
    All,
    DemoteWeak,
    StrongOnly,
}

impl FtsFusionPolicy {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::DemoteWeak => "demote-weak",
            Self::StrongOnly => "strong-only",
        }
    }

    pub(super) fn apply(self, confidence: FtsMatchConfidence, score: f64) -> f64 {
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

pub(super) fn confidence_score(confidence: FtsMatchConfidence) -> u8 {
    match confidence {
        FtsMatchConfidence::DirectTitle => 4,
        FtsMatchConfidence::StrongLexical => 3,
        FtsMatchConfidence::MediumLexical => 2,
        FtsMatchConfidence::WeakLexical => 1,
    }
}

pub(super) fn fts_lane_weight(lane: FtsSearchLane) -> f64 {
    match lane {
        FtsSearchLane::Mixed | FtsSearchLane::TitleAlias => 1.0,
        FtsSearchLane::Facet => 0.35,
    }
}

pub(super) fn fts_confidence_weight(lane: FtsSearchLane, confidence: FtsMatchConfidence) -> f64 {
    if lane != FtsSearchLane::TitleAlias {
        return 1.0;
    }
    match confidence {
        FtsMatchConfidence::DirectTitle => 2.0,
        FtsMatchConfidence::StrongLexical => 1.5,
        FtsMatchConfidence::MediumLexical | FtsMatchConfidence::WeakLexical => 1.0,
    }
}

pub(super) fn classify_fts_hit(
    hit: &FtsSearchHit,
    record: &SearchCandidateRecord,
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
    record: &'a SearchCandidateRecord,
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

pub(super) fn effective_fts_rank(
    rank: u32,
    lane: FtsSearchLane,
    confidence: FtsMatchConfidence,
) -> u32 {
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
    if score.exact_token_match || (score.query_coverage >= 1.0 && score.candidate_coverage >= 0.8) {
        return FtsMatchConfidence::DirectTitle;
    }
    if score.query_coverage >= 0.8
        && score.candidate_coverage >= 0.5
        && score.matched_query_tokens >= 2
    {
        return FtsMatchConfidence::StrongLexical;
    }
    if score.query_coverage >= 0.5 && score.matched_query_tokens >= 2 {
        return FtsMatchConfidence::MediumLexical;
    }
    FtsMatchConfidence::WeakLexical
}

fn title_alias_token_matches(query: &str, candidate: &str) -> bool {
    query == candidate
        || (query.len() >= 3 && candidate.starts_with(query))
        || (candidate.len() >= 3 && query.starts_with(candidate))
}

fn significant_tokens<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    tokens.filter(|token| token.len() > 1).collect()
}

fn high_value_record_tokens(record: &SearchCandidateRecord) -> Vec<String> {
    let mut tokens = Vec::new();
    tokens.extend(tokenize_fts_query(&record.name));
    tokens.extend(
        record
            .traits
            .iter()
            .flat_map(|value| tokenize_fts_query(value)),
    );
    tokens.extend(tokenize_fts_query(record.kind.as_str()));
    tokens.extend(tokenize_fts_query(&record.foundry_record_type));
    tokens.extend(
        record
            .taxonomy_families
            .iter()
            .flat_map(|value| tokenize_fts_query(value)),
    );
    if let Some(category) = &record.system_category {
        tokens.extend(tokenize_fts_query(category));
    }
    if let Some(group) = &record.system_group {
        tokens.extend(tokenize_fts_query(group));
    }
    tokens
}

fn token_coverage(query_tokens: &[&str], candidate_tokens: &[&str]) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }
    let matched = query_tokens
        .iter()
        .filter(|query| candidate_tokens.iter().any(|candidate| *query == candidate))
        .count();
    matched as f64 / query_tokens.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::RecordKind;

    fn test_record_key(value: &str) -> atlas_domain::RecordKey {
        atlas_domain::RecordKey::parse(value).expect("fixture record key should parse")
    }

    fn test_record(key: &str, name: &str, traits: &[&str]) -> SearchCandidateRecord {
        let key = test_record_key(key);
        SearchCandidateRecord {
            key,
            name: name.to_string(),
            foundry_record_type: "spell".to_string(),
            traits: traits.iter().map(|value| (*value).to_string()).collect(),
            kind: RecordKind::Spell,
            taxonomy_families: Vec::new(),
            system_category: None,
            system_group: None,
        }
    }

    #[test]
    fn fts_confidence_distinguishes_direct_and_weak_hits() {
        let direct = test_record("records:a", "Treat Wounds", &["healing"]);
        let strong = test_record("records:b", "Battle Medicine", &["healing", "manipulate"]);
        let weak = test_record("records:c", "Shielded Arm", &["metal"]);
        let mut medium = test_record("records:d", "Shielded Arm", &["mental"]);
        medium.taxonomy_families = vec!["action".to_string()];
        medium.system_category = Some("consumable".to_string());
        medium.system_group = Some("alchemical".to_string());

        assert_eq!(
            classify_fts_texts(
                FtsSearchLane::Mixed,
                &direct,
                &["treat".to_string(), "wounds".to_string()],
                std::iter::empty::<&str>(),
            ),
            FtsMatchConfidence::DirectTitle
        );
        assert_eq!(
            classify_fts_texts(
                FtsSearchLane::Facet,
                &strong,
                &["healing".to_string(), "manipulate".to_string()],
                std::iter::empty::<&str>(),
            ),
            FtsMatchConfidence::StrongLexical
        );
        assert_eq!(
            classify_fts_texts(
                FtsSearchLane::Facet,
                &medium,
                &[
                    "mental".to_string(),
                    "alchemical".to_string(),
                    "missing".to_string(),
                    "other".to_string(),
                ],
                std::iter::empty::<&str>(),
            ),
            FtsMatchConfidence::MediumLexical
        );
        assert_eq!(
            classify_fts_texts(
                FtsSearchLane::Facet,
                &weak,
                &[
                    "low".to_string(),
                    "level".to_string(),
                    "fear".to_string(),
                    "spell".to_string()
                ],
                std::iter::empty::<&str>(),
            ),
            FtsMatchConfidence::WeakLexical
        );
    }

    #[test]
    fn fts_fusion_policy_weights_confidence_levels() {
        assert_eq!(
            FtsFusionPolicy::All.apply(FtsMatchConfidence::WeakLexical, 10.0),
            10.0
        );
        assert_eq!(
            FtsFusionPolicy::DemoteWeak.apply(FtsMatchConfidence::MediumLexical, 10.0),
            5.0
        );
        assert_eq!(
            FtsFusionPolicy::DemoteWeak.apply(FtsMatchConfidence::WeakLexical, 10.0),
            1.0
        );
        assert_eq!(
            FtsFusionPolicy::StrongOnly.apply(FtsMatchConfidence::MediumLexical, 10.0),
            0.0
        );
        assert_eq!(
            FtsFusionPolicy::StrongOnly.apply(FtsMatchConfidence::StrongLexical, 10.0),
            10.0
        );
    }
}
