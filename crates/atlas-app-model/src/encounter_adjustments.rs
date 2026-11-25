//! GM-entered participant modifiers, available target choices, and guarded previews.
use crate::ConditionDurationView;
use serde::{Deserialize, Serialize};
use serde_json::Number;

pub use atlas_domain::{AdjustmentModifierType, AdjustmentTarget};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentTargetView {
    pub target: AdjustmentTarget,
    pub label: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentTimingView {
    pub duration: ConditionDurationView,
    /// Rounds for round durations, exact seconds for all other durations.
    pub remaining: i64,
    pub anchor_available: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentView {
    pub adjustment_key: String,
    pub target: AdjustmentTarget,
    pub label: String,
    pub target_available: bool,
    pub name: String,
    pub amount: i64,
    pub modifier_type: AdjustmentModifierType,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub timing: Option<EncounterAdjustmentTimingView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentsView {
    pub entries: Vec<EncounterAdjustmentView>,
    pub targets: Vec<EncounterAdjustmentTargetView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterAdjustmentTimingInput {
    pub duration: ConditionDurationView,
    /// Set remaining rounds or seconds explicitly. Omit to preserve progress on edits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterAdjustmentInput {
    pub target: AdjustmentTarget,
    pub name: String,
    pub amount: i64,
    pub modifier_type: AdjustmentModifierType,
    pub timing: Option<EncounterAdjustmentTimingInput>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterAdjustmentAction {
    Add {
        value: EncounterAdjustmentInput,
    },
    Edit {
        adjustment_key: String,
        value: EncounterAdjustmentInput,
    },
    End {
        adjustment_key: String,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterAdjustmentRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_reference_revision: Option<u64>,
    pub action: EncounterAdjustmentAction,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentPreviewRow {
    pub label: String,
    #[cfg_attr(feature = "openapi", schema(required, value_type = Option<f64>))]
    pub before: Option<Number>,
    #[cfg_attr(feature = "openapi", schema(required, value_type = Option<f64>))]
    pub after: Option<Number>,
    pub suppressed: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustmentPreviewView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference_revision: Option<u64>,
    pub rows: Vec<EncounterAdjustmentPreviewRow>,
}
