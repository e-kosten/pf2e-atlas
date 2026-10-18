pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("workspace is busy; retry after the current operation finishes")]
    Busy,
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
