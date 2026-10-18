//! Receipt-only participant rolls. These never update initiative or gameplay state.
use super::{Encounters, storage};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{TransactionBehavior, params};
use serde::{Serialize, de::DeserializeOwned};

pub struct ParticipantRollGuard<'a> {
    pub expected_revision: i64,
    pub participant_key: &'a str,
    pub expected_participant_revision: i64,
}

impl Encounters<'_> {
    pub fn participant_roll_receipt<T: DeserializeOwned>(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<T>> {
        super::initiative_rolls::validate_request(id, intent)?;
        super::initiative_rolls::recover(&self.store.connection()?, key, id, intent)
    }

    /// The application supplies bounded evaluation, after source resolution. All guards and
    /// recovery run before evaluation; receipt insertion and evaluation share one transaction.
    pub fn roll_participant<T: Serialize + DeserializeOwned>(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
        guard: ParticipantRollGuard<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<T>,
    ) -> LocalStateResult<T> {
        let ParticipantRollGuard {
            expected_revision,
            participant_key,
            expected_participant_revision,
        } = guard;
        super::initiative_rolls::validate_request(id, intent)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = super::initiative_rolls::recover(&transaction, key, id, intent)? {
            return Ok(receipt);
        }
        let snapshot = storage::get_with_participants(&transaction, key)?
            .ok_or_else(|| LocalStateError::EncounterNotFound(key.into()))?;
        if snapshot.encounter.encounter_key != key {
            return Err(LocalStateError::InvalidRollRequest(
                "use the canonical encounter key",
            ));
        }
        if snapshot.encounter.revision != expected_revision {
            return Err(LocalStateError::StaleEncounterRevision {
                encounter_key: key.into(),
                expected_revision,
                actual_revision: snapshot.encounter.revision,
            });
        }
        let participant = snapshot
            .participants
            .iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| LocalStateError::ParticipantNotFound(participant_key.into()))?;
        if participant.revision != expected_participant_revision {
            return Err(LocalStateError::StaleParticipantRevision {
                participant_key: participant_key.into(),
                expected_revision: expected_participant_revision,
                actual_revision: participant.revision,
            });
        }
        let receipt = evaluate()?;
        transaction.execute("INSERT INTO encounter_roll_receipts(encounter_id,request_id,request_json,receipt_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1", params![key,id,serde_json::to_string(intent)?,serde_json::to_string(&receipt)?])?;
        transaction.commit()?;
        Ok(receipt)
    }
}
