use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeOptionsRequest {
    pub participant_key: String,
    pub reference: crate::EncounterReferenceRequest,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeAttackOptionView {
    pub attack_number: u8,
    pub label: String,
    pub modifier: i32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeOptionsView {
    pub expected_revision: i64,
    pub expected_participant_revision: i64,
    pub title: String,
    pub attacks: Vec<StrikeAttackOptionView>,
    pub unavailable: Option<String>,
    pub damage_available: bool,
    pub damage_unavailable: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum StrikeRollKind {
    Attack,
    Damage,
}
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeDamageRollView {
    pub id: String,
    pub label: String,
    pub formula: Option<String>,
    pub damage_type: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<atlas_dice::RollOutcome>,
    pub unavailable: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeRollView {
    pub request_id: String,
    pub encounter_key: String,
    pub participant_key: String,
    pub title: String,
    pub attack: Option<atlas_dice::RollOutcome>,
    pub damage: Vec<StrikeDamageRollView>,
    pub damage_totals: Vec<StrikeDamageTotalView>,
    pub damage_total: Option<i32>,
    pub persistent_total: Option<i32>,
    pub damage_minimum_adjustment: i32,
    pub persistent_minimum_adjustment: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct StrikeDamageTotalView {
    pub damage_type: String,
    pub persistent: bool,
    pub raw_total: i32,
    pub total: i32,
}
