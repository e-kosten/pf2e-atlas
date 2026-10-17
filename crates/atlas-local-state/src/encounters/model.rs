use atlas_domain::RecordKey;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    pub duration: Option<ConditionDuration>,
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
    pub duration: Option<ConditionDuration>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConditionTiming {
    pub participant_key: String,
    pub event: ConditionTurnEvent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConditionDuration {
    pub amount: i64,
    pub unit: ConditionDurationUnit,
    pub timing: Option<ConditionTiming>,
}
#[derive(Debug, Clone)]
pub struct EncounterTurnUpdate {
    pub turn: EncounterTurnState,
    pub durations: Vec<(i64, i64)>,
}
