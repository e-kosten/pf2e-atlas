use std::collections::HashSet;

use atlas_dice::RollOutcome;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use super::{Encounters, UpdateEncounterParticipant, storage};
use crate::{LocalStateError, LocalStateResult};

/// Source resolution happens before entering the write transaction.
#[derive(Debug, Clone)]
pub struct ResolvedInitiativeRoll {
    pub participant_key: String,
    pub expected_revision: i64,
    pub basis: String,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantInitiativeRoll {
    pub participant_key: String,
    pub display_name: String,
    pub basis: String,
    pub outcome: RollOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeRollReceipt {
    pub request_id: String,
    pub encounter_key: String,
    pub rolls: Vec<ParticipantInitiativeRoll>,
}

impl Encounters<'_> {
    /// Recover before source hydration or revision checks, including lost-response retries.
    pub fn initiative_roll_receipt(
        &self,
        encounter_key: &str,
        request_id: &str,
        request: &serde_json::Value,
    ) -> LocalStateResult<Option<InitiativeRollReceipt>> {
        validate_request(request_id, request)?;
        recover(
            &self.store.connection()?,
            encounter_key,
            request_id,
            request,
        )
    }

    /// Roll and save a whole reviewed batch once. Receipts live until encounter deletion.
    /// `request` is the original application intent, not the derived source inputs.
    pub fn roll_initiative(
        &self,
        encounter_key: &str,
        request_id: &str,
        request: &serde_json::Value,
        expected_revision: i64,
        resolved: &[ResolvedInitiativeRoll],
    ) -> LocalStateResult<InitiativeRollReceipt> {
        validate_request(request_id, request)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = recover(&transaction, encounter_key, request_id, request)? {
            return Ok(receipt);
        }
        let snapshot = storage::get_with_participants(&transaction, encounter_key)?
            .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_key.into()))?;
        if snapshot.encounter.encounter_key != encounter_key {
            return Err(LocalStateError::InvalidRollRequest(
                "use the canonical encounter key",
            ));
        }
        if snapshot.encounter.revision != expected_revision {
            return Err(LocalStateError::StaleEncounterRevision {
                encounter_key: encounter_key.into(),
                expected_revision,
                actual_revision: snapshot.encounter.revision,
            });
        }
        if resolved.is_empty() || resolved.len() > 200 {
            return Err(LocalStateError::InvalidRollRequest(
                "select between 1 and 200 participants",
            ));
        }
        let mut seen = HashSet::new();
        // Validate the entire batch before drawing any dice or changing tie order.
        let mut prepared = Vec::with_capacity(resolved.len());
        for input in resolved {
            if !seen.insert(&input.participant_key) {
                return Err(LocalStateError::InvalidRollRequest(
                    "participant selected more than once",
                ));
            }
            let participant = snapshot
                .participants
                .iter()
                .find(|p| p.participant_key == input.participant_key)
                .ok_or_else(|| {
                    LocalStateError::ParticipantNotFound(input.participant_key.clone())
                })?;
            if participant.revision != input.expected_revision {
                return Err(LocalStateError::StaleParticipantRevision {
                    participant_key: input.participant_key.clone(),
                    expected_revision: input.expected_revision,
                    actual_revision: participant.revision,
                });
            }
            prepared.push(atlas_dice::prepare(&input.formula)?);
        }
        let mut rolls = Vec::with_capacity(resolved.len());
        for (input, prepared) in resolved.iter().zip(prepared) {
            let outcome = prepared.roll()?;
            // Earlier changes may compact ties and bump another participant's revision.
            let p =
                storage::participant(&transaction, &input.participant_key)?.ok_or_else(|| {
                    LocalStateError::ParticipantNotFound(input.participant_key.clone())
                })?;
            rolls.push(ParticipantInitiativeRoll {
                participant_key: p.participant_key.clone(),
                display_name: p.display_name.clone(),
                basis: input.basis.clone(),
                outcome: outcome.clone(),
            });
            storage::update_participant(
                &transaction,
                UpdateEncounterParticipant {
                    expected_revision: p.revision,
                    participant_key: p.participant_key,
                    display_name: p.display_name,
                    side: p.side,
                    participant_variant: p.participant_variant,
                    initiative: Some(i64::from(outcome.total)),
                    max_hp: p.max_hp,
                    current_hp: p.current_hp,
                    temporary_hp: p.temporary_hp,
                    defeated: p.defeated,
                    hidden: p.hidden,
                    note: p.note,
                    hp_origin: p.hp_origin,
                    variant_origin: p.variant_origin,
                },
            )?;
        }
        let receipt = InitiativeRollReceipt {
            request_id: request_id.into(),
            encounter_key: encounter_key.into(),
            rolls,
        };
        transaction.execute(
            "INSERT INTO encounter_roll_receipts(encounter_id,request_id,request_json,receipt_json)
             SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1",
            params![
                encounter_key,
                request_id,
                serde_json::to_string(request)?,
                serde_json::to_string(&receipt)?
            ],
        )?;
        transaction.commit()?;
        Ok(receipt)
    }
}

pub(super) fn validate_request(id: &str, request: &serde_json::Value) -> LocalStateResult<()> {
    if id.is_empty() || id.len() > 128 || serde_json::to_vec(request)?.len() > 64 * 1024 {
        return Err(LocalStateError::InvalidRollRequest(
            "request exceeds supported bounds",
        ));
    }
    Ok(())
}

pub(super) fn recover<T: serde::de::DeserializeOwned>(
    connection: &Connection,
    key: &str,
    id: &str,
    request: &serde_json::Value,
) -> LocalStateResult<Option<T>> {
    let row: Option<(String, String)> = connection
        .query_row(
            "SELECT r.request_json,r.receipt_json FROM encounter_roll_receipts r
         JOIN encounters e ON e.id=r.encounter_id WHERE e.encounter_key=?1 AND r.request_id=?2",
            params![key, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((stored, receipt)) = row else {
        return Ok(None);
    };
    if serde_json::from_str::<serde_json::Value>(&stored)? != *request {
        return Err(LocalStateError::RollRequestConflict);
    }
    Ok(Some(serde_json::from_str(&receipt)?))
}
