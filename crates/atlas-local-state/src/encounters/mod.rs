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
    AddEncounterParticipant, AddEncounterParticipantCondition, ConditionDuration,
    ConditionDurationUnit, ConditionTiming, ConditionTurnEvent, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterStatus, EncounterTurnState, EncounterTurnUpdate,
    EncounterWithParticipants, NewEncounter, ParticipantHpOrigin, ParticipantKind, ParticipantSide,
    ParticipantVariant, ParticipantVariantOrigin, ReorderEncounterParticipant, ReorderPlacement,
    SpellResource, UpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition,
};
pub use service::{EncountersRead, EncountersWrite};

#[cfg(test)]
mod tests;

mod spell_resources;
