pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("workspace does not exist at {0}; run atlas setup")]
    Missing(std::path::PathBuf),
    #[error("workspace is busy; retry after the current operation finishes")]
    Busy,
    #[error("reference publication requires exclusive workspace admission")]
    StructuralPublication,
    #[error("workspace maintenance is incomplete: {0}")]
    Maintenance(String),
    #[error(
        "workspace schema {actual} does not match this application ({expected}); run setup or restart the application"
    )]
    Schema { expected: u32, actual: u32 },
    #[error("unsupported workspace: {0}")]
    Unsupported(String),
    #[error("invalid workspace migration: {0}")]
    Migration(String),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl StorageError {
    pub fn is_contention(&self) -> bool {
        matches!(self, Self::Busy)
            || matches!(self, Self::Sqlite(error) if matches!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
            ))
    }

    pub fn is_access_failure(&self) -> bool {
        matches!(self, Self::Io(_))
            || matches!(self, Self::Sqlite(error) if matches!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::PermissionDenied
                    | rusqlite::ErrorCode::ReadOnly
                    | rusqlite::ErrorCode::CannotOpen
                    | rusqlite::ErrorCode::SystemIoFailure)
            ))
    }
}
