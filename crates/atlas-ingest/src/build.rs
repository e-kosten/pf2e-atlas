use std::time::Instant;

use tracing::info;

use atlas_index::{IndexArtifactWriter, LadybugIndexWriter, SqliteIndexWriter};

use crate::artifact_manifest::{
    ArtifactManifest, ArtifactManifestInput, adjacent_artifact_manifest_path,
    compute_source_position_report, write_artifact_manifest,
};
use crate::embeddings::generation::generate_document_embeddings_for_source;
use crate::error::IngestError;
use crate::index_build_input::index_build_input;
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

    let embedding_model = options.embedding_model()?;
    let index_input = index_build_input(&source);
    let mut artifact_outputs = artifact_outputs_for_options(&options);
    for output in &mut artifact_outputs {
        info!(
            backend = output.label(),
            output = %output.output_path().display(),
            "writing artifact output"
        );
        output
            .write(&index_input, embedding_model)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    let artifact_record_count = source.records.len();
    let source_record_count = source.source_record_count;
    let generated_record_count = artifact_record_count - source_record_count;
    let document_embedding_count = source.document_embeddings.len();
    let source_signature = source.source_signature.clone();
    let source_position =
        compute_source_position_report(&options.source_root, options.manifest_path.as_deref());
    let manifest = ArtifactManifest::new(ArtifactManifestInput {
        source_root: options.source_root.clone(),
        source_signature: source_signature.clone(),
        source_record_count,
        artifact_record_count,
        generated_record_count,
        document_embedding_count,
        embedding_model: options.embedding_model_id.clone(),
        source_position,
    });
    write_artifact_manifest(
        &adjacent_artifact_manifest_path(&options.output_path),
        &manifest,
    )?;
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
        generated_record_count,
        pending_document_embedding_count: source.pending_document_embeddings.len(),
        document_embedding_count,
        reused_document_embedding_count: embedding_report.reused_count,
        generated_document_embedding_count: embedding_report.generated_count,
        document_embedding_tokenization:
            DocumentEmbeddingTokenizationReport::from_embedding_telemetry(
                source.document_embedding_tokenization,
            ),
        embedding_timing: embedding_report.timing,
        build_duration_ms,
        source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}

fn artifact_outputs_for_options(
    options: &BuildArtifactOptions,
) -> Vec<Box<dyn IndexArtifactWriter>> {
    let mut outputs: Vec<Box<dyn IndexArtifactWriter>> = vec![Box::new(SqliteIndexWriter::new(
        options.output_path.clone(),
    ))];
    if let Some(ladybug_output_path) = &options.ladybug_output_path {
        outputs.push(Box::new(LadybugIndexWriter::new(
            ladybug_output_path.clone(),
        )));
    }
    outputs
}
