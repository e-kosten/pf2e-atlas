//! Saved turn resources; app-service owns allocation and permission policy.
use super::{ConditionExtraAction, EncountersWrite, receipt_codec};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnResourceEffect {
    pub condition_id: Option<i64>,
    pub name: String,
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub actions_lost: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnActionAllocation {
    pub base: i64,
    pub adjustments: Vec<TurnActionAdjustment>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnActionAdjustment {
    Quickened { gained: i64 },
    Stunned { value: Option<i64>, lost: i64 },
    Slowed { value: i64, lost: i64, overlap: i64 },
    Inactive { lost: i64 },
    StunnedDuringTurn { value: i64, lost: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnResources {
    /// A GM-granted reaction does not allocate actions or start the first turn.
    #[serde(default)]
    pub before_first_turn: bool,
    /// Older saved turns lack the facts needed to reconstruct their allocation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allocation: Option<TurnActionAllocation>,
    pub actions_open: bool,
    pub pending_reason: Option<String>,
    pub participant_key: String,
    pub turn_key: String,
    pub round_number: i64,
    pub regular_total: i64,
    pub regular_remaining: i64,
    pub extra_total: i64,
    pub extra_remaining: i64,
    pub extra_uses: Vec<ConditionExtraAction>,
    pub reactions_total: Option<i64>,
    pub reactions_remaining: Option<i64>,
    pub effects: Vec<TurnResourceEffect>,
}
impl EncountersWrite<'_> {
    pub(super) fn save_turn_resources(
        &self,
        encounter_key: &str,
        value: &TurnResources,
    ) -> LocalStateResult<()> {
        let changed = self.connection.execute(
            "UPDATE encounter_participants SET turn_resources_json=?3,revision=revision+1 WHERE participant_key=?1 AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?2)",
            params![value.participant_key,encounter_key,receipt_codec::encode(value)?],
        )?;
        if changed != 1 {
            return Err(LocalStateError::ParticipantNotFound(
                value.participant_key.clone(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn older_turn_stock_remains_readable_without_inventing_allocation_facts() {
        let saved: TurnResources = serde_json::from_value(serde_json::json!({
            "actions_open":true,"pending_reason":null,"participant_key":"participant",
            "turn_key":"turn:older","round_number":1,"regular_total":2,
            "regular_remaining":1,"extra_total":0,"extra_remaining":0,"extra_uses":[],
            "reactions_total":1,"reactions_remaining":1,"effects":[]
        }))
        .unwrap();
        assert_eq!(saved.regular_total, 2);
        assert_eq!(saved.regular_remaining, 1);
        assert!(saved.allocation.is_none());
        assert!(!saved.before_first_turn);
        assert_eq!(
            receipt_codec::decode::<TurnResources>(&receipt_codec::encode(&saved).unwrap())
                .unwrap(),
            saved
        );
    }
}
