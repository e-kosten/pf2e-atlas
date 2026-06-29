use std::collections::BTreeSet;

use rusqlite::TransactionBehavior;

use super::model::{
    AddSavedListItemOutcome, ImportSavedList, NewSavedList, NewSavedListItem, ResolvedSavedListItem,
};
use super::model::{SavedList, SavedListWithItems, UpdateSavedList};
use super::storage;
use crate::slug::validate_slug;
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
        validate_slug(&list.slug)?;
        let list = NewSavedList {
            tags: normalize_tags(list.tags),
            ..list
        };
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

    pub fn update(&self, list: UpdateSavedList) -> LocalStateResult<Option<SavedList>> {
        storage::validate_list_ref(&list.list_key)?;
        validate_slug(&list.slug)?;
        let list = UpdateSavedList {
            tags: normalize_tags(list.tags),
            ..list
        };
        let connection = self.store.connection()?;
        if !storage::update_list(&connection, list.clone())? {
            return Ok(None);
        }
        self.get(&list.list_key)
    }

    pub fn import(&self, import: ImportSavedList) -> LocalStateResult<SavedListWithItems> {
        validate_slug(&import.slug)?;
        validate_import_items(&import.items)?;
        let import = ImportSavedList {
            tags: normalize_tags(import.tags),
            ..import
        };
        let mut connection = self.store.connection()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = storage::get(&transaction, &import.slug)?;
        let list_key = match existing {
            Some(existing) if import.replace => {
                storage::update_list(
                    &transaction,
                    UpdateSavedList {
                        list_key: existing.list_key.clone(),
                        slug: import.slug.clone(),
                        name: import.name,
                        description: import.description,
                        tags: import.tags.clone(),
                    },
                )?;
                existing.list_key
            }
            Some(_) => return Err(LocalStateError::ListAlreadyExists(import.slug)),
            None => storage::insert_list(
                &transaction,
                NewSavedList {
                    slug: import.slug,
                    name: import.name,
                    description: import.description,
                    tags: import.tags,
                },
            )?,
        };
        storage::replace_items(&transaction, &list_key, import.items)?;
        let list = storage::get_with_items(&transaction, &list_key)?
            .ok_or_else(|| LocalStateError::ListNotFound(list_key.clone()))?;
        transaction.commit()?;
        Ok(list)
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

fn validate_import_items(items: &[super::model::ImportSavedListItem]) -> LocalStateResult<()> {
    let mut keys = BTreeSet::new();
    for item in items {
        storage::validate_record_key(&item.record_key)?;
        if !keys.insert(item.record_key.as_str()) {
            return Err(LocalStateError::InvalidListImport(format!(
                "duplicate record key `{}`",
                item.record_key
            )));
        }
    }
    Ok(())
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }
        let key = tag.to_lowercase();
        if seen.insert(key) {
            normalized.push(tag.to_string());
        }
    }
    normalized.sort_by_key(|tag| tag.to_lowercase());
    normalized
}
