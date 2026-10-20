#![deny(unsafe_code)]

mod atomic_action;
mod encounters;
mod error;
mod parties;
pub use parties::{PartiesRead, PartiesWrite, Party, PartyMember, PartyMemberInput};
mod references;
mod saved_lists;
mod schema;
mod slug;
mod store;

pub use encounters::SavedOutgoingTurn;
pub use encounters::{
    AddEncounterParticipant, AddEncounterParticipantCondition, CheckDegree, ConditionAction,
    ConditionDuration, ConditionDurationUnit, ConditionEventBoundary, ConditionEventChange,
    ConditionEventOutcome, ConditionExtraAction, ConditionOrigin, ConditionParameters,
    ConditionScope, ConditionSubject, ConditionTiming, ConditionTurnEvent, DamageLethality,
    DamageMultiplier, Encounter, EncounterBoundaryClaim, EncounterBoundaryKind,
    EncounterEventReceipt, EncounterParticipant, EncounterParticipantCondition, EncounterStatus,
    EncounterTurnState, EncounterTurnUpdate, EncounterWithParticipants, EncountersRead,
    EncountersWrite, FrightenedRecovery, InitiativeRollReceipt, LifecycleCheckPurpose,
    MortalityState, NewEncounter, ParticipantEventChange, ParticipantHpOrigin,
    ParticipantInitiativeRoll, ParticipantKind, ParticipantRollGuard, ParticipantSide,
    ParticipantVariant, ParticipantVariantOrigin, PendingConsequence, PersistentDamageOutcome,
    PersistentRecovery, PersistentSelectionChange, PersistentSourceSelection,
    PreparedConditionEvent, ReorderEncounterParticipant, ReorderPlacement, ResolvedInitiativeRoll,
    SavedHpEventContext, SavedPendingConsequence, SpellResource, StoredStrikeDamage,
    StoredStrikeDamageTotal, StoredStrikeOutcome, StoredStrikeReceipt, StoredStrikeRoll,
    UpdateEncounter, UpdateEncounterParticipant, UpdateEncounterParticipantCondition, WakingRule,
};
pub use error::{LocalStateError, LocalStateResult};
pub use saved_lists::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    NewSavedList, ReorderSavedListItem, ResolvedSavedListItem, SavedList, SavedListItem,
    SavedListItemPlacement, SavedListItemSnapshot, SavedListItemStatus, SavedListWithItems,
    SavedListsRead, SavedListsWrite, UpdateSavedList, hydrate_saved_list_item,
};
pub use schema::{
    CONDITION_SYSTEM_SCHEMA_SQL, ENCOUNTER_STATUS_SCHEMA_SQL, HAZARD_PARTICIPATION_SCHEMA_SQL,
    LOCAL_STATE_SCHEMA_SQL, PARTIES_SCHEMA_SQL,
};
pub use slug::derive_slug;
pub use store::{LocalStateRead, LocalStateWrite};
#[cfg(test)]
mod test_support;
#[cfg(test)]
use test_support::TestStore;
