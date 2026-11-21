//! Durable command identity and outcomes are saved with their gameplay writes.
use rusqlite::{Connection, OptionalExtension, params};

use super::{EncounterEventReceipt, receipt_codec};
use crate::{LocalStateError, LocalStateResult};

#[derive(Clone, Copy)]
pub(super) enum Family {
    Turn,
    Event,
    PcAddition,
    RecordAddition,
}

impl Family {
    fn as_str(self) -> &'static str {
        match self {
            Self::Turn => "turn",
            Self::Event => "event",
            Self::PcAddition => "pc_addition",
            Self::RecordAddition => "record_addition",
        }
    }

    fn conflict(self) -> LocalStateError {
        let message = match self {
            Self::RecordAddition => return LocalStateError::RecordAdditionConflict,
            Self::Turn => "request ID already used for another turn intent",
            Self::Event => "event request ID was reused with different intent",
            Self::PcAddition => "PC addition request ID was reused with different intent",
        };
        LocalStateError::InvalidRollRequest(message)
    }
}

pub(super) enum Outcome<'a> {
    RecordAddition,
    Turn(&'a str),
    Event(&'a EncounterEventReceipt),
    PcAddition(&'a super::PcAdditionReceipt),
}

fn validate_request(family: Family, id: &str, intent: &serde_json::Value) -> LocalStateResult<()> {
    if id.is_empty() || id.len() > 128 || serde_json::to_vec(intent)?.len() > 64 * 1024 {
        if matches!(family, Family::RecordAddition) {
            return Err(LocalStateError::InvalidRecordAddition(
                "Use a request ID of 1 to 128 bytes and addition details within 64 KiB.",
            ));
        }
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
    pub(super) fn pc_addition(self) -> LocalStateResult<super::PcAdditionReceipt> {
        let payload = self.payload.ok_or_else(|| {
            LocalStateError::IncompatibleSchema("PC addition receipt has no payload".into())
        })?;
        super::PcAdditionReceipt::decode(&payload)
    }
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
    intents: &[Option<&serde_json::Value>],
) -> LocalStateResult<Option<StoredReceipt>> {
    for intent in intents.iter().flatten() {
        validate_request(family, request_id, intent)?;
    }
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
        let saved: serde_json::Value = serde_json::from_str(&saved)?;
        if !intents.iter().flatten().any(|intent| **intent == saved) {
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
        Outcome::RecordAddition => (Family::RecordAddition, None, None),
        Outcome::Turn(boundary) => (Family::Turn, Some(boundary), None),
        Outcome::Event(event) => (Family::Event, None, Some(receipt_codec::encode(event)?)),
        Outcome::PcAddition(receipt) => (
            Family::PcAddition,
            None,
            Some(receipt_codec::encode(receipt)?),
        ),
    };
    let inserted = connection.execute(
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
    if inserted == 0 {
        return Err(LocalStateError::EncounterNotFound(encounter_key.into()));
    }
    Ok(())
}

impl super::EncountersRead<'_> {
    pub fn record_addition_committed(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<bool> {
        self.get(encounter_key)?
            .filter(|parent| parent.encounter_key == encounter_key)
            .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_key.into()))?;
        Ok(recover(
            self.connection,
            self.owner_id,
            encounter_key,
            Family::RecordAddition,
            request_id,
            &[Some(intent)],
        )?
        .is_some())
    }
}

impl super::EncountersWrite<'_> {
    pub fn save_record_addition_receipt(
        &self,
        encounter_key: &str,
        request_id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<()> {
        validate_request(Family::RecordAddition, request_id, intent)?;
        insert(
            self.connection,
            self.owner_id,
            encounter_key,
            request_id,
            intent,
            Outcome::RecordAddition,
        )
    }
}
