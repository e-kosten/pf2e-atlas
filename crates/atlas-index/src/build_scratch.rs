//! Attempt-local files, protected by the workspace's exclusive update owner.
use crate::IndexError;
use std::path::{Path, PathBuf};

/// Declare this guard before all child streams so their handles close first.
pub struct BuildScratch {
    directory: tempfile::TempDir,
}

impl BuildScratch {
    pub fn new(owner: &atlas_storage::UpdateOwner) -> Result<Self, IndexError> {
        Self::with_parent_marker(owner, |marker| {
            std::fs::write(marker, b"atlas-build-scratch/v1\n")
        })
    }

    fn with_parent_marker(
        owner: &atlas_storage::UpdateOwner,
        write_marker: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> Result<Self, IndexError> {
        let mut name = owner.path().as_os_str().to_owned();
        name.push(".atlas-build-scratch");
        let parent = PathBuf::from(name);
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&parent) {
            Ok(()) => {
                // Until the marker is complete, only this call owns the new
                // directory. A partial marker must not strand a foreign-looking
                // parent that all future attempts would correctly reject.
                let mut initializing = InitializingParent {
                    path: &parent,
                    complete: false,
                };
                write_marker(&parent.join(".atlas-owned-scratch")).map_err(operation)?;
                initializing.complete = true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if !parent
                    .symlink_metadata()
                    .map_err(operation)?
                    .file_type()
                    .is_dir()
                    || !owned_marker(
                        &parent.join(".atlas-owned-scratch"),
                        b"atlas-build-scratch/v1\n",
                    )
                {
                    return Err(IndexError::Invalid(
                        "build scratch parent is not Atlas-owned".into(),
                    ));
                }
            }
            Err(error) => return Err(operation(error)),
        }
        // Update ownership excludes another live Atlas attempt for this workspace.
        // Only direct, marked directories are ours; leave all other entries alone.
        for entry in std::fs::read_dir(&parent).map_err(operation)? {
            let entry = entry.map_err(operation)?;
            if entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("attempt-"))
                && entry.file_type().map_err(operation)?.is_dir()
                && owned_marker(
                    &entry.path().join(".atlas-owned-attempt"),
                    b"atlas-build-attempt/v1\n",
                )
            {
                std::fs::remove_dir_all(entry.path()).map_err(operation)?;
            }
        }
        let directory = tempfile::Builder::new()
            .prefix("attempt-")
            .tempdir_in(parent)
            .map_err(operation)?;
        std::fs::write(
            directory.path().join(".atlas-owned-attempt"),
            b"atlas-build-attempt/v1\n",
        )
        .map_err(operation)?;
        Ok(Self { directory })
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}

struct InitializingParent<'a> {
    path: &'a Path,
    complete: bool,
}

impl Drop for InitializingParent<'_> {
    fn drop(&mut self) {
        if !self.complete {
            // These are the only two entries this initializer can have created.
            // Never recursively delete or clean a pre-existing directory.
            let _ = std::fs::remove_file(self.path.join(".atlas-owned-scratch"));
            let _ = std::fs::remove_dir(self.path);
        }
    }
}

fn owned_marker(path: &Path, expected: &[u8]) -> bool {
    path.symlink_metadata()
        .is_ok_and(|m| m.file_type().is_file() && m.len() == expected.len() as u64)
        && std::fs::read(path).is_ok_and(|bytes| bytes == expected)
}

fn operation(error: std::io::Error) -> IndexError {
    IndexError::Operation(format!("build scratch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_parent_marker_initialization_cleans_partial_write_and_allows_retry() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        let owner = workspace.update_owner().unwrap();
        let mut name = owner.path().as_os_str().to_owned();
        name.push(".atlas-build-scratch");
        let parent = PathBuf::from(name);
        let error = BuildScratch::with_parent_marker(&owner, |marker| {
            std::fs::write(marker, b"partial")?;
            Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "injected marker failure",
            ))
        });
        assert!(
            error
                .err()
                .unwrap()
                .to_string()
                .contains("injected marker failure")
        );
        assert!(!parent.exists());
        let retry = BuildScratch::new(&owner).unwrap();
        assert!(retry.path().exists());
        assert!(owned_marker(
            &parent.join(".atlas-owned-scratch"),
            b"atlas-build-scratch/v1\n"
        ));
    }

    #[test]
    fn distinct_workspace_extensions_cannot_clean_each_others_attempts() {
        let directory = tempfile::tempdir().unwrap();
        let a = atlas_storage::Workspace::at(directory.path().join("same.db")).unwrap();
        let b = atlas_storage::Workspace::at(directory.path().join("same.sqlite")).unwrap();
        let a_owner = a.update_owner().unwrap();
        let b_owner = b.update_owner().unwrap();
        let a_scratch = BuildScratch::new(&a_owner).unwrap();
        let sentinel = a_scratch.path().join("keep");
        std::fs::write(&sentinel, b"owned by a").unwrap();
        let b_scratch = BuildScratch::new(&b_owner).unwrap();
        assert_ne!(a_scratch.path().parent(), b_scratch.path().parent());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"owned by a");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                a_scratch
                    .path()
                    .parent()
                    .unwrap()
                    .metadata()
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o077,
                0
            );
        }
    }

    #[test]
    fn abandoned_cleanup_removes_only_marked_directories() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        let owner = workspace.update_owner().unwrap();
        let first = BuildScratch::new(&owner).unwrap();
        let parent = first.path().parent().unwrap().to_owned();
        drop(first);
        let abandoned = parent.join("attempt-abandoned");
        std::fs::create_dir(&abandoned).unwrap();
        std::fs::write(
            abandoned.join(".atlas-owned-attempt"),
            b"atlas-build-attempt/v1\n",
        )
        .unwrap();
        let foreign = parent.join("attempt-unmarked");
        std::fs::create_dir(&foreign).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&foreign, parent.join("attempt-symlink")).unwrap();
        let next = BuildScratch::new(&owner).unwrap();
        assert!(!abandoned.exists());
        assert!(foreign.exists());
        #[cfg(unix)]
        assert!(
            parent
                .join("attempt-symlink")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(next.path().exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_parent_is_rejected_without_touching_its_target() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        let owner = workspace.update_owner().unwrap();
        let foreign = directory.path().join("foreign");
        std::fs::create_dir(&foreign).unwrap();
        let sentinel = foreign.join("keep");
        std::fs::write(&sentinel, b"unchanged").unwrap();
        let mut name = workspace.path().as_os_str().to_owned();
        name.push(".atlas-build-scratch");
        std::os::unix::fs::symlink(&foreign, std::path::PathBuf::from(name)).unwrap();
        assert!(BuildScratch::new(&owner).is_err());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"unchanged");
    }
}
