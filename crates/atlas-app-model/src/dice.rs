use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RollMode {
    Normal,
    Fortune,
    Misfortune,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InitiativeStatisticView {
    pub key: String,
    pub label: String,
    pub modifier: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InitiativeParticipantOptionsView {
    pub participant_key: String,
    pub expected_revision: i64,
    pub statistics: Vec<InitiativeStatisticView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InitiativeOptionsView {
    pub artifact_generation: Option<String>,
    pub encounter_key: String,
    pub expected_revision: i64,
    pub participants: Vec<InitiativeParticipantOptionsView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InitiativeParticipantRequest {
    pub participant_key: String,
    pub expected_revision: i64,
    pub statistic: String,
    pub modifier_override: Option<i32>,
    pub mode: RollMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct RollInitiativeRequest {
    pub artifact_generation: Option<String>,
    pub request_id: String,
    pub expected_revision: i64,
    pub participants: Vec<InitiativeParticipantRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ParticipantRollView {
    pub participant_key: String,
    pub display_name: String,
    pub basis: String,
    pub outcome: atlas_dice::RollOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InitiativeRollView {
    pub request_id: String,
    pub encounter_key: String,
    pub rolls: Vec<ParticipantRollView>,
}

/// A reference flat check. It does not commit an encounter action or resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RollFlatCheckRequest {
    pub dc: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FlatCheckRollView {
    pub dc: i64,
    pub outcome: atlas_dice::RollOutcome,
    pub success: bool,
}
