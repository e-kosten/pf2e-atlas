use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Null fields follow the PC snapshot; explicit values are GM overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyReferenceView {
    pub party_level: Option<u32>,
    pub party_size: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SetEncounterDifficultyRequest {
    pub expected_revision: i64,
    pub reference: EncounterDifficultyReferenceView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreviewEncounterDifficultyRequest {
    pub expected_revision: i64,
    pub reference: EncounterDifficultyReferenceView,
    pub addition: Option<EncounterDifficultyAddition>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyAddition {
    pub record_ref: String,
    pub quantity: u32,
    pub participant_variant: Option<crate::EncounterParticipantVariantView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyView {
    pub rules: Vec<EncounterDifficultyRuleView>,
    pub reference: EncounterDifficultyReferenceView,
    pub party_level: Option<u32>,
    /// Effective budget count, including any supported mixed-level adjustment.
    pub party_size: Option<u32>,
    pub pc_count: u32,
    pub basis: String,
    pub reference_note: Option<String>,
    pub total_xp: Option<u32>,
    pub known_xp: u32,
    /// Suggested award normalized to a four-PC party; not persisted or distributed.
    pub xp_per_pc: Option<u32>,
    pub award_note: Option<String>,
    pub rating: Option<String>,
    pub budgets: Vec<EncounterDifficultyBudgetView>,
    pub opposition: Vec<EncounterDifficultyOpponentView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyBudgetView {
    pub label: String,
    pub xp: i32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyOpponentView {
    /// None for a prospective addition, which has no saved participant identity.
    pub participant_key: Option<String>,
    pub name: String,
    pub kind: String,
    pub level: Option<i32>,
    pub xp: Option<u32>,
    pub unavailable_reason: Option<String>,
}

/// Curated pinned rules destinations, like the source-linked condition catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EncounterDifficultyRuleView {
    pub title: String,
    pub navigation: crate::RecordNavigationView,
}
