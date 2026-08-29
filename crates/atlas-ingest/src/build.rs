use std::path::{Path, PathBuf};
use std::time::Instant;

use tracing::info;

#[cfg(all(test, feature = "record-round-trip-diagnostic"))]
use atlas_index::{ArtifactPublicationTelemetry, ArtifactReceiptTelemetry, IndexBuildInput};
use atlas_index::{IndexArtifactWriter, SqliteIndexWriter, publish_artifact_pair};

use crate::artifact_manifest::{
    ArtifactManifest, ArtifactManifestInput, adjacent_artifact_manifest_path,
    compute_source_position_report, write_artifact_manifest,
};
use crate::embeddings::generation::generate_document_embeddings_for_source;
use crate::error::IngestError;
use crate::index_build_input::index_build_input;
use crate::source::SourceLoad;
use crate::source::model::{
    BuildArtifactOptions, BuildArtifactReport, DocumentEmbeddingTokenizationReport,
};
use crate::source_pipeline;

pub(crate) struct BuildArtifactValidationOutcome {
    pub(crate) report: BuildArtifactReport,
    #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
    pub(crate) index_input: IndexBuildInput,
    #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
    pub(crate) receipt_telemetry: ArtifactReceiptTelemetry,
    #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
    pub(crate) publication_telemetry: ArtifactPublicationTelemetry,
    #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
    pub(crate) artifact_sha256: String,
    #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
    pub(crate) manifest_stage_ms: u128,
}

pub(crate) fn build_artifact(
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let build_started_at = Instant::now();
    info!(
        source = %options.source_root.display(),
        output = %options.output_path.display(),
        "starting artifact build"
    );
    let source = source_pipeline::load_foundry_source(
        &options.source_root,
        options.manifest_path.as_deref(),
    )?;
    build_artifact_from_source_started(source, options, build_started_at)
        .map(|outcome| outcome.report)
}

pub(crate) fn build_artifact_from_source(
    source: SourceLoad,
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    build_artifact_from_source_started(source, options, Instant::now())
        .map(|outcome| outcome.report)
}

#[cfg(all(test, feature = "record-round-trip-diagnostic"))]
pub(crate) fn build_artifact_from_source_for_validation(
    source: SourceLoad,
    options: BuildArtifactOptions,
) -> Result<BuildArtifactValidationOutcome, IngestError> {
    build_artifact_from_source_started(source, options, Instant::now())
}

fn build_artifact_from_source_started(
    mut source: SourceLoad,
    options: BuildArtifactOptions,
    build_started_at: Instant,
) -> Result<BuildArtifactValidationOutcome, IngestError> {
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

    let embedding_model = options.embedding_model()?;
    let document_embedding_tokenization =
        std::mem::take(&mut source.document_embedding_tokenization);
    let diagnostics = std::mem::take(&mut source.diagnostics);
    let skipped_records = std::mem::take(&mut source.skipped_records);
    let warnings = std::mem::take(&mut source.warnings);
    let index_input = index_build_input(source);
    let staged_artifact = staged_path(&options.output_path, "artifact");
    let staged_manifest = staged_path(&options.output_path, "manifest");
    let output = SqliteIndexWriter::new_for_publication(
        staged_artifact.clone(),
        options.output_path.clone(),
    );
    info!(
        backend = output.label(),
        output = %options.output_path.display(),
        "writing artifact output"
    );
    let receipt = output
        .write(&index_input, embedding_model)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let receipt_telemetry = receipt.telemetry().clone();
    let artifact_record_count = index_input.records.len();
    let source_record_count = index_input.source_record_count;
    let generated_record_count = artifact_record_count - source_record_count;
    let document_embedding_count = index_input.document_embeddings.len();
    let source_signature = index_input.source_signature.clone();
    let source_position =
        compute_source_position_report(&options.source_root, options.manifest_path.as_deref());
    let artifact_sha256 = receipt.artifact_sha256().to_string();
    let manifest_started = Instant::now();
    let manifest = ArtifactManifest::new(ArtifactManifestInput {
        source_root: options.source_root.clone(),
        source_signature: source_signature.clone(),
        source_record_count,
        artifact_record_count,
        generated_record_count,
        document_embedding_count,
        embedding_model: options.embedding_model_id.clone(),
        artifact_sha256: artifact_sha256.clone(),
        source_position,
    });
    write_artifact_manifest(&staged_manifest, &manifest)?;
    let manifest_stage_ms = manifest_started.elapsed().as_millis();
    let publication = publish_artifact_pair(
        receipt,
        &staged_manifest,
        &options.output_path,
        &adjacent_artifact_manifest_path(&options.output_path),
    )
    .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let build_duration_ms = build_started_at.elapsed().as_millis();
    info!(
        output = %options.output_path.display(),
        artifact_records = artifact_record_count,
        packs = index_input.packs.len(),
        document_embeddings = index_input.document_embeddings.len(),
        duration_ms = build_duration_ms,
        "artifact build complete"
    );
    info!(
        write_ms = receipt_telemetry.write_ms,
        deep_validation_ms = receipt_telemetry.deep_validation_ms,
        writer_digest_ms = receipt_telemetry.writer_digest_ms,
        manifest_stage_ms,
        lock_wait_ms = publication.lock_wait_ms,
        recovery_ms = publication.recovery_ms,
        generation_materialization_ms = publication.generation_materialization_ms,
        prior_pair_snapshot_ms = publication.prior_pair_snapshot_ms,
        pair_install_ms = publication.pair_install_ms,
        visible_pair_verification_ms = publication.visible_pair_verification_ms,
        cleanup_ms = publication.cleanup_ms,
        artifact_bytes = receipt_telemetry.artifact_bytes,
        writer_digest_bytes = receipt_telemetry.writer_digest_bytes,
        publication_sha_bytes = publication.publication_sha_bytes,
        generation_copy_bytes = publication.generation_copy_bytes,
        receipt_reuse_count = publication.receipt_reuse_count,
        "artifact publication receipt telemetry"
    );
    let report = BuildArtifactReport {
        output_path: options.output_path,
        pack_count: index_input.packs.len(),
        record_count: artifact_record_count,
        source_record_count,
        artifact_record_count,
        generated_record_count,
        pending_document_embedding_count: index_input.pending_document_embeddings.len(),
        document_embedding_count,
        reused_document_embedding_count: embedding_report.reused_count,
        generated_document_embedding_count: embedding_report.generated_count,
        document_embedding_tokenization:
            DocumentEmbeddingTokenizationReport::from_embedding_telemetry(
                document_embedding_tokenization,
            ),
        embedding_timing: embedding_report.timing,
        build_duration_ms,
        source_signature,
        diagnostics,
        skipped_records,
        warnings,
    };
    Ok(BuildArtifactValidationOutcome {
        report,
        #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
        index_input,
        #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
        receipt_telemetry,
        #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
        publication_telemetry: publication,
        #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
        artifact_sha256,
        #[cfg(all(test, feature = "record-round-trip-diagnostic"))]
        manifest_stage_ms,
    })
}

fn staged_path(target: &Path, kind: &str) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let file = target.file_name().unwrap_or_default().to_string_lossy();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    parent.join(format!(
        ".{file}.{kind}-{}-{nonce}.stage",
        std::process::id()
    ))
}
