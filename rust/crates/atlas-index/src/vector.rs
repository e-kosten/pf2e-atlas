use std::path::Path;

use atlas_artifact::schema::{
    TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX, record_vector_index_create_sql,
    record_vector_index_insert_sql,
};
use atlas_domain::SearchFilterNode;
use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags, params_from_iter};
use thiserror::Error;
use tracing::info;

use crate::contract::{contract_diagnostic, contract_diagnostic_with_code};
use crate::filters::{FilterCompileError, compile_eligible_records_query};
use crate::sql::{count_rows, count_sql, table_exists};
use crate::{
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationReport,
    IndexValidationError, ValidationCode, ValidationStatus, validate_index,
};

pub type VectorExtensionLoader = dyn FnOnce() -> Result<(), String>;

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
    pub rank_distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VectorSearchMode {
    ParentOnly,
    Chunks,
    WeightedChunks,
}

impl VectorSearchMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ParentOnly => "parent-only",
            Self::Chunks => "chunks",
            Self::WeightedChunks => "weighted-chunks",
        }
    }

    fn includes_child_units(self) -> bool {
        !matches!(self, Self::ParentOnly)
    }

    fn uses_rank_weights(self) -> bool {
        matches!(self, Self::WeightedChunks)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VectorQueryError {
    #[error("vector query limit must be greater than zero")]
    InvalidLimit,
    #[error("query vector must not be empty")]
    EmptyQueryVector,
    #[error("vector query failed: {0}")]
    QueryFailed(String),
    #[error(transparent)]
    Filter(#[from] FilterCompileError),
}

pub fn compile_vector_knn_query(
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    mode: VectorSearchMode,
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
        Value::Blob(encode_f32_vector(query_vector)),
    );
    let unit_limit = limit.saturating_mul(20).max(limit).min(1000);
    let limit_placeholder = push_parameter(&mut parameters, Value::Integer(i64::from(unit_limit)));
    let unit_filter = if mode.includes_child_units() {
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
    mode: VectorSearchMode,
) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
    let compiled = compile_vector_knn_query(query_vector, filter, limit, mode)?;
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
                rank_distance: row.get(4)?,
            })
        })
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    let rows = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    Ok(collapse_vector_hits(rows, limit as usize, mode))
}

fn collapse_vector_hits(
    rows: Vec<VectorSearchHit>,
    limit: usize,
    mode: VectorSearchMode,
) -> Vec<VectorSearchHit> {
    let mut grouped = std::collections::BTreeMap::<String, Vec<VectorSearchHit>>::new();
    for hit in rows {
        grouped.entry(hit.record_key.clone()).or_default().push(hit);
    }
    let mut collapsed = grouped
        .into_values()
        .filter_map(|hits| best_record_hit(hits, mode))
        .collect::<Vec<_>>();
    collapsed.sort_by(compare_vector_hits_for_rank);
    collapsed.truncate(limit);
    collapsed
}

fn best_record_hit(hits: Vec<VectorSearchHit>, mode: VectorSearchMode) -> Option<VectorSearchHit> {
    let has_parent = hits.iter().any(|hit| hit.unit_kind == "parent");
    hits.into_iter()
        .map(|mut hit| {
            hit.rank_distance = if mode.uses_rank_weights() {
                adjusted_rank_distance(&hit, has_parent)
            } else {
                hit.distance
            };
            hit
        })
        .min_by(compare_vector_hits_for_rank)
}

fn adjusted_rank_distance(hit: &VectorSearchHit, has_parent: bool) -> f64 {
    let unit_penalty = match hit.unit_kind.as_str() {
        "parent" => 0.0,
        "heading_section" => 0.025,
        "titled_option" => 0.040,
        "activation_block" => 0.055,
        _ => 0.060,
    };
    let missing_parent_penalty = if has_parent { 0.0 } else { 0.025 };
    hit.distance + unit_penalty + missing_parent_penalty
}

fn compare_vector_hits_for_rank(
    left: &VectorSearchHit,
    right: &VectorSearchHit,
) -> std::cmp::Ordering {
    left.rank_distance
        .total_cmp(&right.rank_distance)
        .then_with(|| left.distance.total_cmp(&right.distance))
        .then_with(|| left.record_key.cmp(&right.record_key))
        .then_with(|| left.embedding_unit_key.cmp(&right.embedding_unit_key))
}

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
    write_vector_index_with_loader(path, register_sqlite_vec_extension)
}

pub fn write_vector_index_with_loader(
    path: impl AsRef<Path>,
    loader: impl FnOnce() -> Result<(), String>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    info!(index = %path.display(), "validating base artifact before vector index build");
    let base_report = validate_index(path)?;
    if base_report.status != ValidationStatus::Ok {
        info!(index = %path.display(), "base artifact validation failed; skipping vector index build");
        return Ok(base_report);
    }

    let index = path.display().to_string();
    let summary = metadata_summary_from_report(&base_report);
    if let Err(message) = loader() {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    info!(index = %path.display(), "checking sqlite-vec capability");
    if let Err(message) = probe_sqlite_vec(&connection) {
        info!(index = %path.display(), error = %message, "sqlite-vec capability unavailable");
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }

    info!(index = %path.display(), "creating and populating record_vector_index");
    create_and_populate_vector_index(&connection, &summary)?;
    info!(index = %path.display(), "validating record_vector_index");
    validate_vector_index_with_loaded_connection(index, summary, &connection)
}

pub fn validate_vector_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    validate_vector_index_with_loader(path, register_sqlite_vec_extension)
}

pub fn validate_vector_index_with_loader(
    path: impl AsRef<Path>,
    loader: impl FnOnce() -> Result<(), String>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    info!(index = %path.display(), "validating base artifact before vector index validation");
    let base_report = validate_index(path)?;
    if base_report.status != ValidationStatus::Ok {
        info!(index = %path.display(), "base artifact validation failed; skipping vector validation");
        return Ok(base_report);
    }

    let index = path.display().to_string();
    let summary = metadata_summary_from_report(&base_report);
    if let Err(message) = loader() {
        return Ok(vector_extension_unavailable_report(index, summary, message));
    }
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    info!(index = %path.display(), "checking sqlite-vec capability");
    if let Err(message) = probe_sqlite_vec(&connection) {
        info!(index = %path.display(), error = %message, "sqlite-vec capability unavailable");
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
    info!(
        dimensions,
        table = TABLE_RECORD_VECTOR_INDEX,
        "recreating vector index table"
    );

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
            "SELECT rowid, vector_blob
             FROM document_embedding_cache
             ORDER BY embedding_unit_key",
        )
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = select
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut row_count = 0usize;
    for row in rows {
        let (rowid, vector_blob) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        insert
            .execute((rowid, vector_blob))
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        row_count += 1;
    }
    info!(
        rows = row_count,
        table = TABLE_RECORD_VECTOR_INDEX,
        "populated vector index table"
    );

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

pub(crate) fn register_sqlite_vec_extension() -> Result<(), String> {
    atlas_sqlite_vec::register_sqlite_vec_auto_extension().map_err(|error| error.to_string())
}

fn push_parameter(parameters: &mut Vec<Value>, value: Value) -> String {
    parameters.push(value);
    format!("?{}", parameters.len())
}

fn encode_f32_vector(vector: &[f32]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(std::mem::size_of_val(vector));
    for value in vector {
        blob.extend_from_slice(&value.to_le_bytes());
    }
    blob
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
        fts_tokenizer: report.fts_tokenizer.clone(),
        adjacent_manifest_path: report.adjacent_manifest_path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(unit: &str, record: &str, unit_kind: &str, distance: f64) -> VectorSearchHit {
        VectorSearchHit {
            embedding_unit_key: unit.to_string(),
            record_key: record.to_string(),
            unit_kind: unit_kind.to_string(),
            label: None,
            distance,
            rank_distance: distance,
        }
    }

    #[test]
    fn collapse_vector_hits_keeps_one_unit_per_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit("records:a#parent", "records:a", "parent", 0.1),
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.2,
                ),
                hit("records:b#parent", "records:b", "parent", 0.3),
                hit("records:c#parent", "records:c", "parent", 0.4),
            ],
            2,
            VectorSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed
                .iter()
                .map(|hit| (hit.embedding_unit_key.as_str(), hit.record_key.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("records:a#parent", "records:a"),
                ("records:b#parent", "records:b"),
            ]
        );
    }

    #[test]
    fn collapse_vector_hits_keeps_parent_over_slightly_closer_activation_block() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#activation_block:1",
                    "records:a",
                    "activation_block",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.130),
            ],
            10,
            VectorSearchMode::WeightedChunks,
        );

        assert_eq!(collapsed[0].embedding_unit_key, "records:a#parent");
        assert_eq!(collapsed[0].distance, 0.130);
        assert_eq!(collapsed[0].rank_distance, 0.130);
    }

    #[test]
    fn collapse_vector_hits_allows_much_closer_child_to_recover_record() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#activation_block:1",
                    "records:a",
                    "activation_block",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.200),
            ],
            10,
            VectorSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#activation_block:1"
        );
        assert_eq!(collapsed[0].distance, 0.100);
        assert_eq!(collapsed[0].rank_distance, 0.155);
    }

    #[test]
    fn collapse_vector_hits_prefers_heading_section_over_activation_block_at_similar_distance() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#activation_block:1",
                    "records:a",
                    "activation_block",
                    0.100,
                ),
                hit(
                    "records:b#heading_section:1",
                    "records:b",
                    "heading_section",
                    0.105,
                ),
            ],
            10,
            VectorSearchMode::WeightedChunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:b#heading_section:1"
        );
        assert_eq!(collapsed[0].rank_distance, 0.155);
        assert_eq!(
            collapsed[1].embedding_unit_key,
            "records:a#activation_block:1"
        );
        assert_eq!(collapsed[1].rank_distance, 0.180);
    }

    #[test]
    fn collapse_vector_hits_penalizes_records_without_parent_hit() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:b#parent", "records:b", "parent", 0.145),
            ],
            10,
            VectorSearchMode::WeightedChunks,
        );

        assert_eq!(collapsed[0].embedding_unit_key, "records:b#parent");
        assert_eq!(collapsed[0].rank_distance, 0.145);
        assert_eq!(
            collapsed[1].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[1].rank_distance, 0.150);
    }

    #[test]
    fn collapse_vector_hits_can_rank_chunks_without_unit_weights() {
        let collapsed = collapse_vector_hits(
            vec![
                hit(
                    "records:a#heading_section:1",
                    "records:a",
                    "heading_section",
                    0.100,
                ),
                hit("records:a#parent", "records:a", "parent", 0.120),
            ],
            10,
            VectorSearchMode::Chunks,
        );

        assert_eq!(
            collapsed[0].embedding_unit_key,
            "records:a#heading_section:1"
        );
        assert_eq!(collapsed[0].rank_distance, 0.100);
    }
}
