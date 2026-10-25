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
pub struct EncounterDifficultyView {
    pub reference: EncounterDifficultyReferenceView,
    pub party_level: Option<u32>,
    /// Effective budget count, including any supported mixed-level adjustment.
    pub party_size: Option<u32>,
    pub pc_count: u32,
    pub basis: String,
    pub reference_note: Option<String>,
    pub total_xp: Option<u32>,
    pub known_xp: u32,
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
    pub participant_key: String,
    pub name: String,
    pub kind: String,
    pub level: Option<i32>,
    pub xp: Option<u32>,
    pub unavailable_reason: Option<String>,
}
