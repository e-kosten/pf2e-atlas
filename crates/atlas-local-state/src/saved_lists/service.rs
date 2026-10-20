use std::collections::BTreeSet;

use super::model::{
    AddSavedListItemOutcome, ImportSavedList, NewSavedList, NewSavedListItem, ResolvedSavedListItem,
};
use super::model::{SavedList, SavedListWithItems, UpdateSavedList};
use super::storage;
use crate::slug::validate_slug;
use crate::{LocalStateError, LocalStateResult};

#[derive(Clone, Copy)]
pub struct SavedListsRead<'a> {
    pub(super) connection: &'a rusqlite::Connection,
}
pub struct SavedListsWrite<'a> {
    pub(super) read: SavedListsRead<'a>,
}
impl<'a> std::ops::Deref for SavedListsWrite<'a> {
    type Target = SavedListsRead<'a>;
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
impl<'a> SavedListsRead<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self { connection }
    }
}
impl<'a> SavedListsWrite<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self {
            read: SavedListsRead::from_connection(connection),
        }
    }
}
impl SavedListsWrite<'_> {
    pub fn create(&self, list: NewSavedList) -> LocalStateResult<SavedList> {
        crate::atomic_action::run(self.connection, || {
            validate_slug(&list.slug)?;
            let list = NewSavedList {
                tags: normalize_tags(list.tags),
                ..list
            };
            let transaction = self.connection;
            let list_key = storage::insert_list(transaction, list)?;
            let result = storage::get(transaction, &list_key)?
                .ok_or(LocalStateError::ListNotFound(list_key))?;

            Ok(result)
        })
    }

    pub fn delete(&self, list_ref: &str) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            storage::validate_list_ref(list_ref)?;
            storage::delete(self.connection, list_ref)
        })
    }

    pub fn update(&self, list: UpdateSavedList) -> LocalStateResult<Option<SavedList>> {
        crate::atomic_action::run(self.connection, || {
            storage::validate_list_ref(&list.list_key)?;
            validate_slug(&list.slug)?;
            let list = UpdateSavedList {
                tags: normalize_tags(list.tags),
                ..list
            };
            let transaction = self.connection;
            if !storage::update_list(transaction, list.clone())? {
                return Ok(None);
            }
            let result = storage::get(transaction, &list.list_key)?;

            Ok(result)
        })
    }

    pub fn import(&self, import: ImportSavedList) -> LocalStateResult<SavedListWithItems> {
        crate::atomic_action::run(self.connection, || {
            validate_slug(&import.slug)?;
            validate_import_items(&import.items)?;
            let import = ImportSavedList {
                tags: normalize_tags(import.tags),
                ..import
            };
            let transaction = self.connection;
            let existing = storage::get(transaction, &import.slug)?;
            let list_key = match existing {
                Some(existing) if import.replace => {
                    storage::update_list(
                        transaction,
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
                    transaction,
                    NewSavedList {
                        slug: import.slug,
                        name: import.name,
                        description: import.description,
                        tags: import.tags,
                    },
                )?,
            };
            storage::replace_items(transaction, &list_key, import.items)?;
            let list = storage::get_with_items(transaction, &list_key)?
                .ok_or_else(|| LocalStateError::ListNotFound(list_key.clone()))?;

            Ok(list)
        })
    }

    pub fn add_resolved_item(
        &self,
        list_ref: &str,
        item: ResolvedSavedListItem,
    ) -> LocalStateResult<AddSavedListItemOutcome> {
        crate::atomic_action::run(self.connection, || {
            storage::validate_list_ref(list_ref)?;
            let item = NewSavedListItem {
                record_key: item.record_key.to_string(),
                note: item.note,
                record_title_snapshot: item.title_snapshot,
                record_kind_snapshot: item.kind_snapshot,
            };
            let transaction = self.connection;
            let outcome = storage::add_item(transaction, list_ref, item)?;

            Ok(outcome)
        })
    }

    pub fn update_item_note(
        &self,
        list_ref: &str,
        record_key: &str,
        note: Option<&str>,
    ) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            storage::validate_list_ref(list_ref)?;
            storage::validate_record_key(record_key)?;
            storage::update_item_note(self.connection, list_ref, record_key, note)
        })
    }

    pub fn remove_item(&self, list_ref: &str, record_key: &str) -> LocalStateResult<bool> {
        crate::atomic_action::run(self.connection, || {
            storage::validate_list_ref(list_ref)?;
            storage::validate_record_key(record_key)?;
            let transaction = self.connection;
            let removed = storage::remove_item(transaction, list_ref, record_key)?;

            Ok(removed)
        })
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

impl SavedListsRead<'_> {
    pub fn list(&self) -> LocalStateResult<Vec<SavedList>> {
        let transaction = self.connection;
        let result = storage::list(transaction)?;

        Ok(result)
    }
    pub fn get(&self, list_ref: &str) -> LocalStateResult<Option<SavedList>> {
        storage::validate_list_ref(list_ref)?;
        let transaction = self.connection;
        let result = storage::get(transaction, list_ref)?;

        Ok(result)
    }
    pub fn get_with_items(&self, list_ref: &str) -> LocalStateResult<Option<SavedListWithItems>> {
        storage::validate_list_ref(list_ref)?;
        let transaction = self.connection;
        let result = storage::get_with_items(transaction, list_ref)?;

        Ok(result)
    }
}
