#![deny(unsafe_code)]

use thiserror::Error;
use tracing::info;

use atlas_embedding::{EmbeddingRuntimeConfig, TextEmbedder};

mod aliases;
mod embedding_reuse;
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
    GeneratedDocumentEmbedding, PendingDocumentEmbedding, ReusableDocumentEmbedding,
    generate_document_embeddings, generate_document_embeddings_with_reuse,
    generate_document_embeddings_with_reuse_using,
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
    let mut reused_document_embedding_count = 0;
    let mut generated_document_embedding_count = 0;
    if let Some(cache_root) = options.embedding_cache_root {
        let config = EmbeddingRuntimeConfig::default_model(cache_root);
        let reusable_embeddings = if options.reuse_embeddings && options.output_path.exists() {
            match embedding_reuse::load_reusable_document_embeddings(&options.output_path, &config)
            {
                Ok(reusable_embeddings) => {
                    info!(
                        reusable_document_embeddings = reusable_embeddings.len(),
                        "loaded reusable document embeddings"
                    );
                    Some(reusable_embeddings)
                }
                Err(error) => {
                    info!(
                        reason = %error,
                        "document embedding reuse unavailable"
                    );
                    None
                }
            }
        } else {
            if options.reuse_embeddings {
                info!("document embedding reuse unavailable: existing output artifact not found");
            } else {
                info!("document embedding reuse disabled");
            }
            None
        };
        info!(
            pending_document_embeddings = source.pending_document_embeddings.len(),
            "generating document embeddings"
        );
        let mut embedder = None;
        let generated = generate_document_embeddings_with_reuse_using(
            &source.pending_document_embeddings,
            reusable_embeddings.as_ref(),
            |input| {
                if embedder.is_none() {
                    info!(
                        cache = %config.cache_root.display(),
                        "loading embedding model"
                    );
                    embedder = Some(TextEmbedder::load(&config)?);
                }
                embedder
                    .as_mut()
                    .expect("embedder was initialized")
                    .embed_document(input)
            },
        )
        .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        reused_document_embedding_count = generated.reused_count;
        generated_document_embedding_count = generated.generated_count;
        source.document_embeddings = generated.embeddings;
        info!(
            document_embeddings = source.document_embeddings.len(),
            reused_document_embeddings = reused_document_embedding_count,
            generated_document_embeddings = generated_document_embedding_count,
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
        reused_document_embedding_count,
        generated_document_embedding_count,
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}
