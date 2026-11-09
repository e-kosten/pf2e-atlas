use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollMode {
    Normal,
    Fortune,
    Misfortune,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeStatisticView {
    pub key: String,
    pub label: String,
    pub modifier: i32,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeParticipantOptionsView {
    pub participant_key: String,
    pub expected_revision: i64,
    pub statistics: Vec<InitiativeStatisticView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeOptionsView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub artifact_generation: Option<String>,
    pub encounter_key: String,
    pub expected_revision: i64,
    pub participants: Vec<InitiativeParticipantOptionsView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeParticipantRequest {
    pub participant_key: String,
    pub expected_revision: i64,
    pub statistic: String,
    pub modifier_override: Option<i32>,
    pub mode: RollMode,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollInitiativeRequest {
    pub artifact_generation: Option<String>,
    pub request_id: String,
    pub expected_revision: i64,
    pub participants: Vec<InitiativeParticipantRequest>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantRollView {
    pub participant_key: String,
    pub display_name: String,
    pub basis: String,
    pub outcome: atlas_dice::RollOutcome,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeRollView {
    pub request_id: String,
    pub encounter_key: String,
    pub rolls: Vec<ParticipantRollView>,
}

/// A reference flat check. It does not commit an encounter action or resource.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollFlatCheckRequest {
    pub dc: i64,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatCheckRollView {
    pub dc: i64,
    pub outcome: atlas_dice::RollOutcome,
    pub success: bool,
}
