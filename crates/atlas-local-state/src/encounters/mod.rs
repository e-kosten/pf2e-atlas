mod turn_resources;
pub use turn_resources::{TurnResourceEffect, TurnResources};
mod outgoing_turns;
pub use outgoing_turns::SavedOutgoingTurn;
mod persistent_precedence;
pub use persistent_precedence::PersistentSourcePrecedence;
mod event_model;
mod events;
pub use event_model::{
    CheckDegree, ConditionEventBoundary, ConditionEventChange, ConditionEventOutcome,
    ConditionEventSource, ConsumedConditionApplication, DamageLethality, EncounterBoundaryClaim,
    EncounterBoundaryKind, EncounterEventReceipt, LifecycleCheckPurpose, ParticipantEventChange,
    PendingConsequence, PersistentDamageOutcome, PersistentDamageProtection,
    PersistentPrecedenceChange, PersistentPreparationContext, PreparedConditionEvent,
    PreparedPersistentRecovery, SavedHpEventContext, SavedPendingConsequence,
};
mod condition_parameters;
pub use condition_parameters::{
    ConditionOrigin, ConditionParameters, ConditionSubject, DamageMultiplier, FrightenedRecovery,
    MortalityState, PersistentRecovery, WakingRule,
};
mod strike_receipt;
pub use strike_receipt::{
    StoredStrikeDamage, StoredStrikeDamageTotal, StoredStrikeOutcome, StoredStrikeReceipt,
    StoredStrikeRoll,
};
mod receipt_codec;
mod strike_rolls;
pub use strike_rolls::ParticipantRollGuard;
mod initiative_rolls;
mod model;
pub use initiative_rolls::{
    InitiativeRollReceipt, ParticipantInitiativeRoll, ResolvedInitiativeRoll,
};
mod service;
mod storage;

pub use model::{
    AddEncounterParticipant, AddEncounterParticipantCondition, ConditionAction, ConditionDuration,
    ConditionDurationUnit, ConditionExtraAction, ConditionScope, ConditionTiming,
    ConditionTurnEvent, Encounter, EncounterParticipant, EncounterParticipantCondition,
    EncounterStatus, EncounterTurnState, EncounterTurnUpdate, EncounterWithParticipants,
    NewEncounter, ParticipantHpOrigin, ParticipantKind, ParticipantSide, ParticipantVariant,
    ParticipantVariantOrigin, ReorderEncounterParticipant, ReorderPlacement, SpellResource,
    UpdateEncounter, UpdateEncounterParticipant, UpdateEncounterParticipantCondition,
};
pub use service::{EncountersRead, EncountersWrite};

#[cfg(test)]
mod tests;

mod spell_resources;
