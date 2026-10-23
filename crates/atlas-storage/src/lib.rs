//! Workspace connection admission and transactional migrations.
//!
//! Domain crates own SQL and provide ordered migrations. This crate owns neither
//! reference projections nor user-state models. Sessions must be bounded to a
//! composed operation, never retained by an idle server or during inference.
#![deny(unsafe_code)]

mod backup;
mod error;
mod locking;
mod migration;
mod session;
mod snapshot;
mod workspace;

pub use error::{StorageError, StorageResult};
pub use migration::{DataBackfill, Migration};
pub use session::{MaintenanceSession, ReadSession, UpdateOwner, WriteSession};
pub use snapshot::WorkspaceInitialization;
pub use workspace::Workspace;

#[cfg(test)]
mod tests;
