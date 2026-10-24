//! Durable outgoing-turn identity; rule planning remains in app-service.
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
    pub fn outgoing_turn(
        &self,
        encounter_ref: &str,
        boundary_key: &str,
    ) -> LocalStateResult<Option<SavedOutgoingTurn>> {
        let payload:Option<String> = self.connection.query_row("SELECT t.payload_json FROM encounter_outgoing_turns t JOIN encounters e ON e.id=t.encounter_id WHERE (e.encounter_key=?1 OR e.slug=?1) AND t.boundary_key=?2 AND t.id=(SELECT MAX(id) FROM encounter_outgoing_turns WHERE encounter_id=e.id)",params![encounter_ref,boundary_key],|row|row.get(0)).optional()?;
        payload
            .map(|payload| receipt_codec::decode(&payload))
            .transpose()
    }
    pub fn latest_outgoing_turn(
        &self,
        encounter_ref: &str,
    ) -> LocalStateResult<Option<SavedOutgoingTurn>> {
        let payload: Option<String> = self.connection.query_row(
            "SELECT t.payload_json FROM encounter_outgoing_turns t JOIN encounters e ON e.id=t.encounter_id WHERE (e.encounter_key=?1 OR e.slug=?1) AND t.id=(SELECT MAX(id) FROM encounter_outgoing_turns WHERE encounter_id=e.id) AND t.requires_review=1",
            [encounter_ref], |row| row.get(0)).optional()?;
        payload
            .map(|payload| receipt_codec::decode(&payload))
            .transpose()
    }
}
impl EncountersWrite<'_> {
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
            let Some(snapshot) = storage::get_with_participants(self.connection, encounter_ref)?
            else {
                return Ok(None);
            };
            let prior:Option<String>=self.connection.query_row("SELECT intent_json FROM encounter_outgoing_turns WHERE encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1) AND request_id=?2",params![snapshot.encounter.encounter_key,request_id],|row|row.get(0)).optional()?;
            if let Some(prior) = prior {
                if serde_json::from_str::<serde_json::Value>(&prior)? != *intent {
                    return Err(LocalStateError::InvalidRollRequest(
                        "request ID already used for another turn intent",
                    ));
                }
                return storage::get(self.connection, encounter_ref);
            }
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
            // Even an empty boundary retains the once-only request identity.
            self.connection.execute("INSERT INTO encounter_outgoing_turns(encounter_id,boundary_key,request_id,intent_json,payload_json,requires_review) SELECT id,?2,?3,?4,?5,?6 FROM encounters WHERE encounter_key=?1",params![saved.snapshot.encounter.encounter_key,saved.boundary_key,request_id,serde_json::to_string(intent)?,receipt_codec::encode(&saved)?,i64::from(review_needed&&saved.snapshot.encounter.current_turn_participant_key.is_some()&&saved.next_participant_key.is_some())])?;
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
