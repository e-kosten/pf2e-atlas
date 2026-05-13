use std::fs;
use std::path::Path;

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
    let _ = fs::remove_file(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }

    let mut connection = Connection::open(path)
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
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
