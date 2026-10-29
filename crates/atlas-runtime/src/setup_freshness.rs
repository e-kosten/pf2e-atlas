use crate::ResolvedAtlasPaths;
use atlas_index::SourceArtifactBuildContext;
use atlas_ingest::compute_source_fingerprint;
pub(crate) fn source_is_fresh(
    paths: &ResolvedAtlasPaths,
    manifest: Option<&std::path::Path>,
    context: &SourceArtifactBuildContext,
) -> Result<bool, String> {
    let started = std::time::Instant::now();
    let current =
        compute_source_fingerprint(&paths.source_root, manifest).map_err(|e| e.to_string());
    tracing::info!(target:"atlas_progress",phase="check_source_freshness",
        elapsed_seconds=started.elapsed().as_secs_f64(),success=current.is_ok(),"Checked source freshness");
    let current = current?;
    Ok(current.value == context.source_fingerprint
        && current.file_count == context.source_file_count)
}
