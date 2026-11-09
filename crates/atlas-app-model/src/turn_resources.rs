//! Manual resource controls, including a reaction granted before the first turn.
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnResourcesView {
    pub action_calculation: TurnActionCalculationView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub inherent_conditions: Option<InherentActionConditionsView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub turn_key: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub round_number: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub regular_total: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub regular_remaining: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub extra_total: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub extra_remaining: Option<i64>,
    pub extra_uses: Vec<crate::ConditionExtraActionView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub reactions_total: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub reactions_remaining: Option<i64>,
    pub can_grant_reaction: bool,
    pub start_results: Vec<TurnResourceResultView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub pending_reason: Option<String>,
    pub forecast: crate::RuntimeCountView,
    pub can_edit_actions: bool,
    pub can_spend_actions: crate::RuntimeCapabilityView,
    pub can_spend_reactions: crate::RuntimeCapabilityView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnActionCalculationView {
    ThisTurn {
        #[cfg_attr(feature = "openapi", schema(required))]
        count: Option<crate::RuntimeCountView>,
        changes: Vec<crate::RuntimeAdjustmentView>,
    },
    NextTurn {
        count: crate::RuntimeCountView,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InherentActionConditionsView {
    pub entries: Vec<InherentActionConditionView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable_reason: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InherentActionConditionView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub unavailable_reason: Option<String>,
    pub name: String,
    pub has_value: bool,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub value: Option<i64>,
    pub condition_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub editable_rule_ordinal: Option<usize>,
    pub source_name: String,
    pub source: crate::RecordNavigationView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnResourceResultView {
    pub condition_name: String,
    pub condition_ref: String,
    pub outcome: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnResourceAction {
    GrantReaction,
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnResourceKind {
    Action,
    ExtraAction,
    Reaction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTurnResourcesRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub turn_key: Option<String>,
    pub action: TurnResourceAction,
}
