use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::RecordSurfaceView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSummaryView {
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rarity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<RecordBadgeView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub taxonomy: Vec<RecordBadgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub publication: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub surface: Option<RecordSurfaceView>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordBadgeView {
    pub kind: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordDetailView {
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub presentation: atlas_record::RecordPresentationDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub surface: Option<RecordSurfaceView>,
}
