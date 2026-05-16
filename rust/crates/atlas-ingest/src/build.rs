use std::time::Instant;

use tracing::info;

use crate::artifact::writer;
use crate::embeddings::generation::generate_document_embeddings_for_source;
use crate::error::IngestError;
use crate::source::model::{
    BuildArtifactOptions, BuildArtifactReport, DocumentEmbeddingTokenizationReport,
};
use crate::source_pipeline;

pub(crate) fn build_artifact(
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let build_started_at = Instant::now();
    info!(
        source = %options.source_root.display(),
        output = %options.output_path.display(),
        "starting artifact build"
    );
    let mut source = source_pipeline::load_foundry_source(
        &options.source_root,
        options.manifest_path.as_deref(),
    )?;
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

    let embedding_report = generate_document_embeddings_for_source(&mut source, &options)?;

    info!(
        output = %options.output_path.display(),
        "writing artifact"
    );
    let embedding_model = options.embedding_model()?;
    writer::write_artifact(&options.output_path, &source, embedding_model)?;
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
        reused_document_embedding_count: embedding_report.reused_count,
        generated_document_embedding_count: embedding_report.generated_count,
        document_embedding_tokenization:
            DocumentEmbeddingTokenizationReport::from_embedding_telemetry(
                source.document_embedding_tokenization,
            ),
        embedding_timing: embedding_report.timing,
        build_duration_ms,
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}
