use std::path::Path;

use atlas_artifact::schema::{
    TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX, record_vector_index_create_sql,
    record_vector_index_insert_sql,
};
use rusqlite::{Connection, OpenFlags};

use crate::contract::{contract_diagnostic, contract_diagnostic_with_code};
use crate::sql::{count_rows, count_sql, table_exists};
use crate::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus, validate_index,
};

pub type VectorExtensionLoader = dyn FnOnce(&Connection) -> Result<(), String>;

pub fn validate_vector_index_report(path: impl AsRef<Path>) -> ArtifactValidationReport {
    let path = path.as_ref();
    match validate_vector_index(path) {
        Ok(report) => report,
        Err(error) => crate::validation_report_from_error(path, error),
    }
}

pub fn write_vector_index_report(path: impl AsRef<Path>) -> ArtifactValidationReport {
    let path = path.as_ref();
    match write_vector_index(path) {
        Ok(report) => report,
        Err(error) => crate::validation_report_from_error(path, error),
    }
}

pub fn write_vector_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    write_vector_index_with_loader(path, |_| Ok(()))
}

pub fn write_vector_index_with_loader(
    path: impl AsRef<Path>,
    loader: impl FnOnce(&Connection) -> Result<(), String>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    let base_report = validate_index(path)?;
    if base_report.status != ValidationStatus::Ok {
        return Ok(base_report);
    }

    let index = path.display().to_string();
    let summary = metadata_summary_from_report(&base_report);
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if let Err(message) = loader(&connection).and_then(|()| probe_sqlite_vec(&connection)) {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    create_and_populate_vector_index(&connection, &summary)?;
    validate_vector_index_with_loaded_connection(index, summary, &connection)
}

pub fn validate_vector_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    validate_vector_index_with_loader(path, |_| Ok(()))
}

pub fn validate_vector_index_with_loader(
    path: impl AsRef<Path>,
    loader: impl FnOnce(&Connection) -> Result<(), String>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    let base_report = validate_index(path)?;
    if base_report.status != ValidationStatus::Ok {
        return Ok(base_report);
    }

    let index = path.display().to_string();
    let summary = metadata_summary_from_report(&base_report);
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if let Err(message) = loader(&connection).and_then(|()| probe_sqlite_vec(&connection)) {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    validate_vector_index_with_loaded_connection(index, summary, &connection)
}

fn validate_vector_index_with_loaded_connection(
    index: String,
    summary: ArtifactMetadataSummary,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let mut diagnostics = Vec::new();
    if !table_exists(connection, TABLE_RECORD_VECTOR_INDEX)? {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Schema,
            format!("required vector table `{TABLE_RECORD_VECTOR_INDEX}` is missing"),
            Some(format!("table:{TABLE_RECORD_VECTOR_INDEX}")),
            Some("present".to_string()),
            Some("missing".to_string()),
        ));
    } else {
        validate_vector_index_coverage(connection, &mut diagnostics)?;
    }

    if diagnostics.is_empty() {
        Ok(ArtifactValidationReport::ok(index, summary))
    } else {
        Ok(ArtifactValidationReport::incompatible_metadata(
            index,
            summary,
            diagnostics,
        ))
    }
}

fn vector_extension_unavailable_report(
    index: String,
    summary: ArtifactMetadataSummary,
    message: String,
) -> ArtifactValidationReport {
    ArtifactValidationReport::incompatible_metadata(
        index,
        summary,
        vec![contract_diagnostic_with_code(
            ArtifactContractFamily::Embedding,
            "sqlite-vec extension is unavailable for vector index operations".to_string(),
            Some("sqlite_vec".to_string()),
            Some("available".to_string()),
            Some(message),
            ValidationCode::VectorExtensionUnavailable,
        )],
    )
}

fn create_and_populate_vector_index(
    connection: &Connection,
    summary: &ArtifactMetadataSummary,
) -> Result<(), IndexValidationError> {
    let dimensions = summary
        .embedding_dimensions
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| {
            IndexValidationError::InvalidArtifact(
                "embedding_dimensions metadata is required to build record_vector_index"
                    .to_string(),
            )
        })?;

    connection
        .execute_batch(&format!(
            "DROP TABLE IF EXISTS {table};
             {create_sql}",
            table = TABLE_RECORD_VECTOR_INDEX,
            create_sql = record_vector_index_create_sql(dimensions)
        ))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;

    let insert_sql = record_vector_index_insert_sql();
    let mut insert = connection
        .prepare(&insert_sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut select = connection
        .prepare(
            "SELECT record_key, vector_blob
             FROM document_embedding_cache
             ORDER BY record_key",
        )
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = select
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    for row in rows {
        let (record_key, vector_blob) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        insert
            .execute((record_key.as_str(), vector_blob))
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    }

    Ok(())
}

fn probe_sqlite_vec(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.atlas_vec_capability_probe
             USING vec0(embedding FLOAT[1]);
             DROP TABLE temp.atlas_vec_capability_probe;",
        )
        .map_err(|error| error.to_string())
}

fn validate_vector_index_coverage(
    connection: &Connection,
    diagnostics: &mut Vec<crate::ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let vector_rows = count_rows(connection, TABLE_RECORD_VECTOR_INDEX)?;
    let cache_rows = count_rows(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)?;
    if vector_rows != cache_rows {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Embedding,
            "record vector index row count must match document embedding cache row count"
                .to_string(),
            Some("record_vector_index:document_embedding_cache_count".to_string()),
            Some(cache_rows.to_string()),
            Some(vector_rows.to_string()),
        ));
    }

    for (key, sql, expected) in [
        (
            "record_vector_index:missing_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM document_embedding_cache
               EXCEPT
               SELECT record_key FROM record_vector_index
             )",
            "every document embedding has a vector index row",
        ),
        (
            "record_vector_index:stale_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM record_vector_index
               EXCEPT
               SELECT record_key FROM document_embedding_cache
             )",
            "every vector index row has a document embedding cache row",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Embedding,
                format!("record vector index coverage check `{key}` failed"),
                Some(key.to_string()),
                Some(expected.to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }

    Ok(())
}

fn metadata_summary_from_report(report: &ArtifactValidationReport) -> ArtifactMetadataSummary {
    ArtifactMetadataSummary {
        artifact_contract_version: report.artifact_contract_version.clone(),
        schema_version: report.schema_version.clone(),
        source_kind: report.source_kind.clone(),
        source_signature: report.source_signature.clone(),
        source_record_count: report.source_record_count.clone(),
        artifact_record_count: report.artifact_record_count.clone(),
        generated_record_count: report.generated_record_count.clone(),
        content_hash_algorithm: report.content_hash_algorithm.clone(),
        embedding_provider_family: report.embedding_provider_family.clone(),
        embedding_model_id: report.embedding_model_id.clone(),
        embedding_model_revision: report.embedding_model_revision.clone(),
        embedding_tokenizer_id: report.embedding_tokenizer_id.clone(),
        embedding_pooling: report.embedding_pooling.clone(),
        embedding_normalization: report.embedding_normalization.clone(),
        embedding_dimensions: report.embedding_dimensions.clone(),
        embedding_dtype: report.embedding_dtype.clone(),
        embedding_distance_metric: report.embedding_distance_metric.clone(),
        embedding_document_prefix: report.embedding_document_prefix.clone(),
        embedding_query_prefix: report.embedding_query_prefix.clone(),
        fts_tokenizer: report.fts_tokenizer.clone(),
        adjacent_manifest_path: report.adjacent_manifest_path.clone(),
    }
}
