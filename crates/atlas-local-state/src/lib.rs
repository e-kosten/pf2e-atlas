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
    ActionAttemptReceipt, ActionAttemptStatus, ActionCheckResult, ActionCheckSource,
    ActionCheckStage, ActionCompletion, ActionFactKind, ActionRollMode, ActionTargetMode,
    ConfusionChoice, FrozenAction, FrozenActionCheck, FrozenActionContext, FrozenActionJudgment,
    FrozenActionTarget, PreparedActionAttempt, RetainedStrikeRoll, SavedActionAttempt,
    SavedOutgoingTurn, SpellResourceCost,
};
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
    NewSavedList, ResolvedSavedListItem, SavedList, SavedListItem, SavedListItemSnapshot,
    SavedListItemStatus, SavedListWithItems, SavedListsRead, SavedListsWrite, UpdateSavedList,
    hydrate_saved_list_item,
};
pub use schema::{
    ACTION_ATTEMPTS_SCHEMA_SQL, CONDITION_CONTEXT_SCHEMA_SQL, CONDITION_EVENTS_SCHEMA_SQL,
    CONDITION_IMMUNITY_OVERRIDE_SCHEMA_SQL, ENCOUNTER_BOUNDARIES_SCHEMA_SQL,
    ENCOUNTER_RELATIONSHIPS_SCHEMA_SQL, ENCOUNTER_STATUS_SCHEMA_SQL,
    HAZARD_PARTICIPATION_SCHEMA_SQL, LOCAL_STATE_SCHEMA_SQL, OUTGOING_TURNS_SCHEMA_SQL,
    PARTICIPANT_FACTS_SCHEMA_SQL, PERSISTENT_SELECTIONS_SCHEMA_SQL, TURN_NOTIFICATIONS_SCHEMA_SQL,
};
pub use slug::derive_slug;
pub use store::{LocalStateRead, LocalStateWrite};
#[cfg(test)]
mod test_support;
#[cfg(test)]
use test_support::TestStore;

pub use encounters::{
    AttitudeState, DetectionState, DirectedRelationship, RelationshipCandidate, RelationshipChange,
    RelationshipKnowledge, RelationshipLayer, RelationshipProperty, RelationshipProvenance,
    RelationshipSourceSnapshot, RelationshipValue,
};
