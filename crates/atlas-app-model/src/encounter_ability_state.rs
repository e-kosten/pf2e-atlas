//! Source-defined encounter ability switches, choices and limited uses.
use crate::RecordNavigationView;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterAbilityStateView {
    pub navigation: RecordNavigationView,
    pub rule_ordinal: Option<usize>,
    pub label: String,
    pub state: EncounterAbilityStateKindView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterAbilityStateKindView {
    Option {
        active: bool,
        always_active: bool,
        selection: Option<String>,
        choices: Vec<EncounterAbilityChoiceView>,
    },
    Uses {
        remaining: i64,
        maximum: i64,
        interval: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterAbilityChoiceView {
    pub value: String,
    pub label: String,
    pub consequence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterAbilityStateAction {
    SetOption {
        active: bool,
        selection: Option<String>,
    },
    SpendUse,
    RestoreUse,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
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
