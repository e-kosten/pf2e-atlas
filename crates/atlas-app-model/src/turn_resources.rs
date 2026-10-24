//! Manual resource controls and the recorded allowance of one actual turn.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct TurnResourcesView {
    pub turn_key: Option<String>,
    pub round_number: Option<i64>,
    pub regular_total: Option<i64>,
    pub regular_remaining: Option<i64>,
    pub extra_total: Option<i64>,
    pub extra_remaining: Option<i64>,
    pub extra_uses: Vec<crate::ConditionExtraActionView>,
    pub reactions_total: Option<i64>,
    pub reactions_remaining: Option<i64>,
    pub start_results: Vec<String>,
    pub pending_reason: Option<String>,
    pub forecast: crate::RuntimeCountView,
    pub can_edit_actions: bool,
    pub can_spend_actions: crate::RuntimeCapabilityView,
    pub can_spend_reactions: crate::RuntimeCapabilityView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnResourceAction {
    Spend {
        resource: TurnResourceKind,
    },
    Restore {
        resource: TurnResourceKind,
    },
    Regain,
    Correct {
        resource: TurnResourceKind,
        remaining: i64,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TurnResourceKind {
    Action,
    ExtraAction,
    Reaction,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct UpdateTurnResourcesRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub turn_key: Option<String>,
    pub action: TurnResourceAction,
}
