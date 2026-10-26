use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalStateError {
    #[error("This encounter boundary has already been processed")]
    EncounterBoundaryConflict,
    #[error("Party not found: {0}")]
    PartyNotFound(String),
    #[error("Party changed. Reopen it to review the current members.")]
    PartyConflict,
    #[error("{0}")]
    InvalidParty(&'static str),
    #[error("{0}")]
    InvalidDifficultyReference(&'static str),
    #[error("Local-state mutations require a caller-owned write transaction")]
    ActionRequiresTransaction,
    #[error("Unsupported stored roll receipt version {0}")]
    UnsupportedReceiptVersion(u32),
    #[error("This roll request ID was already used for a different request")]
    RollRequestConflict,
    #[error("Invalid roll request: {0}")]
    InvalidRollRequest(&'static str),
    #[error("{0}")]
    InvalidConditionEvent(String),
    #[error("Dice evaluation failed: {0}")]
    Dice(#[from] atlas_dice::DiceError),
    #[error("Roll receipt is not valid: {0}")]
    RollReceipt(#[from] serde_json::Error),
    #[error("local-state path is not valid UTF-8: {0}")]
    NonUtf8Path(String),
    #[error("invalid saved-list slug `{slug}`: {reason}")]
    InvalidSlug { slug: String, reason: &'static str },
    #[error("invalid saved-list ref `{list_ref}`: {reason}")]
    InvalidListRef {
        list_ref: String,
        reason: &'static str,
    },
    #[error("invalid encounter ref `{encounter_ref}`: {reason}")]
    InvalidEncounterRef {
        encounter_ref: String,
        reason: &'static str,
    },
    #[error("invalid saved-list record key `{record_key}`: {reason}")]
    InvalidRecordKey { record_key: String, reason: String },
    #[error("invalid saved-list import: {0}")]
    InvalidListImport(String),
    #[error("saved list already exists: {0}")]
    ListAlreadyExists(String),
    #[error("saved list not found: {0}")]
    ListNotFound(String),
    #[error("could not allocate a unique saved-list key")]
    ListKeyAllocationFailed,
    #[error("encounter already exists: {0}")]
    EncounterAlreadyExists(String),
    #[error("encounter not found: {0}")]
    EncounterNotFound(String),
    #[error("encounter participant not found: {0}")]
    ParticipantNotFound(String),
    #[error(
        "encounter participant `{participant_key}` changed: expected revision {expected_revision}, actual revision {actual_revision}"
    )]
    StaleParticipantRevision {
        participant_key: String,
        expected_revision: i64,
        actual_revision: i64,
    },
    #[error(
        "encounter `{encounter_key}` changed: expected revision {expected_revision}, actual revision {actual_revision}"
    )]
    StaleEncounterRevision {
        encounter_key: String,
        expected_revision: i64,
        actual_revision: i64,
    },
    #[error("could not allocate a unique encounter key")]
    EncounterKeyAllocationFailed,
    #[error("could not allocate a unique encounter participant key")]
    ParticipantKeyAllocationFailed,
    #[error("unsupported local-state metadata `{key}` value `{value}`")]
    UnsupportedMetadata { key: &'static str, value: String },
    #[error("local-state database is incompatible: {0}")]
    IncompatibleSchema(String),
    #[error("local-state database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("local-state filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("local-state timestamp error: {0}")]
    Timestamp(#[from] time::error::Format),
}

pub type LocalStateResult<T> = Result<T, LocalStateError>;
