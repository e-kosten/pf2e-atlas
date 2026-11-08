//! Previewed encounter reset and full-night recovery.
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterRecoveryAction {
    ResetEncounter,
    ResetParticipant {
        participant_key: String,
    },
    RestOvernight {
        healing_mode: OvernightHealingMode,
        participants: Vec<OvernightParticipantInput>,
    },
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OvernightHealingMode {
    Natural,
    Full,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OvernightParticipantInput {
    pub participant_key: String,
    /// None selects the known Constitution × level amount; zero skips healing.
    pub healing: Option<i64>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewEncounterRecoveryRequest {
    pub expected_encounter_revision: i64,
    pub action: EncounterRecoveryAction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyEncounterRecoveryRequest {
    pub expected_encounter_revision: i64,
    pub action: EncounterRecoveryAction,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterRecoveryParticipantView {
    pub participant_key: String,
    pub display_name: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub current_hp: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub resulting_hp: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub resulting_max_hp: Option<i64>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub healing: Option<i64>,
    pub changes: Vec<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable_reason: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterRecoveryPreviewView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub rules_reference: Option<crate::RecordNavigationView>,
    pub encounter_key: String,
    pub encounter_revision: i64,
    pub participants: Vec<EncounterRecoveryParticipantView>,
    pub changes: Vec<String>,
    pub can_apply: bool,
}
