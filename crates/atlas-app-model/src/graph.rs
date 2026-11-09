use crate::{
    RecordRelationshipView, RecordSummaryView, SearchCoverageView, SearchPageView, SearchResultRow,
};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphContextView {
    pub seed: RecordSummaryView,
    pub outgoing: GraphSectionView,
    pub backlinks: GraphSectionView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphSectionView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub filter: Option<atlas_domain::QueryPredicate>,
    pub records: Vec<RecordSummaryView>,
    pub occurrences: Vec<RecordRelationshipView>,
    pub truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantGroupView {
    pub seed: RecordSummaryView,
    pub base_name: String,
    pub variants: Vec<RecordSummaryView>,
    pub evidence: Vec<VariantEvidenceView>,
    pub ambiguous: bool,
    pub truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantEvidenceView {
    pub record_key: String,
    pub naming_convention: String,
    pub qualifier: String,
    pub compatibility: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResultsView {
    pub sort: atlas_domain::ResultSort,
    pub search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: String,
    pub rows: Vec<SearchResultRow>,
    pub page: SearchPageView,
    pub coverage: SearchCoverageView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimilarRecordsView {
    pub seed: RecordSummaryView,
    pub results: SearchResultsView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordListView {
    pub records: Vec<RecordSummaryView>,
    pub page: SearchPageView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphContextViewRequest {
    pub record_key: String,
    #[serde(rename = "embedded_path")]
    pub owners: Option<Vec<atlas_domain::OwnedContentLocator>>,
    pub source_fingerprint: Option<String>,
    pub outgoing_limit: usize,
    pub backlink_limit: usize,
}

/// Loading belongs to the client request lifecycle; errors and unavailable evidence are not empty.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelatedCategoryView<T> {
    Content { content: T },
    Empty,
    Unavailable { reason: String },
    Error { error: crate::AppError },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedRecordsRequest {
    pub selection: crate::RecordNavigationView,
    pub artifact_generation: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedRecordsView {
    pub show_versions: bool,
    pub selection: crate::RecordNavigationView,
    pub artifact_generation: String,
    pub seed: RecordSummaryView,
    pub references: RelatedCategoryView<GraphContextView>,
    pub remaster_relationships: RelatedCategoryView<crate::RemasterRelationshipsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub source_remaster_context: Option<RelatedCategoryView<crate::RemasterSourceContextView>>,
    pub variants: RelatedCategoryView<VariantGroupView>,
}
