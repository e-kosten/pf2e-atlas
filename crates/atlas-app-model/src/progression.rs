use crate::RecordNavigationView;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionInlineLinkView {
    pub field: String,
    pub text: String,
    pub navigation: RecordNavigationView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordProgressionHintView {
    pub inline_links: Vec<ProgressionInlineLinkView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressionDirection {
    #[default]
    Overview,
    RequiredBy,
    Pool,
    AvailableThrough,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionRequest {
    pub selection: RecordNavigationView,
    pub artifact_generation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub direction: Option<ProgressionDirection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub trait_key: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressionRequirementKind {
    AllOf,
    AnyOf,
    Record,
    Proficiency,
    AttributeMinimum,
    Unresolved,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionRequirementView {
    pub kind: ProgressionRequirementKind,
    pub depth: usize,
    pub label: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub navigation: Option<RecordNavigationView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionWitnessView {
    pub path: String,
    pub alternative: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionEntryView {
    pub navigation: RecordNavigationView,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub level: Option<u8>,
    pub requirements: Vec<ProgressionRequirementView>,
    pub witnesses: Vec<ProgressionWitnessView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgressionPageView {
    pub entries: Vec<ProgressionEntryView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub next_offset: Option<usize>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionPoolView {
    pub trait_key: String,
    pub label: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionView {
    pub selection: RecordNavigationView,
    pub artifact_generation: String,
    pub prerequisites: Vec<ProgressionRequirementView>,
    pub required_by: ProgressionPageView,
    pub ancestry_feat_pools: Vec<ProgressionPoolView>,
    pub pool: ProgressionPageView,
    pub available_through: ProgressionPageView,
}
