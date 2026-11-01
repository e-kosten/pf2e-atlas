//! Bounded prepared changes; gameplay policy belongs to app-service.
use super::{
    AddEncounterParticipantCondition, MortalityState, UpdateEncounterParticipantCondition,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ConditionEventChange {
    Add(AddEncounterParticipantCondition),
    Replace(UpdateEncounterParticipantCondition),
    Remove {
        participant_key: String,
        condition_id: i64,
    },
}
#[derive(Debug, Clone)]
pub struct ConsumedConditionApplication {
    pub participant_key: String,
    pub origin: super::ConditionOrigin,
}
#[derive(Debug, Clone)]
pub struct ParticipantEventChange {
    pub level_override: Option<i64>,
    pub base_death_threshold: i64,
    pub participant_key: String,
    pub expected_revision: i64,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub track_dying: bool,
    pub mortality: MortalityState,
    pub defeated: bool,
    pub hazard_disabled: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PendingConsequence {
    ConfusedDamageRecovery {
        participant_key: String,
        condition_ids: Vec<i64>,
        qualifying_damage: Option<bool>,
    },
    BrokenHpThreshold {
        participant_key: String,
    },
    InitiativePlacement {
        #[serde(default)]
        context: SavedHpEventContext,
        participant_key: String,
        before_hp: i64,
        after_hp: i64,
    },
    HpLifecycle {
        participant_key: String,
        before_hp: i64,
        after_hp: i64,
        damage_amount: i64,
        was_dying: i64,
        was_wounded: i64,
        was_doomed: i64,
        context: SavedHpEventContext,
    },
    DrainedHpLoss {
        participant_key: String,
        magnitude_increase: i64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPendingConsequence {
    pub consequence_key: String,
    pub consequence: PendingConsequence,
}
#[derive(Debug, Clone, Default)]
pub struct PreparedConditionEvent {
    pub manual_trackers: Vec<super::ManualTrackerChange>,
    pub ability_states: Vec<super::AbilityStateChange>,
    pub consumed_applications: Vec<ConsumedConditionApplication>,
    pub turn_resources: Vec<super::TurnResources>,
    pub condition_changes: Vec<ConditionEventChange>,
    pub participant_changes: Vec<ParticipantEventChange>,
    pub pending: Vec<SavedPendingConsequence>,
    pub resolved_pending_keys: Vec<String>,
    pub outcomes: Vec<ConditionEventOutcome>,
    pub precedence_changes: Vec<PersistentPrecedenceChange>,
    pub initiative_moves: Vec<super::ReorderEncounterParticipant>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterEventReceipt {
    pub encounter_key: String,
    pub request_id: String,
    pub added_condition_ids: Vec<i64>,
    pub outcomes: Vec<ConditionEventOutcome>,
    pub pending: Vec<SavedPendingConsequence>,
}

impl ParticipantEventChange {
    pub fn preserving(participant: &super::EncounterParticipant) -> Self {
        Self {
            level_override: participant.level_override,
            base_death_threshold: participant.base_death_threshold,
            participant_key: participant.participant_key.clone(),
            expected_revision: participant.revision,
            current_hp: participant.current_hp,
            temporary_hp: participant.temporary_hp,
            track_dying: participant.track_dying,
            mortality: participant.mortality,
            defeated: participant.defeated,
            hazard_disabled: participant.hazard_disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionEventOutcome {
    PersistentPrepared {
        context: Box<PersistentPreparationContext>,
        groups: Vec<PersistentDamageOutcome>,
        hardness: i64,
        complete: bool,
        recovery: Option<PreparedPersistentRecovery>,
    },
    LifecycleCheck {
        participant_key: String,
        purpose: LifecycleCheckPurpose,
        dc: Option<i64>,
        modifier: Option<i64>,
        roll: Option<atlas_dice::RollOutcome>,
        table_result: Option<i64>,
        degree: CheckDegree,
        table_actions: u8,
    },
    PersistentDamage {
        participant_key: String,
        groups: Vec<PersistentDamageOutcome>,
        total: i64,
        hardness_absorbed: i64,
    },
    PersistentRecovery {
        episode_id: String,
        damage_type: String,
        dc: Option<i64>,
        roll: Option<atlas_dice::RollOutcome>,
        success: bool,
    },
}

/// Source identity and boundary binding; protections are re-evaluated from retained raw dice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentPreparationContext {
    pub participant_key: String,
    pub record_key: Option<String>,
    pub boundary_key: Option<String>,
    pub artifact_generation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedPersistentRecovery {
    pub episode_id: String,
    pub damage_type: String,
    pub dc: i64,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub success: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentDamageOutcome {
    pub episode_id: String,
    pub damage_type: String,
    pub source_condition_id: i64,
    pub roll: Option<atlas_dice::RollOutcome>,
    pub amount: i64,
    pub table_resolved: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protection: Option<PersistentDamageProtection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageLethality {
    Lethal,
    Nonlethal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SavedHpEventContext {
    #[serde(default)]
    pub keep_current_initiative: Option<bool>,
    #[serde(default)]
    pub keep_upright: Option<bool>,
    #[serde(default)]
    pub natural_rest_healing: Option<bool>,
    pub attack_or_spell_damage: Option<bool>,
    pub broken_threshold: Option<i64>,
    pub lethality: Option<DamageLethality>,
    pub critical: Option<bool>,
    pub enemy_source: Option<bool>,
    pub responsible_participant_key: Option<String>,
    pub base_death_threshold: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PersistentPrecedenceChange {
    pub participant_key: String,
    pub episode_id: String,
    pub sources: Vec<ConditionEventSource>,
}

/// A source already saved, or an addition in this event's ordered list of additions.
#[derive(Debug, Clone, Copy)]
pub enum ConditionEventSource {
    Existing(i64),
    Added(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterBoundaryKind {
    PreparedPersistentDamage,
    PreparedPersistentRecovery,
    PersistentDamage,
    PersistentRecovery,
    TurnStart,
    TurnEnd,
}
impl EncounterBoundaryKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PreparedPersistentDamage => "prepared_persistent_damage",
            Self::PreparedPersistentRecovery => "prepared_persistent_recovery",
            Self::PersistentDamage => "persistent_damage",
            Self::PersistentRecovery => "persistent_recovery",
            Self::TurnStart => "turn_start",
            Self::TurnEnd => "turn_end",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterBoundaryClaim {
    pub participant_key: String,
    pub round_number: i64,
    pub kind: EncounterBoundaryKind,
    pub scope_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckDegree {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleCheckPurpose {
    DyingRecovery,
    Retch,
    WakeFromNoise,
    ConfusedDamageRecovery,
}

/// Claims and the optional saved outgoing turn that authorizes their original round.
#[derive(Debug, Clone, Copy)]
pub struct ConditionEventBoundary<'a> {
    pub claims: &'a [EncounterBoundaryClaim],
    pub outgoing_boundary_key: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentDamageProtection {
    pub immune: bool,
    pub weakness: i64,
    pub resistance: i64,
}
