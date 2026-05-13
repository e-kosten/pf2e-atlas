#![deny(unsafe_code)]

use thiserror::Error;

mod aliases;
mod generated_affliction_identity;
mod generated_afflictions;
mod metrics;
mod model;
mod normalize;
mod pipeline;
mod references;
pub mod report;
mod schema;
mod source;
mod variant_taxonomy;
mod variants;
mod writer;

pub use model::{BuildArtifactOptions, BuildArtifactReport, IngestDiagnostics, SkippedRecord};
pub use report::analyze_foundry_source;

pub(crate) use model::{
    ActorSideData, AfflictionFamily, AfflictionOccurrence, AliasSource,
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME, DerivedAfflictionRecordInput,
    FolderDefinition, GeneratedAfflictionBuild, ItemSideData, LoadedPack, LoadedRecord,
    ManifestPack, MetricRow, MetricValue, NormalizedTime, RecordAlias, RecordReferenceIndex,
    ReferenceCandidate, ReferenceEdge, RemasterLink, SourceLoad, SpellSideData, VariantCandidate,
    VariantDiagnosticSource,
};

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("source root is unavailable: {0}")]
    SourceUnavailable(String),
    #[error("source manifest failed to parse: {0}")]
    ManifestParseFailed(String),
    #[error("source record failed to parse: {0}")]
    RecordParseFailed(String),
    #[error("record normalization failed for {path}: {message}")]
    RecordNormalizationFailed { path: String, message: String },
    #[error("source contains no loadable Foundry records")]
    NoRecordsLoaded,
    #[error("artifact write failed: {0}")]
    ArtifactWriteFailed(String),
}

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    let source =
        pipeline::load_foundry_source(&options.source_root, options.manifest_path.as_deref())?;
    if source.records.is_empty() {
        return Err(IngestError::NoRecordsLoaded);
    }
    writer::write_artifact(&options.output_path, &source)?;
    Ok(BuildArtifactReport {
        output_path: options.output_path,
        pack_count: source.packs.len(),
        record_count: source.records.len(),
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}
