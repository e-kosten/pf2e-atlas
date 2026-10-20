//! Optional contextual action queries; absence of a target does not block browsing.
use crate::{EncounterActionCheckView, EncounterActionIssueView, EncounterActionJudgmentView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionKindView {
    Strike,
    Cast,
    Escape,
    ForceOpen,
    Stand,
    Crawl,
    Seek,
    Delay,
    Ready,
    Other,
    Retch,
    Request,
    TravelingExploration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterTargetModeView {
    Direct,
    Area,
    GuessedLocation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterActionTargetView {
    pub participant_key: String,
    pub mode: EncounterTargetModeView,
    pub vision_only_precise: Option<bool>,
    pub distance_feet: Option<serde_json::Number>,
    pub guessed_location_correct: Option<bool>,
    pub visual_effect: Option<bool>,
    pub auditory_effect: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PreviewEncounterActionRequest {
    pub normally_ally: Option<bool>,
    pub participant_key: String,
    pub action: EncounterActionKindView,
    pub reaction: Option<bool>,
    pub target: Option<EncounterActionTargetView>,
    pub judgments: Vec<EncounterActionJudgmentView>,
    pub confusion: Option<EncounterConfusionContextView>,
    pub extra_action: Option<crate::ConditionActionView>,
    pub request_context: Option<crate::EncounterRequestContextView>,
    pub sensory: Option<EncounterSensoryContextView>,
    pub forced_movement: Option<EncounterForcedMovementContextView>,
    pub prone_cover: Option<EncounterProneCoverContextView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterActionPreviewView {
    pub considers_target_ally: Option<bool>,
    pub can_flank: bool,
    pub encounter_key: String,
    pub participant_key: String,
    pub expected_revision: i64,
    pub expected_participant_revision: i64,
    pub issues: Vec<EncounterActionIssueView>,
    pub confusion: Option<EncounterConfusionRequirementsView>,
    pub disruption: Option<EncounterActionCheckView>,
    pub targeting: Option<EncounterTargetingRequirementsView>,
    pub quickened_use: Option<EncounterQuickenedUseView>,
    pub request: Option<crate::EncounterRequestRequirementsView>,
    pub sensory: Option<EncounterSensoryRequirementsView>,
    pub forced_movement: Option<EncounterForcedMovementRequirementsView>,
    pub prone_cover: Option<EncounterProneCoverRequirementsView>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterTargetRequiredInputView {
    GuessedLocationCorrect,
    VisualEffect,
    AuditoryEffect,
    Detection,
    GuessedLocation,
    VisionOnlyPrecise,
    DistanceFeet,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterTargetingRequirementsView {
    pub visual_effects_immune: bool,
    pub auditory_effects_immune: bool,
    pub prevents_effect: bool,
    pub flat_check_dc: Option<i64>,
    pub source_ids: Vec<i64>,
    pub requires_guessed_location: bool,
    pub required_facts: Vec<String>,
    pub required_inputs: Vec<EncounterTargetRequiredInputView>,
    pub secret: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PreviewEncounterStrikeAttemptRequest {
    pub strike: crate::EncounterStrikeOptionsRequest,
    pub judgments: Vec<crate::EncounterActionJudgmentView>,
    pub confusion: Option<EncounterConfusionContextView>,
    pub target: Option<EncounterActionTargetView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct BeginEncounterStrikeAttemptRequest {
    /// Exact original request, used to recover its immutable roll receipt.
    pub existing_roll: Option<crate::RollEncounterStrikeRequest>,
    pub strike: crate::RollEncounterStrikeRequest,
    pub judgments: Vec<crate::EncounterActionJudgmentView>,
    pub confusion: Option<EncounterConfusionContextView>,
    pub target: Option<EncounterActionTargetView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeAttemptPreviewView {
    pub targeting_check: Option<EncounterActionCheckView>,
    pub options: crate::EncounterStrikeOptionsView,
    pub action: EncounterActionPreviewView,
    pub artifact_generation: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeAttemptView {
    pub attempt_key: String,
    pub participant_key: String,
    pub revision: i64,
    pub status: crate::EncounterActionAttemptStatusView,
    pub title: String,
    pub checks: Vec<EncounterActionCheckView>,
    pub pending_check: Option<EncounterActionCheckView>,
    pub outcomes: Vec<crate::EncounterActionCheckResultView>,
    pub attack: Option<atlas_dice::RollOutcome>,
    pub automatic_self_hit: bool,
    pub secret: bool,
    pub shareable_summary: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeAttemptsView {
    pub encounter_key: String,
    pub attempts: Vec<EncounterStrikeAttemptView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterQuickenedUseView {
    pub allowed: Option<bool>,
    pub allowing_source_ids: Vec<i64>,
    pub unknown_source_ids: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterPerceptionBasisView {
    General,
    VisualOnly,
    AuditoryOnly,
    MixedSound,
    Initiative,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterSensoryContextView {
    pub basis: EncounterPerceptionBasisView,
    pub distance_feet: Option<f64>,
    pub requires_movement: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterSensoryRequirementsView {
    pub perception: Option<serde_json::Number>,
    pub critical_failure: bool,
    pub visual_effects_immune: bool,
    pub auditory_effects_immune: bool,
    pub all_terrain_difficult: bool,
    pub required_facts: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterForcedMovementContextView {
    pub holding_in_place: Option<bool>,
    pub difficulty_class: Option<i64>,
    pub check_total: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterForcedMovementOutcomeView {
    Unrestricted,
    NeedsCheck,
    CheckFailed,
    CheckSucceeded,
    NeedsContext,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterForcedMovementRequirementsView {
    pub outcome: EncounterForcedMovementOutcomeView,
    pub difficulty_class: Option<i64>,
    pub source_ids: Vec<i64>,
    pub required_facts: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterProneCoverContextView {
    pub against_ranged_attack: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterProneCoverRequirementsView {
    pub armor_class: Option<serde_json::Number>,
    pub required_facts: Vec<String>,
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
