use super::{EncountersRead, EncountersWrite};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{OptionalExtension, params};

pub(crate) fn valid_creation_identity(request_id: &str, intent_digest: &str) -> bool {
    (1..=128).contains(&request_id.len())
        && intent_digest.len() == 64
        && intent_digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate(request_id: &str, intent_digest: &str) -> LocalStateResult<()> {
    if valid_creation_identity(request_id, intent_digest) {
        Ok(())
    } else {
        Err(LocalStateError::InvalidRecordAddition(
            "Use a request ID of 1 to 128 bytes and a SHA-256 creation intent.",
        ))
    }
}

impl EncountersRead<'_> {
    /// The original encounter may have been deleted after the commit.
    pub fn creation_receipt(
        &self,
        request_id: &str,
        intent_digest: &str,
    ) -> LocalStateResult<Option<String>> {
        validate(request_id, intent_digest)?;
        let receipt: Option<(String, String)> = self.connection.query_row(
            "SELECT intent_digest, encounter_key FROM encounter_creation_receipts WHERE owner_id=?1 AND request_id=?2",
            params![self.owner_id, request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
        match receipt {
            Some((digest, _)) if digest != intent_digest => {
                Err(LocalStateError::RecordAdditionConflict)
            }
            Some((_, key)) => Ok(Some(key)),
            None => Ok(None),
        }
    }
}

impl EncountersWrite<'_> {
    /// Bind a newly created owned encounter to its original request in the creation transaction.
    pub fn save_creation_receipt(
        &self,
        request_id: &str,
        intent_digest: &str,
        encounter_key: &str,
    ) -> LocalStateResult<()> {
        validate(request_id, intent_digest)?;
        let inserted = self.connection.execute(
            "INSERT INTO encounter_creation_receipts(owner_id, request_id, intent_digest, encounter_key)
             SELECT owner_id, ?2, ?3, encounter_key FROM encounters WHERE owner_id=?1 AND encounter_key=?4",
            params![self.owner_id, request_id, intent_digest, encounter_key],
        )?;
        if inserted == 0 {
            return Err(LocalStateError::EncounterNotFound(encounter_key.into()));
        }
        Ok(())
    }
}
