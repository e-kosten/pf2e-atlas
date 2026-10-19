//! CLI reading contract over shared semantic facts; no source reinterpretation.
use atlas_app_model::{
    EmbeddedRecordLinkView, PreparedContentFieldView, RecordBodyView, RecordDetailView,
    RecordNavigationView, RecordRelationshipView, RecordSummaryView,
};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct RecordResponse<'a> {
    pub artifact_generation: &'a str,
    /// Identity of the selected content, including effective rank-preview facts.
    pub identity: &'a RecordSummaryView,
    /// Embedded selections retain their containing record without repeating root identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<&'a RecordSummaryView>,
    pub selected: &'a RecordNavigationView,
    pub content: RecordContent<'a>,
    pub relationships: &'a [RecordRelationshipView],
    pub relationships_truncated: bool,
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
        }
    }
}
