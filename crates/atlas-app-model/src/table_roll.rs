use crate::{RecordDetailRequest, RecordDetailView, RecordNavigationView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollTableRequest {
    pub selection: RecordDetailRequest,
    pub artifact_generation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TableRollView {
    pub outcome: atlas_dice::RollOutcome,
    pub matches: Vec<RecordDetailView>,
    pub invalid_ranges: Vec<RecordNavigationView>,
}
