//! Lifetime turn receipts and one current outgoing context; rules stay in app-service.
use super::{
    Encounter, EncounterTurnUpdate, EncounterWithParticipants, EncountersRead, EncountersWrite,
    receipt_codec, storage,
};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedOutgoingTurn {
    pub boundary_key: String,
    pub snapshot: EncounterWithParticipants,
    pub next_participant_key: Option<String>,
}
impl EncountersRead<'_> {
    pub fn advance_turn_receipt(
        &self,
        encounter_ref: &str,
        request_id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<Encounter>> {
        super::initiative_rolls::validate_request(request_id, intent)?;
        recover_advance(self.connection, encounter_ref, request_id, intent)
    }
    pub fn outgoing_turn(
        &self,
        encounter_ref: &str,
        boundary_key: &str,
    ) -> LocalStateResult<Option<SavedOutgoingTurn>> {
        let payload:Option<String> = self.connection.query_row("SELECT t.payload_json FROM encounter_outgoing_turns t JOIN encounter_turn_receipts r USING(encounter_id,request_id) JOIN encounters e ON e.id=t.encounter_id WHERE (e.encounter_key=?1 OR e.slug=?1) AND r.boundary_key=?2",params![encounter_ref,boundary_key],|row|row.get(0)).optional()?;
        payload
            .map(|payload| receipt_codec::decode(&payload))
            .transpose()
    }
    pub fn latest_outgoing_turn(
        &self,
        encounter_ref: &str,
    ) -> LocalStateResult<Option<SavedOutgoingTurn>> {
        let payload: Option<String> = self.connection.query_row(
            "SELECT t.payload_json FROM encounter_outgoing_turns t JOIN encounters e ON e.id=t.encounter_id WHERE (e.encounter_key=?1 OR e.slug=?1) AND t.requires_review=1",
            [encounter_ref], |row| row.get(0)).optional()?;
        payload
            .map(|payload| receipt_codec::decode(&payload))
            .transpose()
    }
}
impl EncountersWrite<'_> {
    pub fn advance_turn_receipt(
        &self,
        encounter_ref: &str,
        request_id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<Encounter>> {
        super::initiative_rolls::validate_request(request_id, intent)?;
        recover_advance(self.connection, encounter_ref, request_id, intent)
    }
    pub fn advance_turn_once(
        &self,
        encounter_ref: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        choose: impl FnOnce(&EncounterWithParticipants) -> LocalStateResult<(EncounterTurnUpdate, bool)>,
    ) -> LocalStateResult<Option<Encounter>> {
        super::initiative_rolls::validate_request(request_id, intent)?;
        crate::atomic_action::run(self.connection, || {
            if let Some(encounter) =
                recover_advance(self.connection, encounter_ref, request_id, intent)?
            {
                return Ok(Some(encounter));
            }
            let Some(snapshot) = storage::get_with_participants(self.connection, encounter_ref)?
            else {
                return Ok(None);
            };
            if snapshot.encounter.revision != expected_revision {
                return Err(LocalStateError::StaleEncounterRevision {
                    encounter_key: snapshot.encounter.encounter_key,
                    expected_revision,
                    actual_revision: snapshot.encounter.revision,
                });
            }
            let (update, review_needed) = choose(&snapshot)?;
            let saved = SavedOutgoingTurn {
                boundary_key: format!("turn_{:016x}", rand::random::<u64>()),
                next_participant_key: update.turn.participant_key.clone(),
                snapshot,
            };
            self.connection.execute("INSERT INTO encounter_turn_receipts(encounter_id,request_id,boundary_key,intent_json) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1",params![saved.snapshot.encounter.encounter_key,request_id,saved.boundary_key,serde_json::to_string(intent)?])?;
            self.connection.execute("INSERT INTO encounter_outgoing_turns(encounter_id,request_id,payload_json,requires_review) SELECT id,?2,?3,?4 FROM encounters WHERE encounter_key=?1 ON CONFLICT(encounter_id) DO UPDATE SET request_id=excluded.request_id,payload_json=excluded.payload_json,requires_review=excluded.requires_review",params![saved.snapshot.encounter.encounter_key,request_id,receipt_codec::encode(&saved)?,i64::from(review_needed&&saved.snapshot.encounter.current_turn_participant_key.is_some()&&saved.next_participant_key.is_some())])?;
            for (condition_id, amount) in update.durations {
                storage::update_duration_amount(self.connection, condition_id, amount)?;
            }
            storage::set_turn_state(
                self.connection,
                encounter_ref,
                update.turn.participant_key.as_deref(),
                Some(update.turn.round_number),
                update.turn.mark_running,
            )?;
            if let Some(start) = update.start_event {
                let current = storage::get(self.connection, encounter_ref)?
                    .ok_or_else(|| LocalStateError::EncounterNotFound(encounter_ref.into()))?;
                self.apply_claimed_condition_event(
                    &current.encounter_key,
                    &format!("start:{}", saved.boundary_key),
                    intent,
                    current.revision,
                    super::ConditionEventBoundary {
                        claims: &update.start_claims,
                        outgoing_boundary_key: None,
                    },
                    start,
                )?;
            }
            storage::get(self.connection, encounter_ref)
        })
    }
}

fn recover_advance(
    connection: &rusqlite::Connection,
    encounter_ref: &str,
    request_id: &str,
    intent: &serde_json::Value,
) -> LocalStateResult<Option<Encounter>> {
    let prior: Option<String> = connection.query_row(
        "SELECT t.intent_json FROM encounter_turn_receipts t JOIN encounters e ON e.id=t.encounter_id WHERE (e.encounter_key=?1 OR e.slug=?1) AND t.request_id=?2",
        params![encounter_ref, request_id], |row| row.get(0),
    ).optional()?;
    let Some(prior) = prior else {
        return Ok(None);
    };
    if serde_json::from_str::<serde_json::Value>(&prior)? != *intent {
        return Err(LocalStateError::InvalidRollRequest(
            "request ID already used for another turn intent",
        ));
    }
    storage::get(connection, encounter_ref)
}
