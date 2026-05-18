use std::path::{Path, PathBuf};

use atlas_index::ArtifactValidationReport;
use atlas_ingest::{
    ArtifactManifest, SourceFingerprint, compute_source_fingerprint, read_artifact_manifest,
    source_git_commit_if_clean,
};

use crate::ResolvedAtlasPaths;

pub(crate) enum ManifestFreshness {
    Fresh(String),
    Stale,
    Unavailable,
}

pub(crate) fn manifest_source_signature(
    paths: &ResolvedAtlasPaths,
    validation: &ArtifactValidationReport,
) -> ManifestFreshness {
    let Some(relative_manifest_path) = validation.adjacent_manifest_path.as_deref() else {
        return ManifestFreshness::Unavailable;
    };
    let manifest_path =
        match resolve_adjacent_manifest_path(&paths.index_path, relative_manifest_path) {
            Some(path) => path,
            None => return ManifestFreshness::Unavailable,
        };
    let manifest = match read_artifact_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(_) => return ManifestFreshness::Unavailable,
    };
    if !manifest_matches_artifact(&manifest, validation) {
        return ManifestFreshness::Unavailable;
    }

    if let Some(manifest_git_commit) = manifest.source.git_commit.as_deref() {
        let current_git_commit = match source_git_commit_if_clean(&paths.source_root) {
            Ok(commit) => commit,
            Err(_) => return ManifestFreshness::Unavailable,
        };
        return if manifest_git_commit == current_git_commit {
            ManifestFreshness::Fresh(manifest.source.signature)
        } else {
            ManifestFreshness::Stale
        };
    }

    if manifest
        .source
        .fingerprint
        .as_ref()
        .is_some_and(|manifest_fingerprint| {
            compute_source_fingerprint(&paths.source_root, None).is_ok_and(|current_fingerprint| {
                fingerprints_match(manifest_fingerprint, &current_fingerprint)
            })
        })
    {
        ManifestFreshness::Fresh(manifest.source.signature)
    } else {
        ManifestFreshness::Stale
    }
}

fn resolve_adjacent_manifest_path(
    artifact_path: &Path,
    relative_manifest_path: &str,
) -> Option<PathBuf> {
    let path = Path::new(relative_manifest_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(
        artifact_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path),
    )
}

fn manifest_matches_artifact(
    manifest: &ArtifactManifest,
    validation: &ArtifactValidationReport,
) -> bool {
    validation.source_signature.as_deref() == Some(manifest.source.signature.as_str())
        && optional_usize_matches(
            validation.source_record_count.as_deref(),
            manifest.source.record_count,
        )
        && optional_usize_matches(
            validation.artifact_record_count.as_deref(),
            manifest.build.artifact_record_count,
        )
        && optional_usize_matches(
            validation.generated_record_count.as_deref(),
            manifest.build.generated_record_count,
        )
}

fn fingerprints_match(left: &SourceFingerprint, right: &SourceFingerprint) -> bool {
    left.kind == right.kind && left.value == right.value
}

fn optional_usize_matches(value: Option<&str>, expected: usize) -> bool {
    value.and_then(|value| value.parse::<usize>().ok()) == Some(expected)
}
