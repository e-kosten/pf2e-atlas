use atlas_domain::RecordKey;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Encounter {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterParticipant {
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
    pub hidden: bool,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub conditions: Vec<EncounterParticipantCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterParticipantCondition {
    pub condition_id: i64,
    pub condition_key: Option<String>,
    pub name: String,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub duration_rounds: Option<i64>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterWithParticipants {
    pub encounter: Encounter,
    pub participants: Vec<EncounterParticipant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterParticipantSpellState {
    pub initialized: bool,
    pub resources: Vec<EncounterSpellResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterSpellResource {
    pub target: EncounterSpellResourceTarget,
    pub maximum: i64,
    pub initial_remaining: i64,
    pub remaining: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EncounterSpellResourceTarget {
    PreparedSlot {
        entry_id: String,
        spell_occurrence_id: String,
        rank: i64,
        slot_id: String,
    },
    SpontaneousPool {
        entry_id: String,
        rank: i64,
    },
    InnateUse {
        entry_id: Option<String>,
        spell_occurrence_id: String,
    },
    FocusPool {
        resource_id: String,
    },
}

impl EncounterSpellResourceTarget {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::PreparedSlot { .. } => "prepared_slot",
            Self::SpontaneousPool { .. } => "spontaneous_pool",
            Self::InnateUse { .. } => "innate_use",
            Self::FocusPool { .. } => "focus_pool",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterSpellResourceOperation {
    CastOne,
    RestoreOne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterSpellResourceMutation {
    pub before: EncounterSpellResource,
    pub after: EncounterSpellResource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncounterParticipantReset {
    pub participant: EncounterParticipant,
    pub cleared_current_turn: bool,
    pub reset_domains: Vec<EncounterParticipantResetDomain>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterParticipantResetDomain {
    HitPoints,
    Defeated,
    Conditions,
    InitiativeTurnState,
    VariantAdjustments,
    ActionBudget,
    SpellResources,
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
    pub participant_key: String,
    pub display_name: String,
    pub side: ParticipantSide,
    pub participant_variant: ParticipantVariant,
    pub initiative: Option<i64>,
    pub max_hp: Option<i64>,
    pub current_hp: Option<i64>,
    pub temporary_hp: i64,
    pub defeated: bool,
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
    pub duration_rounds: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEncounterParticipantCondition {
    pub condition_id: i64,
    pub condition_key: Option<String>,
    pub name: String,
    pub value: Option<i64>,
    pub source_participant_key: Option<String>,
    pub duration_rounds: Option<i64>,
    pub note: Option<String>,
}
