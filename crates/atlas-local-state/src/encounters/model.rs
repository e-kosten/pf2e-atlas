use super::{ConditionOrigin, ConditionParameters, MortalityState};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
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

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "complete" => Self::Complete,
            "archived" => Self::Archived,
            _ => Self::Draft,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterParticipant {
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

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "elite" => Self::Elite,
            "weak" => Self::Weak,
            _ => Self::Normal,
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

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "hazard" => Self::Hazard,
            "pc" => Self::Pc,
            _ => Self::Creature,
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

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "pc" => Self::Pc,
            "ally" => Self::Ally,
            "neutral" => Self::Neutral,
            "hazard" => Self::Hazard,
            _ => Self::Enemy,
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
    pub encounter_key: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub note: Option<String>,
    pub status: EncounterStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddEncounterParticipant {
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
    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "derived_pristine" => Self::DerivedPristine,
            "derived_edited" => Self::DerivedEdited,
            "unknown" => Self::Unknown,
            _ => Self::Explicit,
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
    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "default_unadjusted" => Self::DefaultUnadjusted,
            "inherited_known" => Self::InheritedKnown,
            "inherited_unknown" => Self::InheritedUnknown,
            _ => Self::Explicit,
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
    pub amount: i64,
    pub unit: ConditionDurationUnit,
    pub timing: Option<ConditionTiming>,
}
#[derive(Debug, Clone)]
pub struct EncounterTurnUpdate {
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
