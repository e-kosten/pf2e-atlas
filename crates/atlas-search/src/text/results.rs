use std::collections::BTreeSet;

use atlas_domain::RecordKey;
use atlas_index::FtsSearchHit;
use atlas_record::PersistedRecord;

use crate::FusionOptions;
use crate::fusion::{FusedRankedHit, TextSearchExplain, identity_explain};
use crate::query::TextQueryAnalysis;
use crate::records::{RecordResolutionMatchKind, RecordResolutionResult};
use crate::semantic::SemanticSearchHit;

use super::request::RetrievalMode;

#[derive(Debug, Clone, PartialEq)]
pub struct TextSearchResult {
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
                explain: explain.then(|| identity_explain(index)),
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
        .map(|identity| identity.record.key.clone())
        .collect::<BTreeSet<_>>();
    keys.extend(fts_hits.iter().map(|hit| hit.record_key.clone()));
    keys.extend(vector_hits.iter().map(|hit| hit.record_key.clone()));
    keys.into_iter().collect()
}
