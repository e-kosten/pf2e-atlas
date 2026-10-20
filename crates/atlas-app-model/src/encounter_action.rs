//! Optional contextual action queries; absence of a target does not block browsing.
use crate::{EncounterActionCheckView, EncounterActionIssueView, EncounterActionJudgmentView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterRetchPreviewRequest {
    pub participant_key: String,
    pub judgments: Vec<EncounterActionJudgmentView>,
    pub confusion: Option<EncounterConfusionContextView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterRetchPreviewView {
    pub issues: Vec<EncounterActionIssueView>,
    pub confusion: Option<EncounterConfusionRequirementsView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeActionContextView {
    pub issues: Vec<EncounterActionIssueView>,
    pub confused: bool,
    pub disruption: Option<EncounterActionCheckView>,
    pub targeting: Option<EncounterTargetingRequirementsView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterTargetingRequirementsView {
    pub prevents_effect: bool,
    pub requires_guessed_location: bool,
    pub check_senses: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterStrikeContextRequest {
    pub strike: crate::EncounterStrikeOptionsRequest,
    pub target_key: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeContextView {
    pub targeting_check: Option<EncounterActionCheckView>,
    pub options: crate::EncounterStrikeOptionsView,
    pub action: EncounterStrikeActionContextView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterConfusionContextView {
    Offensive {
        viable_target_keys: Vec<String>,
        random_target_key: Option<String>,
        offensive: Option<bool>,
    },
    Facilitate {
        note: Option<String>,
    },
    NoViableTargets,
    UnableToAttack,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterConfusionRequirementsView {
    pub source_ids: Vec<i64>,
    pub required_facts: Vec<String>,
    pub selected_target_key: Option<String>,
    pub automatic_self_hit: bool,
    pub wastes_action: bool,
}
