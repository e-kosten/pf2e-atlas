#![deny(unsafe_code)]

mod error;
mod paths;
mod profile_assets;
mod retrieval;
mod runtime;
mod setup;
mod setup_clean;
mod setup_freshness;
mod setup_model;
#[cfg(test)]
mod tests;

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
