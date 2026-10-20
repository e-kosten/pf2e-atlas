//! Explicit condition recovery and qualifying table events.
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CheckDegreeView {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LifecycleCheckInputView {
    Roll,
    TableDegree { degree: CheckDegreeView },
    TableResult { total: i64, natural: Option<i64> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionLifecycleActionView {
    DyingRecovery {
        check: LifecycleCheckInputView,
    },
    Retch {
        #[serde(default)]
        confusion: Option<crate::EncounterConfusionContextView>,
        #[serde(default)]
        judgments: Vec<crate::EncounterActionJudgmentView>,
        condition_id: i64,
        dc: Option<i64>,
        fortitude_modifier: Option<i64>,
        actions_spent: bool,
        check: LifecycleCheckInputView,
    },
    FrightenedEndTurn {
        decisions: Vec<FrightenedSourceDecisionView>,
    },
    TreatWounds {
        healing_received: i64,
    },
    FullHpRest {
        minutes: u32,
    },
    OvernightRest,
    StunnedActionsLost {
        condition_id: i64,
        actions_lost: i64,
    },
    StableRecovery {
        minutes: u32,
    },
    Wake {
        cause: WakeCauseView,
    },
    WakeFromNoise {
        dc: i64,
        perception_modifier: Option<i64>,
        check: LifecycleCheckInputView,
    },
    ConfusedDamageRecovery {
        consequence_key: String,
        condition_ids: Vec<i64>,
        qualifying_damage: bool,
        check: Option<LifecycleCheckInputView>,
    },
    ResolveBroken {
        consequence_key: String,
        threshold: i64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FrightenedSourceDecisionView {
    pub condition_id: i64,
    pub reduce: bool,
    pub minimum: Option<i64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WakeCauseView {
    Interact,
    RestInterrupted,
    RestCompleted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ApplyConditionLifecycleRequest {
    pub boundary_key: Option<String>,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub action: ConditionLifecycleActionView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LifecycleCheckResultView {
    pub dc: Option<i64>,
    pub modifier: Option<i64>,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub table_result: Option<i64>,
    pub degree: CheckDegreeView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleResultView {
    pub encounter_key: String,
    pub request_id: String,
    pub check: Option<LifecycleCheckResultView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PendingConditionRecoveryView {
    ConfusedDamage {
        consequence_key: String,
        participant_key: String,
        condition_ids: Vec<i64>,
        qualifying_damage: Option<bool>,
    },
    BrokenThreshold {
        consequence_key: String,
        participant_key: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrightenedRecoveryView {
    EndTurn { minimum: i64 },
    Held,
    SourceDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ConditionLifecycleActionKindView {
    DyingRecovery,
    Retch,
    FrightenedEndTurn,
    TreatWounds,
    FullHpRest,
    OvernightRest,
    StunnedActionsLost,
    StableRecovery,
    Wake,
    WakeFromNoise,
    ConfusedDamageRecovery,
    ResolveBroken,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleInputKindView {
    Check,
    SourceDc,
    CurrentFortitude,
    CurrentPerception,
    ActionsSpent,
    RestMinutes,
    HealingReceived,
    ActionsLost,
    FrightenedDecisions,
    Threshold,
    QualifyingDamage,
    SourceSelection,
    ActionJudgments,
    WakeCause,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleOptionView {
    pub kind: ConditionLifecycleActionKindView,
    pub label: String,
    pub condition_id: Option<i64>,
    pub consequence_key: Option<String>,
    pub qualifying_damage: Option<bool>,
    pub source_condition_ids: Vec<i64>,
    pub dc: Option<i64>,
    pub modifier: Option<i64>,
    pub actions_lost: Option<i64>,
    pub required_inputs: Vec<LifecycleInputKindView>,
    pub issues: Vec<crate::EncounterActionIssueView>,
    pub available: bool,
    pub unavailable_reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionLifecycleActionsView {
    pub encounter_key: String,
    pub encounter_revision: i64,
    pub participant_key: String,
    pub actions: Vec<ConditionLifecycleOptionView>,
}
