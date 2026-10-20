#![deny(unsafe_code)]

mod error;
mod operations;
mod paths;
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

pub use error::{RuntimeError, RuntimeErrorKind, RuntimePathTarget, RuntimePlatform};
pub use paths::{AtlasPathMode, AtlasPathOverrides, ResolvedAtlasPaths, ResolvedPathMode};
pub use retrieval::SearchProfileReadiness;
pub use runtime::{AtlasRuntime, AtlasRuntimeOptions};
pub use setup_model::{
    RuntimeSetupCleanOptions, RuntimeSetupCleanReport, RuntimeSetupOptions, RuntimeSetupReport,
    SetupAction, SetupActionKind, SetupActionStatus, SetupBuildReport, SetupCleanTarget,
    SetupCleanTargetKind, SetupCleanTargetStatus, SetupEmbeddingReport, SetupExitClass,
    SetupPathsReport, SetupReadiness, SetupReadinessItem, SetupReadinessStatus, SetupTarget,
};
pub use workspace::{WORKSPACE_SCHEMA_VERSION, initialize_workspace};

pub use operations::{StateRead, WorkspaceRead, WorkspaceWrite};

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
