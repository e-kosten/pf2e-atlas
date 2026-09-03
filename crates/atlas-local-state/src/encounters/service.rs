use rusqlite::TransactionBehavior;

use super::model::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterParticipantReset, EncounterParticipantSpellState,
    EncounterSpellResource, EncounterSpellResourceMutation, EncounterSpellResourceOperation,
    EncounterSpellResourceTarget, EncounterWithParticipants, NewEncounter,
    ReorderEncounterParticipant, UpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition,
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
        let connection = self.store.connection()?;
        let encounter_key = storage::insert_encounter(&connection, encounter)?;
        self.get(&encounter_key)?
            .ok_or(LocalStateError::EncounterNotFound(encounter_key))
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
        let connection = self.store.connection()?;
        storage::get_with_participants(&connection, encounter_ref)
    }

    pub fn update(&self, encounter: UpdateEncounter) -> LocalStateResult<Option<Encounter>> {
        validate_ref(&encounter.encounter_key)?;
        let connection = self.store.connection()?;
        if !storage::update_encounter(&connection, encounter.clone())? {
            return Ok(None);
        }
        self.get(&encounter.encounter_key)
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
        storage::capture_participant_baseline(&transaction, &participant_key)?;
        transaction.commit()?;
        self.participant(&participant_key)?
            .ok_or(LocalStateError::ParticipantNotFound(participant_key))
    }

    pub fn add_participant_with_spell_resources(
        &self,
        encounter_ref: &str,
        participant: AddEncounterParticipant,
        spell_resources: &[EncounterSpellResource],
    ) -> LocalStateResult<EncounterParticipant> {
        validate_ref(encounter_ref)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let participant_key = storage::add_participant(&transaction, encounter_ref, participant)?;
        storage::capture_participant_baseline(&transaction, &participant_key)?;
        storage::initialize_spell_state(&transaction, &participant_key, spell_resources)?;
        transaction.commit()?;
        self.participant(&participant_key)?
            .ok_or(LocalStateError::ParticipantNotFound(participant_key))
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
        transaction.commit()?;
        self.participant(&participant.participant_key)
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
        transaction.commit()?;
        self.participant(&reorder.participant_key)
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
        transaction.commit()?;
        self.condition(condition_id)?
            .ok_or(LocalStateError::ParticipantNotFound(
                condition_id.to_string(),
            ))
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
        transaction.commit()?;
        self.condition(condition.condition_id)
    }

    pub fn remove_condition(&self, condition_id: i64) -> LocalStateResult<bool> {
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let removed = storage::remove_condition(&transaction, condition_id)?;
        transaction.commit()?;
        Ok(removed)
    }

    pub fn set_current_turn(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
    ) -> LocalStateResult<bool> {
        validate_ref(encounter_ref)?;
        let connection = self.store.connection()?;
        storage::set_current_turn(&connection, encounter_ref, participant_key)
    }

    pub fn set_turn_state(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
        round_number: i64,
        mark_running: bool,
    ) -> LocalStateResult<bool> {
        validate_ref(encounter_ref)?;
        let connection = self.store.connection()?;
        storage::set_turn_state(
            &connection,
            encounter_ref,
            participant_key,
            Some(round_number),
            mark_running,
        )
    }

    pub fn initialize_spell_state(
        &self,
        participant_key: &str,
        resources: &[EncounterSpellResource],
    ) -> LocalStateResult<EncounterParticipantSpellState> {
        validate_ref(participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let state = storage::initialize_spell_state(&transaction, participant_key, resources)?;
        transaction.commit()?;
        Ok(state)
    }

    pub fn spell_state(
        &self,
        participant_key: &str,
    ) -> LocalStateResult<EncounterParticipantSpellState> {
        validate_ref(participant_key)?;
        let connection = self.store.connection()?;
        storage::spell_state(&connection, participant_key)
    }

    pub fn mutate_spell_resource(
        &self,
        participant_key: &str,
        target: &EncounterSpellResourceTarget,
        operation: EncounterSpellResourceOperation,
    ) -> LocalStateResult<EncounterSpellResourceMutation> {
        validate_ref(participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mutation =
            storage::mutate_spell_resource(&transaction, participant_key, target, operation)?;
        transaction.commit()?;
        Ok(mutation)
    }

    pub fn participant_reset_available(&self, participant_key: &str) -> LocalStateResult<bool> {
        validate_ref(participant_key)?;
        let connection = self.store.connection()?;
        storage::has_participant_baseline(&connection, participant_key)
    }

    pub fn reset_participant(
        &self,
        participant_key: &str,
    ) -> LocalStateResult<EncounterParticipantReset> {
        validate_ref(participant_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let reset = storage::reset_participant(&transaction, participant_key)?;
        transaction.commit()?;
        Ok(reset)
    }

    fn participant(&self, participant_key: &str) -> LocalStateResult<Option<EncounterParticipant>> {
        let connection = self.store.connection()?;
        storage::participant(&connection, participant_key)
    }

    fn condition(
        &self,
        condition_id: i64,
    ) -> LocalStateResult<Option<EncounterParticipantCondition>> {
        let connection = self.store.connection()?;
        let encounters = storage::list(&connection)?;
        for encounter in encounters {
            let Some(detail) =
                storage::get_with_participants(&connection, &encounter.encounter_key)?
            else {
                continue;
            };
            for participant in detail.participants {
                if let Some(condition) = participant
                    .conditions
                    .into_iter()
                    .find(|condition| condition.condition_id == condition_id)
                {
                    return Ok(Some(condition));
                }
            }
        }
        Ok(None)
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
