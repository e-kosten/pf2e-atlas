//! Owner-scoped domain access borrowed from a caller-owned workspace transaction.
//! Each mutation has a savepoint, while final commit stays caller-owned.
use crate::{
    EncountersRead, EncountersWrite, LocalStateError, LocalStateResult, SavedListsRead,
    SavedListsWrite,
};
use atlas_storage::{ReadSession, WriteSession};
use rusqlite::{Connection, OptionalExtension};

pub struct LocalStateRead<'a> {
    session: &'a ReadSession,
    owner_id: i64,
}
pub struct LocalStateWrite<'a> {
    session: &'a WriteSession,
    owner_id: i64,
}

pub(crate) fn local_owner(connection: &Connection) -> LocalStateResult<i64> {
    connection
        .query_row(
            "SELECT id FROM principals WHERE principal_key = 'local'",
            [],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| LocalStateError::IncompatibleSchema("missing seeded local principal".into()))
}

impl<'a> LocalStateRead<'a> {
    pub fn for_local(session: &'a ReadSession) -> LocalStateResult<Self> {
        Ok(Self {
            session,
            owner_id: local_owner(session.connection())?,
        })
    }
    pub fn parties(&self) -> crate::PartiesRead<'a> {
        crate::PartiesRead::from_connection(self.session.connection(), self.owner_id)
    }
    pub fn saved_lists(&self) -> SavedListsRead<'a> {
        SavedListsRead::from_connection(self.session.connection(), self.owner_id)
    }
    pub fn encounters(&self) -> EncountersRead<'a> {
        EncountersRead::from_connection(self.session.connection(), self.owner_id)
    }
}

impl<'a> LocalStateWrite<'a> {
    pub fn for_local(session: &'a WriteSession) -> LocalStateResult<Self> {
        Ok(Self {
            session,
            owner_id: local_owner(session.connection())?,
        })
    }
    pub fn parties(&self) -> crate::PartiesWrite<'a> {
        crate::PartiesWrite::from_connection(self.session.connection(), self.owner_id)
    }
    pub fn saved_lists(&self) -> SavedListsWrite<'a> {
        SavedListsWrite::from_connection(self.session.connection(), self.owner_id)
    }
    pub fn encounters(&self) -> EncountersWrite<'a> {
        EncountersWrite::from_connection(self.session.connection(), self.owner_id)
    }
}
