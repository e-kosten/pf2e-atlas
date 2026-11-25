use crate::{
    PreparedContentFieldView, RecordNavigationView, RecordRelationshipView, RecordSelection,
};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollTableRequest {
    pub selection: RecordSelection,
    pub artifact_generation: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRollView {
    pub outcome: atlas_dice::RollOutcome,
    pub matches: Vec<TableRollMatchView>,
    pub invalid_ranges: Vec<RecordNavigationView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRollMatchView {
    pub title: String,
    pub navigation: RecordNavigationView,
    pub content: Vec<PreparedContentFieldView>,
    pub relationships: Vec<RecordRelationshipView>,
}
