/// Optional GM overrides; absent values follow the encounter's PC snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct EncounterDifficultyReference {
    pub party_level: Option<u32>,
    pub party_size: Option<u32>,
}

use super::{ConditionOrigin, ConditionParameters, MortalityState};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    #[serde(default = "default_round_duration_seconds")]
    pub round_duration_seconds: i64,
    #[serde(default)]
    pub difficulty_reference: EncounterDifficultyReference,
    pub revision: i64,
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub note: Option<String>,
    pub status: EncounterStatus,
    pub round_number: i64,
    pub current_turn_participant_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Encounter metadata and its saved roster size, without participant detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterWithParticipantCount {
    pub encounter: Encounter,
    pub participant_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterStatus {
    Draft,
    Running,
    Complete,
    Archived,
}

impl EncounterStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Archived => "archived",
        }
    }

    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "running" => Ok(Self::Running),
            "complete" => Ok(Self::Complete),
            "archived" => Ok(Self::Archived),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "status",
                value: value.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterParticipant {
    #[serde(default)]
    pub manual_trackers: Vec<super::ManualTracker>,
    #[serde(default)]
    pub ability_states: Vec<super::ParticipantAbilityState>,
    #[serde(default)]
    pub turn_resources: Option<super::TurnResources>,
    pub level_override: Option<i64>,
    pub base_death_threshold: i64,
    pub persistent_precedence: Vec<super::PersistentSourcePrecedence>,
    pub track_dying: bool,
    pub mortality: MortalityState,
    pub spell_resources: Vec<SpellResource>,
    pub expended_spells: Vec<String>,
    pub revision: i64,
    pub hp_origin: ParticipantHpOrigin,
    pub variant_origin: ParticipantVariantOrigin,
    pub participant_key: String,
    pub record_key: Option<String>,
    pub participant_kind: ParticipantKind,
    pub participant_variant: ParticipantVariant,
    pub position: i64,
    pub display_name: String,
    pub record_title_snapshot: Option<String>,
    pub record_kind_snapshot: Option<String>,
    pub side: ParticipantSide,
    pub initiative: Option<i64>,
    pub initiative_order: i64,
    pub max_hp: Option<i64>,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hazard_disabled: bool,
    pub hidden: bool,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub conditions: Vec<EncounterParticipantCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantVariant {
    Normal,
    Elite,
    Weak,
}

impl ParticipantVariant {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Elite => "elite",
            Self::Weak => "weak",
        }
    }

    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "normal" => Ok(Self::Normal),
            "elite" => Ok(Self::Elite),
            "weak" => Ok(Self::Weak),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "participant_variant",
                value: value.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionScope {
    #[default]
    Everyone,
    SelectedParticipants {
        participant_keys: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConditionExtraAction {
    #[default]
    Unspecified,
    Unrestricted,
    Restricted {
        actions: Vec<ConditionAction>,
        guidance: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionAction {
    Stride,
    Strike,
    Step,
    Leap,
    Stand,
    Interact,
    CastSpell,
    Sustain,
    Escape,
    Crawl,
    Fly,
    Swim,
    Climb,
    Burrow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterParticipantCondition {
    pub condition_id: i64,
    pub condition_key: Option<String>,
    pub name: String,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub scope: ConditionScope,
    pub extra_action: ConditionExtraAction,
    pub immunity_override: bool,
    pub parameters: ConditionParameters,
    pub origin: Option<ConditionOrigin>,
    pub duration: Option<ConditionDuration>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantKind {
    Creature,
    Hazard,
    Pc,
}

impl ParticipantKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Creature => "creature",
            Self::Hazard => "hazard",
            Self::Pc => "pc",
        }
    }

    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "creature" => Ok(Self::Creature),
            "hazard" => Ok(Self::Hazard),
            "pc" => Ok(Self::Pc),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "participant_kind",
                value: value.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantSide {
    Pc,
    Ally,
    Enemy,
    Neutral,
    Hazard,
}

impl ParticipantSide {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pc => "pc",
            Self::Ally => "ally",
            Self::Enemy => "enemy",
            Self::Neutral => "neutral",
            Self::Hazard => "hazard",
        }
    }

    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "pc" => Ok(Self::Pc),
            "ally" => Ok(Self::Ally),
            "enemy" => Ok(Self::Enemy),
            "neutral" => Ok(Self::Neutral),
            "hazard" => Ok(Self::Hazard),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "side",
                value: value.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterWithParticipants {
    pub encounter: Encounter,
    pub participants: Vec<EncounterParticipant>,
}

/// An application-chosen turn transition, persisted atomically with its snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterTurnState {
    pub participant_key: Option<String>,
    pub round_number: i64,
    pub mark_running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEncounter {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEncounter {
    pub round_duration_seconds: Option<i64>,
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub note: Option<String>,
    pub status: EncounterStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddEncounterParticipant {
    pub level_override: Option<i64>,
    pub spell_resources: Vec<SpellResource>,
    pub participant_variant: ParticipantVariant,
    pub hp_origin: ParticipantHpOrigin,
    pub variant_origin: ParticipantVariantOrigin,
    pub record_key: Option<RecordKey>,
    pub participant_kind: ParticipantKind,
    pub display_name: String,
    pub record_title_snapshot: Option<String>,
    pub record_kind_snapshot: Option<String>,
    pub side: ParticipantSide,
    pub initiative: Option<i64>,
    pub max_hp: Option<i64>,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEncounterParticipant {
    pub expected_revision: i64,
    pub hp_origin: ParticipantHpOrigin,
    pub variant_origin: ParticipantVariantOrigin,
    pub participant_key: String,
    pub display_name: String,
    pub side: ParticipantSide,
    pub participant_variant: ParticipantVariant,
    pub initiative: Option<i64>,
    pub max_hp: Option<i64>,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
    pub hazard_disabled: bool,
    pub hidden: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReorderPlacement {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReorderEncounterParticipant {
    pub participant_key: String,
    pub target_participant_key: String,
    pub placement: ReorderPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddEncounterParticipantCondition {
    pub participant_key: String,
    pub condition_key: Option<String>,
    pub name: String,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub scope: ConditionScope,
    pub extra_action: ConditionExtraAction,
    pub immunity_override: bool,
    pub parameters: ConditionParameters,
    pub origin: Option<ConditionOrigin>,
    pub duration: Option<ConditionDuration>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEncounterParticipantCondition {
    pub expected_encounter_revision: i64,
    pub participant_key: String,
    pub condition_id: i64,
    pub condition_key: Option<String>,
    pub name: String,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub scope: ConditionScope,
    pub extra_action: ConditionExtraAction,
    pub immunity_override: bool,
    pub parameters: ConditionParameters,
    pub origin: Option<ConditionOrigin>,
    pub duration: Option<ConditionDuration>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantHpOrigin {
    DerivedPristine,
    DerivedEdited,
    #[default]
    Explicit,
    Unknown,
}
impl ParticipantHpOrigin {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DerivedPristine => "derived_pristine",
            Self::DerivedEdited => "derived_edited",
            Self::Explicit => "explicit",
            Self::Unknown => "unknown",
        }
    }
    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "derived_pristine" => Ok(Self::DerivedPristine),
            "derived_edited" => Ok(Self::DerivedEdited),
            "explicit" => Ok(Self::Explicit),
            "unknown" => Ok(Self::Unknown),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "hp_origin",
                value: value.into(),
            }),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantVariantOrigin {
    DefaultUnadjusted,
    InheritedKnown,
    InheritedUnknown,
    #[default]
    Explicit,
}
impl ParticipantVariantOrigin {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DefaultUnadjusted => "default_unadjusted",
            Self::InheritedKnown => "inherited_known",
            Self::InheritedUnknown => "inherited_unknown",
            Self::Explicit => "explicit",
        }
    }
    pub(crate) fn from_str(value: &str) -> crate::LocalStateResult<Self> {
        match value {
            "default_unadjusted" => Ok(Self::DefaultUnadjusted),
            "inherited_known" => Ok(Self::InheritedKnown),
            "inherited_unknown" => Ok(Self::InheritedUnknown),
            "explicit" => Ok(Self::Explicit),
            _ => Err(crate::LocalStateError::UnsupportedMetadata {
                key: "variant_origin",
                value: value.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionDurationUnit {
    Rounds,
    Minutes,
    Hours,
    Days,
}
impl ConditionDurationUnit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Rounds => "rounds",
            Self::Minutes => "minutes",
            Self::Hours => "hours",
            Self::Days => "days",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionTurnEvent {
    Start,
    End,
}
impl ConditionTurnEvent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionTiming {
    pub participant_key: String,
    pub event: ConditionTurnEvent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionDuration {
    /// Exact seconds remaining for anchored non-round durations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_seconds: Option<i64>,
    pub amount: i64,
    pub unit: ConditionDurationUnit,
    pub timing: Option<ConditionTiming>,
}
#[derive(Debug, Clone)]
pub struct EncounterTurnUpdate {
    pub manual_trackers: Vec<super::ManualTrackerChange>,
    pub start_event: Option<super::PreparedConditionEvent>,
    pub start_claims: Vec<super::EncounterBoundaryClaim>,
    pub turn: EncounterTurnState,
    pub durations: Vec<(i64, i64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellResource {
    pub key: String,
    pub capacity: i64,
    pub remaining: i64,
    pub undo_remaining: Option<i64>,
}

fn default_round_duration_seconds() -> i64 {
    6
}
