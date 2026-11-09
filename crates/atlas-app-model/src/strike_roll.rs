use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterStrikeOptionsRequest {
    pub participant_key: String,
    pub reference: crate::EncounterReferenceRequest,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikeAttackOptionView {
    pub attack_number: u8,
    pub label: String,
    pub modifier: i32,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StrikeRollAction {
    Attack {
        #[cfg_attr(feature = "openapi", schema(minimum = 1, maximum = 3))]
        attack_number: u8,
        mode: crate::RollMode,
        modifier_override: Option<i32>,
    },
    Damage {},
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollEncounterStrikeRequest {
    pub request_id: String,
    pub expected_revision: i64,
    pub participant_key: String,
    pub expected_participant_revision: i64,
    pub reference: crate::EncounterReferenceRequest,
    pub action: StrikeRollAction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterStrikeRollView {
    pub request_id: String,
    pub encounter_key: String,
    pub participant_key: String,
    pub title: String,
    pub result: StrikeRollResult,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StrikeRollResult {
    Attack {
        attack: atlas_dice::RollOutcome,
        attack_modifier: i32,
    },
    Damage {
        damage: Vec<StrikeDamageRollView>,
        damage_totals: Vec<StrikeDamageTotalView>,
        #[cfg_attr(feature = "openapi", schema(required))]
        damage_total: Option<i32>,
        #[cfg_attr(feature = "openapi", schema(required))]
        persistent_total: Option<i32>,
        damage_minimum_adjustment: i32,
        persistent_minimum_adjustment: i32,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikeDamageTotalView {
    pub damage_type: String,
    pub persistent: bool,
    pub raw_total: i32,
    pub total: i32,
}
