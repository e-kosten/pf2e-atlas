#![deny(unsafe_code)]
mod audit;
mod build;
mod build_model;
#[cfg(test)]
mod build_tests;
mod error;
mod index_build_input;
#[cfg(test)]
mod passage_benchmark;
mod report;
mod source;
mod source_aliases;
mod source_compatibility;
mod source_enrichment;
mod source_fingerprint;
mod source_loading;
pub use audit::{
    SourcePathAuditOptions, SourcePathAuditPathReport, SourcePathAuditReport,
    SourcePathAuditSample, SourcePathAuditValueType, audit_source_paths,
};
pub use build_model::{BuildArtifactOptions, BuildArtifactReport, ProfileBuildReport};
pub use error::IngestError;
pub use report::{SourceAnalysisReport, analyze_foundry_source};
pub use source_compatibility::validate_source_compatibility;
pub use source_enrichment::{
    EnrichedDocumentOutcome, EnrichedFoundryDocument, EnrichedFoundryPack, EnrichedFoundrySource,
    SourceEnrichmentContext, SourceEnrichmentReport, enrich_loaded_source,
};
pub use source_fingerprint::{
    SourceFingerprint, compute_source_fingerprint, source_git_commit_if_clean,
};
pub use source_loading::{
    LoadedFoundryDocument, LoadedFoundryPack, LoadedFoundrySource, QuarantinedSourceFile,
    SourceDocumentCounts, SourceFileProvenance, SourceLoadFailure, SourceLoadFailureStage,
    SourceLoadingReport, SourceMetadata, SourcePackMetadata, load_foundry_documents,
    load_foundry_documents_with_reuse,
};
pub fn build_artifact(
    workspace: &atlas_storage::Workspace,
    schema: u32,
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    build::build_artifact(workspace, schema, options)
}

mod traits;
