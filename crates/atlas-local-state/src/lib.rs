#![deny(unsafe_code)]

mod atomic_action;
mod encounters;
mod error;
mod parties;
pub use parties::{PartiesRead, PartiesWrite, Party, PartyMember, PartyMemberInput};
mod references;
mod release;
pub use release::upgrade_release_owners;
mod saved_lists;
mod slug;
mod store;
pub mod workspace_recovery;

pub use encounters::{
    AbilityStateChange, AbilityStateValue, AddEncounterParticipant,
    AddEncounterParticipantCondition, CheckDegree, ConditionAction, ConditionDuration,
    ConditionDurationUnit, ConditionEventBoundary, ConditionEventChange, ConditionEventOutcome,
    ConditionEventSource, ConditionExtraAction, ConditionOrigin, ConditionParameters,
    ConditionScope, ConditionSubject, ConditionTiming, ConditionTurnEvent,
    ConsumedConditionApplication, DamageLethality, DamageMultiplier, Encounter,
    EncounterBoundaryClaim, EncounterBoundaryKind, EncounterEventReceipt, EncounterParticipant,
    EncounterParticipantCondition, EncounterStatus, EncounterTurnState, EncounterTurnUpdate,
    EncounterWithParticipantCount, EncounterWithParticipants, EncountersRead, EncountersWrite,
    FrightenedRecovery, InitiativeRollReceipt, LifecycleCheckPurpose, MortalityState, NewEncounter,
    ParticipantAbilityState, ParticipantEventChange, ParticipantHpOrigin,
    ParticipantInitiativeRoll, ParticipantKind, ParticipantSide, ParticipantVariant,
    ParticipantVariantOrigin, PendingConsequence, PersistentDamageOutcome,
    PersistentDamageProtection, PersistentPrecedenceChange, PersistentRecovery,
    PersistentSourcePrecedence, PreparedConditionEvent, ReorderEncounterParticipant,
    ResolvedInitiativeRoll, SavedHpEventContext, SavedPendingConsequence, SpellResource,
    UpdateEncounter, UpdateEncounterParticipant, UpdateEncounterParticipantCondition, WakingRule,
};
pub use encounters::{
    ApplyEncounterRecovery, EncounterDocument, EncounterRecordSource, ParticipantGameplay,
};
pub use encounters::{
    EncounterDifficultyReference, SavedOutgoingTurn, TurnActionAdjustment, TurnActionAllocation,
    TurnResourceEffect, TurnResources,
};
pub use encounters::{ManualTracker, ManualTrackerChange, TrackerProgress};
pub use error::{LocalStateError, LocalStateResult};
pub use saved_lists::{
    AddSavedListItemOutcome, HydratedSavedListItem, ImportSavedList, ImportSavedListItem,
    NewSavedList, ReorderSavedListItem, ResolvedSavedListItem, SavedList, SavedListItem,
    SavedListItemSnapshot, SavedListItemStatus, SavedListWithItems, SavedListsRead,
    SavedListsWrite, UpdateSavedList, hydrate_saved_list_item,
};
pub use slug::derive_slug;
pub use store::{LocalStateRead, LocalStateWrite};
#[cfg(test)]
mod test_support;
#[cfg(test)]
use test_support::TestStore;
