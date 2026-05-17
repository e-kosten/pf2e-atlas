use atlas_artifact::{
    schema::{TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX},
    storage::encode_f32_vector_blob,
};
use atlas_domain::SearchFilterNode;
use atlas_embedding::EmbeddingUnitKind;
use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};
use thiserror::Error;

use crate::contract::{contract_diagnostic, contract_diagnostic_with_code};
use crate::filters::{FilterCompileError, compile_eligible_records_query};
use crate::sql::{count_rows, count_sql, table_exists};
use crate::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus,
};

#[derive(Debug, Clone, PartialEq)]
pub struct VectorKnnQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchHit {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VectorQueryError {
    #[error("vector query limit must be greater than zero")]
    InvalidLimit,
    #[error("query vector must not be empty")]
    EmptyQueryVector,
    #[error("vector query failed: {0}")]
    QueryFailed(String),
    #[error("vector query returned invalid unit kind: {0}")]
    InvalidUnitKind(String),
    #[error(transparent)]
    Filter(#[from] FilterCompileError),
}

pub(crate) fn compile_vector_knn_query(
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    include_child_units: bool,
) -> Result<VectorKnnQuery, VectorQueryError> {
    if limit == 0 {
        return Err(VectorQueryError::InvalidLimit);
    }
    if query_vector.is_empty() {
        return Err(VectorQueryError::EmptyQueryVector);
    }

    let eligible = compile_eligible_records_query(filter)?;
    let mut parameters = eligible.parameters;
    let vector_placeholder = push_parameter(
        &mut parameters,
        Value::Blob(encode_f32_vector_blob(query_vector)),
    );
    let limit_placeholder = push_parameter(&mut parameters, Value::Integer(i64::from(limit)));
    let unit_filter = if include_child_units {
        ""
    } else {
        "AND candidate.unit_kind = 'parent'"
    };
    let sql = format!(
        "WITH eligible(record_key) AS ({eligible_sql})
         SELECT e.embedding_unit_key, e.record_key, e.unit_kind, e.label, v.distance
         FROM {vector_table} v
         JOIN {cache_table} e ON e.rowid = v.rowid
         WHERE v.embedding MATCH {vector_placeholder}
           AND k = {limit_placeholder}
           AND v.rowid IN (
             SELECT candidate.rowid
             FROM {cache_table} candidate
             WHERE candidate.record_key IN (SELECT record_key FROM eligible)
               {unit_filter}
           )
         ORDER BY v.distance ASC",
        eligible_sql = eligible.sql,
        vector_table = TABLE_RECORD_VECTOR_INDEX,
        cache_table = TABLE_DOCUMENT_EMBEDDING_CACHE,
        unit_filter = unit_filter,
    );

    Ok(VectorKnnQuery { sql, parameters })
}

pub fn query_vector_index(
    connection: &Connection,
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    include_child_units: bool,
) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
    let compiled = compile_vector_knn_query(query_vector, filter, limit, include_child_units)?;
    let mut statement = connection
        .prepare(&compiled.sql)
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map(params_from_iter(compiled.parameters.iter()), |row| {
            Ok(VectorSearchHit {
                embedding_unit_key: row.get(0)?,
                record_key: row.get(1)?,
                unit_kind: row.get(2)?,
                label: row.get(3)?,
                distance: row.get(4)?,
            })
        })
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    let rows = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|mut hit| {
            hit.unit_kind = parse_embedding_unit_kind(hit.unit_kind)?;
            Ok(hit)
        })
        .collect()
}

fn parse_embedding_unit_kind(value: String) -> Result<String, VectorQueryError> {
    value
        .parse::<EmbeddingUnitKind>()
        .map(|unit_kind| unit_kind.as_str().to_string())
        .map_err(|_| VectorQueryError::InvalidUnitKind(value))
}

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

fn validate_vector_index_with_loaded_connection(
    index: String,
    summary: ArtifactMetadataSummary,
    connection: &Connection,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let mut diagnostics = Vec::new();
    if !table_exists(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)? {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Schema,
            format!("required embedding table `{TABLE_DOCUMENT_EMBEDDING_CACHE}` is missing"),
            Some(format!("table:{TABLE_DOCUMENT_EMBEDDING_CACHE}")),
            Some("present".to_string()),
            Some("missing".to_string()),
        ));
    }
    if !table_exists(connection, TABLE_RECORD_VECTOR_INDEX)? {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Schema,
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

fn probe_sqlite_vec(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.atlas_vec_capability_probe
             USING vec0(embedding FLOAT[1]);
             DROP TABLE temp.atlas_vec_capability_probe;",
        )
        .map_err(|error| error.to_string())
}

pub(crate) fn register_sqlite_vec_extension() -> Result<(), String> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().map_err(|error| error.to_string())
}

fn push_parameter(parameters: &mut Vec<Value>, value: Value) -> String {
    parameters.push(value);
    format!("?{}", parameters.len())
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
        embedding_unit_policy_version: report.embedding_unit_policy_version.clone(),
        fts_tokenizer: report.fts_tokenizer.clone(),
        adjacent_manifest_path: report.adjacent_manifest_path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_embedding_unit_kind_to_public_string() {
        assert_eq!(
            parse_embedding_unit_kind("heading_section".to_string())
                .expect("known unit kind should parse"),
            "heading_section"
        );
    }

    #[test]
    fn rejects_unknown_embedding_unit_kind() {
        assert!(parse_embedding_unit_kind("unknown".to_string()).is_err());
    }
}
