#![deny(unsafe_code)]

mod artifact;
mod artifact_manifest;
mod build;
mod diagnostics;
mod embedding_reuse;
mod embeddings;
mod error;
mod generated;
mod records;
mod report;
mod source;
mod source_pipeline;

pub use artifact_manifest::{
    ADJACENT_ARTIFACT_MANIFEST_PATH, ARTIFACT_MANIFEST_VERSION, ArtifactManifest,
    SourceFingerprint, SourcePositionReport, adjacent_artifact_manifest_path,
    compute_source_fingerprint, compute_source_position_report, read_artifact_manifest,
    source_git_commit_if_clean,
};
pub use diagnostics::{DroppedInlineMacroDiagnostic, IngestDiagnostics};
pub use error::IngestError;
pub use report::{
    SourceAnalysisEmbeddingReport, SourceAnalysisMetricReport, SourceAnalysisRelationshipReport,
    SourceAnalysisReport, SourceAnalysisSideDataReport, SourceAnalysisSourceReport,
    SourceAnalysisTextReport, analyze_foundry_source, build_artifact_json,
};
pub use source::model::{
    BuildArtifactOptions, BuildArtifactReport, DocumentEmbeddingRecordTruncationCoverageReport,
    DocumentEmbeddingSectionTruncationReport, DocumentEmbeddingTokenizationReport,
    DocumentEmbeddingTruncationExampleReport, DocumentEmbeddingUnitKindTruncationReport,
    EmbeddingTimingReport, SkippedRecord,
};

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    build::build_artifact(options)
}
