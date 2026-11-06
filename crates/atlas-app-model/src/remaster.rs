use crate::{RecordNavigationView, RelatedCategoryView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterRelationshipsView {
    pub relationships: Vec<RemasterRelationshipView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterRelationshipView {
    pub id: String,
    pub label: String,
    pub parts: Vec<RemasterInlinePartView>,
    pub explanation_links: Vec<RemasterExplanationLinkView>,
    pub truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterExplanationLinkView {
    pub label: String,
    pub navigation: RecordNavigationView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemasterInlinePartView {
    Text {
        text: String,
    },
    Reference {
        label: String,
        navigation: Box<RecordNavigationView>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RemasterSourceContextView {
    pub heading: String,
    pub navigation: RecordNavigationView,
    pub relationships: RelatedCategoryView<RemasterRelationshipsView>,
}
