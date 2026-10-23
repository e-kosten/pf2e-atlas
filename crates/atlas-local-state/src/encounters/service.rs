use super::model::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterTurnState, EncounterWithParticipants, NewEncounter,
    ReorderEncounterParticipant, UpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition,
};
use super::storage;
use crate::{LocalStateError, LocalStateResult};

#[derive(Clone, Copy)]
pub struct EncountersRead<'a> {
    pub(super) connection: &'a rusqlite::Connection,
}
pub struct EncountersWrite<'a> {
    pub(super) read: EncountersRead<'a>,
}
impl<'a> std::ops::Deref for EncountersWrite<'a> {
    type Target = EncountersRead<'a>;
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
impl<'a> EncountersRead<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self { connection }
    }
}
impl<'a> EncountersWrite<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self {
            read: EncountersRead::from_connection(connection),
        }
    }
}
impl EncountersWrite<'_> {
    pub fn create(&self, encounter: NewEncounter) -> LocalStateResult<Encounter> {
        crate::atomic_action::run(self.connection, || {
            let transaction = self.connection;
            let encounter_key = storage::insert_encounter(transaction, encounter)?;
            let result = storage::get(transaction, &encounter_key)?
                .ok_or(LocalStateError::EncounterNotFound(encounter_key))?;

            Ok(result)
        })
    }

    pub fn update(&self, encounter: UpdateEncounter) -> LocalStateResult<Option<Encounter>> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(&encounter.encounter_key)?;
            let transaction = self.connection;
            if !storage::update_encounter(transaction, encounter.clone())? {
                return Ok(None);
            }
            let result = storage::get(transaction, &encounter.encounter_key)?;

            Ok(result)
        })
    }

    pub fn delete(&self, encounter_ref: &str) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            storage::delete(self.connection, encounter_ref)
        })
    }

    pub fn add_participant(
        &self,
        encounter_ref: &str,
        participant: AddEncounterParticipant,
    ) -> LocalStateResult<EncounterParticipant> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            let transaction = self.connection;
            let participant_key =
                storage::add_participant(transaction, encounter_ref, participant)?;
            let result = storage::participant(transaction, &participant_key)?
                .ok_or(LocalStateError::ParticipantNotFound(participant_key))?;

            Ok(result)
        })
    }

    /// Add one resolved quantity as a single durable operation.
    pub fn add_participants(
        &self,
        encounter_ref: &str,
        participants: Vec<AddEncounterParticipant>,
    ) -> LocalStateResult<Vec<EncounterParticipant>> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            let transaction = self.connection;
            if storage::get(transaction, encounter_ref)?.is_none() {
                return Err(LocalStateError::EncounterNotFound(
                    encounter_ref.to_string(),
                ));
            }
            let mut result = Vec::with_capacity(participants.len());
            for participant in participants {
                let key = storage::add_participant(transaction, encounter_ref, participant)?;
                result.push(
                    storage::participant(transaction, &key)?
                        .ok_or(LocalStateError::ParticipantNotFound(key))?,
                );
            }

            Ok(result)
        })
    }

    pub fn update_participant(
        &self,
        participant: UpdateEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        crate::atomic_action::run(self.connection, || {
            let transaction = self.connection;
            if !storage::update_participant(transaction, participant.clone())? {
                return Ok(None);
            }
            let result = storage::participant(transaction, &participant.participant_key)?;

            Ok(result)
        })
    }

    pub fn reorder_participant(
        &self,
        reorder: ReorderEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(&reorder.participant_key)?;
            validate_ref(&reorder.target_participant_key)?;
            let transaction = self.connection;
            if !storage::reorder_participant(transaction, reorder.clone())? {
                return Ok(None);
            }
            let result = storage::participant(transaction, &reorder.participant_key)?;

            Ok(result)
        })
    }

    pub fn remove_participant(&self, participant_key: &str) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(participant_key)?;
            let transaction = self.connection;
            let removed = storage::remove_participant(transaction, participant_key)?;

            Ok(removed)
        })
    }

    pub fn add_condition(
        &self,
        condition: AddEncounterParticipantCondition,
    ) -> LocalStateResult<EncounterParticipantCondition> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(&condition.participant_key)?;
            let transaction = self.connection;
            let condition_id = storage::add_condition(transaction, condition)?;
            let result = storage::condition(transaction, condition_id)?.ok_or(
                LocalStateError::ParticipantNotFound(condition_id.to_string()),
            )?;

            Ok(result)
        })
    }

    pub fn update_condition(
        &self,
        condition: UpdateEncounterParticipantCondition,
    ) -> LocalStateResult<Option<EncounterParticipantCondition>> {
        crate::atomic_action::run(self.connection, || {
            let transaction = self.connection;
            if !storage::update_condition(transaction, condition.clone())? {
                return Ok(None);
            }
            let result = storage::condition(transaction, condition.condition_id)?;

            Ok(result)
        })
    }

    pub fn remove_condition(
        &self,
        participant_key: &str,
        condition_id: i64,
    ) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            let transaction = self.connection;
            let removed = storage::remove_condition(transaction, participant_key, condition_id)?;

            Ok(removed)
        })
    }

    pub fn set_current_turn(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
    ) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            let transaction = self.connection;
            let result = storage::set_current_turn(transaction, encounter_ref, participant_key)?;

            Ok(result)
        })
    }

    pub fn set_turn_state(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
        round_number: i64,
        mark_running: bool,
    ) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            let transaction = self.connection;
            let result = storage::set_turn_state(
                transaction,
                encounter_ref,
                participant_key,
                Some(round_number),
                mark_running,
            )?;

            Ok(result)
        })
    }

    /// Choose a turn using the same locked snapshot that is updated.
    /// The callback must be pure; resolve immutable source facts before this call.
    pub fn advance_turn(
        &self,
        encounter_ref: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> EncounterTurnState,
    ) -> LocalStateResult<Option<Encounter>> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            let transaction = self.connection;
            let Some(snapshot) = storage::get_with_participants(transaction, encounter_ref)? else {
                return Ok(None);
            };
            let turn = choose(&snapshot);
            storage::set_turn_state(
                transaction,
                encounter_ref,
                turn.participant_key.as_deref(),
                Some(turn.round_number),
                turn.mark_running,
            )?;
            let result = storage::get(transaction, encounter_ref)?;

            Ok(result)
        })
    }

    /// Remove a participant and apply the application's replacement turn atomically.
    /// The pure callback sees the complete pre-removal snapshot.
    pub fn remove_participant_and_select_turn(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> Option<EncounterTurnState>,
    ) -> LocalStateResult<Option<Encounter>> {
        crate::atomic_action::run(self.connection, || {
            validate_ref(encounter_ref)?;
            validate_ref(participant_key)?;
            let transaction = self.connection;
            let Some(snapshot) = storage::get_with_participants(transaction, encounter_ref)? else {
                return Ok(None);
            };
            if !snapshot
                .participants
                .iter()
                .any(|row| row.participant_key == participant_key)
            {
                return Ok(None);
            }
            let replacement = choose(&snapshot);
            storage::remove_participant(transaction, participant_key)?;
            if let Some(turn) = replacement {
                storage::set_turn_state(
                    transaction,
                    encounter_ref,
                    turn.participant_key.as_deref(),
                    Some(turn.round_number),
                    turn.mark_running,
                )?;
            }
            let result = storage::get(transaction, encounter_ref)?;

            Ok(result)
        })
    }
}

fn validate_ref(value: &str) -> LocalStateResult<()> {
    if value.trim().is_empty() {
        return Err(LocalStateError::InvalidEncounterRef {
            encounter_ref: value.to_string(),
            reason: "encounter ref must not be empty",
        });
    }
    Ok(())
}

impl EncountersRead<'_> {
    pub fn list(&self) -> LocalStateResult<Vec<Encounter>> {
        storage::list(self.connection)
    }
    pub fn get(&self, encounter_ref: &str) -> LocalStateResult<Option<Encounter>> {
        validate_ref(encounter_ref)?;
        storage::get(self.connection, encounter_ref)
    }
    pub fn get_with_participants(
        &self,
        encounter_ref: &str,
    ) -> LocalStateResult<Option<EncounterWithParticipants>> {
        validate_ref(encounter_ref)?;
        let transaction = self.connection;
        let result = storage::get_with_participants(transaction, encounter_ref)?;

        Ok(result)
    }
}
