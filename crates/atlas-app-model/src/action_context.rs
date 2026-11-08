//! Shared condition advice and input facts for live encounter actions.
use crate::RecordNavigationView;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionFactView {
    Reaction,
    Manipulate,
    Auditory,
    Concentrate,
    Move,
    Attack,
    MentalOnly,
    RelatesToFascination,
    FurthersEscape,
    VoluntaryIngestion,
    FollowsControllerInstructions,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterCastingRemindersRequest {
    pub participant_key: String,
    pub selection: RecordNavigationView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncounterActionIssueKindView {
    Prohibited,
    NeedsFact,
    NeedsValue,
    NeedsContext,
    NeedsConfusion,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterActionIssueView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub subject: Option<crate::ConditionSubjectView>,
    pub source_key: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_condition_id: Option<i64>,
    pub condition_name: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub controller_key: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub instructions: Option<String>,
    pub kind: EncounterActionIssueKindView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub fact: Option<EncounterActionFactView>,
    pub reason: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterActionCheckSourceView {
    pub source_key: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub condition_id: Option<i64>,
    pub name: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterActionCheckView {
    pub dc: i64,
    pub sources: Vec<EncounterActionCheckSourceView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterCastingRemindersView {
    pub advisories: Vec<crate::RuntimeEffectNoteView>,
}
