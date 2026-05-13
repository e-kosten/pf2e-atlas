use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tracing::info;

mod embeddings;
mod labels;
mod metadata;
mod metric_catalogs;
mod packs;
mod records;
mod relationships;

use embeddings::write_document_embedding_cache;
use metadata::write_artifact_metadata;
use metric_catalogs::write_metric_catalogs;
use packs::write_packs;
use records::write_records;
use relationships::{write_record_aliases, write_reference_edges, write_remaster_links};

use crate::{IngestError, SourceLoad, schema};

pub(crate) fn write_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
    info!(output = %path.display(), "preparing artifact output");
    let output = ArtifactOutput::prepare(path)?;

    let mut connection = Connection::open(output.temp_path())
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let transaction = connection
        .transaction()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    info!("creating artifact schema");
    schema::create_artifact_schema(&transaction)?;
    info!("writing artifact metadata");
    write_artifact_metadata(
        &transaction,
        source.source_record_count,
        source.records.len(),
        source.records.len() - source.source_record_count,
        &source.source_signature,
    )?;
    info!(packs = source.packs.len(), "writing packs");
    write_packs(&transaction, &source.packs)?;
    info!(records = source.records.len(), "writing records");
    write_records(&transaction, &source.records, &source.remaster_links)?;
    info!(
        reference_edges = source.references.len(),
        "writing reference edges"
    );
    write_reference_edges(&transaction, &source.references)?;
    info!(aliases = source.aliases.len(), "writing record aliases");
    write_record_aliases(&transaction, &source.aliases)?;
    info!(
        remaster_links = source.remaster_links.len(),
        "writing remaster links"
    );
    write_remaster_links(&transaction, &source.remaster_links)?;
    info!(
        document_embeddings = source.document_embeddings.len(),
        "writing document embedding cache"
    );
    write_document_embedding_cache(&transaction, &source.document_embeddings)?;
    info!("writing metric catalogs");
    write_metric_catalogs(&transaction)?;
    info!("committing artifact");
    transaction
        .commit()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    drop(connection);
    output.commit()
}

struct ArtifactOutput {
    target_path: PathBuf,
    temp_path: PathBuf,
}

impl ArtifactOutput {
    fn prepare(target_path: &Path) -> Result<Self, IngestError> {
        if let Some(parent) = target_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
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

    fn commit(self) -> Result<(), IngestError> {
        remove_sqlite_files(&self.target_path)?;
        move_sqlite_files(&self.temp_path, &self.target_path)
    }
}

impl Drop for ArtifactOutput {
    fn drop(&mut self) {
        let _ = remove_sqlite_files(&self.temp_path);
    }
}

fn temp_artifact_path(target_path: &Path) -> Result<PathBuf, IngestError> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = target_path
        .file_name()
        .ok_or_else(|| {
            IngestError::ArtifactWriteFailed("artifact output path has no file name".to_string())
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

fn remove_sqlite_files(path: &Path) -> Result<(), IngestError> {
    for sqlite_path in sqlite_paths(path) {
        match fs::remove_file(&sqlite_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(IngestError::ArtifactWriteFailed(error.to_string())),
        }
    }
    Ok(())
}

fn move_sqlite_files(source_path: &Path, target_path: &Path) -> Result<(), IngestError> {
    let source_paths = sqlite_paths(source_path);
    let target_paths = sqlite_paths(target_path);

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
