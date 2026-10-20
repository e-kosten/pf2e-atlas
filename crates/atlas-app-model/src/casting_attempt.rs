//! Saved casting attempts separate resource cost, interruption and spell effects.
use crate::RecordNavigationView;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionFactView {
    Reaction,
    Manipulate,
    Auditory,
    Concentrate,
    Move,
    Attack,
    MentalOnly,
    OpensOwnBonds,
    RelatesToFascination,
    FurthersEscape,
    Cantrip,
    VoluntaryIngestion,
    FollowsControllerInstructions,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterActionJudgmentView {
    pub source_condition_id: Option<i64>,
    pub fact: EncounterActionFactView,
    pub value: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PreviewEncounterCastingAttemptRequest {
    pub participant_key: String,
    pub selection: RecordNavigationView,
    pub target: Option<crate::EncounterActionTargetView>,
    pub judgments: Vec<EncounterActionJudgmentView>,
    pub confusion: Option<crate::EncounterConfusionContextView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct BeginEncounterCastingAttemptRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    pub artifact_generation: String,
    pub preview: PreviewEncounterCastingAttemptRequest,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterActionCheckInputView {
    Roll,
    TableResult { result: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ResolveEncounterActionCheckRequest {
    pub request_id: String,
    pub attempt_key: String,
    pub expected_attempt_revision: i64,
    pub outcome: EncounterActionCheckInputView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AbandonEncounterActionAttemptRequest {
    pub request_id: String,
    pub attempt_key: String,
    pub expected_attempt_revision: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionIssueKindView {
    Prohibited,
    NeedsFact,
    NeedsValue,
    NeedsContext,
    NeedsConfusion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterActionIssueView {
    pub subject: Option<crate::ConditionSubjectView>,
    pub source_condition_id: i64,
    pub condition_name: String,
    pub controller_key: Option<String>,
    pub instructions: Option<String>,
    pub kind: EncounterActionIssueKindView,
    pub fact: Option<EncounterActionFactView>,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterActionCheckSourceView {
    pub condition_id: i64,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterActionCheckView {
    pub dc: i64,
    pub sources: Vec<EncounterActionCheckSourceView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CastingResourceCostView {
    pub resource_key: String,
    pub label: String,
    pub before: i64,
    pub after: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterCastingPreviewView {
    pub advisories: Vec<crate::RuntimeEffectNoteView>,
    pub encounter_key: String,
    pub participant_key: String,
    pub expected_encounter_revision: i64,
    pub expected_participant_revision: i64,
    pub artifact_generation: String,
    pub spell_title: String,
    pub rank: u8,
    pub cost: Option<CastingResourceCostView>,
    pub issues: Vec<EncounterActionIssueView>,
    pub confusion: Option<crate::EncounterConfusionRequirementsView>,
    pub disruption: Option<EncounterActionCheckView>,
    pub pending_attempt_key: Option<String>,
    pub targeting: Option<crate::EncounterTargetingRequirementsView>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionAttemptStatusView {
    AwaitingCheck,
    Ready,
    Disrupted,
    Abandoned,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterActionCheckResultView {
    pub roll: Option<atlas_dice::RollOutcome>,
    pub table_result: Option<i64>,
    pub success: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterCastingAttemptView {
    pub attempt_key: String,
    pub participant_key: String,
    pub revision: i64,
    pub status: EncounterActionAttemptStatusView,
    pub spell_title: String,
    pub rank: u8,
    pub cost: Option<CastingResourceCostView>,
    pub checks: Vec<EncounterActionCheckView>,
    pub pending_check: Option<EncounterActionCheckView>,
    pub outcomes: Vec<EncounterActionCheckResultView>,
    pub automatic_self_hit: bool,
    pub secret: bool,
    pub shareable_summary: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterCastingAttemptsView {
    pub encounter_key: String,
    pub attempts: Vec<EncounterCastingAttemptView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterActionMutationView {
    pub encounter_key: String,
    pub attempt_key: String,
}
