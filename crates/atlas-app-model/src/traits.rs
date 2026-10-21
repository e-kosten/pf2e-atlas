//! Addressable vocabulary reading, distinct from Foundry record identity.
use crate::{
    RecordNavigationView, RecordSummaryView, SearchCoverageView, SearchPageRequest, SearchPageView,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TraitDetailRequest {
    pub trait_key: String,
    pub expected_artifact_generation: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TraitDetailView {
    pub artifact_generation: String,
    pub trait_key: String,
    pub label: String,
    pub content: Option<TraitContentView>,
    pub catalogs: Vec<String>,
    pub source_revision: Option<String>,
    pub indexing_locale: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TraitAssociationKind {
    Members,
    References,
    Similar,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TraitRecordsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sort: Option<atlas_domain::ResultSort>,
    pub trait_key: String,
    pub kind: TraitAssociationKind,
    pub page: SearchPageRequest,
    pub expected_artifact_generation: String,
    pub search_profile: Option<atlas_domain::SearchProfileId>,
    pub ranking: Option<atlas_domain::SearchRanking>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TraitRecordView {
    pub source_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reference_direction: Option<String>,
    pub record: RecordSummaryView,
    pub navigation: RecordNavigationView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TraitRecordsPage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sort: Option<atlas_domain::ResultSort>,
    pub search_profile: Option<atlas_domain::SearchProfileId>,
    pub artifact_generation: String,
    pub trait_key: String,
    pub kind: TraitAssociationKind,
    pub rows: Vec<TraitRecordView>,
    pub page: SearchPageView,
    pub coverage: Option<SearchCoverageView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TraitContentView {
    pub body: crate::PreparedFieldBodyView,
    pub references: Vec<crate::PreparedReferenceView>,
}
