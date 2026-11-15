//! Explicit saved source inputs, not interpreted notes or effective statistics.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind")]
pub enum ConditionParameters {
    #[serde(rename = "fleeing")]
    Fleeing { subject: Option<ConditionSubject> },
    #[serde(rename = "fascinated")]
    Fascinated { subject: Option<ConditionSubject> },
    #[serde(rename = "attitude")]
    Attitude,
    #[serde(rename = "frightened")]
    Frightened { recovery: FrightenedRecovery },
    #[serde(rename = "broken")]
    Broken { threshold: Option<i64> },
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "persistent_damage_reminder")]
    PersistentDamageReminder {
        formula: Option<String>,
        damage_type: Option<String>,
        multiplier: DamageMultiplier,
        recovery: PersistentRecovery,
    },
    #[serde(rename = "persistent_damage")]
    PersistentDamage {
        formula: String,
        damage_type: String,
        multiplier: DamageMultiplier,
        recovery: PersistentRecovery,
        episode_id: String,
    },
    #[serde(rename = "sickened")]
    Sickened { recovery_dc: Option<i64> },
    #[serde(rename = "unconscious")]
    Unconscious { waking: WakingRule },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageMultiplier {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "double")]
    Double,
    #[serde(rename = "half")]
    Half,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PersistentRecovery {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "specified")]
    Specified { dc: i64 },
    #[serde(rename = "source_dependent")]
    SourceDependent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WakingRule {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "source_controlled")]
    SourceControlled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionOrigin {
    pub application_id: String,
    pub grant_slot: String,
    pub participant_key: Option<String>,
    pub record_key: Option<String>,
    pub activity_id: Option<String>,
    pub artifact_generation: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MortalityState {
    #[default]
    #[serde(rename = "alive")]
    Alive,
    #[serde(rename = "dead")]
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum FrightenedRecovery {
    #[serde(rename = "end_turn")]
    EndTurn { minimum: i64 },
    #[serde(rename = "held")]
    Held,
    #[serde(rename = "source_dependent")]
    SourceDependent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConditionSubject {
    #[serde(rename = "participant")]
    Participant { participant_key: String },
    #[serde(rename = "other")]
    Other { description: String },
}
