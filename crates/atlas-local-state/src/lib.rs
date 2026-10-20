#![deny(unsafe_code)]

mod atomic_action;
mod encounters;
mod error;
mod references;
mod saved_lists;
mod schema;
mod slug;
mod store;

pub use encounters::{
    AddEncounterParticipant, AddEncounterParticipantCondition, ConditionDuration,
    ConditionDurationUnit, ConditionTiming, ConditionTurnEvent, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterStatus, EncounterTurnState, EncounterTurnUpdate,
    EncounterWithParticipants, EncountersRead, EncountersWrite, InitiativeRollReceipt,
    NewEncounter, ParticipantHpOrigin, ParticipantInitiativeRoll, ParticipantKind,
    ParticipantRollGuard, ParticipantSide, ParticipantVariant, ParticipantVariantOrigin,
    ReorderEncounterParticipant, ReorderPlacement, ResolvedInitiativeRoll, SpellResource,
    StoredStrikeDamage, StoredStrikeDamageTotal, StoredStrikeOutcome, StoredStrikeReceipt,
    StoredStrikeRoll, UpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition,
};
pub use error::{LocalStateError, LocalStateResult};
pub use saved_lists::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    NewSavedList, ReorderSavedListItem, ResolvedSavedListItem, SavedList, SavedListItem,
    SavedListItemPlacement, SavedListItemSnapshot, SavedListItemStatus, SavedListWithItems,
    SavedListsRead, SavedListsWrite, UpdateSavedList, hydrate_saved_list_item,
};
pub use schema::{
    ENCOUNTER_STATUS_SCHEMA_SQL, HAZARD_PARTICIPATION_SCHEMA_SQL, LOCAL_STATE_SCHEMA_SQL,
};
pub use slug::derive_slug;
pub use store::{LocalStateRead, LocalStateWrite};
#[cfg(test)]
mod test_support;
#[cfg(test)]
use test_support::TestStore;
