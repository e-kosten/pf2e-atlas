use crate::DamageMultiplierView;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageSourceView {
    pub condition_id: i64,
    pub formula: String,
    /// Validated expression for unrolled dice; never random outcomes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub roll_preview: Option<atlas_dice::RollPreviewExpression>,
    pub multiplier: DamageMultiplierView,
    pub recovery_dc: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageGroupView {
    /// Present only when current damage protection facts establish immunity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub immunity_note: Option<String>,
    pub episode_id: String,
    pub damage_type: String,
    pub sources: Vec<PersistentDamageSourceView>,
    /// The first source in the backend-resolved precedence order. Groups are nonempty.
    pub selected_source_id: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
    /// Explicit GM adjudication, including assisted recovery.
    pub dc_override: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PersistentRecoveryResolution {
    #[default]
    Roll,
    Clear,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PersistentRecoveryMode {
    EndTurn,
    Explicit,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentEpisodeReviewView {
    pub damage_resolution: PersistentDamageResolutionView,
    pub end_turn_recovery_unavailable_reason: Option<String>,
    pub end_turn_roll_available: bool,
    pub end_turn_roll_unavailable_reason: Option<String>,
    pub episode_id: String,
    pub end_turn_recovery_available: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentDamageInput {
    Roll,
    TableResult { amount: i64 },
    Prepared { draft: PersistentDamageDraft },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PreparedPersistentRecoveryInput {
    pub episode_id: String,
    pub dc_override: Option<i64>,
}

/// A partial damage draft. At most one new damage roll or recovery check per request.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PreparePersistentDamageRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub boundary_key: Option<String>,
    pub artifact_generation: Option<String>,
    pub groups: Vec<PersistentDamageGroupInput>,
    pub recovery: Option<PreparedPersistentRecoveryInput>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PreparedPersistentDamageView {
    pub damage: PersistentDamageResultView,
    pub complete: bool,
    pub recovery: Option<PersistentRecoveryResultView>,
    pub draft_context: PersistentDraftContext,
    pub recovery_draft: Option<PersistentRecoveryDraft>,
}

/// The turn and participant reviewed by the GM before submitting browser-held rolls.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PersistentDraftContext {
    pub encounter_key: String,
    pub participant_key: String,
    pub boundary_key: Option<String>,
    pub round_number: i64,
    pub record_key: Option<String>,
    pub artifact_generation: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PersistentDamageDraft {
    pub context: PersistentDraftContext,
    pub source_condition_id: i64,
    pub roll: atlas_dice::RollOutcome,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PersistentRecoveryDraft {
    pub context: PersistentDraftContext,
    pub source_condition_id: i64,
    pub dc: i64,
    pub dc_override: Option<i64>,
    pub roll: Option<atlas_dice::RollOutcome>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentRecoveryInput {
    Clear {
        episode_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        draft: Option<PersistentRecoveryDraft>,
    },
    Prepared {
        episode_id: String,
        draft: PersistentRecoveryDraft,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageGroupInput {
    pub episode_id: String,
    pub damage: PersistentDamageInput,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ApplyPersistentDamageRequest {
    pub boundary_key: Option<String>,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub artifact_generation: Option<String>,
    pub groups: Vec<PersistentDamageGroupInput>,
    #[serde(default)]
    pub recoveries: Vec<PersistentRecoveryInput>,
    pub context: crate::HpEventContextView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageResultView {
    pub encounter_key: String,
    pub request_id: String,
    pub groups: Vec<PersistentDamageOutcomeView>,
    pub total: i64,
    pub hardness_absorbed: i64,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamageProtectionView {
    pub immune: bool,
    pub weakness: i64,
    pub resistance: i64,
}
