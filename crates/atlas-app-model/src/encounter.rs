use crate::{
    ConditionEditorKindView, ConditionEventIntentView, ConditionOriginView,
    ConditionParametersView, EncounterDamageTypeView, MortalityStateView,
};
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterConditionEligibilityView {
    Available,
    Immune { reason: String },
    NeedsContext { reason: String },
}

use serde::{Deserialize, Serialize};

use crate::{RecordPresentationView, RecordSummaryView};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterSummaryView {
    pub round_duration_seconds: i64,
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub description: Option<String>,
    pub status: EncounterStatusView,
    pub round_number: i64,
    pub participant_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterIndexView {
    pub encounters: Vec<EncounterSummaryView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionCatalogView {
    pub standard_persistent_recovery_dc: i64,
    pub damage_types: Vec<EncounterDamageTypeView>,
    pub conditions: Vec<EncounterConditionDefinitionView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionDurationUnit {
    Rounds,
    Minutes,
    Hours,
    Days,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionTurnEvent {
    Start,
    End,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionTimingView {
    pub participant_key: String,
    pub event: ConditionTurnEvent,
}

/// Condition rounds use a remaining amount; tracker durations retain the restart amount.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionDurationView {
    /// Remaining turns at the encounter’s current round length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub remaining_rounds: Option<i64>,
    pub amount: i64,
    pub unit: ConditionDurationUnit,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub timing: Option<ConditionTimingView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterTurnReviewView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub next_turn_unavailable_reason: Option<EncounterNextTurnUnavailableReason>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub outgoing: Option<EncounterTurnBoundaryView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub incoming: Option<EncounterTurnBoundaryView>,
    pub encounter_key: String,
    pub expected_revision: i64,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub current_participant_key: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub next_participant_key: Option<String>,
    pub reminders: Vec<EncounterTurnReminderView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterTurnReminderView {
    TrackerElapsed {
        participant_key: String,
        participant_name: String,
        tracker_key: String,
        tracker_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "openapi", schema(nullable = false))]
        result: Option<String>,
    },
    /// Confirmed current-state or boundary-receipt result for a retained reminder.
    Resolved {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        result: String,
    },
    ConditionRecovery {
        participant_key: String,
        participant_name: String,
        condition_ids: Vec<i64>,
        condition_name: String,
        /// Current effective value; unavailable if any contributing source is unknown.
        #[cfg_attr(feature = "openapi", schema(required))]
        condition_value: Option<i64>,
    },
    PersistentDamage {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        #[cfg_attr(feature = "openapi", schema(required))]
        note: Option<String>,
    },
    DurationEnding {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        timing: ConditionTimingView,
    },
    DurationEnded {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
    },
    DurationReminder {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        duration: ConditionDurationView,
    },
    TimingUnavailable {
        participant_key: String,
        participant_name: String,
        condition_id: i64,
        condition_name: String,
        duration: ConditionDurationView,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvanceEncounterTurnRequest {
    pub request_id: String,
    pub expected_revision: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionScopeView {
    #[default]
    Everyone,
    SelectedParticipants {
        participant_keys: Vec<String>,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionExtraActionView {
    #[default]
    Unspecified,
    Unrestricted,
    Restricted {
        actions: Vec<ConditionActionView>,
        #[cfg_attr(feature = "openapi", schema(required))]
        guidance: Option<String>,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionActionView {
    Stride,
    Strike,
    Step,
    Leap,
    Stand,
    Interact,
    CastSpell,
    Sustain,
    Escape,
    Crawl,
    Fly,
    Swim,
    Climb,
    Burrow,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionDefinitionView {
    pub editor_kind: ConditionEditorKindView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub scope_label: Option<String>,
    pub supports_extra_action: bool,
    pub eligibility: EncounterConditionEligibilityView,
    pub supports_duration_instead_of_value: bool,
    pub condition_ref: String,
    pub name: String,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: Vec<EncounterConditionApplicabilityView>,
    pub categories: Vec<EncounterConditionCategoryView>,
    pub has_value: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub default_value: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterConditionAutomationLevelView {
    Automated,
    Tracked,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterConditionApplicabilityView {
    Creature,
    Hazard,
    Object,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterConditionCategoryView {
    ActionEconomy,
    Attitude,
    DeathAndDying,
    Detection,
    ObjectState,
    RuntimeState,
    StatModifier,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterStatusView {
    Draft,
    Running,
    Complete,
    Archived,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterParticipantKindView {
    Creature,
    Hazard,
    Pc,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterParticipantSideView {
    Pc,
    Ally,
    Enemy,
    Neutral,
    Hazard,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterParticipantVariantView {
    Normal,
    Elite,
    Weak,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatModifierTypeView {
    Adjustment,
    Status,
    Circumstance,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ActionBudgetView {
    /// Status when the default allocation loses Quickened's extra action first.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub extra_action_unavailable_reason: Option<String>,
    pub quickened_sources: Vec<ConditionExtraActionView>,
    pub actions: RuntimeCountView,
    pub reactions: RuntimeCountView,
    pub can_act: RuntimeCapabilityView,
    pub can_react: RuntimeCapabilityView,
    pub notes: Vec<RuntimeEffectNoteView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub explanation: Option<String>,
    pub base_value: i64,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub adjusted_value: Option<i64>,
    pub segments: Vec<RuntimeCountSegmentView>,
    pub adjustments: Vec<RuntimeAdjustmentView>,
    pub suppressed_adjustments: Vec<RuntimeAdjustmentView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCountSegmentView {
    pub label: String,
    pub value: i64,
    pub restricted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reason: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeAdjustmentView {
    pub source: String,
    pub label: String,
    pub value: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reason: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeCapabilityView {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reason: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeEffectNoteView {
    pub source: String,
    pub label: String,
    pub reason: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StatModifierView {
    pub source: String,
    pub label: String,
    pub modifier_type: StatModifierTypeView,
    pub value: i64,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterParticipantStatusView {
    Unchecked,
    Active,
    Unresolved,
    Manual,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateEncounterRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterCreateView {
    pub encounter: EncounterSummaryView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterDetailView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference_error: Option<crate::AppError>,
    pub difficulty: crate::EncounterDifficultyView,
    pub revision: i64,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub artifact_generation: Option<String>,
    pub encounter: EncounterSummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub current_turn_participant_key: Option<String>,
    pub participants: Vec<EncounterParticipantView>,
}

/// A complete selected reference in one current participant's runtime context.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterReferenceRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<i64>,
    pub artifact_generation: String,
    pub selection: crate::RecordSelection,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct UpdateEncounterRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub slug: Option<crate::FieldEdit<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub name: Option<crate::FieldEdit<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub description: Option<crate::FieldEdit<Option<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub note: Option<crate::FieldEdit<Option<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub status: Option<crate::FieldEdit<EncounterStatusView>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::field_edit::optional_edit"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub round_duration_seconds: Option<crate::FieldEdit<i64>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterMutationView {
    pub encounter_key: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantView {
    pub manual_trackers: Vec<crate::EncounterManualTrackerView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub ability_states: Option<Vec<crate::EncounterAbilityStateView>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub turn_resources: Option<crate::TurnResourcesView>,
    pub knockout_options: Vec<crate::KnockoutOptionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub knockout_unavailable_reason: Option<String>,
    pub effective_max_hp: crate::NumberFactView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub level_override: Option<i64>,
    pub base_death_threshold: i64,
    pub persistent_damage_groups: Vec<crate::PersistentDamageGroupView>,
    pub track_dying: bool,
    pub mortality: MortalityStateView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub cast_unavailable_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub spellcasting: Option<EncounterSpellcastingView>,
    pub revision: i64,
    pub hp_origin: String,
    pub variant_origin: String,
    pub participant_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record_key: Option<String>,
    pub participant_kind: EncounterParticipantKindView,
    pub participant_variant: EncounterParticipantVariantView,
    pub status: EncounterParticipantStatusView,
    pub position: i64,
    pub display_name: String,
    pub side: EncounterParticipantSideView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub initiative: Option<i64>,
    pub initiative_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hazard_disabled: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub note: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub note_hint: Option<String>,
    pub conditions: Vec<EncounterParticipantConditionView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub inherent_conditions: Option<InherentConditionsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub presentation: Option<RecordPresentationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record: Option<RecordSummaryView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InherentConditionsView {
    pub entries: Vec<InherentConditionView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable_reason: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InherentConditionView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub unavailable_reason: Option<String>,
    pub name: String,
    pub has_value: bool,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub value: Option<i64>,
    pub condition_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub editable_rule_ordinal: Option<usize>,
    pub source_name: String,
    pub source: crate::RecordNavigationView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub guidance: Vec<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantConditionView {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub guidance: Vec<String>,
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub condition_key: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub source_participant_key: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required))]
    pub scope: ConditionScopeView,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required))]
    pub extra_action: ConditionExtraActionView,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required))]
    pub immunity_override: bool,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required))]
    pub parameters: ConditionParametersView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub origin: Option<ConditionOriginView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct AddEncounterRecordParticipantRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub creature_scenario: Option<CreatureScenarioContext>,
    pub record_ref: String,
    pub quantity: u32,
    /// Omitted inherits the source adjustment; a supplied choice replaces it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_variant: Option<EncounterParticipantVariantView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiative: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureScenarioContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub use_id: Option<String>,
    pub artifact_generation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub expected: Option<PreparedCreatureChoice>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedCreatureChoice {
    pub record_key: String,
    pub initial_adjustment: EncounterParticipantVariantView,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct AddEncounterManualParticipantRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiative: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterParticipantEdit {
    pub expected_revision: i64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub display_name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub side: Option<EncounterParticipantSideView>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub participant_variant: Option<EncounterParticipantVariantView>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nullable_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<i64>))]
    pub initiative: Option<Option<i64>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub defeated: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub hazard_disabled: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub hidden: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nullable_participant_field"
    )]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub note: Option<Option<String>>,
}

fn present_participant_field<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

fn present_nullable_participant_field<'de, D, T>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer).map(Some)
}

/// A supplied exact value; null explicitly clears current/maximum availability.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterHpValueInput {
    #[serde(deserialize_with = "Option::deserialize")]
    #[cfg_attr(feature = "openapi", schema(required))]
    pub input: Option<String>,
}

/// A deliberate HP operation, interpreted against the checked current participant.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterHpAction {
    Damage {
        amount: String,
    },
    Heal {
        amount: String,
    },
    SetCurrent {
        #[serde(deserialize_with = "Option::deserialize")]
        #[cfg_attr(feature = "openapi", schema(required))]
        input: Option<String>,
    },
    SetMaximum {
        #[serde(deserialize_with = "Option::deserialize")]
        #[cfg_attr(feature = "openapi", schema(required))]
        input: Option<String>,
    },
    SetTemporary {
        input: String,
    },
    /// Correct all supplied values atomically; omitted fields retain their intent.
    SetValues {
        #[serde(skip_serializing_if = "Option::is_none")]
        current: Option<EncounterHpValueInput>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<EncounterHpValueInput>,
        #[serde(skip_serializing_if = "Option::is_none")]
        temporary: Option<String>,
    },
    UseDerivedMaximum,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyEncounterHpActionRequest {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<crate::HpEventContextView>,
    pub expected_revision: i64,
    pub action: EncounterHpAction,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReorderEncounterParticipantRequest {
    pub participant_key: String,
    pub target_participant_key: String,
    pub placement: atlas_domain::ReorderPlacement,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct AddEncounterParticipantConditionRequest {
    pub request_id: String,
    /// Reference revision used by the confirmed preview, when it consulted source data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_reference_revision: Option<u64>,
    pub expected_encounter_revision: i64,
    #[serde(default)]
    pub event_intent: ConditionEventIntentView,
    #[serde(default)]
    pub decisions: crate::ConditionApplicationDecisionsView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_participant_key: Option<String>,
    #[serde(default)]
    pub scope: ConditionScopeView,
    #[serde(default)]
    pub extra_action: ConditionExtraActionView,
    #[serde(default)]
    pub immunity_override: bool,
    #[serde(default)]
    pub parameters: crate::ConditionParametersInputView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<ConditionOriginView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct UpdateEncounterParticipantConditionRequest {
    pub request_id: String,
    /// Reference revision used by the confirmed preview, when it consulted source data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_reference_revision: Option<u64>,
    #[serde(default)]
    pub event_intent: ConditionEventIntentView,
    #[serde(default)]
    pub decisions: crate::ConditionApplicationDecisionsView,
    pub expected_encounter_revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_ref: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_participant_key: Option<String>,
    #[serde(default)]
    pub scope: ConditionScopeView,
    #[serde(default)]
    pub extra_action: ConditionExtraActionView,
    #[serde(default)]
    pub immunity_override: bool,
    #[serde(default)]
    pub parameters: crate::ConditionParametersInputView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<ConditionOriginView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<ConditionDurationView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct SetEncounterTurnRequest {
    pub participant_key: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeleteEncounterView {
    pub encounter_key: String,
    pub slug: String,
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn condition_duration_rejects_fractional_missing_and_retired_inputs() {
        let request = json!({"request_id":"duration","expected_encounter_revision":1,"name":"Custom","duration":{"amount":2,"unit":"rounds","timing":{"participant_key":"p","event":"end"}}});
        let parsed =
            serde_json::from_value::<AddEncounterParticipantConditionRequest>(request.clone())
                .unwrap();
        assert_eq!(parsed.duration.unwrap().amount, 2);
        for duration in [
            json!({"unit":"rounds"}),
            json!({"amount":1.5,"unit":"minutes"}),
            json!({"amount":null,"unit":"hours"}),
            json!({"amount":2,"unit":"seconds"}),
            json!({"amount":2,"unit":"rounds","timing":{"participant_key":"p","event":"sometime"}}),
        ] {
            let mut bad = request.clone();
            bad["duration"] = duration;
            assert!(
                serde_json::from_value::<AddEncounterParticipantConditionRequest>(bad).is_err()
            );
        }
        let mut retired = request;
        retired.as_object_mut().unwrap().remove("duration");
        assert!(
            serde_json::from_value::<AddEncounterParticipantConditionRequest>(retired.clone())
                .is_ok()
        );
        retired["duration_rounds"] = json!(2);
        assert!(
            serde_json::from_value::<AddEncounterParticipantConditionRequest>(retired).is_err()
        );
    }

    #[test]
    fn hp_action_requires_revision_and_explicit_correction_input() {
        let mut request = json!({
            "request_id":"hp-action", "expected_revision":7,
            "action":{"kind":"damage","amount":"1+2"},
        });
        let decoded =
            serde_json::from_value::<ApplyEncounterHpActionRequest>(request.clone()).unwrap();
        assert_eq!(decoded.expected_revision, 7);
        request.as_object_mut().unwrap().remove("expected_revision");
        assert!(serde_json::from_value::<ApplyEncounterHpActionRequest>(request).is_err());
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_current"})).is_err()
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_maximum"})).is_err()
        );
        assert_eq!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"set_current","input":null}))
                .unwrap(),
            EncounterHpAction::SetCurrent { input: None }
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(json!({"kind":"damage","amount":12}))
                .is_err()
        );
        assert!(
            serde_json::from_value::<EncounterHpAction>(
                json!({"kind":"heal","amount":"2","defeated":false})
            )
            .is_err()
        );
    }

    #[test]
    fn atomic_hp_values_distinguish_omitted_fields_from_explicit_unknown() {
        let parsed: EncounterHpAction = serde_json::from_value(
            json!({"kind":"set_values","current":{"input":null},"temporary":"0"}),
        )
        .unwrap();
        assert_eq!(
            parsed,
            EncounterHpAction::SetValues {
                current: Some(EncounterHpValueInput { input: None }),
                maximum: None,
                temporary: Some("0".into())
            }
        );
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            json!({"kind":"set_values","current":{"input":null},"temporary":"0"})
        );
        for bad in [
            json!({"kind":"set_values","current":{}}),
            json!({"kind":"set_values","maximum":{"input":5}}),
            json!({"kind":"set_values","temporary":0}),
            json!({"kind":"set_values","current":{"input":"4","value":4}}),
        ] {
            assert!(serde_json::from_value::<EncounterHpAction>(bad).is_err());
        }
    }

    #[test]
    fn participant_edit_distinguishes_omitted_clear_and_set_values() {
        let omitted = json!({"expected_revision": 7});
        let clear = json!({"expected_revision": 7, "initiative": null, "note": null});
        let set =
            json!({"expected_revision": 7, "initiative": 0, "note": "Ready", "hidden": false});
        for (wire, initiative, note) in [
            (omitted, None, None),
            (clear, Some(None), Some(None)),
            (set, Some(Some(0)), Some(Some("Ready".to_owned()))),
        ] {
            let edit: UpdateEncounterParticipantEdit =
                serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(edit.initiative, initiative);
            assert_eq!(edit.note, note);
            assert_eq!(serde_json::to_value(edit).unwrap(), wire);
        }
        for invalid in [
            json!({"display_name": "One"}),
            json!({"expected_revision": 7, "side": null}),
            json!({"expected_revision": 7, "defeated": null}),
            json!({"expected_revision": 7, "current_hp": 10}),
        ] {
            assert!(serde_json::from_value::<UpdateEncounterParticipantEdit>(invalid).is_err());
        }
    }

    #[test]
    fn mutation_receipt_contains_only_durable_encounter_identity() {
        let receipt = EncounterMutationView {
            encounter_key: "encounter_one".into(),
        };
        assert_eq!(
            serde_json::to_value(receipt).unwrap(),
            json!({"encounter_key":"encounter_one"})
        );
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterSpellcastingView {
    pub resources: Vec<EncounterSpellResourceView>,
    pub spells: Vec<EncounterSpellView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterSpellResourceScope {
    Spell,
    Rank { rank: u8 },
    Entry,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterSpellResourceView {
    pub scope: EncounterSpellResourceScope,
    pub key: String,
    pub entry_id: String,
    pub label: String,
    pub capacity: i64,
    pub remaining: i64,
    pub can_undo: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterSpellView {
    pub navigation: crate::RecordNavigationView,
    pub entry_id: String,
    pub title: String,
    pub ranks: Vec<EncounterSpellRankView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub expended: Option<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterSpellRankView {
    pub rank: u8,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub resource_key: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable_reason: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyEncounterSpellActionRequest {
    pub expected_revision: i64,
    pub action: EncounterSpellAction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EncounterSpellAction {
    SetExpended {
        selection: crate::RecordNavigationView,
        expended: bool,
    },
    Cast {
        selection: crate::RecordNavigationView,
    },
    Undo {
        resource_key: String,
    },
    SetEntryResources {
        entry_id: String,
        remaining: std::collections::BTreeMap<String, i64>,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterNextTurnUnavailableReason {
    NoInitiativeParticipants,
    NoActiveParticipants,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterTurnBoundaryView {
    pub source_revision: i64,
    pub event: ConditionTurnEvent,
    pub participant_removed: bool,
    pub damage_applied: bool,
    pub recovered_episode_ids: Vec<String>,
    pub boundary_key: String,
    pub participant_key: String,
    pub participant_name: String,
    pub round_number: i64,
    pub reminders: Vec<EncounterTurnReminderView>,
    pub persistent_damage_groups: Vec<crate::PersistentDamageGroupView>,
}

#[cfg(test)]
mod tracker_reminder_tests {
    use super::EncounterTurnReminderView;
    #[test]
    fn tracker_elapsed_reminder_omits_an_unresolved_result_and_retains_canonical_status() {
        let reminder = EncounterTurnReminderView::TrackerElapsed {
            participant_key: "dragon".into(),
            participant_name: "Dragon".into(),
            tracker_key: "breath".into(),
            tracker_name: "Breath".into(),
            result: None,
        };
        let mut wire = serde_json::to_value(&reminder).unwrap();
        assert!(wire.get("result").is_none());
        assert_eq!(
            serde_json::from_value::<EncounterTurnReminderView>(wire.clone()).unwrap(),
            reminder
        );
        wire["result"] = serde_json::json!("Removed");
        assert!(
            matches!(serde_json::from_value::<EncounterTurnReminderView>(wire).unwrap(), EncounterTurnReminderView::TrackerElapsed { result: Some(result), .. } if result == "Removed")
        );
    }
}

#[cfg(test)]
mod turn_boundary_wire_tests {
    use super::{ConditionTurnEvent, EncounterTurnBoundaryView};
    #[test]
    fn turn_boundary_source_revision_is_a_required_json_integer() {
        let boundary = EncounterTurnBoundaryView {
            source_revision: 17,
            event: ConditionTurnEvent::Start,
            participant_removed: false,
            damage_applied: false,
            recovered_episode_ids: vec![],
            boundary_key: "boundary".into(),
            participant_key: "dragon".into(),
            participant_name: "Dragon".into(),
            round_number: 2,
            reminders: vec![],
            persistent_damage_groups: vec![],
        };
        let mut json = serde_json::to_value(&boundary).unwrap();
        assert_eq!(json["source_revision"].as_i64(), Some(17));
        assert_eq!(
            serde_json::from_value::<EncounterTurnBoundaryView>(json.clone()).unwrap(),
            boundary
        );
        json.as_object_mut().unwrap().remove("source_revision");
        assert!(serde_json::from_value::<EncounterTurnBoundaryView>(json).is_err());
    }
}
