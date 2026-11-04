//! Attempt-local files, protected by the workspace's exclusive update owner.
use crate::IndexError;
use std::{fs, path::Path};

/// Declare this guard before all child streams so their handles close first.
pub struct BuildScratch {
    directory: tempfile::TempDir,
}

impl BuildScratch {
    pub fn new(owner: &atlas_storage::UpdateOwner) -> Result<Self, IndexError> {
        Self::new_in(owner, &std::env::temp_dir())
    }

    fn new_in(owner: &atlas_storage::UpdateOwner, temp_root: &Path) -> Result<Self, IndexError> {
        let prefix = attempt_prefix(owner);
        let marker = format!(".{prefix}owned");
        reclaim_abandoned(temp_root, &prefix, &marker);
        let mut builder = tempfile::Builder::new();
        builder.prefix(&prefix);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(fs::Permissions::from_mode(0o700));
        }
        let directory = builder.tempdir_in(temp_root).map_err(operation)?;
        fs::File::create(directory.path().join(marker)).map_err(operation)?;
        Ok(Self { directory })
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}

fn attempt_prefix(owner: &atlas_storage::UpdateOwner) -> String {
    let path = owner.path().as_os_str();
    #[cfg(unix)]
    let workspace = {
        use std::os::unix::ffi::OsStrExt;
        crate::codec::sha256(path.as_bytes())
    };
    #[cfg(windows)]
    let workspace = {
        use std::os::windows::ffi::OsStrExt;
        let bytes: Vec<_> = path.encode_wide().flat_map(u16::to_le_bytes).collect();
        crate::codec::sha256(&bytes)
    };
    #[cfg(not(any(unix, windows)))]
    let workspace = crate::codec::sha256(path.as_encoded_bytes());
    format!("pf2e-atlas-build-v1-{workspace}-")
}

fn reclaim_abandoned(temp_root: &Path, prefix: &str, marker: &str) {
    let entries = match fs::read_dir(temp_root) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::debug!(?temp_root, %error, "Could not scan for abandoned build scratch");
            return;
        }
    };
    // The caller holds the update lock, which excludes other live attempts for this workspace.
    for entry in entries.flatten() {
        let matches_workspace = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(prefix));
        if !matches_workspace || !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let path = entry.path();
        if !path
            .join(marker)
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_file() && m.len() == 0)
        {
            continue;
        }
        if let Err(error) = fs::remove_dir_all(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(?path, %error, "Could not remove abandoned build scratch");
        }
    }
}

fn operation(error: std::io::Error) -> IndexError {
    IndexError::Operation(format!("build scratch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_workspace_extensions_cannot_clean_each_others_attempts() {
        let directory = tempfile::tempdir().unwrap();
        let a = atlas_storage::Workspace::at(directory.path().join("same.db")).unwrap();
        let b = atlas_storage::Workspace::at(directory.path().join("same.sqlite")).unwrap();
        let a_owner = a.update_owner().unwrap();
        let b_owner = b.update_owner().unwrap();
        let a_scratch = BuildScratch::new_in(&a_owner, directory.path()).unwrap();
        let sentinel = a_scratch.path().join("keep");
        fs::write(&sentinel, b"owned by a").unwrap();
        let b_scratch = BuildScratch::new_in(&b_owner, directory.path()).unwrap();
        assert_ne!(a_scratch.path(), b_scratch.path());
        assert_eq!(fs::read(sentinel).unwrap(), b"owned by a");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                a_scratch.path().metadata().unwrap().permissions().mode() & 0o077,
                0
            );
        }
    }

    #[test]
    fn next_attempt_removes_only_marked_directories_for_its_workspace() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        let owner = workspace.update_owner().unwrap();
        let first = BuildScratch::new_in(&owner, directory.path()).unwrap();
        let abandoned = first.directory.keep();
        drop(owner);

        let owner = workspace.update_owner().unwrap();
        let prefix = attempt_prefix(&owner);
        let unmarked = directory.path().join(format!("{prefix}unmarked"));
        fs::create_dir(&unmarked).unwrap();
        let wrong_marker = directory.path().join(format!("{prefix}wrong-marker"));
        fs::create_dir(&wrong_marker).unwrap();
        fs::File::create(wrong_marker.join(".another-workspace-owned")).unwrap();
        #[cfg(unix)]
        {
            let target = directory.path().join("symlink-target");
            fs::create_dir(&target).unwrap();
            fs::File::create(target.join(format!(".{prefix}owned"))).unwrap();
            std::os::unix::fs::symlink(&target, directory.path().join(format!("{prefix}symlink")))
                .unwrap();
        }

        let next = BuildScratch::new_in(&owner, directory.path()).unwrap();
        assert!(!abandoned.exists());
        assert!(unmarked.exists());
        assert!(wrong_marker.exists());
        #[cfg(unix)]
        assert!(
            directory
                .path()
                .join(format!("{prefix}symlink"))
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        let path = next.path().to_owned();
        assert!(path.is_dir());
        drop(next);
        assert!(!path.exists());
    }

    #[test]
    fn obsolete_scratch_parent_does_not_block_os_temp_attempt() {
        let directory = tempfile::tempdir().unwrap();
        let workspace =
            atlas_storage::Workspace::at(directory.path().join("workspace.sqlite")).unwrap();
        let owner = workspace.update_owner().unwrap();
        let stranded = directory
            .path()
            .join("workspace.sqlite.atlas-build-scratch");
        fs::create_dir(&stranded).unwrap();
        let scratch = BuildScratch::new(&owner).unwrap();
        assert_eq!(
            scratch.path().parent(),
            Some(std::env::temp_dir().as_path())
        );
        assert!(stranded.is_dir());
    }
}
