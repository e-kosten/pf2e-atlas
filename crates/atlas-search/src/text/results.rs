use std::collections::BTreeSet;

use atlas_domain::RecordKey;
use atlas_index::FtsSearchHit;
use atlas_record::AtlasRecord;

use crate::fusion::{FusedRankedHit, FusionOptions, TextSearchExplain, identity_explain};
use crate::page::SearchPageInfo;
use crate::query::TextQueryDiagnostics;
use crate::records::{RecordResolutionMatchKind, RecordResolutionResult};
use crate::semantic::SemanticSearchHit;

use super::request::RetrievalMode;

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchResult {
    pub retrieval: RetrievalMode,
    pub fusion: FusionOptions,
    pub records: Vec<TextSearchRecord>,
    pub total: u64,
    pub page: SearchPageInfo,
    pub diagnostics: Option<TextSearchDiagnostics>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchDiagnostics {
    pub query: TextQueryDiagnostics,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchRecord {
    pub record: AtlasRecord,
    pub match_info: TextSearchMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSearchMatch {
    Identity {
        retrieval: RetrievalMode,
        identity_match_kind: RecordResolutionMatchKind,
        diagnostics: Option<TextSearchMatchDiagnostics>,
    },
    Ranked {
        retrieval: RetrievalMode,
        diagnostics: Option<TextSearchMatchDiagnostics>,
    },
}

impl TextSearchMatch {
    pub const fn diagnostics(&self) -> Option<&TextSearchMatchDiagnostics> {
        match self {
            Self::Identity { diagnostics, .. } | Self::Ranked { diagnostics, .. } => {
                diagnostics.as_ref()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchMatchDiagnostics {
    pub rank: u32,
    pub fused_score: Option<f64>,
    pub fts_rank: Option<u32>,
    pub fts_score: Option<f64>,
    pub fts_lane: Option<&'static str>,
    pub fts_confidence: Option<&'static str>,
    pub vector_rank: Option<u32>,
    pub vector_distance: Option<f64>,
    pub vector_rank_distance: Option<f64>,
    pub vector_unit_kind: Option<String>,
    pub vector_label: Option<String>,
}

impl From<TextSearchExplain> for TextSearchMatchDiagnostics {
    fn from(explain: TextSearchExplain) -> Self {
        Self {
            rank: explain.rank,
            fused_score: explain.fused_score,
            fts_rank: explain.fts_rank,
            fts_score: explain.fts_score,
            fts_lane: explain.fts_lane.map(|lane| lane.as_str()),
            fts_confidence: explain.fts_confidence.map(|confidence| confidence.as_str()),
            vector_rank: explain.vector_rank,
            vector_distance: explain.vector_distance,
            vector_rank_distance: explain.vector_rank_distance,
            vector_unit_kind: explain.vector_unit_kind,
            vector_label: explain.vector_label,
        }
    }
}

pub(super) enum TextSearchResultItem {
    Identity(Box<TextSearchRecord>),
    Ranked(FusedRankedHit),
}

pub(super) fn identity_records(
    identity_matches: Vec<RecordResolutionResult>,
    retrieval: RetrievalMode,
    explain: bool,
) -> Vec<TextSearchRecord> {
    identity_matches
        .into_iter()
        .enumerate()
        .map(|(index, identity)| TextSearchRecord {
            record: identity.record,
            match_info: TextSearchMatch::Identity {
                retrieval,
                identity_match_kind: identity.match_kind,
                diagnostics: explain
                    .then(|| TextSearchMatchDiagnostics::from(identity_explain(index))),
            },
        })
        .collect()
}

pub(super) fn candidate_keys(
    identity_matches: &[RecordResolutionResult],
    fts_hits: &[FtsSearchHit],
    vector_hits: &[SemanticSearchHit],
) -> Vec<RecordKey> {
    let mut keys = identity_matches
        .iter()
        .map(|identity| identity.record.identity.key.clone())
        .collect::<BTreeSet<_>>();
    keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    keys.extend(vector_hits.iter().map(|hit| hit.record_key.clone()));
    keys.into_iter().collect()
}
