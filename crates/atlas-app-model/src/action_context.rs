//! Shared condition advice and input facts for live encounter actions.
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
pub struct EncounterCastingRemindersRequest {
    pub participant_key: String,
    pub selection: RecordNavigationView,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterCastingRemindersView {
    pub advisories: Vec<crate::RuntimeEffectNoteView>,
}
