//! Optional contextual action queries; absence of a target does not block browsing.
use crate::{EncounterActionCheckView, EncounterActionIssueView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeActionContextView {
    pub issues: Vec<EncounterActionIssueView>,
    pub confused: bool,
    pub disruption: Option<EncounterActionCheckView>,
    pub targeting: Option<EncounterTargetingRequirementsView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterTargetingRequirementsView {
    pub prevents_effect: bool,
    pub requires_guessed_location: bool,
    pub check_senses: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EncounterStrikeContextRequest {
    pub strike: crate::EncounterStrikeOptionsRequest,
    pub target_key: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterStrikeContextView {
    pub targeting_check: Option<EncounterActionCheckView>,
    pub options: crate::EncounterStrikeOptionsView,
    pub action: EncounterStrikeActionContextView,
}
