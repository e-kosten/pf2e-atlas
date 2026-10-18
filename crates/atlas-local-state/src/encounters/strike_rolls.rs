//! Durable strike rolls. These never update initiative or gameplay state.
use super::{EncountersRead, EncountersWrite, storage};
use super::{StoredStrikeReceipt, StoredStrikeRoll};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::params;

pub struct ParticipantRollGuard<'a> {
    pub expected_revision: i64,
    pub participant_key: &'a str,
    pub expected_participant_revision: i64,
}

impl EncountersWrite<'_> {
    /// The application supplies bounded evaluation, after source resolution. All guards and
    /// recovery run before evaluation; receipt insertion and evaluation share one transaction.
    pub fn roll_strike(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
        guard: ParticipantRollGuard<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<StoredStrikeRoll>,
    ) -> LocalStateResult<StoredStrikeReceipt> {
        crate::atomic_action::run(self.connection, || {
            let ParticipantRollGuard {
                expected_revision,
                participant_key,
                expected_participant_revision,
            } = guard;
            super::initiative_rolls::validate_request(id, intent)?;
            let transaction = self.connection;
            if let Some(receipt) = recover(transaction, key, id, intent)? {
                return Ok(receipt);
            }
            let snapshot = storage::get_with_participants(transaction, key)?
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
            let receipt = StoredStrikeReceipt {
                request_id: id.into(),
                encounter_key: key.into(),
                participant_key: participant_key.into(),
                roll: evaluate()?,
            };
            receipt.validate()?;
            transaction.execute("INSERT INTO encounter_roll_receipts(encounter_id,request_id,request_json,receipt_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1", params![key,id,serde_json::to_string(intent)?,super::strike_receipt::encode(&receipt)?])?;

            Ok(receipt)
        })
    }
}

impl EncountersRead<'_> {
    pub fn strike_roll_receipt(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<StoredStrikeReceipt>> {
        super::initiative_rolls::validate_request(id, intent)?;
        recover(self.connection, key, id, intent)
    }
}

fn recover(
    connection: &rusqlite::Connection,
    key: &str,
    id: &str,
    intent: &serde_json::Value,
) -> LocalStateResult<Option<StoredStrikeReceipt>> {
    let payload: Option<super::strike_receipt::StrikePayload<StoredStrikeReceipt>> =
        super::initiative_rolls::recover(connection, key, id, intent)?;
    let receipt = payload.map(|payload| payload.receipt);
    if let Some(value) = &receipt {
        value.validate()?;
        if value.request_id != id || value.encounter_key != key {
            return Err(LocalStateError::InvalidRollRequest(
                "stored strike receipt identity does not match its request",
            ));
        }
    }
    Ok(receipt)
}
