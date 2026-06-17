use rusqlite::TransactionBehavior;

use super::model::{
    AddSavedListItemOutcome, NewSavedList, NewSavedListItem, ResolvedSavedListItem,
};
use super::model::{SavedList, SavedListWithItems};
use super::storage;
use crate::{LocalStateError, LocalStateResult, LocalStateStore};

#[derive(Debug, Clone, Copy)]
pub struct SavedLists<'a> {
    store: &'a LocalStateStore,
}

impl<'a> SavedLists<'a> {
    pub(crate) fn new(store: &'a LocalStateStore) -> Self {
        Self { store }
    }

    pub fn create(&self, list: NewSavedList) -> LocalStateResult<SavedList> {
        storage::validate_slug(&list.slug)?;
        let connection = self.store.connection()?;
        let list_key = storage::insert_list(&connection, list)?;
        self.get(&list_key)?
            .ok_or(LocalStateError::ListNotFound(list_key))
    }

    pub fn list(&self) -> LocalStateResult<Vec<SavedList>> {
        let connection = self.store.connection()?;
        storage::list(&connection)
    }

    pub fn get(&self, list_ref: &str) -> LocalStateResult<Option<SavedList>> {
        storage::validate_list_ref(list_ref)?;
        let connection = self.store.connection()?;
        storage::get(&connection, list_ref)
    }

    pub fn get_with_items(&self, list_ref: &str) -> LocalStateResult<Option<SavedListWithItems>> {
        storage::validate_list_ref(list_ref)?;
        let connection = self.store.connection()?;
        storage::get_with_items(&connection, list_ref)
    }

    pub fn delete(&self, list_ref: &str) -> LocalStateResult<bool> {
        storage::validate_list_ref(list_ref)?;
        let connection = self.store.connection()?;
        storage::delete(&connection, list_ref)
    }

    pub fn add_resolved_item(
        &self,
        list_ref: &str,
        item: ResolvedSavedListItem,
    ) -> LocalStateResult<AddSavedListItemOutcome> {
        storage::validate_list_ref(list_ref)?;
        let item = NewSavedListItem {
            record_key: item.record_key.to_string(),
            note: item.note,
            record_title_snapshot: item.title_snapshot,
            record_kind_snapshot: item.kind_snapshot,
        };
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let outcome = storage::add_item(&transaction, list_ref, item)?;
        transaction.commit()?;
        Ok(outcome)
    }

    pub fn remove_item(&self, list_ref: &str, record_key: &str) -> LocalStateResult<bool> {
        storage::validate_list_ref(list_ref)?;
        storage::validate_record_key(record_key)?;
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let removed = storage::remove_item(&transaction, list_ref, record_key)?;
        transaction.commit()?;
        Ok(removed)
    }
}
