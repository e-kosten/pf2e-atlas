use crate::{
    RecordRelationshipView, RecordSummaryView, SearchCoverageView, SearchPageView, SearchResultRow,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct GraphContextView {
    pub seed: RecordSummaryView,
    pub outgoing: GraphSectionView,
    pub backlinks: GraphSectionView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct GraphSectionView {
    pub records: Vec<RecordSummaryView>,
    pub occurrences: Vec<RecordRelationshipView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterLinksView {
    pub seed: RecordSummaryView,
    pub links: Vec<RemasterLinkView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterLinkView {
    pub legacy_record: RecordSummaryView,
    pub remaster_record: RecordSummaryView,
    pub evidence: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct VariantGroupView {
    pub seed: RecordSummaryView,
    pub base_name: String,
    pub variants: Vec<RecordSummaryView>,
    pub evidence: Vec<VariantEvidenceView>,
    pub ambiguous: bool,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct VariantEvidenceView {
    pub record_key: String,
    pub naming_convention: String,
    pub qualifier: String,
    pub compatibility: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SearchResultsView {
    pub search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: String,
    pub rows: Vec<SearchResultRow>,
    pub page: SearchPageView,
    pub coverage: SearchCoverageView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SimilarRecordsView {
    pub seed: RecordSummaryView,
    pub results: SearchResultsView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordListView {
    pub records: Vec<RecordSummaryView>,
    pub page: SearchPageView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct GraphContextViewRequest {
    pub record_key: String,
    #[serde(rename = "embedded_path")]
    pub owners: Option<Vec<atlas_record::source_content::OwnedContentLocator>>,
    pub source_fingerprint: Option<String>,
    pub outgoing_limit: usize,
    pub backlink_limit: usize,
}

/// Loading belongs to the client request lifecycle; errors and unavailable evidence are not empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelatedCategoryView<T> {
    Content { content: T },
    Empty,
    Unavailable { reason: String },
    Error { error: crate::AppError },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelatedRecordsRequest {
    pub selection: crate::RecordNavigationView,
    pub artifact_generation: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RelatedRecordsView {
    pub selection: crate::RecordNavigationView,
    pub artifact_generation: String,
    pub seed: RecordSummaryView,
    pub references: RelatedCategoryView<GraphContextView>,
    pub counterparts: RelatedCategoryView<RemasterLinksView>,
    pub variants: RelatedCategoryView<VariantGroupView>,
}
