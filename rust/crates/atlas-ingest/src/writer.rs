use std::fs;
use std::path::Path;

use rusqlite::Connection;

mod labels;
mod metadata;
mod metric_catalogs;
mod packs;
mod records;
mod relationships;

use metadata::write_artifact_metadata;
use metric_catalogs::write_metric_catalogs;
use packs::write_packs;
use records::write_records;
use relationships::{write_record_aliases, write_reference_edges, write_remaster_links};

use crate::{IngestError, SourceLoad, schema};

pub(crate) fn write_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
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
    schema::create_artifact_schema(&transaction)?;
    write_artifact_metadata(
        &transaction,
        source.source_record_count,
        source.records.len(),
        source.records.len() - source.source_record_count,
        &source.source_signature,
    )?;
    write_packs(&transaction, &source.packs)?;
    write_records(&transaction, &source.records, &source.remaster_links)?;
    write_reference_edges(&transaction, &source.references)?;
    write_record_aliases(&transaction, &source.aliases)?;
    write_remaster_links(&transaction, &source.remaster_links)?;
    write_metric_catalogs(&transaction)?;
    transaction
        .commit()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
