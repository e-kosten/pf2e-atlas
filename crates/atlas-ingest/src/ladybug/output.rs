use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::info;

use crate::error::IngestError;

pub(crate) fn ladybug_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

pub(crate) fn ladybug_progress_message(phase: &'static str, message: impl std::fmt::Display) {
    info!(target: "atlas_progress", phase, "{}", message);
}

pub(crate) fn ladybug_search_index_progress(
    step: usize,
    total_steps: usize,
    message: &'static str,
) {
    ladybug_progress_message(
        "ladybug_search_indexes",
        format_args!("({step}/{total_steps}) {message}"),
    );
}

pub(crate) fn ladybug_write_error(error: impl std::fmt::Display) -> IngestError {
    IngestError::ArtifactWriteFailed(format!("LadybugDB write failed: {error}"))
}

pub(crate) struct LadybugOutput {
    target_path: PathBuf,
    temp_path: PathBuf,
    staging_path: PathBuf,
}

impl LadybugOutput {
    pub(crate) fn prepare(target_path: &Path) -> Result<Self, IngestError> {
        if let Some(parent) = target_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        let temp_path = temp_ladybug_path(target_path)?;
        remove_ladybug_files(&temp_path)?;
        Ok(Self {
            target_path: target_path.to_path_buf(),
            staging_path: path_with_suffix(&temp_path, ".parquet-staging"),
            temp_path,
        })
    }

    pub(crate) fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    pub(crate) fn staging_path(&self) -> &Path {
        &self.staging_path
    }

    pub(crate) fn commit(self) -> Result<(), IngestError> {
        remove_ladybug_files(&self.target_path)?;
        move_ladybug_files(&self.temp_path, &self.target_path)
    }
}

impl Drop for LadybugOutput {
    fn drop(&mut self) {
        let _ = remove_ladybug_files(&self.temp_path);
        let _ = remove_path_if_exists(&self.staging_path);
    }
}

fn temp_ladybug_path(target_path: &Path) -> Result<PathBuf, IngestError> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = target_path
        .file_name()
        .ok_or_else(|| {
            IngestError::ArtifactWriteFailed("LadybugDB output path has no file name".to_string())
        })?
        .to_string_lossy();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
        .as_nanos();
    Ok(parent.join(format!(
        "{file_name}.rebuild-{}-{timestamp}",
        std::process::id()
    )))
}

fn remove_path_if_exists(path: &Path) -> Result<(), IngestError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string())),
        Ok(_) => fs::remove_file(path)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(IngestError::ArtifactWriteFailed(error.to_string())),
    }
}

fn ladybug_paths(path: &Path) -> [PathBuf; 5] {
    [
        path.to_path_buf(),
        path_with_suffix(path, ".wal"),
        path_with_suffix(path, ".wal.checkpoint"),
        path_with_suffix(path, ".shadow"),
        path_with_suffix(path, ".tmp"),
    ]
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = OsString::from(path.as_os_str());
    value.push(suffix);
    PathBuf::from(value)
}

fn remove_ladybug_files(path: &Path) -> Result<(), IngestError> {
    for ladybug_path in ladybug_paths(path) {
        remove_path_if_exists(&ladybug_path)?;
    }
    Ok(())
}

fn move_ladybug_files(source_path: &Path, target_path: &Path) -> Result<(), IngestError> {
    let source_paths = ladybug_paths(source_path);
    let target_paths = ladybug_paths(target_path);

    fs::rename(&source_paths[0], &target_paths[0])
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for (source, target) in source_paths.iter().zip(target_paths.iter()).skip(1) {
        if source.exists() {
            fs::rename(source, target)
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
    }

    Ok(())
}
