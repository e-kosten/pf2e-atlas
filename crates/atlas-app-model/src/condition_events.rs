//! Condition source inputs and explicit saved-event contracts.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionParametersView {
    Fleeing {
        subject: Option<ConditionSubjectView>,
    },
    Fascinated {
        subject: Option<ConditionSubjectView>,
    },
    Attitude {
        supernatural: bool,
    },
    Frightened {
        recovery: crate::FrightenedRecoveryView,
    },
    Broken {
        threshold: Option<i64>,
    },
    #[default]
    None,
    PersistentDamageReminder {
        formula: Option<String>,
        damage_type: Option<String>,
        multiplier: DamageMultiplierView,
        recovery: PersistentRecoveryView,
    },
    PersistentDamage {
        formula: String,
        damage_type: String,
        multiplier: DamageMultiplierView,
        recovery: PersistentRecoveryView,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        episode_id: Option<String>,
    },
    Sickened {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        recovery_dc: Option<i64>,
    },
    Unconscious {
        waking: WakingRuleView,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DamageMultiplierView {
    Normal,
    Double,
    Half,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentRecoveryView {
    Standard,
    Specified { dc: i64 },
    SourceDependent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WakingRuleView {
    Normal,
    SourceControlled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionOriginView {
    pub application_id: String,
    pub grant_slot: String,
    pub participant_key: Option<String>,
    pub record_key: Option<String>,
    pub activity_id: Option<String>,
    pub artifact_generation: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MortalityStateView {
    Alive,
    Dead,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConditionEventIntentView {
    #[default]
    Apply,
    Correction,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConditionRemovalIntentView {
    #[default]
    RemoveEffect,
    Correction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RemoveEncounterParticipantConditionRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub intent: ConditionRemovalIntentView,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ConditionEditorKindView {
    Fleeing,
    Fascinated,
    Attitude,
    Frightened,
    Broken,
    Ordinary,
    PersistentDamage,
    Stunned,
    Sickened,
    Unconscious,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterDamageTypeView {
    pub value: String,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionEventPreviewView {
    pub encounter_key: String,
    pub encounter_revision: i64,
    pub consequences: Vec<ConditionEventConsequenceView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionEventConsequenceView {
    PronePrevented {
        participant_key: String,
        reason: String,
    },
    DropHeldItems {
        participant_key: String,
    },
    DirectHpLoss {
        participant_key: String,
        amount: i64,
    },
    BecomeProne {
        participant_key: String,
    },
    DrainedLevelRequired {
        participant_key: String,
        magnitude_increase: i64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ResolveDrainedConsequenceRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub consequence_key: String,
    pub level: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PendingConditionConsequenceView {
    pub consequence_key: String,
    pub participant_key: String,
    pub magnitude_increase: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SetEncounterLifeSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mortality: Option<MortalityStateView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level_override: Option<LevelOverrideInputView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub base_death_threshold: Option<i64>,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub track_dying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
pub struct ConditionApplicationDecisionsView {
    #[serde(default)]
    pub keep_upright: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LevelOverrideInputView {
    pub value: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionSubjectView {
    Participant { participant_key: String },
    Other { description: String },
}
