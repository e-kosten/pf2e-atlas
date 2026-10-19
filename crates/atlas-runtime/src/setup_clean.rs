use std::fs;
use std::path::{Component, Path, PathBuf};

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
            &paths.index_path,
        ));
    }
    if options.embedding_cache {
        targets.push(clean_directory_target(
            SetupCleanTargetKind::EmbeddingCache,
            &paths.embedding_cache_root,
            options.check,
            &paths.index_path,
        ));
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
    workspace: &Path,
) -> SetupCleanTarget {
    let display_path = path.display().to_string();
    if let Err(reason) = ensure_workspace_outside_target(path, workspace) {
        return SetupCleanTarget::with_reason(
            kind,
            SetupCleanTargetStatus::Failed,
            display_path,
            reason,
        );
    }
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

/// Protect both a configured workspace alias and its physical location. Checking
/// their parent directories also protects adjacent SQLite journals, backups, and
/// stable admission/publisher lock files without duplicating storage filenames.
fn ensure_workspace_outside_target(target: &Path, workspace: &Path) -> Result<(), String> {
    let resolve = |path: &Path| {
        resolve_without_creating(path)
            .map_err(|error| format!("cannot safely resolve {}: {error}", path.display()))
    };
    let target = resolve(target)?;
    let physical_workspace = resolve(workspace)?;
    let configured_parent = resolve(
        workspace
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    let physical_parent = physical_workspace
        .parent()
        .ok_or_else(|| "workspace has no parent directory".to_owned())?;
    if configured_parent.starts_with(&target)
        || physical_parent.starts_with(&target)
        || physical_workspace == target
    {
        return Err("cleanup target contains the workspace or its backup/lock files; choose a separate cache directory".into());
    }
    Ok(())
}

/// Canonicalize existing components in order (including symlinks before `..`),
/// retaining nonexistent suffixes without creating directories. A broken alias
/// or inaccessible component is an error, not evidence that cleanup is safe.
fn resolve_without_creating(path: &Path) -> std::io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            component => {
                resolved.push(component.as_os_str());
                match fs::symlink_metadata(&resolved) {
                    Ok(_) => resolved = fs::canonicalize(&resolved)?,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolvedPathMode;

    fn paths(root: &Path) -> ResolvedAtlasPaths {
        ResolvedAtlasPaths {
            mode: ResolvedPathMode::Global,
            repo_root: None,
            managed_source_root: root.join("managed-source"),
            source_root_explicit: true,
            source_root: root.join("source"),
            embedding_cache_root: root.join("models"),
            index_path: root.join("state/workspace.sqlite"),
        }
    }

    #[test]
    fn source_and_model_cleanup_refuse_workspace_containment_in_check_and_execution() {
        for in_source in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let mut paths = paths(dir.path());
            let target = if in_source {
                &paths.source_root
            } else {
                &paths.embedding_cache_root
            };
            fs::create_dir_all(target).unwrap();
            paths.index_path = target.join("workspace.sqlite");
            fs::write(&paths.index_path, "saved state").unwrap();
            let backup = target.join("workspace.sqlite.upgrade-backup.sqlite");
            fs::write(&backup, "backup").unwrap();
            for check in [true, false] {
                let report = clean_setup(
                    &paths,
                    RuntimeSetupCleanOptions {
                        source: in_source,
                        embedding_cache: !in_source,
                        check,
                    },
                );
                assert_eq!(report.targets[0].status, SetupCleanTargetStatus::Failed);
                assert_eq!(
                    fs::read_to_string(&paths.index_path).unwrap(),
                    "saved state"
                );
                assert_eq!(fs::read_to_string(&backup).unwrap(), "backup");
            }
            // Even if only sidecars remain, the directory cannot be removed.
            fs::remove_file(&paths.index_path).unwrap();
            let report = clean_setup(
                &paths,
                RuntimeSetupCleanOptions {
                    source: in_source,
                    embedding_cache: !in_source,
                    check: false,
                },
            );
            assert_eq!(report.targets[0].status, SetupCleanTargetStatus::Failed);
            assert!(backup.exists());
        }
    }

    #[test]
    fn unrelated_cache_cleanup_preserves_workspace_and_check_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths(dir.path());
        fs::create_dir_all(&paths.source_root).unwrap();
        fs::create_dir_all(&paths.embedding_cache_root).unwrap();
        let report = clean_setup(
            &paths,
            RuntimeSetupCleanOptions {
                source: true,
                embedding_cache: true,
                check: true,
            },
        );
        assert!(
            report
                .targets
                .iter()
                .all(|target| target.status == SetupCleanTargetStatus::Planned)
        );
        assert!(!paths.index_path.parent().unwrap().exists());
        fs::create_dir_all(paths.index_path.parent().unwrap()).unwrap();
        fs::write(&paths.index_path, "saved state").unwrap();
        let report = clean_setup(
            &paths,
            RuntimeSetupCleanOptions {
                source: true,
                embedding_cache: true,
                check: false,
            },
        );
        assert!(
            report
                .targets
                .iter()
                .all(|target| target.status == SetupCleanTargetStatus::Removed)
        );
        assert_eq!(fs::read_to_string(paths.index_path).unwrap(), "saved state");
    }

    #[test]
    fn relative_paths_and_missing_suffixes_are_resolved_without_creation() {
        let dir = tempfile::TempDir::new_in(std::env::current_dir().unwrap()).unwrap();
        let relative = dir
            .path()
            .strip_prefix(std::env::current_dir().unwrap())
            .unwrap();
        let target = relative.join("cache");
        let workspace = relative.join("cache/not-created/state.sqlite");
        assert!(ensure_workspace_outside_target(&target, &workspace).is_err());
        assert!(!dir.path().join("cache").exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_aliases_and_broken_aliases_fail_closed() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let other = dir.path().join("other");
        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&other).unwrap();
        let workspace = cache.join("workspace.sqlite");
        fs::write(&workspace, "saved").unwrap();
        let alias = dir.path().join("alias");
        symlink(&cache, &alias).unwrap();
        assert!(ensure_workspace_outside_target(&alias, &workspace).is_err());
        assert!(ensure_workspace_outside_target(&cache, &alias.join("workspace.sqlite")).is_err());
        let file_alias = other.join("workspace.sqlite");
        symlink(&workspace, &file_alias).unwrap();
        assert!(ensure_workspace_outside_target(&other, &file_alias).is_err());
        assert!(ensure_workspace_outside_target(&cache, &file_alias).is_err());
        let broken = dir.path().join("broken");
        symlink(dir.path().join("absent"), &broken).unwrap();
        assert!(ensure_workspace_outside_target(&other, &broken).is_err());
        assert!(ensure_workspace_outside_target(&broken, &workspace).is_err());
        assert_eq!(fs::read_to_string(workspace).unwrap(), "saved");
    }
}
