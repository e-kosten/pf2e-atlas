use std::collections::{BTreeMap, BTreeSet};

use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::{FtsQuery, FtsSearchHit};
use atlas_record::PersistedRecord;
use serde::{Deserialize, Serialize};

use crate::fusion::{
    DEFAULT_FTS_FUSION_POLICY, FusionInput, FusionMethod, FusionOptions, TextSearchExplain,
    fuse_ranked_hits, identity_explain,
};
use crate::query::{TextQueryAnalysis, analyze_text_query};
use crate::resolution::{RecordResolutionMatchKind, RecordResolutionResult};
use crate::semantic::{SemanticSearchHit, SemanticSearchMode};
use crate::{AtlasRetrievalService, SearchError};

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

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRequest<'a> {
    pub query: &'a str,
    pub exclude: Option<&'a str>,
    pub filter: Option<&'a SearchFilterNode>,
    pub limit: u32,
    pub offset: u32,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub fts_top_k: u32,
    pub vector_top_k: u32,
    pub explain: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchPage {
    pub query: TextQueryAnalysis,
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub records: Vec<TextSearchRecord>,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRecord {
    pub record: PersistedRecord,
    pub match_info: TextSearchMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSearchMatch {
    Identity {
        retrieval: RetrievalMode,
        identity_match_kind: RecordResolutionMatchKind,
        explain: Option<TextSearchExplain>,
    },
    Ranked {
        retrieval: RetrievalMode,
        explain: Option<TextSearchExplain>,
    },
}

#[derive(Debug, Clone)]
pub enum AtlasSearchRequest<'a> {
    Semantic {
        query: &'a str,
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
        mode: SemanticSearchMode,
    },
    FilterOnly {
        filter: Option<&'a SearchFilterNode>,
        limit: u32,
    },
    Text(TextSearchRequest<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtlasSearchResult {
    Semantic(crate::semantic::SemanticSearchResult),
    Text(TextSearchPage),
}

impl AtlasRetrievalService {
    pub fn text_search(
        &mut self,
        request: TextSearchRequest<'_>,
    ) -> Result<TextSearchPage, SearchError> {
        validate_text_search_request(&request)?;
        let resolved_filter = self.index.resolve_metric_filters(request.filter)?;
        let filter = resolved_filter.as_ref().or(request.filter);
        let query = analyze_text_query(request.query, request.exclude);
        let fts_query = FtsQuery::from_tokens(query.fts_tokens.clone());
        let exclude_query = FtsQuery::from_tokens(query.exclude_tokens.clone());
        let identity_matches = self.resolve_record(request.query, filter)?;
        let fts_hits = if request.retrieval.uses_fts() {
            match fts_query.as_ref() {
                Some(fts_query) => {
                    self.index
                        .query_precision_fts_index(fts_query, filter, request.fts_top_k)?
                }
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let vector_hits = if request.retrieval.uses_vector() {
            self.semantic(
                request.query,
                filter,
                request.vector_top_k,
                SemanticSearchMode::WeightedChunks,
            )?
        } else {
            Vec::new()
        };
        let excluded_keys = match exclude_query.as_ref() {
            Some(exclude_query) => self
                .index
                .query_fts_candidate_record_keys(
                    exclude_query,
                    &candidate_keys(&identity_matches, &fts_hits, &vector_hits),
                )?
                .into_iter()
                .collect::<BTreeSet<_>>(),
            None => BTreeSet::new(),
        };
        let identity_matches = identity_matches
            .into_iter()
            .filter(|identity| !excluded_keys.contains(&identity.record.key))
            .collect::<Vec<_>>();
        let identity_keys = identity_matches
            .iter()
            .map(|identity| identity.record.key.clone())
            .collect::<BTreeSet<_>>();
        let fusion_candidate_keys = candidate_keys(&identity_matches, &fts_hits, &vector_hits)
            .into_iter()
            .filter(|key| !excluded_keys.contains(key))
            .collect::<Vec<_>>();
        let candidate_records = self.index.load_records_by_key(&fusion_candidate_keys)?;
        let records_by_key = candidate_records
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let fused = fuse_ranked_hits(FusionInput {
            fts_hits: &fts_hits,
            vector_hits: &vector_hits,
            records_by_key: &records_by_key,
            fts_tokens: &query.fts_tokens,
            identity_keys: &identity_keys,
            excluded_keys: &excluded_keys,
            retrieval: request.retrieval,
            fusion: request.fusion,
            fts_policy: DEFAULT_FTS_FUSION_POLICY,
            explain: request.explain,
            identity_count: identity_matches.len(),
        });
        let total = identity_matches.len() + fused.len();
        let mut all_matches = identity_matches
            .into_iter()
            .enumerate()
            .map(|(index, identity)| TextSearchRecord {
                record: identity.record,
                match_info: TextSearchMatch::Identity {
                    retrieval: request.retrieval,
                    identity_match_kind: identity.match_kind,
                    explain: request.explain.then(|| identity_explain(index)),
                },
            })
            .collect::<Vec<_>>();

        all_matches.extend(fused.into_iter().filter_map(|ranked| {
            records_by_key
                .get(&ranked.record_key)
                .map(|record| TextSearchRecord {
                    record: record.clone(),
                    match_info: TextSearchMatch::Ranked {
                        retrieval: request.retrieval,
                        explain: ranked.explain,
                    },
                })
        }));

        let mut page_records = all_matches
            .into_iter()
            .skip(request.offset as usize)
            .take(request.limit as usize)
            .collect::<Vec<_>>();
        self.enrich_text_record_reference_labels(&mut page_records)?;

        Ok(TextSearchPage {
            query,
            retrieval: request.retrieval,
            fusion: request.fusion,
            records: page_records,
            total: total as u64,
        })
    }
}

fn validate_text_search_request(request: &TextSearchRequest<'_>) -> Result<(), SearchError> {
    if let Some(filter) = request.filter {
        filter
            .validate()
            .map_err(|error| SearchError::InvalidSearchOptions(error.to_string()))?;
    }
    if request.fusion.method == FusionMethod::Rrf
        && ((request.fusion.fts_weight - 1.0).abs() > f64::EPSILON
            || (request.fusion.vector_weight - 1.0).abs() > f64::EPSILON)
    {
        return Err(SearchError::InvalidSearchOptions(
            "unweighted rrf does not accept lane weights; use weighted-rrf".to_string(),
        ));
    }
    if request.fusion.rank_constant <= 0.0
        || request.fusion.fts_weight < 0.0
        || request.fusion.vector_weight < 0.0
    {
        return Err(SearchError::InvalidSearchOptions(
            "fusion weights must be non-negative and rank constant must be greater than zero"
                .to_string(),
        ));
    }
    Ok(())
}

fn candidate_keys(
    identity_matches: &[RecordResolutionResult],
    fts_hits: &[FtsSearchHit],
    vector_hits: &[SemanticSearchHit],
) -> Vec<RecordKey> {
    let mut keys = identity_matches
        .iter()
        .map(|identity| identity.record.key.clone())
        .collect::<BTreeSet<_>>();
    keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    keys.extend(
        vector_hits
            .iter()
            .filter_map(|hit| RecordKey::parse(&hit.record_key).ok()),
    );
    keys.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unweighted_rrf_rejects_lane_weights_at_runtime_boundary() {
        let request = TextSearchRequest {
            query: "healing",
            exclude: None,
            filter: None,
            limit: 10,
            offset: 0,
            retrieval: RetrievalMode::Fts,
            fusion: FusionOptions {
                method: FusionMethod::Rrf,
                fts_weight: 2.0,
                vector_weight: 1.0,
                rank_constant: 60.0,
            },
            fts_top_k: 10,
            vector_top_k: 10,
            explain: false,
        };

        assert!(matches!(
            validate_text_search_request(&request),
            Err(SearchError::InvalidSearchOptions(_))
        ));
    }
}
