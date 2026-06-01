use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{IndexArtifactWriter, IndexBuildInput};
use atlas_embedding::EmbeddingModelId;
use diesel::{Connection, SqliteConnection};
use tracing::info;

mod discovery_catalogs;
mod embeddings;
mod labels;
mod metadata;
mod metric_catalogs;
mod models;
mod packs;
mod records;
mod relationships;
mod schema;
mod vector_index;

use discovery_catalogs::write_discovery_catalogs;
use embeddings::write_document_embedding_cache;
use metadata::write_artifact_metadata;
use metric_catalogs::write_metric_catalogs;
use packs::write_packs;
use records::write_records;
use relationships::{
    write_record_aliases, write_reference_edges, write_reference_occurrences, write_remaster_links,
};
use vector_index::write_record_vector_index;

use crate::IndexWriteError;

pub struct SqliteIndexWriter {
    path: PathBuf,
}

impl SqliteIndexWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl IndexArtifactWriter for SqliteIndexWriter {
    fn label(&self) -> &'static str {
        "SQLite"
    }

    fn output_path(&self) -> &Path {
        &self.path
    }

    fn write(
        &self,
        input: &IndexBuildInput<'_>,
        embedding_model: EmbeddingModelId,
    ) -> Result<(), IndexWriteError> {
        write_artifact(&self.path, input, embedding_model)
    }
}

fn write_artifact(
    path: &Path,
    input: &IndexBuildInput<'_>,
    embedding_model: EmbeddingModelId,
) -> Result<(), IndexWriteError> {
    artifact_progress("artifact_write", "Preparing artifact output");
    info!(output = %path.display(), "preparing artifact output");
    let output = ArtifactOutput::prepare(path)?;

    if !input.document_embeddings.is_empty() {
        artifact_progress("artifact_write", "Loading sqlite vector extension");
        atlas_sqlite_vec::register_sqlite_vec_auto_extension()
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    let database_url = output.temp_path().to_string_lossy().into_owned();
    let mut connection = SqliteConnection::establish(&database_url)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    connection.transaction::<_, IndexWriteError, _>(|connection| {
        artifact_progress("artifact_write", "Creating artifact schema");
        info!("creating artifact schema");
        schema::create_artifact_schema(connection)?;
        artifact_progress("artifact_write", "Writing artifact metadata");
        info!("writing artifact metadata");
        write_artifact_metadata(
            connection,
            input.source_record_count,
            input.artifact_record_count(),
            input.generated_record_count()?,
            input.source_signature,
            embedding_model,
        )?;
        artifact_progress("artifact_write", "Writing packs");
        info!(packs = input.packs.len(), "writing packs");
        write_packs(connection, &input.packs)?;
        artifact_progress("artifact_write", "Writing records");
        info!(records = input.records.len(), "writing records");
        write_records(
            connection,
            &input.records,
            input.aliases,
            input.remaster_links,
        )?;
        artifact_progress("artifact_write", "Writing reference edges");
        info!(
            reference_edges = input.references.len(),
            "writing reference edges"
        );
        write_reference_edges(connection, input.references)?;
        artifact_progress("artifact_write", "Writing reference occurrences");
        info!(
            records = input.records.len(),
            "writing reference occurrences"
        );
        write_reference_occurrences(connection, &input.records)?;
        artifact_progress("artifact_write", "Writing record aliases");
        info!(aliases = input.aliases.len(), "writing record aliases");
        write_record_aliases(connection, input.aliases)?;
        artifact_progress("artifact_write", "Writing remaster links");
        info!(
            remaster_links = input.remaster_links.len(),
            "writing remaster links"
        );
        write_remaster_links(connection, input.remaster_links)?;
        artifact_progress("artifact_write", "Writing document embedding cache");
        info!(
            document_embeddings = input.document_embeddings.len(),
            "writing document embedding cache"
        );
        write_document_embedding_cache(connection, input.document_embeddings)?;
        if !input.document_embeddings.is_empty() {
            artifact_progress("artifact_write", "Writing record vector index");
            info!(
                document_embeddings = input.document_embeddings.len(),
                "writing record vector index"
            );
            write_record_vector_index(connection)?;
        }
        artifact_progress("artifact_write", "Writing metric catalogs");
        info!("writing metric catalogs");
        write_metric_catalogs(connection)?;
        artifact_progress("artifact_write", "Writing filter discovery catalogs");
        info!("writing filter discovery catalogs");
        write_discovery_catalogs(connection)?;
        artifact_progress("artifact_write", "Finalizing SQLite artifact tables");
        info!("committing SQLite artifact tables");
        Ok(())
    })?;
    drop(connection);

    artifact_progress("artifact_write", "Publishing artifact");
    info!("publishing artifact");
    output.commit()
}

fn artifact_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

struct ArtifactOutput {
    target_path: PathBuf,
    temp_path: PathBuf,
}

impl ArtifactOutput {
    fn prepare(target_path: &Path) -> Result<Self, IndexWriteError> {
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

    fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    fn commit(self) -> Result<(), IndexWriteError> {
        let backup_path = backup_artifact_path(&self.target_path)?;
        remove_sqlite_files(&backup_path)?;
        move_existing_sqlite_files(&self.target_path, &backup_path)?;
        match move_required_sqlite_files(&self.temp_path, &self.target_path) {
            Ok(()) => {
                remove_sqlite_files(&backup_path)?;
                Ok(())
            }
            Err(error) => {
                let _ = remove_sqlite_files(&self.target_path);
                match move_existing_sqlite_files(&backup_path, &self.target_path) {
                    Ok(()) => Err(error),
                    Err(restore_error) => Err(IndexWriteError::WriteFailed(format!(
                        "{error}; also failed to restore previous artifact: {restore_error}"
                    ))),
                }
            }
        }
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

fn backup_artifact_path(target_path: &Path) -> Result<PathBuf, IndexWriteError> {
    suffixed_artifact_path(target_path, "publish-backup")
}

fn suffixed_artifact_path(target_path: &Path, purpose: &str) -> Result<PathBuf, IndexWriteError> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = target_path
        .file_name()
        .ok_or_else(|| {
            IndexWriteError::WriteFailed("artifact output path has no file name".to_string())
        })?
        .to_string_lossy();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?
        .as_nanos();
    Ok(parent.join(format!(
        "{file_name}.{purpose}-{}-{timestamp}",
        std::process::id()
    )))
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

fn remove_sqlite_files(path: &Path) -> Result<(), IndexWriteError> {
    for sqlite_path in sqlite_paths(path) {
        match fs::remove_file(&sqlite_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(IndexWriteError::WriteFailed(error.to_string())),
        }
    }
    Ok(())
}

fn move_required_sqlite_files(
    source_path: &Path,
    target_path: &Path,
) -> Result<(), IndexWriteError> {
    let source_paths = sqlite_paths(source_path);
    let target_paths = sqlite_paths(target_path);

    fs::rename(&source_paths[0], &target_paths[0])
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;

    for (source, target) in source_paths.iter().zip(target_paths.iter()).skip(1) {
        if source.exists() {
            fs::rename(source, target)
                .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        }
    }

    Ok(())
}

fn move_existing_sqlite_files(
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

fn restore_moved_sqlite_files(moved: &mut Vec<(PathBuf, PathBuf)>) -> Result<(), std::io::Error> {
    while let Some((source, target)) = moved.pop() {
        if target.exists() {
            fs::rename(target, source)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_embedding::EmbeddingModelId;

    #[test]
    fn failed_artifact_write_preserves_existing_target_and_cleans_temp()
    -> Result<(), Box<dyn std::error::Error>> {
        let target_path = unique_temp_path("failed-artifact-write.sqlite");
        fs::write(&target_path, b"existing artifact")?;
        let input = IndexBuildInput {
            source_signature: "fixture",
            source_record_count: 1,
            packs: Vec::new(),
            records: Vec::new(),
            references: &[],
            aliases: &[],
            remaster_links: &[],
            pending_document_embeddings: &[],
            document_embeddings: &[],
        };

        let error = write_artifact(&target_path, &input, EmbeddingModelId::BgeSmallEnV15)
            .expect_err("invalid input should fail before publish");

        assert!(matches!(error, IndexWriteError::InvalidInput(_)));
        assert_eq!(fs::read(&target_path)?, b"existing artifact");
        let parent = target_path.parent().expect("temp file should have parent");
        let file_name = target_path
            .file_name()
            .expect("temp file should have file name")
            .to_string_lossy();
        let leftovers = fs::read_dir(parent)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{file_name}.rebuild-"))
            })
            .collect::<Vec<_>>();
        assert!(leftovers.is_empty());

        let _ = fs::remove_file(&target_path);
        Ok(())
    }

    #[test]
    fn failed_backup_move_restores_partially_moved_target() -> Result<(), Box<dyn std::error::Error>>
    {
        let target_path = unique_temp_path("backup-source.sqlite");
        let backup_path = unique_temp_path("backup-target.sqlite");
        fs::write(&target_path, b"main")?;
        fs::write(sqlite_paths(&target_path)[1].as_path(), b"wal")?;
        fs::create_dir(sqlite_paths(&backup_path)[1].as_path())?;

        let error = move_existing_sqlite_files(&target_path, &backup_path)
            .expect_err("backup should fail after moving the main database");

        assert!(error.to_string().contains("Is a directory"));
        assert_eq!(fs::read(&target_path)?, b"main");
        assert_eq!(fs::read(sqlite_paths(&target_path)[1].as_path())?, b"wal");
        assert!(!backup_path.exists());

        let _ = fs::remove_file(&target_path);
        let _ = fs::remove_file(sqlite_paths(&target_path)[1].as_path());
        let _ = fs::remove_dir(sqlite_paths(&backup_path)[1].as_path());
        Ok(())
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "atlas-index-{name}-{}-{timestamp}",
            std::process::id()
        ))
    }
}
