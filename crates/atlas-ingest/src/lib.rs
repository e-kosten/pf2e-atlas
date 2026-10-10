#![deny(unsafe_code)]
mod audit;
mod build;
mod build_model;
#[cfg(test)]
mod build_tests;
mod error;
mod index_build_input;
mod report;
mod source;
mod source_aliases;
mod source_embedding_reuse;
mod source_enrichment;
mod source_fingerprint;
mod source_loading;
pub use audit::{
    SourcePathAuditOptions, SourcePathAuditPathReport, SourcePathAuditReport,
    SourcePathAuditSample, SourcePathAuditValueType, audit_source_paths,
};
pub use build_model::{BuildArtifactOptions, BuildArtifactReport};
pub use error::IngestError;
pub use report::{SourceAnalysisReport, analyze_foundry_source};
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
};
pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    build::build_artifact(options)
}
