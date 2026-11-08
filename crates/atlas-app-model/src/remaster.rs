use crate::{RecordNavigationView, RelatedCategoryView};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemasterRelationshipsView {
    pub relationships: Vec<RemasterRelationshipView>,
    pub truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemasterRelationshipView {
    pub id: String,
    pub label: String,
    pub parts: Vec<RemasterInlinePartView>,
    pub explanation_links: Vec<RemasterExplanationLinkView>,
    pub truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemasterExplanationLinkView {
    pub label: String,
    pub navigation: RecordNavigationView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemasterSourceContextView {
    pub heading: String,
    pub navigation: RecordNavigationView,
    pub relationships: RelatedCategoryView<RemasterRelationshipsView>,
}
