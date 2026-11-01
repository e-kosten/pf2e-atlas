use crate::*;
use rusqlite::{Connection, TransactionBehavior};
use std::path::{Path, PathBuf};
#[derive(Clone)]
pub(crate) struct TestStore {
    path: PathBuf,
}
impl TestStore {
    pub fn open(path: impl Into<PathBuf>) -> LocalStateResult<Self> {
        let store = Self { path: path.into() };
        let conn = store.connection()?;
        if !conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='principals')",
            [],
            |r| r.get::<_, bool>(0),
        )? {
            conn.execute_batch("CREATE TABLE record_identity(id INTEGER PRIMARY KEY,key TEXT UNIQUE,last_known_name TEXT,availability TEXT NOT NULL);")?;
            conn.execute_batch(LOCAL_STATE_SCHEMA_SQL)?;
            conn.execute_batch(ENCOUNTER_STATUS_SCHEMA_SQL)?;
            conn.execute_batch(ENCOUNTER_DIFFICULTY_SCHEMA_SQL)?;
            conn.execute_batch(ABILITY_STATE_SCHEMA_SQL)?;
            conn.execute_batch(HAZARD_PARTICIPATION_SCHEMA_SQL)?;
            conn.execute_batch(CONDITION_SYSTEM_SCHEMA_SQL)?;
            conn.execute_batch(PERSISTENT_PREPARATION_SCHEMA_SQL)?;
            conn.execute_batch(TURN_RESOURCES_SCHEMA_SQL)?;
            conn.execute_batch(CONSUMED_CONDITION_APPLICATIONS_SCHEMA_SQL)?;
            conn.execute_batch(ENCOUNTER_RECOVERY_SCHEMA_SQL)?;
            conn.execute_batch(MANUAL_TRACKERS_SCHEMA_SQL)?;
            conn.execute_batch(ENCOUNTER_CLOCK_SCHEMA_SQL)?;
            backfill_encounter_clock(&conn).expect("fresh workspace clock backfill");
            conn.pragma_update(None, "foreign_keys", false)?;
            let transaction = conn.unchecked_transaction()?;
            backfill_saved_state_integrity(&transaction)
                .map_err(|error| LocalStateError::IncompatibleSchema(error.to_string()))?;
            transaction.commit()?;
            conn.pragma_update(None, "foreign_keys", true)?;
            conn.execute_batch(TURN_RECEIPTS_SCHEMA_SQL)?;
        }
        Ok(store)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn connection(&self) -> LocalStateResult<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA foreign_keys=ON")?;
        Ok(conn)
    }
    pub fn saved_lists(&self) -> TestSavedLists<'_> {
        TestSavedLists(self)
    }
    pub fn encounters(&self) -> TestEncounters<'_> {
        TestEncounters(self)
    }
}
pub(crate) struct TestSavedLists<'a>(&'a TestStore);
impl TestSavedLists<'_> {
    pub fn create(&self, list: NewSavedList) -> LocalStateResult<SavedList> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.create(list)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn get(&self, list_ref: &str) -> LocalStateResult<Option<SavedList>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let domain = SavedListsRead::from_connection(&transaction);
        let result = domain.get(list_ref)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn get_with_items(&self, list_ref: &str) -> LocalStateResult<Option<SavedListWithItems>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let domain = SavedListsRead::from_connection(&transaction);
        let result = domain.get_with_items(list_ref)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn delete(&self, list_ref: &str) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.delete(list_ref)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn update(&self, list: UpdateSavedList) -> LocalStateResult<Option<SavedList>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.update(list)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn import(&self, import: ImportSavedList) -> LocalStateResult<SavedListWithItems> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.import(import)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn reorder_item(
        &self,
        list_ref: &str,
        reorder: ReorderSavedListItem,
    ) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result =
            SavedListsWrite::from_connection(&transaction).reorder_item(list_ref, reorder)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn add_resolved_item(
        &self,
        list_ref: &str,
        item: ResolvedSavedListItem,
    ) -> LocalStateResult<AddSavedListItemOutcome> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.add_resolved_item(list_ref, item)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn remove_item(&self, list_ref: &str, record_key: &str) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = SavedListsWrite::from_connection(&transaction);
        let result = domain.remove_item(list_ref, record_key)?;
        transaction.commit()?;
        Ok(result)
    }
}
pub(crate) struct TestEncounters<'a>(&'a TestStore);
impl TestEncounters<'_> {
    pub fn apply_condition_event(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
        revision: i64,
        changes: PreparedConditionEvent,
    ) -> LocalStateResult<EncounterEventReceipt> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let receipt = EncountersWrite::from_connection(&transaction)
            .apply_condition_event(key, id, intent, revision, changes)?;
        transaction.commit()?;
        Ok(receipt)
    }
    pub fn condition_event_receipt(
        &self,
        key: &str,
        id: &str,
        intent: &serde_json::Value,
    ) -> LocalStateResult<Option<EncounterEventReceipt>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let receipt = EncountersRead::from_connection(&transaction)
            .condition_event_receipt(key, id, intent)?;
        transaction.commit()?;
        Ok(receipt)
    }

    pub fn replace_spell_resources(
        &self,
        participant: &str,
        expected_revision: i64,
        resources: &[SpellResource],
    ) -> LocalStateResult<()> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        domain.replace_spell_resources(participant, expected_revision, resources)?;
        transaction.commit()?;
        Ok(())
    }
    pub fn set_spell_expended(
        &self,
        participant: &str,
        expected_revision: i64,
        spell: &str,
        expended: bool,
    ) -> LocalStateResult<()> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        domain.set_spell_expended(participant, expected_revision, spell, expended)?;
        transaction.commit()?;
        Ok(())
    }
    pub fn initiative_roll_receipt(
        &self,
        encounter_key: &str,
        request_id: &str,
        request: &serde_json::Value,
    ) -> LocalStateResult<Option<InitiativeRollReceipt>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let domain = EncountersRead::from_connection(&transaction);
        let result = domain.initiative_roll_receipt(encounter_key, request_id, request)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn roll_initiative(
        &self,
        encounter_key: &str,
        request_id: &str,
        request: &serde_json::Value,
        expected_revision: i64,
        resolved: &[ResolvedInitiativeRoll],
    ) -> LocalStateResult<InitiativeRollReceipt> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.roll_initiative(
            encounter_key,
            request_id,
            request,
            expected_revision,
            resolved,
        )?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn create(&self, encounter: NewEncounter) -> LocalStateResult<Encounter> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.create(encounter)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn get(&self, encounter_ref: &str) -> LocalStateResult<Option<Encounter>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let domain = EncountersRead::from_connection(&transaction);
        let result = domain.get(encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn get_with_participants(
        &self,
        encounter_ref: &str,
    ) -> LocalStateResult<Option<EncounterWithParticipants>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let domain = EncountersRead::from_connection(&transaction);
        let result = domain.get_with_participants(encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn delete(&self, encounter_ref: &str) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.delete(encounter_ref)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn add_participant(
        &self,
        encounter_ref: &str,
        participant: AddEncounterParticipant,
    ) -> LocalStateResult<EncounterParticipant> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.add_participant(encounter_ref, participant)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn add_participants(
        &self,
        encounter_ref: &str,
        participants: Vec<AddEncounterParticipant>,
    ) -> LocalStateResult<Vec<EncounterParticipant>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.add_participants(encounter_ref, participants)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn update_participant(
        &self,
        participant: UpdateEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.update_participant(participant)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn reorder_participant(
        &self,
        reorder: ReorderEncounterParticipant,
    ) -> LocalStateResult<Option<EncounterParticipant>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.reorder_participant(reorder)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn add_condition(
        &self,
        condition: AddEncounterParticipantCondition,
    ) -> LocalStateResult<EncounterParticipantCondition> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.add_condition(condition)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn update_condition(
        &self,
        condition: UpdateEncounterParticipantCondition,
    ) -> LocalStateResult<Option<EncounterParticipantCondition>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.update_condition(condition)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn remove_condition(
        &self,
        participant_key: &str,
        condition_id: i64,
    ) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.remove_condition(participant_key, condition_id)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn set_current_turn(
        &self,
        encounter_ref: &str,
        participant_key: Option<&str>,
    ) -> LocalStateResult<bool> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.set_current_turn(encounter_ref, participant_key)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn advance_turn(
        &self,
        encounter_ref: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> EncounterTurnState,
    ) -> LocalStateResult<Option<Encounter>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.advance_turn(encounter_ref, choose)?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn advance_turn_once(
        &self,
        encounter_ref: &str,
        request_id: &str,
        intent: &serde_json::Value,
        expected_revision: i64,
        choose: impl FnOnce(&EncounterWithParticipants) -> LocalStateResult<(EncounterTurnUpdate, bool)>,
    ) -> LocalStateResult<Option<Encounter>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result = domain.advance_turn_once(
            encounter_ref,
            request_id,
            intent,
            expected_revision,
            choose,
        )?;
        transaction.commit()?;
        Ok(result)
    }
    pub fn remove_participant_and_select_turn(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        choose: impl FnOnce(&EncounterWithParticipants) -> Option<EncounterTurnState>,
    ) -> LocalStateResult<Option<Encounter>> {
        let mut connection = self.0.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let domain = EncountersWrite::from_connection(&transaction);
        let result =
            domain.remove_participant_and_select_turn(encounter_ref, participant_key, choose)?;
        transaction.commit()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> LocalStateResult<Connection> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE record_identity(id INTEGER PRIMARY KEY,key TEXT UNIQUE,last_known_name TEXT,availability TEXT NOT NULL);")?;
        connection.execute_batch(LOCAL_STATE_SCHEMA_SQL)?;
        connection.execute_batch(ENCOUNTER_STATUS_SCHEMA_SQL)?;
        connection.execute_batch(ENCOUNTER_DIFFICULTY_SCHEMA_SQL)?;
        connection.execute_batch(ABILITY_STATE_SCHEMA_SQL)?;
        connection.execute_batch(HAZARD_PARTICIPATION_SCHEMA_SQL)?;
        connection.execute_batch(CONDITION_SYSTEM_SCHEMA_SQL)?;
        connection.execute_batch(ENCOUNTER_RECOVERY_SCHEMA_SQL)?;
        connection.execute_batch(MANUAL_TRACKERS_SCHEMA_SQL)?;
        connection.execute_batch(ENCOUNTER_CLOCK_SCHEMA_SQL)?;
        backfill_encounter_clock(&connection).expect("fresh workspace clock backfill");
        Ok(connection)
    }

    #[test]
    fn outer_transaction_rolls_back_multiple_domain_actions() -> LocalStateResult<()> {
        let mut connection = connection()?;
        {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let lists = SavedListsWrite::from_connection(&tx);
            lists.create(NewSavedList {
                slug: "kept-together".into(),
                name: "List".into(),
                description: None,
                tags: vec![],
            })?;
            let encounters = EncountersWrite::from_connection(&tx);
            encounters.create(NewEncounter {
                slug: "same-action".into(),
                name: "Encounter".into(),
                description: None,
                note: None,
            })?;
            assert_eq!(lists.list()?.len(), 1);
            assert_eq!(encounters.list()?.len(), 1);
            // Dropping the caller's transaction abandons the complete composed action.
        }
        assert!(
            SavedListsRead::from_connection(&connection)
                .list()?
                .is_empty()
        );
        assert!(
            EncountersRead::from_connection(&connection)
                .list()?
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn unknown_import_retains_identity_and_available_identity_is_not_replaced()
    -> LocalStateResult<()> {
        let mut connection = connection()?;
        connection.execute("INSERT INTO record_identity(key,last_known_name,availability) VALUES('spells:known','Current name','available')", [])?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let lists = SavedListsWrite::from_connection(&tx);
        lists.import(ImportSavedList {
            slug: "imported".into(),
            name: "Imported".into(),
            description: None,
            tags: vec![],
            replace: false,
            items: ["spells:known", "spells:removed"]
                .into_iter()
                .map(|key| ImportSavedListItem {
                    record_key: key.into(),
                    note: None,
                    record_title_snapshot: "Saved name".into(),
                    record_kind_snapshot: Some("spell".into()),
                })
                .collect(),
        })?;
        tx.commit()?;
        let known: (String, String) = connection.query_row(
            "SELECT last_known_name,availability FROM record_identity WHERE key='spells:known'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(known, ("Current name".into(), "available".into()));
        let missing: (String, String) = connection.query_row(
            "SELECT last_known_name,availability FROM record_identity WHERE key='spells:removed'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        assert_eq!(missing, ("Saved name".into(), "unavailable".into()));
        assert!(
            connection
                .execute("DELETE FROM record_identity WHERE key='spells:removed'", [])
                .is_err()
        );
        connection.execute(
            "UPDATE record_identity SET availability='unavailable' WHERE key='spells:known'",
            [],
        )?;
        let saved = SavedListsRead::from_connection(&connection)
            .get_with_items("imported")?
            .unwrap();
        assert_eq!(saved.items.len(), 2);
        assert_eq!(saved.items[0].record_title_snapshot, "Saved name");
        Ok(())
    }

    #[test]
    fn owner_scopes_slugs_without_reusing_stable_keys() -> LocalStateResult<()> {
        let mut connection = connection()?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        SavedListsWrite::from_connection(&tx).create(NewSavedList {
            slug: "same".into(),
            name: "Local".into(),
            description: None,
            tags: vec![],
        })?;
        tx.execute("INSERT INTO principals VALUES(2,'other','Other')", [])?;
        tx.execute("INSERT INTO saved_lists(owner_id,list_key,slug,name,created_at,updated_at) VALUES(2,'list_other','same','Other','now','now')", [])?;
        assert_eq!(
            SavedListsRead::from_connection(&tx)
                .get("same")?
                .unwrap()
                .name,
            "Local"
        );
        assert_eq!(SavedListsRead::from_connection(&tx).list()?.len(), 1);
        assert!(tx.execute("INSERT INTO saved_lists(owner_id,list_key,slug,name,created_at,updated_at) VALUES(1,'list_duplicate','same','Duplicate','now','now')", []).is_err());
        tx.commit()?;
        Ok(())
    }
}
