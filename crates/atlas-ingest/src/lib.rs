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
pub use source::dto::{
    ActorType, FullItemSource, ItemSource, ItemType, NpcSource, PF2E_SOURCE_CONTRACT_VERSION,
    PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SYSTEM_ID, PF2E_SOURCE_PINNED_SYSTEM_VERSION,
    SerializedSourceObject, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity,
    SourceParentContext, SourcePresence, SourceVersionMetadata, VersionedItemSource,
    VersionedNpcSource, parse_item_source, parse_npc_source, pinned_source_version_metadata,
    validate_pinned_source_version,
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
