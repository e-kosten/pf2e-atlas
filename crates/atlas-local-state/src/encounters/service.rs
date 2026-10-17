use rusqlite::TransactionBehavior;

use super::model::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterTurnState, EncounterTurnUpdate,
    EncounterWithParticipants, NewEncounter, ReorderEncounterParticipant, UpdateEncounter,
    UpdateEncounterParticipant, UpdateEncounterParticipantCondition,
};
use super::storage;
use crate::{LocalStateError, LocalStateResult, LocalStateStore};

#[derive(Debug, Clone, Copy)]
pub struct Encounters<'a> {
    store: &'a LocalStateStore,
}

impl<'a> Encounters<'a> {
    pub(crate) fn new(store: &'a LocalStateStore) -> Self {
        Self { store }
    }

    pub fn create(&self, encounter: NewEncounter) -> LocalStateResult<Encounter> {
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let encounter_key = storage::insert_encounter(&transaction, encounter)?;
        let result = storage::get(&transaction, &encounter_key)?
            .ok_or(LocalStateError::EncounterNotFound(encounter_key))?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn list(&self) -> LocalStateResult<Vec<Encounter>> {
        let connection = self.store.connection()?;
        storage::list(&connection)
    }

    pub fn get(&self, encounter_ref: &str) -> LocalStateResult<Option<Encounter>> {
        validate_ref(encounter_ref)?;
        let connection = self.store.connection()?;
        storage::get(&connection, encounter_ref)
    }

    pub fn get_with_participants(
        &self,
        encounter_ref: &str,
    ) -> LocalStateResult<Option<EncounterWithParticipants>> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction()?;
        let result = storage::get_with_participants(&transaction, encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn update(&self, encounter: UpdateEncounter) -> LocalStateResult<Option<Encounter>> {
        validate_ref(&encounter.encounter_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !storage::update_encounter(&transaction, encounter.clone())? {
            return Ok(None);
        }
        let result = storage::get(&transaction, &encounter.encounter_key)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn delete(&self, encounter_ref: &str) -> LocalStateResult<bool> {
        validate_ref(encounter_ref)?;
        let connection = self.store.connection()?;
        storage::delete(&connection, encounter_ref)
    }

    pub fn add_participant(
        &self,
        encounter_ref: &str,
        participant: AddEncounterParticipant,
    ) -> LocalStateResult<EncounterParticipant> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let participant_key = storage::add_participant(&transaction, encounter_ref, participant)?;
        let result = storage::participant(&transaction, &participant_key)?
            .ok_or(LocalStateError::ParticipantNotFound(participant_key))?;
        transaction.commit()?;
        Ok(result)
    }

    /// Add one resolved quantity as a single durable operation.
    pub fn add_participants(
        &self,
        encounter_ref: &str,
        participants: Vec<AddEncounterParticipant>,
    ) -> LocalStateResult<Vec<EncounterParticipant>> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if storage::get(&transaction, encounter_ref)?.is_none() {
            return Err(LocalStateError::EncounterNotFound(
                encounter_ref.to_string(),
            ));
        }
        let mut result = Vec::with_capacity(participants.len());
        for participant in participants {
            let key = storage::add_participant(&transaction, encounter_ref, participant)?;
            result.push(
                storage::participant(&transaction, &key)?
                    .ok_or(LocalStateError::ParticipantNotFound(key))?,
            );
        }
        transaction.commit()?;
        Ok(result)
    }

    pub fn update_participant(
        &self,
        participant: UpdateEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !storage::update_participant(&transaction, participant.clone())? {
            transaction.commit()?;
            return Ok(None);
        }
        let result = storage::participant(&transaction, &participant.participant_key)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn reorder_participant(
        &self,
        reorder: ReorderEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        validate_ref(&reorder.participant_key)?;
        validate_ref(&reorder.target_participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !storage::reorder_participant(&transaction, reorder.clone())? {
            transaction.commit()?;
            return Ok(None);
        }
        let result = storage::participant(&transaction, &reorder.participant_key)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn remove_participant(&self, participant_key: &str) -> LocalStateResult<bool> {
        validate_ref(participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let removed = storage::remove_participant(&transaction, participant_key)?;
        transaction.commit()?;
        Ok(removed)
    }

    pub fn add_condition(
        &self,
        condition: AddEncounterParticipantCondition,
    ) -> LocalStateResult<EncounterParticipantCondition> {
        validate_ref(&condition.participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let condition_id = storage::add_condition(&transaction, condition)?;
        let result = storage::condition(&transaction, condition_id)?.ok_or(
            LocalStateError::ParticipantNotFound(condition_id.to_string()),
        )?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn update_condition(
        &self,
        condition: UpdateEncounterParticipantCondition,
    ) -> LocalStateResult<Option<EncounterParticipantCondition>> {
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !storage::update_condition(&transaction, condition.clone())? {
            transaction.commit()?;
            return Ok(None);
        }
        let result = storage::condition(&transaction, condition.condition_id)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn remove_condition(
        &self,
        participant_key: &str,
        condition_id: i64,
    ) -> LocalStateResult<bool> {
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let removed = storage::remove_condition(&transaction, participant_key, condition_id)?;
        transaction.commit()?;
        Ok(removed)
    }

    pub fn set_current_turn(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
    ) -> LocalStateResult<bool> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = storage::set_current_turn(&transaction, encounter_ref, participant_key)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn set_turn_state(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
        round_number: i64,
        mark_running: bool,
    ) -> LocalStateResult<bool> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = storage::set_turn_state(
            &transaction,
            encounter_ref,
            participant_key,
            Some(round_number),
            mark_running,
        )?;
        transaction.commit()?;
        Ok(result)
    }

    /// Choose a turn using the same locked snapshot that is updated.
    /// The callback must be pure; resolve immutable source facts before this call.
    pub fn advance_turn(
        &self,
        encounter_ref: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> EncounterTurnState,
    ) -> LocalStateResult<Option<Encounter>> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let Some(snapshot) = storage::get_with_participants(&transaction, encounter_ref)? else {
            return Ok(None);
        };
        let turn = choose(&snapshot);
        storage::set_turn_state(
            &transaction,
            encounter_ref,
            turn.participant_key.as_deref(),
            Some(turn.round_number),
            turn.mark_running,
        )?;
        let result = storage::get(&transaction, encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }

    /// Check a reviewed revision and apply turn/timing in one write transaction.
    /// The callback is pure and may only plan updates from this locked snapshot.
    pub fn advance_reviewed_turn(
        &self,
        encounter_ref: &str,
        expected_revision: i64,
        choose: impl FnOnce(&EncounterWithParticipants) -> EncounterTurnUpdate,
    ) -> LocalStateResult<Option<Encounter>> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let Some(snapshot) = storage::get_with_participants(&transaction, encounter_ref)? else {
            return Ok(None);
        };
        if snapshot.encounter.revision != expected_revision {
            return Err(LocalStateError::StaleEncounterRevision {
                encounter_key: snapshot.encounter.encounter_key,
                expected_revision,
                actual_revision: snapshot.encounter.revision,
            });
        }
        let update = choose(&snapshot);
        for (condition_id, amount) in update.durations {
            storage::update_duration_amount(&transaction, condition_id, amount)?;
        }
        storage::set_turn_state(
            &transaction,
            encounter_ref,
            update.turn.participant_key.as_deref(),
            Some(update.turn.round_number),
            update.turn.mark_running,
        )?;
        let result = storage::get(&transaction, encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }

    /// Remove a participant and apply the application's replacement turn atomically.
    /// The pure callback sees the complete pre-removal snapshot.
    pub fn remove_participant_and_select_turn(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> Option<EncounterTurnState>,
    ) -> LocalStateResult<Option<Encounter>> {
        validate_ref(encounter_ref)?;
        validate_ref(participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let Some(snapshot) = storage::get_with_participants(&transaction, encounter_ref)? else {
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
        storage::remove_participant(&transaction, participant_key)?;
        if let Some(turn) = replacement {
            storage::set_turn_state(
                &transaction,
                encounter_ref,
                turn.participant_key.as_deref(),
                Some(turn.round_number),
                turn.mark_running,
            )?;
        }
        let result = storage::get(&transaction, encounter_ref)?;
        transaction.commit()?;
        Ok(result)
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
