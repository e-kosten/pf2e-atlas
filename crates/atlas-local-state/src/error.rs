use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalStateError {
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
    #[error("could not allocate a unique encounter key")]
    EncounterKeyAllocationFailed,
    #[error("could not allocate a unique encounter participant key")]
    ParticipantKeyAllocationFailed,
    #[error("encounter spell resource was not found for participant `{0}`")]
    SpellResourceNotFound(String),
    #[error("encounter spell resource is exhausted for participant `{0}`")]
    SpellResourceExhausted(String),
    #[error("encounter spell resource is already at its original value for participant `{0}`")]
    SpellResourceAtBaseline(String),
    #[error("encounter participant `{0}` has no creation baseline and cannot be reset")]
    ParticipantResetBaselineUnavailable(String),
    #[cfg(test)]
    #[error("injected reset failure after participant write")]
    InjectedResetFailure,
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
