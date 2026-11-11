use serde::{Deserialize, Serialize};

/// Null fields follow the PC snapshot; explicit values are GM overrides.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EncounterDifficultyReferenceView {
    #[cfg_attr(feature = "openapi", schema(required))]
    pub party_level: Option<u32>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub party_size: Option<u32>,
}
/// Omitted or null fields follow the PC snapshot.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EncounterDifficultyReferenceInput {
    pub party_level: Option<u32>,
    pub party_size: Option<u32>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetEncounterDifficultyRequest {
    pub expected_revision: i64,
    pub reference: EncounterDifficultyReferenceInput,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewEncounterDifficultyRequest {
    pub expected_revision: i64,
    pub reference: EncounterDifficultyReferenceInput,
    pub addition: Option<EncounterDifficultyAddition>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterDifficultyAddition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub creature_scenario: Option<crate::CreatureScenarioContext>,
    pub record_ref: String,
    pub quantity: u32,
    pub participant_variant: Option<crate::EncounterParticipantVariantView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterDifficultyView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub prepared_creature_choice: Option<crate::PreparedCreatureChoice>,
    pub rules: Vec<EncounterDifficultyRuleView>,
    pub reference: EncounterDifficultyReferenceView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub party_level: Option<u32>,
    /// Effective budget count, including any supported mixed-level adjustment.
    #[cfg_attr(feature = "openapi", schema(required))]
    pub party_size: Option<u32>,
    pub pc_count: u32,
    pub basis: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub reference_note: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub total_xp: Option<u32>,
    pub known_xp: u32,
    /// Suggested award normalized to a four-PC party; not persisted or distributed.
    #[cfg_attr(feature = "openapi", schema(required))]
    pub xp_per_pc: Option<u32>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub award_note: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub rating: Option<String>,
    pub budgets: Vec<EncounterDifficultyBudgetView>,
    pub opposition: Vec<EncounterDifficultyOpponentView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterDifficultyBudgetView {
    pub label: String,
    pub xp: i32,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterDifficultyOpponentView {
    /// None for a prospective addition, which has no saved participant identity.
    #[cfg_attr(feature = "openapi", schema(required))]
    pub participant_key: Option<String>,
    pub name: String,
    pub kind: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub level: Option<i32>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub xp: Option<u32>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unavailable_reason: Option<String>,
}

/// Curated pinned rules destinations, like the source-linked condition catalog.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterDifficultyRuleView {
    pub title: String,
    pub navigation: crate::RecordNavigationView,
}
