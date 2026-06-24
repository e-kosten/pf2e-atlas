use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::RecordSummaryView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterSummaryView {
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    pub status: EncounterStatusView,
    pub round_number: i64,
    pub participant_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterIndexView {
    pub encounters: Vec<EncounterSummaryView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterStatusView {
    Draft,
    Running,
    Complete,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantKindView {
    Creature,
    Hazard,
    Pc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantSideView {
    Pc,
    Ally,
    Enemy,
    Neutral,
    Hazard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantVariantView {
    Normal,
    Elite,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum StatModifierTypeView {
    Adjustment,
    Status,
    Circumstance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct StatBlockView {
    pub record_key: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub adjusted_level: Option<i64>,
    pub values: Vec<StatValueView>,
    pub activities: Vec<MechanicActivityView>,
    pub unapplied_effects: Vec<UnappliedEffectView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct StatValueView {
    pub target: String,
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub modifiers: Vec<StatModifierView>,
    pub suppressed_modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct StatModifierView {
    pub source: String,
    pub label: String,
    pub modifier_type: StatModifierTypeView,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UnappliedEffectView {
    pub source: String,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct MechanicActivityView {
    pub activity_id: String,
    pub label: String,
    pub kind: MechanicActivityKindView,
    pub usage: MechanicActivityUsageView,
    pub rolls: Vec<ActivityRollView>,
    pub damage: Vec<DamageExpressionView>,
    pub modes: Vec<MechanicActivityModeView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct MechanicActivityModeView {
    pub mode_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub time: Option<String>,
    pub damage: Vec<DamageExpressionView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum MechanicActivityKindView {
    Strike,
    Spell,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum MechanicActivityUsageView {
    Unlimited,
    Limited,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ActivityRollView {
    pub roll_id: String,
    pub label: String,
    pub base_value: i64,
    pub adjusted_value: i64,
    pub surface: ActivityRollSurfaceView,
    pub modifiers: Vec<StatModifierView>,
    pub suppressed_modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ActivityRollSurfaceView {
    AttackRoll,
    Dc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DamageExpressionView {
    pub damage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    pub formula: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub adjusted_formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub damage_type: Option<String>,
    pub effect_kind: DamageEffectKindView,
    pub modifiers: Vec<StatModifierView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum DamageEffectKindView {
    Damage,
    Healing,
    DamageOrHealing,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterParticipantStatusView {
    Active,
    Unresolved,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreateEncounterRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterCreateView {
    pub encounter: EncounterSummaryView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterDetailView {
    pub encounter: EncounterSummaryView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_turn_participant_key: Option<String>,
    pub participants: Vec<EncounterParticipantView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEncounterRequest {
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub status: EncounterStatusView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterUpdateView {
    pub encounter: EncounterSummaryView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantView {
    pub participant_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_key: Option<String>,
    pub participant_kind: EncounterParticipantKindView,
    pub participant_variant: EncounterParticipantVariantView,
    pub status: EncounterParticipantStatusView,
    pub position: i64,
    pub display_name: String,
    pub side: EncounterParticipantSideView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
    pub initiative_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub note_hint: Option<String>,
    pub conditions: Vec<EncounterParticipantConditionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub stat_block: Option<StatBlockView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record: Option<RecordSummaryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterParticipantConditionView {
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_key: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration_rounds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddEncounterRecordParticipantRequest {
    pub encounter_ref: String,
    pub record_ref: String,
    pub quantity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddEncounterManualParticipantRequest {
    pub encounter_ref: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEncounterParticipantRequest {
    pub participant_key: String,
    pub display_name: String,
    pub side: EncounterParticipantSideView,
    pub participant_variant: EncounterParticipantVariantView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub initiative: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_hp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ReorderEncounterParticipantPlacementView {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ReorderEncounterParticipantRequest {
    pub participant_key: String,
    pub target_participant_key: String,
    pub placement: ReorderEncounterParticipantPlacementView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AddEncounterParticipantConditionRequest {
    pub participant_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration_rounds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEncounterParticipantConditionRequest {
    pub condition_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub condition_ref: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_participant_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub duration_rounds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SetEncounterTurnRequest {
    pub encounter_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub participant_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct DeleteEncounterView {
    pub encounter_key: String,
    pub slug: String,
    pub deleted: bool,
}
