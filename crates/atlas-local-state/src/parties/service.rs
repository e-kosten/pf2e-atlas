use super::storage;
use crate::{LocalStateError, LocalStateResult, Party, PartyMemberInput};
use rusqlite::params;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct PartiesRead<'a> {
    pub(super) connection: &'a rusqlite::Connection,
    pub(super) owner_id: i64,
}
pub struct PartiesWrite<'a> {
    pub(super) read: PartiesRead<'a>,
}
impl<'a> std::ops::Deref for PartiesWrite<'a> {
    type Target = PartiesRead<'a>;
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
impl<'a> PartiesRead<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection, owner_id: i64) -> Self {
        Self {
            connection,
            owner_id,
        }
    }
    pub fn list(&self) -> LocalStateResult<Vec<Party>> {
        storage::list(self.connection, self.owner_id)
    }
    pub fn get(&self, key: &str) -> LocalStateResult<Party> {
        storage::get(self.connection, self.owner_id, key)?
            .ok_or_else(|| LocalStateError::PartyNotFound(key.into()))
    }
    pub fn get_at_revision(&self, key: &str, revision: i64) -> LocalStateResult<Party> {
        let party = self.get(key)?;
        if party.revision != revision {
            return Err(LocalStateError::PartyConflict);
        }
        Ok(party)
    }
}
impl<'a> PartiesWrite<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection, owner_id: i64) -> Self {
        Self {
            read: PartiesRead::from_connection(connection, owner_id),
        }
    }
    pub fn create(&self, name: &str, members: &[PartyMemberInput]) -> LocalStateResult<Party> {
        crate::atomic_action::run(self.connection, || {
            validate(name, members)?;
            let key = storage::new_key("party");
            self.connection.execute(
                "INSERT INTO parties(party_key,name,owner_id) VALUES(?1,?2,?3)",
                params![key, name.trim(), self.owner_id],
            )?;
            storage::replace_members(self.connection, &key, members)?;
            self.get(&key)
        })
    }
    pub fn update(
        &self,
        key: &str,
        revision: i64,
        name: &str,
        members: &[PartyMemberInput],
    ) -> LocalStateResult<Party> {
        crate::atomic_action::run(self.connection, || {
            let existing = self.get_at_revision(key, revision)?;
            validate(name, members)?;
            self.connection.execute(
                "UPDATE parties SET name=?1,revision=revision+1 WHERE party_key=?2 AND owner_id=?3",
                params![name.trim(), key, self.owner_id],
            )?;
            storage::replace_members(self.connection, &existing.party_key, members)?;
            self.get(key)
        })
    }
    pub fn delete(&self, key: &str, revision: i64) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            self.get_at_revision(key, revision)?;
            self.connection.execute(
                "DELETE FROM parties WHERE party_key=?1 AND owner_id=?2",
                params![key, self.owner_id],
            )?;
            Ok(())
        })
    }
}
fn validate(name: &str, members: &[PartyMemberInput]) -> LocalStateResult<()> {
    let valid_name = |s: &str| !s.trim().is_empty() && s.trim().chars().count() <= 255;
    if !valid_name(name) {
        return Err(LocalStateError::InvalidParty(
            "Enter a party name of 1 to 255 characters.",
        ));
    }
    validate_member_count(members.len())?;
    let mut keys = HashSet::new();
    for member in members {
        if !keys.insert(&member.character_key) {
            return Err(LocalStateError::InvalidParty(
                "Character was supplied twice.",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_member_count(count: usize) -> LocalStateResult<()> {
    if count > 50 {
        return Err(LocalStateError::InvalidParty(
            "A party can contain at most 50 PCs.",
        ));
    }
    Ok(())
}
