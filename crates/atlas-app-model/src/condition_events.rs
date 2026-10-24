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
    Attitude,
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
/// Editable parameters exclude resolved persistent-episode identity and admission state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionParametersInputView {
    Fleeing {
        subject: Option<ConditionSubjectView>,
    },
    Fascinated {
        subject: Option<ConditionSubjectView>,
    },
    Attitude,
    Frightened {
        recovery: crate::FrightenedRecoveryView,
    },
    Broken {
        threshold: Option<i64>,
    },
    #[default]
    None,
    PersistentDamage {
        formula: Option<String>,
        damage_type: Option<String>,
        multiplier: DamageMultiplierView,
        recovery: PersistentRecoveryView,
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
pub struct ConditionHpChangeView {
    pub current_before: Option<i64>,
    pub current_after: Option<i64>,
    pub maximum_before: Option<i64>,
    pub maximum_after: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ConditionEventPreviewView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub persistent_precedence: Option<PersistentDamagePrecedenceView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hp_change: Option<ConditionHpChangeView>,
    pub encounter_key: String,
    pub encounter_revision: i64,
    pub consequences: Vec<ConditionEventConsequenceView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionEventConsequenceView {
    StunnedActionsLost {
        participant_key: String,
        before: i64,
        after: i64,
        stunned_after: i64,
    },
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
    pub apply_hp_loss: bool,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub consequence_key: String,
    pub level: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PendingConditionConsequenceView {
    pub condition_ref: String,
    pub consequence_key: String,
    pub participant_key: String,
    pub magnitude_increase: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SetEncounterLifeSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub knockout_context: Option<crate::HpEventContextView>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub persistent_precedence: Option<Vec<PersistentDamageSourceRefView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistentDamageSourceRefView {
    Edited,
    Existing { condition_id: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamagePrecedenceView {
    pub damage_type: String,
    pub sources: Vec<PersistentDamagePrecedenceEntryView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PersistentDamagePrecedenceEntryView {
    pub source: PersistentDamageSourceRefView,
    pub formula: String,
    pub multiplier: DamageMultiplierView,
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
