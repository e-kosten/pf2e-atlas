//! Optional contextual action queries; absence of a target does not block browsing.
use crate::{EncounterActionCheckView, EncounterActionIssueView};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterStrikeActionContextView {
    pub issues: Vec<EncounterActionIssueView>,
    pub confused: bool,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub disruption: Option<EncounterActionCheckView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub targeting: Option<EncounterTargetingRequirementsView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterTargetingRequirementsView {
    pub prevents_effect: bool,
    pub requires_guessed_location: bool,
    pub check_senses: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterStrikeContextRequest {
    pub strike: crate::EncounterStrikeOptionsRequest,
    pub target_key: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterStrikeContextView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub targeting_check: Option<EncounterActionCheckView>,
    pub options: crate::EncounterStrikeOptionsView,
    pub action: EncounterStrikeActionContextView,
}
