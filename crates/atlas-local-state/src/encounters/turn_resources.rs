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
pub struct TurnResources {
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
