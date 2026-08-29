use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::IndexWriteError;

pub(super) fn sqlite_database_url(path: &Path) -> Result<String, IndexWriteError> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        IndexWriteError::WriteFailed(format!(
            "SQLite artifact path is not valid UTF-8: {}",
            path.display()
        ))
    })
}

pub(super) fn sqlite_payload_path(
    path: &Path,
    field: &'static str,
) -> Result<String, IndexWriteError> {
    path.to_str().map(str::to_string).ok_or_else(|| {
        IndexWriteError::WriteFailed(format!(
            "{field} path is not valid UTF-8: {}",
            path.display()
        ))
    })
}

pub(super) struct ArtifactOutput {
    target_path: PathBuf,
    temp_path: PathBuf,
}

impl ArtifactOutput {
    pub(super) fn prepare(target_path: &Path) -> Result<Self, IndexWriteError> {
        if let Some(parent) = target_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        }

        let temp_path = temp_artifact_path(target_path)?;
        remove_sqlite_files(&temp_path)?;

        Ok(Self {
            target_path: target_path.to_path_buf(),
            temp_path,
        })
    }

    pub(super) fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    pub(super) fn commit(self) -> Result<(), IndexWriteError> {
        let temp_paths = sqlite_paths(&self.temp_path);
        if temp_paths[1].exists() || temp_paths[2].exists() {
            return Err(IndexWriteError::WriteFailed(
                "staged SQLite artifact still has WAL companions and cannot be atomically published"
                    .to_string(),
            ));
        }
        fs::File::open(&temp_paths[0])
            .and_then(|file| file.sync_all())
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let mut permissions = fs::metadata(&temp_paths[0])
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&temp_paths[0], permissions)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        fs::rename(&temp_paths[0], &self.target_path)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        fs::File::open(self.target_path.parent().unwrap_or_else(|| Path::new(".")))
            .and_then(|directory| directory.sync_all())
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
    }
}

impl Drop for ArtifactOutput {
    fn drop(&mut self) {
        let _ = remove_sqlite_files(&self.temp_path);
    }
}

fn temp_artifact_path(target_path: &Path) -> Result<PathBuf, IndexWriteError> {
    suffixed_artifact_path(target_path, "rebuild")
}

fn suffixed_artifact_path(target_path: &Path, purpose: &str) -> Result<PathBuf, IndexWriteError> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = target_path.file_name().ok_or_else(|| {
        IndexWriteError::WriteFailed("artifact output path has no file name".to_string())
    })?;
    let suffix = format!(
        ".{purpose}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
            .as_nanos()
    );
    let mut suffixed_name = OsString::from(file_name);
    suffixed_name.push(suffix);
    Ok(parent.join(suffixed_name))
}

pub(super) fn sqlite_paths(path: &Path) -> [PathBuf; 3] {
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

pub(super) fn remove_sqlite_files(path: &Path) -> Result<(), IndexWriteError> {
    for sqlite_path in sqlite_paths(path) {
        match fs::remove_file(&sqlite_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(IndexWriteError::WriteFailed(error.to_string())),
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn move_existing_sqlite_files(
    source_path: &Path,
    target_path: &Path,
) -> Result<(), IndexWriteError> {
    let source_paths = sqlite_paths(source_path);
    let target_paths = sqlite_paths(target_path);
    let mut moved = Vec::<(PathBuf, PathBuf)>::new();
    for (source, target) in source_paths.iter().zip(target_paths.iter()) {
        if source.exists() {
            if let Err(error) = fs::rename(source, target) {
                let restore_result = restore_moved_sqlite_files(&mut moved);
                return Err(match restore_result {
                    Ok(()) => IndexWriteError::WriteFailed(error.to_string()),
                    Err(restore_error) => IndexWriteError::WriteFailed(format!(
                        "{error}; also failed to restore partially moved artifact: {restore_error}"
                    )),
                });
            }
            moved.push((source.clone(), target.clone()));
        }
    }

    Ok(())
}

#[cfg(test)]
fn restore_moved_sqlite_files(moved: &mut Vec<(PathBuf, PathBuf)>) -> Result<(), std::io::Error> {
    while let Some((source, target)) = moved.pop() {
        if target.exists() {
            fs::rename(target, source)?;
        }
    }
    Ok(())
}
