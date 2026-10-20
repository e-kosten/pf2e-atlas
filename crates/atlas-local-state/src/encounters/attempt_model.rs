//! Versioned domain facts for an action already begun; no application DTOs.
use super::SpellResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionFactKind {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenActionJudgment {
    pub source_condition_id: Option<i64>,
    pub fact: ActionFactKind,
    pub value: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCheckSource {
    pub condition_id: i64,
    pub name: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCheckStage {
    Disruption,
    Targeting,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenActionCheck {
    pub stage: ActionCheckStage,
    pub dc: i64,
    pub sources: Vec<ActionCheckSource>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellResourceCost {
    pub resource_key: String,
    pub label: String,
    pub before: i64,
    pub after: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRollMode {
    Normal,
    Fortune,
    Misfortune,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenAction {
    Cast {
        spell_id: String,
        rank: u8,
    },
    Strike {
        activity_id: String,
        attack_number: u8,
        modifier: Option<i32>,
        automatic_self_hit: bool,
        mode: ActionRollMode,
        #[serde(default)]
        retained_roll: Option<RetainedStrikeRoll>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedStrikeRoll {
    pub request_id: String,
    pub outcome: atlas_dice::RollOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionTargetMode {
    Direct,
    Area,
    GuessedLocation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenActionTarget {
    pub participant_key: String,
    pub mode: ActionTargetMode,
    pub detection: super::RelationshipKnowledge,
    pub vision_only_precise: Option<bool>,
    pub distance_feet: Option<serde_json::Number>,
    pub guessed_location_correct: Option<bool>,
    pub visual_effect: Option<bool>,
    pub auditory_effect: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConfusionChoice {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenActionContext {
    pub participant_key: String,
    pub source_record_key: String,
    pub title: String,
    pub action: FrozenAction,
    pub target: Option<FrozenActionTarget>,
    pub confusion: Option<ConfusionChoice>,
    pub source_fingerprint: String,
    pub artifact_generation: String,
    pub reference_revision: u64,
    pub judgments: Vec<FrozenActionJudgment>,
    pub cost: Option<SpellResourceCost>,
    pub checks: Vec<FrozenActionCheck>,
}
#[derive(Debug, Clone)]
pub struct PreparedActionAttempt {
    pub context: FrozenActionContext,
    pub resources: Option<Vec<SpellResource>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAttemptStatus {
    AwaitingCheck,
    Ready,
    Disrupted,
    Abandoned,
}
impl ActionAttemptStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::AwaitingCheck => "awaiting_check",
            Self::Ready => "ready",
            Self::Disrupted => "disrupted",
            Self::Abandoned => "abandoned",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCheckResult {
    pub roll: Option<atlas_dice::RollOutcome>,
    pub table_result: Option<i64>,
    pub success: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ActionCompletion {
    Cast,
    StrikeRoll { roll: atlas_dice::RollOutcome },
    AutomaticSelfHit,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedActionAttempt {
    pub attempt_key: String,
    pub revision: i64,
    pub status: ActionAttemptStatus,
    pub context: FrozenActionContext,
    pub outcomes: Vec<ActionCheckResult>,
    pub completion: Option<ActionCompletion>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionAttemptReceipt {
    pub encounter_key: String,
    pub request_id: String,
    pub attempt_key: String,
}
