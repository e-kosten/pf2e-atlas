use std::time::Instant;

use atlas_embedding::{
    EmbeddingRuntimeConfig, TextEmbedder, TextEmbeddingTokenizer,
    apply_document_embedding_token_budget, generate_document_embeddings_with_reuse_using_batch,
};
use tracing::{debug, info};

use crate::embedding_reuse;
use crate::error::IngestError;
use crate::source::SourceLoad;
use crate::source::model::{BuildArtifactOptions, EmbeddingTimingReport};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocumentEmbeddingGenerationReport {
    pub(crate) reused_count: usize,
    pub(crate) generated_count: usize,
    pub(crate) timing: EmbeddingTimingReport,
}

pub(crate) fn generate_document_embeddings_for_source(
    source: &mut SourceLoad,
    options: &BuildArtifactOptions,
) -> Result<DocumentEmbeddingGenerationReport, IngestError> {
    let Some(cache_root) = options.embedding_cache_root.clone() else {
        return Ok(DocumentEmbeddingGenerationReport::default());
    };

    let config = EmbeddingRuntimeConfig::new(options.embedding_model()?, cache_root);
    embedding_progress(
        "document_embeddings",
        "Checking reusable document embeddings",
    );
    let reusable_embeddings = if options.reuse_embeddings && options.output_path.exists() {
        match embedding_reuse::load_reusable_document_embeddings(&options.output_path, &config) {
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

    let mut timing = EmbeddingTimingReport::default();
    embedding_progress("document_embeddings", "Loading embedding tokenizer");
    let tokenizer = TextEmbeddingTokenizer::load(&config)
        .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
    embedding_progress("document_embeddings", "Applying document token budget");
    let tokenization_started_at = Instant::now();
    source.document_embedding_tokenization =
        apply_document_embedding_token_budget(&mut source.pending_document_embeddings, &tokenizer)
            .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
    timing.tokenization_duration_ms = tokenization_started_at.elapsed().as_millis();
    debug!(
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
        debug!(
            record_key = %example.record_key,
            embedding_tokens = example.token_count,
            max_embedding_tokens = example.max_token_count,
            "document embedding input exceeds tokenizer limit"
        );
    }
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
        "document embedding tokenization complete"
    );

    embedding_progress("document_embeddings", "Generating document embeddings");
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
                embedding_progress("document_embeddings", "Loading embedding model");
                info!(
                    cache = %config.cache_root.display(),
                    "loading embedding model"
                );
                let load_started_at = Instant::now();
                embedder = Some(TextEmbedder::load(&config)?);
                timing.model_load_duration_ms = load_started_at.elapsed().as_millis();
            }
            let batch_started_at = Instant::now();
            let vectors = match embedder.as_mut() {
                Some(embedder) => embedder.embed_documents(inputs)?,
                None => {
                    return Err(atlas_embedding::EmbeddingError::ModelRunFailed(
                        "embedder was not initialized".to_string(),
                    ));
                }
            };
            batch_durations_ms.push(batch_started_at.elapsed().as_millis());
            Ok::<Vec<Vec<f32>>, atlas_embedding::EmbeddingError>(vectors)
        },
    )
    .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?;
    timing.generation_duration_ms = generation_started_at.elapsed().as_millis();
    apply_batch_timing(&mut timing, batch_durations_ms);
    let reused_count = generated.reused_count;
    let generated_count = generated.generated_count;
    source.document_embeddings = generated.embeddings;
    info!(
        document_embeddings = source.document_embeddings.len(),
        reused_document_embeddings = reused_count,
        generated_document_embeddings = generated_count,
        "generated document embeddings"
    );

    Ok(DocumentEmbeddingGenerationReport {
        reused_count,
        generated_count,
        timing,
    })
}

fn embedding_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
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
