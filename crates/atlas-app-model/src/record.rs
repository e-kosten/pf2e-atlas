use crate::{RecordSurfaceProfileView, RecordSurfaceView};
use atlas_domain::SourcePassageAddress;
use atlas_record::source_content::{OwnedContentLocator, SourceContentLocator};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordSummaryView {
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    pub source_type: Option<String>,
    pub level_label: Option<String>,
    pub level_basis: Option<String>,
    pub rarity: Option<String>,
    pub traits: Vec<RecordBadgeView>,
    pub publication: Option<String>,
    pub pack: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordBadgeView {
    pub kind: String,
    pub label: String,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordNavigationView {
    pub record_key: String,
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub passage: Option<SourcePassageAddress>,
    pub source_fingerprint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordDetailRequest {
    pub record_key: String,
    #[serde(default)]
    pub owners: Vec<OwnedContentLocator>,
    #[serde(default)]
    pub fields: Vec<String>,
    pub passage: Option<SourcePassageAddress>,
    pub source_fingerprint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordDetailView {
    pub record: RecordSummaryView,
    pub surface: RecordSurfaceView,
    pub selected: RecordNavigationView,
    pub relationships: Vec<RecordRelationshipView>,
    pub relationships_truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordRelationshipView {
    pub source: SourceContentLocator,
    pub ordinal: usize,
    pub origin: String,
    pub kind: String,
    pub authored_target: Option<String>,
    pub occurrence_path: Option<String>,
    pub audiences: Vec<String>,
    pub availability: Option<atlas_domain::QueryFieldState>,
    pub status: String,
    pub target: Option<RecordNavigationView>,
    pub url: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordResolutionAmbiguousView {
    pub record_ref: String,
    pub matches: Vec<RecordResolutionCandidateView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordResolutionCandidateView {
    pub record: RecordSummaryView,
    pub query: String,
    pub normalized_query: String,
    pub match_kind: String,
    pub matched_text: String,
    pub evidence: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RecordRefResolutionView {
    Key(atlas_domain::RecordKey),
    Miss,
    Ambiguous(Vec<RecordResolutionCandidateView>),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordSurfaceRequest {
    pub selection: RecordDetailRequest,
    pub profile: RecordSurfaceProfileView,
}
