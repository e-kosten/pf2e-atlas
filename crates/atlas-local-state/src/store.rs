//! Domain access borrowed from a caller-owned workspace transaction.
//! Each mutation has a savepoint: errors undo that action, while final commit stays caller-owned.
use crate::{EncountersRead, EncountersWrite, SavedListsRead, SavedListsWrite};
use atlas_storage::{ReadSession, WriteSession};

pub struct LocalStateRead<'a> {
    session: &'a ReadSession,
}
pub struct LocalStateWrite<'a> {
    session: &'a WriteSession,
}
impl<'a> LocalStateRead<'a> {
    pub fn new(session: &'a ReadSession) -> Self {
        Self { session }
    }
    pub fn saved_lists(&self) -> SavedListsRead<'a> {
        SavedListsRead::from_connection(self.session.connection())
    }
    pub fn encounters(&self) -> EncountersRead<'a> {
        EncountersRead::from_connection(self.session.connection())
    }
}
impl<'a> LocalStateWrite<'a> {
    pub fn new(session: &'a WriteSession) -> Self {
        Self { session }
    }
    pub fn saved_lists(&self) -> SavedListsWrite<'a> {
        SavedListsWrite::from_connection(self.session.connection())
    }
    pub fn encounters(&self) -> EncountersWrite<'a> {
        EncountersWrite::from_connection(self.session.connection())
    }
}
