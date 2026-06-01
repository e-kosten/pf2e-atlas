use rusqlite::Connection;

use crate::artifact::inventory::{TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX};
use crate::artifact::validation::{
    artifact_validation_diagnostic, artifact_validation_diagnostic_with_code,
};
use crate::read::search::vector::extension::probe_sqlite_vec;
use crate::sql::{count_rows, count_sql, table_exists};
use crate::{
    ArtifactMetadataSummary, ArtifactValidationFamily, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus,
};

pub(crate) fn validate_vector_index_connection(
    index: String,
    base_report: ArtifactValidationReport,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    if base_report.status != ValidationStatus::Ok {
        return Ok(base_report);
    }

    let summary = metadata_summary_from_report(&base_report);
    if let Err(message) = probe_sqlite_vec(connection) {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    validate_vector_index_with_loaded_connection(index, summary, connection)
}

pub(crate) fn validate_embedding_readiness_connection(
    index: String,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let metadata_report = crate::validate_index_metadata_connection(index.clone(), connection)?;
    if metadata_report.status != ValidationStatus::Ok {
        return Ok(metadata_report);
    }

    let summary = metadata_summary_from_report(&metadata_report);
    if let Err(message) = probe_sqlite_vec(connection) {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    validate_vector_index_with_loaded_connection(index, summary, connection)
}

pub(crate) fn check_embedding_readiness_connection(
    index: String,
    base_report: ArtifactValidationReport,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    if base_report.status != ValidationStatus::Ok {
        return Ok(base_report);
    }

    let summary = metadata_summary_from_report(&base_report);
    if let Err(message) = probe_sqlite_vec(connection) {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    let mut diagnostics = Vec::new();
    if !table_exists(connection, TABLE_RECORD_VECTOR_INDEX)? {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Schema,
            format!("required vector table `{TABLE_RECORD_VECTOR_INDEX}` is missing"),
            Some(format!("table:{TABLE_RECORD_VECTOR_INDEX}")),
            Some("present".to_string()),
            Some("missing".to_string()),
        ));
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

pub(crate) fn vector_extension_unavailable_report_from_base(
    index: String,
    base_report: ArtifactValidationReport,
    message: String,
) -> ArtifactValidationReport {
    if base_report.status != ValidationStatus::Ok {
        return base_report;
    }
    vector_extension_unavailable_report(index, metadata_summary_from_report(&base_report), message)
}

fn validate_vector_index_with_loaded_connection(
    index: String,
    summary: ArtifactMetadataSummary,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let mut diagnostics = Vec::new();
    if !table_exists(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)? {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Schema,
            format!("required embedding table `{TABLE_DOCUMENT_EMBEDDING_CACHE}` is missing"),
            Some(format!("table:{TABLE_DOCUMENT_EMBEDDING_CACHE}")),
            Some("present".to_string()),
            Some("missing".to_string()),
        ));
    }
    if !table_exists(connection, TABLE_RECORD_VECTOR_INDEX)? {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Schema,
            format!("required vector table `{TABLE_RECORD_VECTOR_INDEX}` is missing"),
            Some(format!("table:{TABLE_RECORD_VECTOR_INDEX}")),
            Some("present".to_string()),
            Some("missing".to_string()),
        ));
    }
    if diagnostics.is_empty() {
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
        vec![artifact_validation_diagnostic_with_code(
            ArtifactValidationFamily::Embedding,
            "sqlite-vec extension is unavailable for vector index operations".to_string(),
            Some("sqlite_vec".to_string()),
            Some("available".to_string()),
            Some(message),
            ValidationCode::VectorExtensionUnavailable,
        )],
    )
}

fn validate_vector_index_coverage(
    connection: &Connection,
    diagnostics: &mut Vec<crate::ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let vector_rows = count_rows(connection, TABLE_RECORD_VECTOR_INDEX)?;
    let cache_rows = count_rows(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)?;
    if vector_rows != cache_rows {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Embedding,
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
               SELECT rowid FROM document_embedding_cache
               EXCEPT
               SELECT rowid FROM record_vector_index
             )",
            "every document embedding has a vector index row",
        ),
        (
            "record_vector_index:stale_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT rowid FROM record_vector_index
               EXCEPT
               SELECT rowid FROM document_embedding_cache
             )",
            "every vector index row has a document embedding cache row",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Embedding,
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
        embedding_unit_policy_version: report.embedding_unit_policy_version.clone(),
        fts_tokenizer: report.fts_tokenizer.clone(),
        adjacent_manifest_path: report.adjacent_manifest_path.clone(),
    }
}
