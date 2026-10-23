//! Explicit saved source inputs, not interpreted notes or effective statistics.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionParameters {
    Fleeing {
        subject: Option<ConditionSubject>,
    },
    Fascinated {
        subject: Option<ConditionSubject>,
    },
    Attitude,
    Frightened {
        recovery: FrightenedRecovery,
    },
    Broken {
        threshold: Option<i64>,
    },
    #[default]
    None,
    PersistentDamageReminder {
        formula: Option<String>,
        damage_type: Option<String>,
        multiplier: DamageMultiplier,
        recovery: PersistentRecovery,
    },
    PersistentDamage {
        formula: String,
        damage_type: String,
        multiplier: DamageMultiplier,
        recovery: PersistentRecovery,
        episode_id: String,
    },
    Sickened {
        recovery_dc: Option<i64>,
    },
    Unconscious {
        waking: WakingRule,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageMultiplier {
    Normal,
    Double,
    Half,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentRecovery {
    Standard,
    Specified { dc: i64 },
    SourceDependent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WakingRule {
    Normal,
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
#[serde(rename_all = "snake_case")]
pub enum MortalityState {
    #[default]
    Alive,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrightenedRecovery {
    EndTurn { minimum: i64 },
    Held,
    SourceDependent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionSubject {
    Participant { participant_key: String },
    Other { description: String },
}
