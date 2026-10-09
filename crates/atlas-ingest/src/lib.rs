#![deny(unsafe_code)]

mod artifact_manifest;
mod audit;
mod build;
mod diagnostics;
mod embedding_reuse;
mod embeddings;
mod error;
mod generated;
mod index_build_input;
mod records;
mod report;
mod source;
mod source_enrichment;
mod source_loading;
mod source_pipeline;

pub use artifact_manifest::{
    ADJACENT_ARTIFACT_MANIFEST_PATH, ARTIFACT_MANIFEST_VERSION, ArtifactManifest,
    SourceFingerprint, SourcePositionReport, adjacent_artifact_manifest_path,
    compute_source_fingerprint, compute_source_position_report, read_artifact_manifest,
    source_git_commit_if_clean,
};
pub use audit::{
    SourcePathAuditOptions, SourcePathAuditPathReport, SourcePathAuditReport,
    SourcePathAuditSample, SourcePathAuditValueType, SourcePathCoverageStatus, audit_source_paths,
};
pub use diagnostics::{DroppedInlineMacroDiagnostic, IngestDiagnostics};
pub use error::IngestError;
pub use report::{
    SourceAnalysisEmbeddingReport, SourceAnalysisMechanicsReport, SourceAnalysisMetricReport,
    SourceAnalysisRelationshipReport, SourceAnalysisReport, SourceAnalysisSourceReport,
    SourceAnalysisTextReport, analyze_foundry_source,
};
pub use source::model::{
    BuildArtifactOptions, BuildArtifactReport, DocumentEmbeddingRecordTruncationCoverageReport,
    DocumentEmbeddingSectionTruncationReport, DocumentEmbeddingTokenizationReport,
    DocumentEmbeddingTruncationExampleReport, DocumentEmbeddingUnitKindTruncationReport,
    EmbeddingTimingReport, SkippedRecord,
};
pub use source_enrichment::{
    EnrichedDocumentOutcome, EnrichedFoundryDocument, EnrichedFoundryPack, EnrichedFoundrySource,
    SourceEnrichmentContext, SourceEnrichmentReport, enrich_loaded_source,
};
pub use source_loading::{
    LoadedFoundryDocument, LoadedFoundryPack, LoadedFoundrySource, QuarantinedSourceFile,
    SourceDocumentCounts, SourceFileProvenance, SourceLoadFailure, SourceLoadFailureStage,
    SourceLoadingReport, SourceMetadata, SourcePackMetadata, load_foundry_documents,
};

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    build::build_artifact(options)
}
