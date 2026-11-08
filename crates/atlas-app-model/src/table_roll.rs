use crate::{RecordDetailRequest, RecordDetailView, RecordNavigationView};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollTableRequest {
    pub selection: RecordDetailRequest,
    pub artifact_generation: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableRollView {
    pub outcome: atlas_dice::RollOutcome,
    pub matches: Vec<RecordDetailView>,
    pub invalid_ranges: Vec<RecordNavigationView>,
}
