use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{IndexArtifactWriter, IndexBuildInput};
use atlas_embedding::EmbeddingModelId;
use rusqlite::Connection;
use tracing::info;

mod discovery_catalogs;
mod embeddings;
mod labels;
mod metadata;
mod metric_catalogs;
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
    let mut connection = Connection::open(output.temp_path())
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let transaction = connection
        .transaction()
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    artifact_progress("artifact_write", "Creating artifact schema");
    info!("creating artifact schema");
    schema::create_artifact_schema(&transaction)?;
    artifact_progress("artifact_write", "Writing artifact metadata");
    info!("writing artifact metadata");
    write_artifact_metadata(
        &transaction,
        input.source_record_count,
        input.artifact_record_count(),
        input.generated_record_count()?,
        input.source_signature,
        embedding_model,
    )?;
    artifact_progress("artifact_write", "Writing packs");
    info!(packs = input.packs.len(), "writing packs");
    write_packs(&transaction, &input.packs)?;
    artifact_progress("artifact_write", "Writing records");
    info!(records = input.records.len(), "writing records");
    write_records(
        &transaction,
        &input.records,
        input.aliases,
        input.remaster_links,
    )?;
    artifact_progress("artifact_write", "Writing reference edges");
    info!(
        reference_edges = input.references.len(),
        "writing reference edges"
    );
    write_reference_edges(&transaction, input.references)?;
    artifact_progress("artifact_write", "Writing reference occurrences");
    info!(
        records = input.records.len(),
        "writing reference occurrences"
    );
    write_reference_occurrences(&transaction, &input.records)?;
    artifact_progress("artifact_write", "Writing record aliases");
    info!(aliases = input.aliases.len(), "writing record aliases");
    write_record_aliases(&transaction, input.aliases)?;
    artifact_progress("artifact_write", "Writing remaster links");
    info!(
        remaster_links = input.remaster_links.len(),
        "writing remaster links"
    );
    write_remaster_links(&transaction, input.remaster_links)?;
    artifact_progress("artifact_write", "Writing document embedding cache");
    info!(
        document_embeddings = input.document_embeddings.len(),
        "writing document embedding cache"
    );
    write_document_embedding_cache(&transaction, input.document_embeddings)?;
    if !input.document_embeddings.is_empty() {
        artifact_progress("artifact_write", "Writing record vector index");
        info!(
            document_embeddings = input.document_embeddings.len(),
            "writing record vector index"
        );
        write_record_vector_index(&transaction)?;
    }
    artifact_progress("artifact_write", "Writing metric catalogs");
    info!("writing metric catalogs");
    write_metric_catalogs(&transaction)?;
    artifact_progress("artifact_write", "Writing filter discovery catalogs");
    info!("writing filter discovery catalogs");
    write_discovery_catalogs(&transaction)?;
    artifact_progress("artifact_write", "Finalizing SQLite artifact tables");
    info!("committing SQLite artifact tables");
    transaction
        .commit()
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
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
        remove_sqlite_files(&self.target_path)?;
        move_sqlite_files(&self.temp_path, &self.target_path)
    }
}

impl Drop for ArtifactOutput {
    fn drop(&mut self) {
        let _ = remove_sqlite_files(&self.temp_path);
    }
}

fn temp_artifact_path(target_path: &Path) -> Result<PathBuf, IndexWriteError> {
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
        "{file_name}.rebuild-{}-{timestamp}",
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

fn move_sqlite_files(source_path: &Path, target_path: &Path) -> Result<(), IndexWriteError> {
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
