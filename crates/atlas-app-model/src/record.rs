use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSummaryView {
    pub surface: crate::RecordSurfaceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordResolutionAmbiguousView {
    pub record_ref: String,
    pub matches: Vec<RecordResolutionCandidateView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordResolutionCandidateView {
    pub record: RecordSummaryView,
    pub query: String,
    pub normalized_query: String,
    pub match_kind: String,
    pub matched_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordDetailView {
    pub surface: crate::RecordSurfaceView,
}
