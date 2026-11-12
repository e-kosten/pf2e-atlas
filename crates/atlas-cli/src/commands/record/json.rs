//! CLI reading contract over shared semantic facts; no source reinterpretation.
use atlas_app_model::{
    EmbeddedRecordLinkView, PreparedContentFieldView, RecordBodyView, RecordDetailView,
    RecordNavigationView, RecordRelationshipView, RecordSummaryView,
};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct RecordResponse<'a> {
    pub artifact_generation: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_record: Option<&'a atlas_app_model::CanonicalRecordView>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    pub alternate_publications: &'a [String],
    /// Identity of the selected content, including effective rank-preview facts.
    pub identity: &'a RecordSummaryView,
    /// Embedded selections retain their containing record without repeating root identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<&'a RecordSummaryView>,
    pub selected: &'a RecordNavigationView,
    pub content: RecordContent<'a>,
    pub relationships: &'a [RecordRelationshipView],
    pub relationships_truncated: bool,
    pub remaster_relationships:
        &'a atlas_app_model::RelatedCategoryView<atlas_app_model::RemasterRelationshipsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_remaster_context: Option<
        &'a atlas_app_model::RelatedCategoryView<atlas_app_model::RemasterSourceContextView>,
    >,
}

#[derive(Serialize)]
pub(super) struct RecordContent<'a> {
    pub classification: &'a str,
    pub fields: &'a [PreparedContentFieldView],
    #[serde(rename = "embedded")]
    pub owned: &'a [EmbeddedRecordLinkView],
    pub body: &'a RecordBodyView,
}

impl<'a> From<&'a RecordDetailView> for RecordResponse<'a> {
    fn from(detail: &'a RecordDetailView) -> Self {
        Self {
            artifact_generation: &detail.artifact_generation,
            canonical_record: detail.canonical_record.as_ref(),
            alternate_publications: &detail.alternate_publications,
            identity: &detail.presentation.identity,
            parent: (!detail.selected.owners.is_empty()).then_some(&detail.record),
            selected: &detail.selected,
            content: RecordContent {
                classification: &detail.presentation.classification,
                fields: &detail.presentation.content,
                owned: &detail.presentation.owned,
                body: &detail.presentation.body,
            },
            relationships: &detail.relationships,
            relationships_truncated: detail.relationships_truncated,
            remaster_relationships: &detail.remaster_relationships,
            source_remaster_context: detail.source_remaster_context.as_ref(),
        }
    }
}
