#![deny(unsafe_code)]

mod encounters;
mod error;
mod saved_lists;
mod schema;
mod slug;
mod store;

pub const LOCAL_STATE_SCHEMA_VERSION: &str = "7";
pub const LOCAL_STATE_CONTRACT_VERSION: &str = "pf2e-atlas-local-state/v1";

pub use encounters::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterParticipantReset, EncounterParticipantResetDomain,
    EncounterParticipantSpellState, EncounterSpellResource, EncounterSpellResourceMutation,
    EncounterSpellResourceOperation, EncounterSpellResourceTarget, EncounterStatus,
    EncounterWithParticipants, Encounters, NewEncounter, ParticipantKind, ParticipantSide,
    ParticipantVariant, ReorderEncounterParticipant, ReorderPlacement, UpdateEncounter,
    UpdateEncounterParticipant, UpdateEncounterParticipantCondition,
};
pub use error::{LocalStateError, LocalStateResult};
pub use saved_lists::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    NewSavedList, ResolvedSavedListItem, SavedList, SavedListItem, SavedListItemSnapshot,
    SavedListItemStatus, SavedListWithItems, SavedLists, UpdateSavedList, hydrate_saved_list_item,
};
pub use slug::derive_slug;
pub use store::LocalStateStore;
