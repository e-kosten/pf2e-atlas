#![deny(unsafe_code)]

use std::time::Instant;

use thiserror::Error;
use tracing::info;

use atlas_embedding::{EmbeddingRuntimeConfig, TextEmbedder, TextEmbeddingTokenizer};

mod aliases;
mod embedding_reuse;
mod embedding_units;
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
    DocumentEmbeddingSectionTruncation, DocumentEmbeddingTokenizationTelemetry,
    DocumentEmbeddingTruncationExample, EmbeddingUnitKind, GeneratedDocumentEmbedding,
    PendingDocumentEmbedding, ReusableDocumentEmbedding, apply_document_embedding_token_budget,
    generate_document_embeddings, generate_document_embeddings_with_reuse,
    generate_document_embeddings_with_reuse_using,
    generate_document_embeddings_with_reuse_using_batch,
};
pub use model::{DroppedInlineMacroDiagnostic, IngestDiagnostics};
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
use source_model::EmbeddingTimingReport;
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
    let build_started_at = Instant::now();
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
    let mut embedding_timing = EmbeddingTimingReport::default();
    if let Some(cache_root) = options.embedding_cache_root {
        let config = EmbeddingRuntimeConfig::new(options.embedding_model, cache_root);
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
        let tokenizer = TextEmbeddingTokenizer::load(&config)
            .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        let tokenization_started_at = Instant::now();
        source.document_embedding_tokenization = apply_document_embedding_token_budget(
            &mut source.pending_document_embeddings,
            &tokenizer,
        )
        .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        embedding_timing.tokenization_duration_ms = tokenization_started_at.elapsed().as_millis();
        info!(
            document_embeddings = source.document_embedding_tokenization.document_count,
            truncated_document_embeddings = source
                .document_embedding_tokenization
                .truncated_document_count,
            max_embedding_tokens = source
                .document_embedding_tokenization
                .max_token_count
                .unwrap_or(0),
            max_observed_embedding_tokens = source
                .document_embedding_tokenization
                .max_observed_token_count,
            "analyzed document embedding tokenization"
        );
        for example in &source.document_embedding_tokenization.truncated_examples {
            info!(
                record_key = %example.record_key,
                embedding_tokens = example.token_count,
                max_embedding_tokens = example.max_token_count,
                "document embedding input exceeds tokenizer limit"
            );
        }
        info!(
            pending_document_embeddings = source.pending_document_embeddings.len(),
            "generating document embeddings"
        );
        let mut embedder = None;
        let generation_started_at = Instant::now();
        let mut batch_durations_ms = Vec::new();
        let generated = generate_document_embeddings_with_reuse_using_batch(
            &source.pending_document_embeddings,
            reusable_embeddings.as_ref(),
            options.embedding_batch_size,
            |inputs| {
                if embedder.is_none() {
                    info!(
                        cache = %config.cache_root.display(),
                        "loading embedding model"
                    );
                    let load_started_at = Instant::now();
                    embedder = Some(TextEmbedder::load(&config)?);
                    embedding_timing.model_load_duration_ms = load_started_at.elapsed().as_millis();
                }
                let batch_started_at = Instant::now();
                let vectors = embedder
                    .as_mut()
                    .expect("embedder was initialized")
                    .embed_documents(inputs)?;
                batch_durations_ms.push(batch_started_at.elapsed().as_millis());
                Ok::<Vec<Vec<f32>>, atlas_embedding::EmbeddingError>(vectors)
            },
        )
        .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
        embedding_timing.generation_duration_ms = generation_started_at.elapsed().as_millis();
        apply_batch_timing(&mut embedding_timing, batch_durations_ms);
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
    writer::write_artifact(&options.output_path, &source, options.embedding_model)?;
    let artifact_record_count = source.records.len();
    let source_record_count = source.source_record_count;
    let build_duration_ms = build_started_at.elapsed().as_millis();
    info!(
        output = %options.output_path.display(),
        artifact_records = artifact_record_count,
        packs = source.packs.len(),
        document_embeddings = source.document_embeddings.len(),
        duration_ms = build_duration_ms,
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
        document_embedding_tokenization: source.document_embedding_tokenization,
        embedding_timing,
        build_duration_ms,
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}

fn apply_batch_timing(report: &mut EmbeddingTimingReport, mut batch_durations_ms: Vec<u128>) {
    report.batch_count = batch_durations_ms.len();
    if batch_durations_ms.is_empty() {
        return;
    }
    batch_durations_ms.sort_unstable();
    report.batch_duration_min_ms = batch_durations_ms.first().copied();
    report.batch_duration_p50_ms = percentile(&batch_durations_ms, 50);
    report.batch_duration_p95_ms = percentile(&batch_durations_ms, 95);
    report.batch_duration_max_ms = batch_durations_ms.last().copied();
}

fn percentile(sorted: &[u128], percentile: usize) -> Option<u128> {
    if sorted.is_empty() {
        return None;
    }
    let index = ((sorted.len() - 1) * percentile).div_ceil(100);
    sorted.get(index).copied()
}
