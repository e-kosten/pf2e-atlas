use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ResolvedAtlasPaths;
use crate::setup_model::{
    RuntimeSetupCleanOptions, RuntimeSetupCleanReport, SetupCleanTarget, SetupCleanTargetKind,
    SetupCleanTargetStatus,
};

pub(crate) fn clean_setup(
    paths: &ResolvedAtlasPaths,
    options: RuntimeSetupCleanOptions,
) -> RuntimeSetupCleanReport {
    let mut targets = Vec::new();
    if options.source {
        targets.push(clean_directory_target(
            SetupCleanTargetKind::Source,
            &paths.source_root,
            options.check,
        ));
    }
    if options.embedding_cache {
        targets.push(clean_directory_target(
            SetupCleanTargetKind::EmbeddingCache,
            &paths.embedding_cache_root,
            options.check,
        ));
    }
    if options.artifact {
        targets.push(clean_artifact_target(&paths.index_path, options.check));
    }

    RuntimeSetupCleanReport {
        path_mode: paths.mode.as_str(),
        repo_root: paths
            .repo_root
            .as_ref()
            .map(|path| path.display().to_string()),
        check: options.check,
        targets,
    }
}

fn clean_directory_target(
    kind: SetupCleanTargetKind,
    path: &Path,
    check: bool,
) -> SetupCleanTarget {
    let display_path = path.display().to_string();
    if !path.exists() {
        return SetupCleanTarget::with_reason(
            kind,
            SetupCleanTargetStatus::Skipped,
            display_path,
            "path does not exist",
        );
    }
    if check {
        return SetupCleanTarget::new(kind, SetupCleanTargetStatus::Planned, display_path);
    }
    match fs::remove_dir_all(path) {
        Ok(()) => SetupCleanTarget::new(kind, SetupCleanTargetStatus::Removed, display_path),
        Err(error) => SetupCleanTarget::with_reason(
            kind,
            SetupCleanTargetStatus::Failed,
            display_path,
            error.to_string(),
        ),
    }
}

fn clean_artifact_target(path: &Path, check: bool) -> SetupCleanTarget {
    let display_path = path.display().to_string();
    let existing_paths = sqlite_paths(path)
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if existing_paths.is_empty() {
        return SetupCleanTarget::with_reason(
            SetupCleanTargetKind::Artifact,
            SetupCleanTargetStatus::Skipped,
            display_path,
            "artifact files do not exist",
        );
    }
    if check {
        return SetupCleanTarget::new(
            SetupCleanTargetKind::Artifact,
            SetupCleanTargetStatus::Planned,
            display_path,
        );
    }
    for artifact_path in existing_paths {
        if let Err(error) = fs::remove_file(&artifact_path) {
            return SetupCleanTarget::with_reason(
                SetupCleanTargetKind::Artifact,
                SetupCleanTargetStatus::Failed,
                display_path,
                format!("failed to remove {}: {error}", artifact_path.display()),
            );
        }
    }
    SetupCleanTarget::new(
        SetupCleanTargetKind::Artifact,
        SetupCleanTargetStatus::Removed,
        display_path,
    )
}

fn sqlite_paths(path: &Path) -> [PathBuf; 3] {
    let mut wal_path = OsString::from(path.as_os_str());
    wal_path.push("-wal");
    let mut shm_path = OsString::from(path.as_os_str());
    shm_path.push("-shm");

    [
        path.to_path_buf(),
        PathBuf::from(wal_path),
        PathBuf::from(shm_path),
    ]
}
