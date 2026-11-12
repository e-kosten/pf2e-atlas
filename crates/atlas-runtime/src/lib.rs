#![deny(unsafe_code)]

mod desktop_reference;
mod embedding_seeds;
mod error;
mod operations;
mod paths;
pub use atlas_ingest::EmbeddingCoverageGap;
pub use embedding_seeds::{
    EmbeddingSeedPin, EmbeddingSeedPins, parse_embedding_seed_pins, published_embedding_seed_pins,
};
#[cfg(feature = "seed-tools")]
pub use embedding_seeds::{LocalEmbeddingSeed, download_embedding_seed};
mod profile_assets;
mod retrieval;
mod runtime;
mod setup;
mod setup_clean;
mod setup_freshness;
mod setup_model;
mod source_acquisition;
mod source_configuration;
#[cfg(test)]
mod tests;
mod workspace;
mod workspace_archive;
pub use workspace_archive::{RestorePreflight, ValidatedArchive, WorkspaceArchive};

pub use desktop_reference::{
    DesktopModelSelection, DesktopReferenceOutcome, DesktopReferenceStatus, DesktopRefreshError,
};
pub use error::{RuntimeError, RuntimeErrorKind, RuntimePathTarget, RuntimePlatform};
pub use paths::{AtlasPathMode, AtlasPathOverrides, ResolvedAtlasPaths, ResolvedPathMode};
pub use retrieval::SearchProfileReadiness;
pub use runtime::{AtlasRuntime, AtlasRuntimeOptions};
pub use setup_model::{
    DesktopSetupStage, RuntimeSetupCleanOptions, RuntimeSetupCleanReport, RuntimeSetupOptions,
    RuntimeSetupReport, SetupAction, SetupActionKind, SetupActionStatus, SetupBuildReport,
    SetupCleanTarget, SetupCleanTargetKind, SetupCleanTargetStatus, SetupEmbeddingReport,
    SetupExitClass, SetupPathsReport, SetupPublication, SetupReadiness, SetupReadinessItem,
    SetupReadinessStatus, SetupTarget,
};
pub use workspace::{WORKSPACE_SCHEMA_VERSION, initialize_workspace};

pub use operations::{ExactRecordRead, StateRead, WorkspaceRead, WorkspaceWrite};

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use atlas_storage::{StorageError, Workspace};
