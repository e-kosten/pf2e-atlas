use crate::ResolvedAtlasPaths;
use atlas_index::SourceArtifactBuildContext;
use atlas_ingest::compute_source_fingerprint;
pub(crate) fn source_is_fresh(
    paths: &ResolvedAtlasPaths,
    context: &SourceArtifactBuildContext,
) -> Result<bool, String> {
    let current =
        compute_source_fingerprint(&paths.source_root, None).map_err(|e| e.to_string())?;
    Ok(current.value == context.source_fingerprint
        && current.file_count == context.source_file_count)
}
