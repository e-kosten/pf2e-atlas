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
mod source_coverage;
mod source_pipeline;
mod validation;

pub use artifact_manifest::{
    ADJACENT_ARTIFACT_MANIFEST_PATH, ARTIFACT_MANIFEST_VERSION, ArtifactManifest,
    SourceFingerprint, SourcePositionReport, adjacent_artifact_manifest_path,
    compute_source_fingerprint, compute_source_position_report, read_artifact_manifest,
    source_git_commit_if_clean,
};
pub use audit::{
    RetrievalPredicateInventoryEntry, SourceCoverageDiagnostic, SourceCoverageDiagnosticKind,
    SourcePathAuditClosureFailure, SourcePathAuditDiff, SourcePathAuditDiffEntry,
    SourcePathAuditDispositionChange, SourcePathAuditEnforcement, SourcePathAuditFilters,
    SourcePathAuditMode, SourcePathAuditObservationMismatch, SourcePathAuditOptions,
    SourcePathAuditPathReport, SourcePathAuditReport, SourcePathAuditSample,
    SourcePathAuditSummary, SourcePathAuditValueType, SourcePathCoverageDisposition,
    audit_source_paths, disposition_label,
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
    PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE, PF2E_SOURCE_PINNED_SYSTEM_ID,
    PF2E_SOURCE_PINNED_SYSTEM_VERSION, SourceDiagnostic, SourceDiagnosticKind, SourceIdentity,
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
pub use source_coverage::{
    ATLAS_SOURCE_LEAF_COVERAGE_VERSION, CoverageContractError, CoverageFailure,
    CoverageFailureCode, CoverageReport, ExpectedSourceShape, FinalOwnerContract, FinalOwnerStage,
    FixtureContract, FixturePrevalence, FixtureProvenance, MapKeyPolicy,
    PF2E_SOURCE_LEAF_PREVALENCE_SHA256, PF2E_SOURCE_LEAF_PREVALENCE_VERSION,
    PF2E_TYPE_REGISTRY_ENTRY_COUNT, PF2E_TYPE_REGISTRY_SHA256, PF2E_TYPE_REGISTRY_VERSION,
    ReaderContract, SourceDocumentRole, SourceLeafContract, SourceLeafCoverageLedger,
    SourceLeafDisposition, SourceLeafIdentity, SourceLeafKind, SourceLeafReceipt,
    SourceLeafSelector, SourceParentContextSelector, SourcePin, SourcePrevalence,
    SpellArtifactReceiptOperations, SurfaceContract, SurfaceDecision, SurfaceDisposition,
    capture_registered_source_leaf_receipt, capture_registered_spell_source_leaf_receipts,
    evaluate_source_leaf_coverage, lint_source_leaf_ledger, lint_source_leaf_ledgers,
    parse_source_leaf_ledger,
};
pub use validation::{
    AssertionInventoryEntry, ExhaustiveValidationOptions, ExhaustiveValidationReport,
    run_exhaustive_validation,
};

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    build::build_artifact(options)
}
