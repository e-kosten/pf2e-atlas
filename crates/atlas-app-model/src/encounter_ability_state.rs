//! Source-defined encounter ability switches, choices and limited uses.
use crate::RecordNavigationView;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterAbilityStateView {
    pub navigation: RecordNavigationView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub rule_ordinal: Option<usize>,
    pub label: String,
    pub state: EncounterAbilityStateKindView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterAbilityStateKindView {
    Option {
        active: bool,
        always_active: bool,
        #[cfg_attr(feature = "openapi", schema(required))]
        selection: Option<String>,
        choices: Vec<EncounterAbilityChoiceView>,
    },
    Uses {
        remaining: i64,
        maximum: i64,
        interval: String,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterAbilityChoiceView {
    pub value: String,
    pub label: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub consequence: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterAbilityStateAction {
    SetOption {
        active: bool,
        selection: Option<String>,
    },
    SetConditionValue {
        value: Option<i64>,
    },
    SpendUse,
    RestoreUse,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterAbilityStateRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    pub participant_key: String,
    pub navigation: RecordNavigationView,
    pub rule_ordinal: Option<usize>,
    pub action: EncounterAbilityStateAction,
}
