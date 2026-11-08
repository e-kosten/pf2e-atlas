//! Durable command identity and outcomes are saved with their gameplay writes.
use rusqlite::{Connection, OptionalExtension, params};

use super::{EncounterEventReceipt, receipt_codec};
use crate::{LocalStateError, LocalStateResult};

#[derive(Clone, Copy)]
pub(super) enum Family {
    Turn,
    Event,
}

impl Family {
    fn as_str(self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::Event => "event",
        }
    }

    fn conflict(self) -> LocalStateError {
        LocalStateError::InvalidRollRequest(match self {
            Self::Turn => "request ID already used for another turn intent",
            Self::Event => "event request ID was reused with different intent",
        })
    }
}

pub(super) enum Outcome<'a> {
    Turn(&'a str),
    Event(&'a EncounterEventReceipt),
}

fn validate_request(id: &str, intent: &serde_json::Value) -> LocalStateResult<()> {
    if id.is_empty() || id.len() > 128 || serde_json::to_vec(intent)?.len() > 64 * 1024 {
        return Err(LocalStateError::InvalidRollRequest(
            "request exceeds supported bounds",
        ));
    }
    Ok(())
}

pub(super) struct StoredReceipt {
    payload: Option<String>,
}

impl StoredReceipt {
    pub(super) fn event(self) -> LocalStateResult<EncounterEventReceipt> {
        let payload = self.payload.ok_or_else(|| {
            LocalStateError::IncompatibleSchema("event receipt has no payload".into())
        })?;
        receipt_codec::decode(&payload)
    }
}

pub(super) fn recover(
    connection: &Connection,
    owner_id: i64,
    encounter_ref: &str,
    family: Family,
    request_id: &str,
    intent: &serde_json::Value,
) -> LocalStateResult<Option<StoredReceipt>> {
    validate_request(request_id, intent)?;
    let row: Option<(String, Option<String>)> = connection
        .query_row(
            "SELECT r.intent_json,r.receipt_json FROM encounter_command_receipts r
             JOIN encounters e ON e.id=r.encounter_id
             WHERE (e.encounter_key=?1 OR (?2='turn' AND e.slug=?1))
               AND r.family=?2 AND r.request_id=?3 AND e.owner_id=?4",
            params![encounter_ref, family.as_str(), request_id, owner_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    row.map(|(saved, payload)| {
        if serde_json::from_str::<serde_json::Value>(&saved)? != *intent {
            return Err(family.conflict());
        }
        Ok(StoredReceipt { payload })
    })
    .transpose()
}

pub(super) fn insert(
    connection: &Connection,
    owner_id: i64,
    encounter_key: &str,
    request_id: &str,
    intent: &serde_json::Value,
    outcome: Outcome<'_>,
) -> LocalStateResult<()> {
    let (family, boundary, receipt) = match outcome {
        Outcome::Turn(boundary) => (Family::Turn, Some(boundary), None),
        Outcome::Event(event) => (Family::Event, None, Some(receipt_codec::encode(event)?)),
    };
    connection.execute(
        "INSERT INTO encounter_command_receipts
             (encounter_id,family,request_id,intent_json,boundary_key,receipt_json)
         SELECT id,?2,?3,?4,?5,?6 FROM encounters WHERE encounter_key=?1 AND owner_id=?7",
        params![
            encounter_key,
            family.as_str(),
            request_id,
            serde_json::to_string(intent)?,
            boundary,
            receipt,
            owner_id,
        ],
    )?;
    Ok(())
}
