use crate::DamageMultiplierView;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageSourceView {
    pub condition_id: i64,
    pub formula: String,
    pub multiplier: DamageMultiplierView,
    pub recovery_dc: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageGroupView {
    /// Present only when current damage protection facts establish immunity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub immunity_note: Option<String>,
    pub episode_id: String,
    pub damage_type: String,
    pub sources: Vec<PersistentDamageSourceView>,
    /// None means the GM chooses which source deals the greater amount.
    pub selected_source_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RecoverPersistentDamageRequest {
    #[serde(default)]
    pub resolution: PersistentRecoveryResolution,
    pub boundary_key: Option<String>,
    pub mode: PersistentRecoveryMode,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub episode_id: String,
    pub selected_source_id: Option<i64>,
    /// Explicit GM adjudication, including assisted recovery.
    pub dc_override: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PersistentRecoveryResolution {
    #[default]
    Roll,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PersistentRecoveryMode {
    EndTurn,
    Explicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentReviewView {
    pub encounter_revision: i64,
    pub damage_groups: Vec<PersistentDamageGroupView>,
    pub round_number: i64,
    pub damage_applied: bool,
    pub episodes: Vec<PersistentEpisodeReviewView>,
    pub damage_result: Option<PersistentDamageResultView>,
    pub recovery_results: Vec<PersistentRecoveryResultView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentEpisodeReviewView {
    pub damage_resolution: PersistentDamageResolutionView,
    pub end_turn_recovery_unavailable_reason: Option<String>,
    pub end_turn_roll_available: bool,
    pub end_turn_roll_unavailable_reason: Option<String>,
    pub episode_id: String,
    pub end_turn_recovery_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentRecoveryResultView {
    pub encounter_key: String,
    pub request_id: String,
    pub episode_id: String,
    pub damage_type: String,
    pub dc: Option<i64>,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentDamageInput {
    Roll,
    TableResult { amount: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageGroupInput {
    pub episode_id: String,
    pub selected_source_id: Option<i64>,
    pub damage: PersistentDamageInput,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ApplyPersistentDamageRequest {
    pub boundary_key: Option<String>,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub artifact_generation: Option<String>,
    pub groups: Vec<PersistentDamageGroupInput>,
    pub context: crate::HpEventContextView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageResultView {
    pub encounter_key: String,
    pub request_id: String,
    pub groups: Vec<PersistentDamageOutcomeView>,
    pub total: i64,
    pub hardness_absorbed: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageOutcomeView {
    pub episode_id: String,
    pub damage_type: String,
    pub source_condition_id: i64,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub amount: i64,
    pub table_resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub protection: Option<PersistentDamageProtectionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentDamageResolutionView {
    Known {
        protection: PersistentDamageProtectionView,
    },
    Table {
        reason: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageProtectionView {
    pub immune: bool,
    pub weakness: i64,
    pub resistance: i64,
}
