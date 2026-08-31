use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::RecordSurfaceView;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionCatalogView {
    pub conditions: Vec<EncounterConditionDefinitionView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct EncounterConditionDefinitionView {
    pub condition_ref: String,
    pub name: String,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: Vec<EncounterConditionApplicabilityView>,
    pub categories: Vec<EncounterConditionCategoryView>,
    pub has_value: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default_value: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionAutomationLevelView {
    Automated,
    Tracked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionApplicabilityView {
    Creature,
    Hazard,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum EncounterConditionCategoryView {
    ActionEconomy,
    Attitude,
    DeathAndDying,
    Detection,
    ObjectState,
    RuntimeState,
    StatModifier,
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
    pub defeated: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
    pub note_hint: Option<String>,
    pub surface: RecordSurfaceView,
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
