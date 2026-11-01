//! Explicit, source-independent encounter countdowns.
use crate::ConditionDurationView;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterManualTrackerView {
    pub tracker_key: String,
    pub name: String,
    pub duration: ConditionDurationView,
    pub remaining_rounds: i64,
    pub timing_label: String,
    pub timing_available: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterManualTrackerAction {
    Add {
        name: String,
        duration: ConditionDurationView,
    },
    Edit {
        tracker_key: String,
        name: String,
        duration: ConditionDurationView,
    },
    Restart {
        tracker_key: String,
    },
    Remove {
        tracker_key: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterManualTrackerRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    pub participant_key: String,
    pub action: EncounterManualTrackerAction,
}
