//! Explicit, source-independent encounter countdowns.
use crate::ConditionDurationView;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterManualTrackerView {
    pub tracker_key: String,
    pub name: String,
    pub duration: ConditionDurationView,
    pub remaining_rounds: i64,
    pub timing_label: String,
    pub timing_available: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterManualTrackerRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    pub action: EncounterManualTrackerAction,
}
