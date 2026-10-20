use super::storage;
use crate::{LocalStateError, LocalStateResult, Party, PartyMemberInput};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub struct PartiesRead<'a> {
    connection: &'a rusqlite::Connection,
}
pub struct PartiesWrite<'a> {
    read: PartiesRead<'a>,
}
impl<'a> std::ops::Deref for PartiesWrite<'a> {
    type Target = PartiesRead<'a>;
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}
impl<'a> PartiesRead<'a> {
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self { connection }
    }
    pub fn list(&self) -> LocalStateResult<Vec<Party>> {
        storage::list(self.connection)
    }
    pub fn get(&self, key: &str) -> LocalStateResult<Party> {
        storage::get(self.connection, key)?
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
    pub(crate) fn from_connection(connection: &'a rusqlite::Connection) -> Self {
        Self {
            read: PartiesRead::from_connection(connection),
        }
    }
    pub fn create(&self, name: &str, members: &[PartyMemberInput]) -> LocalStateResult<Party> {
        crate::atomic_action::run(self.connection, || {
            validate(name, members, None)?;
            let key = storage::new_key("party");
            self.connection.execute(
                "INSERT INTO parties(party_key,name) VALUES(?1,?2)",
                [&key, name.trim()],
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
            validate(name, members, Some(&existing))?;
            self.connection.execute(
                "UPDATE parties SET name=?1,revision=revision+1 WHERE party_key=?2 AND owner_id=1",
                [name.trim(), key],
            )?;
            storage::replace_members(self.connection, key, members)?;
            self.get(key)
        })
    }
    pub fn delete(&self, key: &str, revision: i64) -> LocalStateResult<()> {
        crate::atomic_action::run(self.connection, || {
            self.get_at_revision(key, revision)?;
            self.connection.execute(
                "DELETE FROM parties WHERE party_key=?1 AND owner_id=1",
                [key],
            )?;
            Ok(())
        })
    }
}
fn validate(
    name: &str,
    members: &[PartyMemberInput],
    existing: Option<&Party>,
) -> LocalStateResult<()> {
    let valid_name = |s: &str| !s.trim().is_empty() && s.trim().chars().count() <= 255;
    if !valid_name(name) {
        return Err(LocalStateError::InvalidParty(
            "Enter a party name of 1 to 255 characters.",
        ));
    }
    if members.len() > 50 {
        return Err(LocalStateError::InvalidParty(
            "A party can contain at most 50 PCs.",
        ));
    }
    let mut keys = HashSet::new();
    for member in members {
        if !valid_name(&member.name) {
            return Err(LocalStateError::InvalidParty(
                "Enter a PC name of 1 to 255 characters.",
            ));
        }
        if let Some(key) = &member.member_key
            && (!keys.insert(key)
                || !existing.is_some_and(|p| p.members.iter().any(|m| &m.member_key == key)))
        {
            return Err(LocalStateError::InvalidParty(
                "Member does not belong to this party or was supplied twice.",
            ));
        }
    }
    Ok(())
}
