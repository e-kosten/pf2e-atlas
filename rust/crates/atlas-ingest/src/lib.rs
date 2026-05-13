#![deny(unsafe_code)]

pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::fs;
pub(crate) use std::path::Path;

pub(crate) use atlas_domain::{
    MetricDomain, PackName, RecordFamily, RecordId, RecordKey, RemasterLinkSource, TextStatus,
};
pub(crate) use serde_json::{Value, json};
use thiserror::Error;

mod aliases;
mod generated_afflictions;
mod metrics;
mod model;
mod normalize;
mod references;
pub mod report;
mod schema;
mod source;
mod variants;
mod writer;

pub use model::{
    ActorSideData, AliasSource, BuildArtifactOptions, BuildArtifactReport, IngestDiagnostics,
    ItemSideData, LoadedPack, LoadedRecord, MetricRow, MetricValue, NormalizedTime, RecordAlias,
    ReferenceCandidate, ReferenceEdge, RemasterLink, SkippedRecord, SourceLoad, SpellSideData,
};
pub use source::load_foundry_source;
pub use writer::{read_artifact_counts, write_artifact};

pub(crate) use model::{
    AfflictionFamily, AfflictionOccurrence, DERIVED_AFFLICTION_INSTANCES_PACK_LABEL,
    DERIVED_AFFLICTION_INSTANCES_PACK_NAME, DERIVED_AFFLICTIONS_PACK_LABEL,
    DERIVED_AFFLICTIONS_PACK_NAME, DerivedAfflictionRecordInput, FolderDefinition,
    GeneratedAfflictionBuild, ManifestPack, RecordReferenceIndex, VariantCandidate,
    VariantDiagnosticSource,
};
pub(crate) use normalize::{
    extract_damage_types, extract_disable_skills, extract_sense_types, extract_speed_types,
    extract_traits, normalize_text, normalized_pointer_string, parse_bulk_value,
    parse_hands_requirement, pointer_bool, pointer_string, string_array_at_pointer, string_field,
    strip_markup, typed_collection,
};
pub(crate) use references::extract_reference_candidates_from_text;
pub(crate) use references::{record_by_key, reference_pack_and_locator, resolve_record_key};
pub(crate) use writer::{alias_source_label, remaster_link_source_label};

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
    let source = load_foundry_source(&options.source_root, options.manifest_path.as_deref())?;
    if source.records.is_empty() {
        return Err(IngestError::NoRecordsLoaded);
    }
    write_artifact(&options.output_path, &source)?;
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
