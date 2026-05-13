#![deny(unsafe_code)]

use thiserror::Error;
use tracing::info;

use atlas_embedding::{EmbeddingRuntimeConfig, TextEmbedder};

mod aliases;
mod embeddings;
mod generated_affliction_identity;
mod generated_affliction_model;
mod generated_afflictions;
mod metrics;
mod model;
mod normalize;
mod pipeline;
mod record_model;
mod references;
pub mod report;
mod schema;
mod side_data;
mod source;
mod source_model;
mod variant_taxonomy;
mod variants;
mod writer;

pub use embeddings::{
    GeneratedDocumentEmbedding, PendingDocumentEmbedding, generate_document_embeddings,
};
pub use model::IngestDiagnostics;
pub use report::analyze_foundry_source;
pub use source_model::{BuildArtifactOptions, BuildArtifactReport, SkippedRecord};

pub(crate) use generated_affliction_model::{
    AfflictionFamily, AfflictionOccurrence, DerivedAfflictionRecordInput, GeneratedAfflictionBuild,
};
pub(crate) use model::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME, FolderDefinition,
    VariantCandidate, VariantDiagnosticSource,
};
pub(crate) use record_model::{
    ActorSideData, AliasSource, ItemSideData, LoadedRecord, MetricRow, MetricValue, NormalizedTime,
    RecordAlias, RecordReferenceIndex, ReferenceCandidate, ReferenceEdge, RemasterLink,
    SpellSideData,
};
pub(crate) use source_model::{LoadedPack, ManifestPack, ParsedManifest, SourceLoad};

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
    #[error("document embedding generation failed: {0}")]
    DocumentEmbeddingFailed(String),
}

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    info!(
        source = %options.source_root.display(),
        output = %options.output_path.display(),
        "starting artifact build"
    );
    let mut source =
        pipeline::load_foundry_source(&options.source_root, options.manifest_path.as_deref())?;
    info!(
        packs = source.packs.len(),
        source_records = source.source_record_count,
        artifact_records = source.records.len(),
        pending_document_embeddings = source.pending_document_embeddings.len(),
        "loaded and normalized source"
    );
    if source.records.is_empty() {
        return Err(IngestError::NoRecordsLoaded);
    }
    if let Some(cache_root) = options.embedding_cache_root {
        info!(
            cache = %cache_root.display(),
            pending_document_embeddings = source.pending_document_embeddings.len(),
            "loading embedding model"
        );
        let config = EmbeddingRuntimeConfig::default_model(cache_root);
        let mut embedder = TextEmbedder::load(&config)
            .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        info!(
            pending_document_embeddings = source.pending_document_embeddings.len(),
            "generating document embeddings"
        );
        source.document_embeddings =
            generate_document_embeddings(&source.pending_document_embeddings, &mut embedder)
                .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        info!(
            document_embeddings = source.document_embeddings.len(),
            "generated document embeddings"
        );
    }
    info!(
        output = %options.output_path.display(),
        "writing artifact"
    );
    writer::write_artifact(&options.output_path, &source)?;
    let artifact_record_count = source.records.len();
    let source_record_count = source.source_record_count;
    info!(
        output = %options.output_path.display(),
        artifact_records = artifact_record_count,
        packs = source.packs.len(),
        document_embeddings = source.document_embeddings.len(),
        "artifact build complete"
    );
    Ok(BuildArtifactReport {
        output_path: options.output_path,
        pack_count: source.packs.len(),
        record_count: artifact_record_count,
        source_record_count,
        artifact_record_count,
        generated_record_count: artifact_record_count - source_record_count,
        pending_document_embedding_count: source.pending_document_embeddings.len(),
        document_embedding_count: source.document_embeddings.len(),
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}
