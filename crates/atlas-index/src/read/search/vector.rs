use crate::artifact::inventory::{TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX};
use crate::artifact::storage::{decode_f32_vector_blob, encode_f32_vector_blob};
use crate::read::sql::SqlBindValue;
use crate::sqlite::SqliteIndexReader;
use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::EmbeddingUnitKind;
use diesel::sql_types::{BigInt, Binary, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};
use rusqlite::Connection;
use thiserror::Error;

use crate::artifact::validation::{
    artifact_validation_diagnostic, artifact_validation_diagnostic_with_code,
};
use crate::read::search::filters::{FilterCompileError, SqliteEligibleRecordKeyset};
use crate::read::sql::bind_sql_query;
use crate::sql::{count_rows, count_sql, table_exists};
use crate::{
    ArtifactMetadataSummary, ArtifactValidationFamily, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus,
};

#[derive(Debug, Clone, PartialEq)]
pub struct VectorKnnQuery {
    pub sql: String,
    pub parameters: Vec<SqlBindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchHit {
    pub record_key: String,
    pub unit_kind: String,
    pub label: Option<String>,
    pub distance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordEmbeddingVector {
    pub unit_kind: String,
    pub label: Option<String>,
    pub ordinal: i64,
    pub vector: Vec<f32>,
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
    #[error(
        "stored embedding vector for `{record_key}` ({unit_kind} #{ordinal}) had invalid vector data: {message}"
    )]
    InvalidStoredVector {
        record_key: String,
        unit_kind: String,
        ordinal: i64,
        message: String,
    },
    #[error(
        "stored embedding vector for `{record_key}` ({unit_kind} #{ordinal}) declared {declared_dimensions} dimensions but decoded to {decoded_dimensions}"
    )]
    InvalidStoredDimensions {
        record_key: String,
        unit_kind: String,
        ordinal: i64,
        declared_dimensions: usize,
        decoded_dimensions: usize,
    },
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

    let unit_filter = if include_child_units {
        ""
    } else {
        "AND candidate.unit_kind = 'parent'"
    };
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let vector_placeholder = builder.push_blob(encode_f32_vector_blob(query_vector));
            let limit_placeholder = builder.push_integer(i64::from(limit));
            format!(
                "SELECT e.record_key, e.unit_kind, e.label, v.distance
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
                vector_table = TABLE_RECORD_VECTOR_INDEX,
                cache_table = TABLE_DOCUMENT_EMBEDDING_CACHE,
                unit_filter = unit_filter,
            )
        });

    Ok(VectorKnnQuery {
        sql: query.sql,
        parameters: query.parameters,
    })
}

pub fn query_vector_index(
    connection: &mut SqliteConnection,
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    include_child_units: bool,
) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
    let compiled = compile_vector_knn_query(query_vector, filter, limit, include_child_units)?;
    let rows = bind_sql_query(compiled.sql, &compiled.parameters)
        .load::<VectorSearchHitRow>(connection)
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(VectorSearchHit {
                record_key: row.record_key,
                unit_kind: parse_embedding_unit_kind(row.unit_kind)?,
                label: row.label,
                distance: row.distance,
            })
        })
        .collect()
}

pub fn load_record_embedding_vectors(
    connection: &mut SqliteConnection,
    record_key: &RecordKey,
) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
    let rows = bind_sql_query(
        "SELECT unit_kind, label, ordinal, dimensions, vector_blob
         FROM document_embedding_cache
         WHERE record_key = ?1
         ORDER BY ordinal ASC, embedding_unit_key ASC"
            .to_string(),
        &[SqlBindValue::Text(record_key.to_string())],
    )
    .load::<RecordEmbeddingVectorRow>(connection)
    .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    let record_key = record_key.to_string();
    rows.into_iter()
        .map(|row| {
            let vector = decode_f32_vector_blob(&row.vector_blob).map_err(|error| {
                VectorQueryError::InvalidStoredVector {
                    record_key: record_key.clone(),
                    unit_kind: row.unit_kind.clone(),
                    ordinal: row.ordinal,
                    message: error.to_string(),
                }
            })?;
            let declared_dimensions = usize::try_from(row.dimensions).map_err(|error| {
                VectorQueryError::QueryFailed(format!(
                    "stored embedding vector for `{record_key}` ({} #{}) had invalid dimensions `{}`: {error}",
                    row.unit_kind, row.ordinal, row.dimensions
                ))
            })?;
            if declared_dimensions != vector.len() {
                return Err(VectorQueryError::InvalidStoredDimensions {
                    record_key: record_key.clone(),
                    unit_kind: row.unit_kind.clone(),
                    ordinal: row.ordinal,
                    declared_dimensions,
                    decoded_dimensions: vector.len(),
                });
            }
            Ok(RecordEmbeddingVector {
                unit_kind: parse_embedding_unit_kind(row.unit_kind)?,
                label: row.label,
                ordinal: row.ordinal,
                vector,
            })
        })
        .collect()
}

#[derive(QueryableByName)]
struct VectorSearchHitRow {
    #[diesel(sql_type = Text)]
    record_key: String,
    #[diesel(sql_type = Text)]
    unit_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    label: Option<String>,
    #[diesel(sql_type = Double)]
    distance: f64,
}

#[derive(QueryableByName)]
struct RecordEmbeddingVectorRow {
    #[diesel(sql_type = Text)]
    unit_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    label: Option<String>,
    #[diesel(sql_type = BigInt)]
    ordinal: i64,
    #[diesel(sql_type = BigInt)]
    dimensions: i64,
    #[diesel(sql_type = Binary)]
    vector_blob: Vec<u8>,
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

impl SqliteIndexReader {
    pub fn validate_vector_index(&self) -> Result<ArtifactValidationReport, IndexValidationError> {
        validate_vector_index_connection(
            self.path().display().to_string(),
            self.validate()?,
            &self.validation_connection()?,
        )
    }

    pub fn vector_validation_report(&self) -> ArtifactValidationReport {
        match self.validate_vector_index() {
            Ok(report) => report,
            Err(error) => crate::validation_report_from_error(self.path(), error),
        }
    }

    pub fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
        self.with_diesel_connection(|connection| {
            query_vector_index(connection, query_vector, filter, limit, include_child_units)
        })
    }

    pub fn load_record_embedding_vectors(
        &self,
        record_key: &RecordKey,
    ) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
        self.with_diesel_connection(|connection| {
            load_record_embedding_vectors(connection, record_key)
        })
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
