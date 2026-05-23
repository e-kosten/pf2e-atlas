use std::fs;
use std::path::Path;
use std::time::Instant;

use lbug::{Connection, Database, SystemConfig};

use crate::error::IngestError;
use crate::ladybug::embeddings::{LadybugEmbedding, ladybug_embeddings};
use crate::ladybug::nodes::{write_graph_node_parquet, write_pack_parquet};
use crate::ladybug::output::{
    LadybugOutput, ladybug_progress, ladybug_progress_message, ladybug_write_error,
};
use crate::ladybug::parquet::{copy_from_parquet, recreate_dir};
use crate::ladybug::relationships::write_graph_relationship_parquet;
use crate::ladybug::schema::{create_schema, create_search_indexes};
use crate::progress::elapsed_display;
use crate::source::SourceLoad;

pub(crate) fn write_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
    ladybug_progress("ladybug_write", "Preparing LadybugDB output");
    let started_at = Instant::now();
    let output = LadybugOutput::prepare(path)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!("Collecting LadybugDB embedding units"),
    );
    let embeddings = ladybug_embeddings(source)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Collected {} LadybugDB embedding units in {}",
            embeddings.len(),
            elapsed_display(started_at)
        ),
    );
    let dimensions = embeddings
        .first()
        .map_or(384, |embedding| embedding.dimensions);
    let database =
        Database::new(output.temp_path(), SystemConfig::default()).map_err(ladybug_write_error)?;
    let connection = Connection::new(&database).map_err(ladybug_write_error)?;

    ladybug_progress("ladybug_write", "Creating LadybugDB graph schema");
    create_schema(&connection, dimensions)?;
    ladybug_progress(
        "ladybug_write",
        "Writing LadybugDB graph Parquet staging files",
    );
    write_parquet_staging_and_copy(&connection, output.staging_path(), source, &embeddings)?;
    if std::env::var_os("ATLAS_LADYBUG_CREATE_SEARCH_INDEXES").is_some() {
        ladybug_progress("ladybug_write", "Creating LadybugDB search indexes");
        create_search_indexes(&connection, !embeddings.is_empty())?;
    }
    connection
        .query("CHECKPOINT;")
        .map_err(ladybug_write_error)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Checkpointed LadybugDB output in {}",
            elapsed_display(started_at)
        ),
    );

    drop(connection);
    drop(database);
    output.commit()
}

fn write_parquet_staging_and_copy(
    connection: &Connection<'_>,
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    let started_at = Instant::now();
    recreate_dir(staging_path)?;
    write_parquet_staging_files(staging_path, source, embeddings)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Wrote LadybugDB Parquet staging files in {}",
            elapsed_display(started_at)
        ),
    );

    for (table, file_name) in [
        ("Pack", "pack.parquet"),
        ("Record", "record.parquet"),
        ("SearchDocument", "search_document.parquet"),
        ("EmbeddingUnit", "embedding_unit.parquet"),
        ("ContentUnit", "content_unit.parquet"),
        ("EvidenceUnit", "evidence_unit.parquet"),
        ("Trait", "trait.parquet"),
        ("FilterValue", "filter_value.parquet"),
        ("Publication", "publication.parquet"),
        ("Alias", "alias.parquet"),
        ("Metric", "metric.parquet"),
        ("VariantGroup", "variant_group.parquet"),
        ("FROM_PACK", "from_pack.parquet"),
        ("PUBLISHED_IN", "published_in.parquet"),
        ("HAS_SEARCH_DOCUMENT", "has_search_document.parquet"),
        ("HAS_CONTENT_UNIT", "has_content_unit.parquet"),
        ("HAS_EVIDENCE_UNIT", "has_evidence_unit.parquet"),
        ("CONTENT_REFERENCES", "content_references.parquet"),
        ("EVIDENCE_REFERENCES", "evidence_references.parquet"),
        ("HAS_EVIDENCE_EMBEDDING", "has_evidence_embedding.parquet"),
        ("HAS_TRAIT", "has_trait.parquet"),
        ("HAS_FILTER_VALUE", "has_filter_value.parquet"),
        ("HAS_ALIAS", "has_alias.parquet"),
        ("HAS_METRIC", "has_metric.parquet"),
        ("HAS_EMBEDDING_UNIT", "has_embedding_unit.parquet"),
        ("REFERENCES", "references.parquet"),
        ("REMASTERED_BY", "remastered_by.parquet"),
        ("IN_VARIANT_GROUP", "in_variant_group.parquet"),
    ] {
        copy_from_parquet(connection, table, &staging_path.join(file_name))?;
    }

    fs::remove_dir_all(staging_path)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Copied LadybugDB Parquet staging files in {}",
            elapsed_display(started_at)
        ),
    );
    Ok(())
}

fn write_parquet_staging_files(
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    write_pack_parquet(staging_path, &source.packs)?;
    write_graph_node_parquet(staging_path, source, embeddings)?;
    write_graph_relationship_parquet(staging_path, source, embeddings)?;
    Ok(())
}
