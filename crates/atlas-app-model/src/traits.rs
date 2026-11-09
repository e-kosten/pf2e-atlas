//! Addressable vocabulary reading, distinct from Foundry record identity.
use crate::{RecordNavigationView, RecordSummaryView, SearchPageRequest, SearchPageView};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitDetailRequest {
    pub trait_key: String,
    pub expected_artifact_generation: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDetailView {
    pub artifact_generation: String,
    pub trait_key: String,
    pub label: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub content: Option<crate::ReferenceContentView>,
    pub catalogs: Vec<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_revision: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub indexing_locale: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraitAssociationKind {
    Members,
    References,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitRecordsRequest {
    pub trait_key: String,
    pub kind: TraitAssociationKind,
    pub page: SearchPageRequest,
    pub expected_artifact_generation: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitRecordView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference_direction: Option<String>,
    pub record: RecordSummaryView,
    pub navigation: RecordNavigationView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitRecordsPage {
    pub artifact_generation: String,
    pub trait_key: String,
    pub kind: TraitAssociationKind,
    pub rows: Vec<TraitRecordView>,
    pub page: SearchPageView,
}
