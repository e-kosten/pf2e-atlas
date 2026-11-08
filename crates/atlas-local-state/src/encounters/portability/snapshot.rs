use super::*;
use rusqlite::{Connection, OptionalExtension};

impl EncountersRead<'_> {
    pub fn export_document(&self, encounter_ref: &str) -> LocalStateResult<EncounterDocument> {
        let snapshot = self
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_ref.into()))?;
        let connection = self.connection;
        let id =
            storage::encounter_id(connection, self.owner_id, &snapshot.encounter.encounter_key)?
                .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_ref.into()))?;
        let baselines = rows(
            connection,
            "SELECT b.participant_key,b.payload_json FROM encounter_participant_baselines b
             JOIN encounter_participants p USING(participant_key)
             WHERE p.encounter_id=?1 ORDER BY b.participant_key",
            id,
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?
        .into_iter()
        .map(|(participant_key, payload)| {
            Ok(Baseline {
                gameplay: decode(&payload, "participant baseline")?,
                participant_key,
            })
        })
        .collect::<LocalStateResult<_>>()?;
        let pending = rows(connection,
            "SELECT consequence_key,participant_key,payload_json FROM encounter_pending_consequences
             WHERE encounter_id=?1 ORDER BY consequence_key",
            id, |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?
            .into_iter().map(|(consequence_key, participant_key, payload)| {
                let consequence: PendingConsequence = decode(&payload, "pending consequence")?;
                if consequence_participant(&consequence) != participant_key {
                    return Err(invalid("pending consequence owner disagrees with its payload"));
                }
                Ok(SavedPendingConsequence { consequence_key, consequence })
            }).collect::<LocalStateResult<_>>()?;
        let applications = rows(connection,
            "SELECT participant_key,application_id,grant_slot,condition_id FROM encounter_condition_applications
             WHERE encounter_id=?1 ORDER BY participant_key,application_id,grant_slot",
            id, |row| Ok(Application {
                participant_key: row.get(0)?,
                application_id: row.get(1)?,
                grant_slot: row.get(2)?,
                condition_id: row.get(3)?,
            }))?;
        let claims = rows(connection,
            "SELECT participant_key,round_number,kind,scope_key,request_id FROM encounter_boundary_claims
             WHERE encounter_id=?1 ORDER BY participant_key,round_number,kind,scope_key",
            id, |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)))?
            .into_iter().map(|(participant_key, round_number, kind, scope_key, request_id)| {
                let kind = serde_json::from_value(Value::String(kind))
                    .map_err(|err| invalid(format!("boundary claim: {err}")))?;
                Ok(Claim { claim: EncounterBoundaryClaim { participant_key, round_number, kind, scope_key }, request_id })
            }).collect::<LocalStateResult<_>>()?;
        let receipts = rows(connection,
            "SELECT family,request_id,intent_json,boundary_key,receipt_json FROM encounter_command_receipts
             WHERE encounter_id=?1 ORDER BY family,request_id",
            id, |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?,
                row.get::<_, String>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?)))?
            .into_iter().map(|(family, request_id, intent, boundary, payload)| {
                let outcome = match (family.as_str(), boundary, payload) {
                    ("turn", Some(boundary_key), None) => ReceiptOutcome::Turn { boundary_key },
                    ("event", None, Some(payload)) => ReceiptOutcome::Event {
                        receipt: decode(&payload, &format!("event receipt {request_id}"))?,
                    },
                    _ => return Err(invalid(format!("unsupported receipt {family}/{request_id}"))),
                };
                let intent = serde_json::from_str(&intent)
                    .map_err(|err| invalid(format!("receipt {request_id} intent: {err}")))?;
                Ok(Receipt { request_id, intent, outcome })
            }).collect::<LocalStateResult<_>>()?;
        let outgoing = connection.query_row(
            "SELECT request_id,requires_review,payload_json FROM encounter_outgoing_turns WHERE encounter_id=?1",
            [id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?, row.get::<_, String>(2)?)))
            .optional()?.map(|(request_id, requires_review, payload)| {
                Ok::<_, LocalStateError>(Outgoing { request_id, requires_review, turn: decode(&payload, "outgoing turn")? })
            }).transpose()?;
        Ok(EncounterDocument {
            version: VERSION,
            snapshot,
            baselines,
            pending,
            applications,
            claims,
            receipts,
            outgoing,
            source_fingerprint: None,
            record_sources: Vec::new(),
        })
    }
}

fn rows<T>(
    connection: &Connection,
    sql: &str,
    id: i64,
    read: impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
) -> LocalStateResult<Vec<T>> {
    let mut statement = connection.prepare(sql)?;
    let mut result = Vec::new();
    for row in statement.query_map([id], read)? {
        if result.len() == ENTRY_LIMIT {
            return Err(invalid("encounter exceeds 100,000 entries"));
        }
        result.push(row?);
    }
    Ok(result)
}

fn decode<T: serde::de::DeserializeOwned>(payload: &str, location: &str) -> LocalStateResult<T> {
    receipt_codec::decode(payload).map_err(|err| invalid(format!("{location}: {err}")))
}

pub(super) fn consequence_participant(value: &PendingConsequence) -> &str {
    match value {
        PendingConsequence::ConfusedDamageRecovery {
            participant_key, ..
        }
        | PendingConsequence::BrokenHpThreshold { participant_key }
        | PendingConsequence::InitiativePlacement {
            participant_key, ..
        }
        | PendingConsequence::HpLifecycle {
            participant_key, ..
        }
        | PendingConsequence::DrainedHpLoss {
            participant_key, ..
        } => participant_key,
    }
}
