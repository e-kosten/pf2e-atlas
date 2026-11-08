use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeOptionsRequest {
    pub participant_key: String,
    pub reference: crate::EncounterReferenceRequest,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeAttackOptionView {
    pub attack_number: u8,
    pub label: String,
    pub modifier: i32,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeOptionsView {
    pub applied: Vec<crate::RuntimeEffectNoteView>,
    pub conditional: Vec<crate::RuntimeEffectNoteView>,
    pub expected_revision: i64,
    pub expected_participant_revision: i64,
    pub title: String,
    pub attacks: Vec<StrikeAttackOptionView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable: Option<String>,
    pub damage_available: bool,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub damage_unavailable: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StrikeRollKind {
    Attack,
    Damage,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollEncounterStrikeRequest {
    pub request_id: String,
    pub expected_revision: i64,
    pub participant_key: String,
    pub expected_participant_revision: i64,
    pub reference: crate::EncounterReferenceRequest,
    pub attack_number: u8,
    pub mode: crate::RollMode,
    pub modifier_override: Option<i32>,
    pub roll_kind: StrikeRollKind,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeDamageRollView {
    pub id: String,
    pub label: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub formula: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub damage_type: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub category: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub outcome: Option<atlas_dice::RollOutcome>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeRollView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub attack_modifier: Option<i32>,
    pub request_id: String,
    pub encounter_key: String,
    pub participant_key: String,
    pub title: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub attack: Option<atlas_dice::RollOutcome>,
    pub damage: Vec<StrikeDamageRollView>,
    pub damage_totals: Vec<StrikeDamageTotalView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub damage_total: Option<i32>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub persistent_total: Option<i32>,
    pub damage_minimum_adjustment: i32,
    pub persistent_minimum_adjustment: i32,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeDamageTotalView {
    pub damage_type: String,
    pub persistent: bool,
    pub raw_total: i32,
    pub total: i32,
}
