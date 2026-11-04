use atlas_index::SourceArtifactBuildContext;
use atlas_ingest::{BuildArtifactOptions, CapturedBuildInputs, IngestError};

pub(crate) fn capture_build_inputs(
    options: BuildArtifactOptions,
) -> Result<CapturedBuildInputs, IngestError> {
    let started = std::time::Instant::now();
    let captured = CapturedBuildInputs::capture(options);
    tracing::info!(target:"atlas_progress",phase="check_source_freshness",
        elapsed_seconds=started.elapsed().as_secs_f64(),success=captured.is_ok(),"Captured source inputs");
    captured
}
pub(crate) fn source_is_fresh(
    inputs: &CapturedBuildInputs,
    context: &SourceArtifactBuildContext,
) -> bool {
    let fingerprint = inputs.fingerprint();
    fingerprint.value == context.source_fingerprint
        && fingerprint.file_count == context.source_file_count
}
