//! HP events distinguish amounts already resolved at the table from corrections.
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DamageLethalityView {
    Lethal,
    Nonlethal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
pub struct HpEventContextView {
    #[serde(default)]
    pub keep_current_initiative: Option<bool>,
    #[serde(default)]
    pub keep_upright: Option<bool>,
    #[serde(default)]
    pub natural_rest_healing: Option<bool>,
    pub attack_or_spell_damage: Option<bool>,
    pub lethality: Option<DamageLethalityView>,
    pub critical: Option<bool>,
    pub enemy_source: Option<bool>,
    pub responsible_participant_key: Option<String>,
    pub base_death_threshold: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ResolveEncounterHpLifecycleRequest {
    #[serde(default)]
    pub resolution: HpLifecycleResolutionView,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub consequence_key: String,
    pub context: HpEventContextView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PendingHpLifecycleView {
    #[serde(default)]
    #[ts(optional)]
    pub unavailable_reason: Option<String>,
    pub knockout_options: Vec<KnockoutOptionView>,
    pub required_facts: Vec<HpLifecycleRequiredFactView>,
    pub initiative_only: bool,
    pub consequence_key: String,
    pub participant_key: String,
    pub before_hp: i64,
    pub after_hp: i64,
    pub context: HpEventContextView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum HpLifecycleRequiredFactView {
    Lethality,
    Critical,
    EnemySource,
    ResponsibleParticipant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum HpLifecycleResolutionView {
    #[default]
    Apply,
    KeepCurrentState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct KnockoutOptionView {
    pub label: String,
    pub result: String,
    pub context: HpEventContextView,
}
