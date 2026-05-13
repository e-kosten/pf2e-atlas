#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use atlas_domain::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION,
    ArtifactContractFamily, ArtifactMetadataSummary, ArtifactValidationDiagnostic,
    ArtifactValidationReport, EXPECTED_CONTENT_HASH_ALGORITHM, EXPECTED_EMBEDDING_DIMENSIONS,
    EXPECTED_EMBEDDING_DISTANCE_METRIC, EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
    EXPECTED_EMBEDDING_DTYPE, EXPECTED_EMBEDDING_MODEL_ID, EXPECTED_EMBEDDING_MODEL_REVISION,
    EXPECTED_EMBEDDING_NORMALIZATION, EXPECTED_EMBEDDING_POOLING,
    EXPECTED_EMBEDDING_PROVIDER_FAMILY, EXPECTED_EMBEDDING_QUERY_PREFIX,
    EXPECTED_EMBEDDING_TOKENIZER_ID, EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND,
    LEGACY_METADATA_TABLE, REQUIRED_ARTIFACT_METADATA_KEYS, ValidationCode, artifact_metadata_keys,
};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexValidationError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
    #[error("index artifact metadata is invalid: {0}")]
    InvalidArtifact(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexInspectionReport {
    pub status: String,
    pub index: String,
    pub validation: ArtifactValidationReport,
    pub tables: BTreeMap<String, usize>,
    pub records: RecordCoverageReport,
    pub text: TextCoverageReport,
    pub taxonomy: TaxonomyCoverageReport,
    pub variants: VariantCoverageReport,
    pub relationships: RelationshipCoverageReport,
    pub metrics: MetricCoverageReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordCoverageReport {
    pub total_records: usize,
    pub search_canonical_records: usize,
    pub by_record_family: BTreeMap<String, usize>,
    pub by_foundry_taxonomy: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextCoverageReport {
    pub records_with_description: usize,
    pub records_with_blurb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaxonomyCoverageReport {
    pub records_with_taxonomy_families: usize,
    pub distinct_taxonomy_families: usize,
    pub top_taxonomy_families: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VariantCoverageReport {
    pub grouped_records: usize,
    pub distinct_groups: usize,
    pub by_source: BTreeMap<String, usize>,
    pub by_axis: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationshipCoverageReport {
    pub reference_edges: usize,
    pub record_aliases: usize,
    pub remaster_links: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetricCoverageReport {
    pub metric_rows_by_domain: BTreeMap<String, usize>,
    pub metric_keys_by_domain: BTreeMap<String, usize>,
    pub metric_value_catalog_rows: usize,
}

pub fn validate_index(
    path: impl AsRef<Path>,
) -> Result<ArtifactValidationReport, IndexValidationError> {
    let path = path.as_ref();
    let index = path.display().to_string();
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if !table_exists(&connection, ARTIFACT_METADATA_TABLE)? {
        let legacy_schema_version = if table_exists(&connection, LEGACY_METADATA_TABLE)? {
            metadata_value(&connection, LEGACY_METADATA_TABLE, "schema_version")?
        } else {
            None
        };
        return Ok(ArtifactValidationReport::missing_artifact_metadata(
            index,
            legacy_schema_version,
        ));
    }

    let metadata = read_metadata(&connection, ARTIFACT_METADATA_TABLE)?;
    let summary = summarize_metadata(&metadata);
    let missing_keys = REQUIRED_ARTIFACT_METADATA_KEYS
        .iter()
        .filter(|key| {
            metadata
                .get(**key)
                .is_none_or(|value| is_missing_value(key, value))
        })
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();

    if !missing_keys.is_empty() {
        return Ok(ArtifactValidationReport::missing_required_metadata(
            index,
            summary,
            missing_keys,
        ));
    }

    let diagnostics = validate_metadata_values(&metadata);
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

pub fn inspect_index(
    path: impl AsRef<Path>,
) -> Result<IndexInspectionReport, IndexValidationError> {
    let path = path.as_ref();
    let validation = validate_index(path)?;
    if validation.status != atlas_domain::ValidationStatus::Ok {
        return Err(IndexValidationError::InvalidArtifact(validation.message));
    }

    let index = path.display().to_string();
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    Ok(IndexInspectionReport {
        status: "ok".to_string(),
        index,
        validation,
        tables: inspect_tables(&connection)?,
        records: inspect_records(&connection)?,
        text: inspect_text(&connection)?,
        taxonomy: inspect_taxonomy(&connection)?,
        variants: inspect_variants(&connection)?,
        relationships: inspect_relationships(&connection)?,
        metrics: inspect_metrics(&connection)?,
    })
}

fn is_missing_value(key: &str, value: &str) -> bool {
    if matches!(
        key,
        artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX
            | artifact_metadata_keys::EMBEDDING_QUERY_PREFIX
    ) {
        false
    } else {
        value.trim().is_empty()
    }
}

fn table_exists(connection: &Connection, table: &str) -> Result<bool, IndexValidationError> {
    let mut statement = connection
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([table])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    rows.next()
        .map(|row| row.is_some())
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

fn inspect_tables(
    connection: &Connection,
) -> Result<BTreeMap<String, usize>, IndexValidationError> {
    let table_names = [
        "artifact_metadata",
        "packs",
        "records",
        "record_traits",
        "records_fts",
        "reference_edges",
        "record_aliases",
        "remaster_links",
        "record_metrics",
        "metric_key_catalog",
        "metric_value_catalog",
        "actor_records",
        "item_records",
        "spell_records",
    ];
    let mut tables = BTreeMap::new();
    for table in table_names {
        tables.insert(table.to_string(), count_rows(connection, table)?);
    }
    Ok(tables)
}

fn inspect_records(connection: &Connection) -> Result<RecordCoverageReport, IndexValidationError> {
    Ok(RecordCoverageReport {
        total_records: count_rows(connection, "records")?,
        search_canonical_records: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE is_search_canonical = 1",
        )?,
        by_record_family: count_grouped(
            connection,
            "SELECT record_family, COUNT(*) FROM records GROUP BY record_family",
        )?,
        by_foundry_taxonomy: count_grouped(
            connection,
            "SELECT foundry_document_type || '|' || foundry_record_type, COUNT(*)
             FROM records
             GROUP BY foundry_document_type, foundry_record_type",
        )?,
    })
}

fn inspect_text(connection: &Connection) -> Result<TextCoverageReport, IndexValidationError> {
    Ok(TextCoverageReport {
        records_with_description: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE description_text IS NOT NULL AND TRIM(description_text) <> ''",
        )?,
        records_with_blurb: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE blurb_text IS NOT NULL AND TRIM(blurb_text) <> ''",
        )?,
    })
}

fn inspect_taxonomy(
    connection: &Connection,
) -> Result<TaxonomyCoverageReport, IndexValidationError> {
    Ok(TaxonomyCoverageReport {
        records_with_taxonomy_families: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE taxonomy_families_json <> '[]'",
        )?,
        distinct_taxonomy_families: count_sql(
            connection,
            "SELECT COUNT(DISTINCT taxonomy_family.value)
             FROM records, json_each(records.taxonomy_families_json) AS taxonomy_family",
        )?,
        top_taxonomy_families: count_grouped_limited(
            connection,
            "SELECT taxonomy_family.value, COUNT(*)
             FROM records, json_each(records.taxonomy_families_json) AS taxonomy_family
             GROUP BY taxonomy_family.value
             ORDER BY COUNT(*) DESC, taxonomy_family.value ASC
             LIMIT 20",
        )?,
    })
}

fn inspect_variants(
    connection: &Connection,
) -> Result<VariantCoverageReport, IndexValidationError> {
    Ok(VariantCoverageReport {
        grouped_records: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE variant_group_key IS NOT NULL",
        )?,
        distinct_groups: count_sql(
            connection,
            "SELECT COUNT(DISTINCT variant_group_key) FROM records WHERE variant_group_key IS NOT NULL",
        )?,
        by_source: count_grouped(
            connection,
            "SELECT variant_source, COUNT(*)
             FROM records
             WHERE variant_group_key IS NOT NULL
             GROUP BY variant_source",
        )?,
        by_axis: count_grouped(
            connection,
            "SELECT variant_axis.value, COUNT(*)
             FROM records, json_each(records.variant_axes_json) AS variant_axis
             WHERE records.variant_group_key IS NOT NULL
             GROUP BY variant_axis.value",
        )?,
    })
}

fn inspect_relationships(
    connection: &Connection,
) -> Result<RelationshipCoverageReport, IndexValidationError> {
    Ok(RelationshipCoverageReport {
        reference_edges: count_rows(connection, "reference_edges")?,
        record_aliases: count_rows(connection, "record_aliases")?,
        remaster_links: count_rows(connection, "remaster_links")?,
    })
}

fn inspect_metrics(connection: &Connection) -> Result<MetricCoverageReport, IndexValidationError> {
    Ok(MetricCoverageReport {
        metric_rows_by_domain: count_grouped(
            connection,
            "SELECT metric_domain, COUNT(*) FROM record_metrics GROUP BY metric_domain",
        )?,
        metric_keys_by_domain: count_grouped(
            connection,
            "SELECT metric_domain, COUNT(DISTINCT metric_key)
             FROM record_metrics
             GROUP BY metric_domain",
        )?,
        metric_value_catalog_rows: count_rows(connection, "metric_value_catalog")?,
    })
}

fn count_rows(connection: &Connection, table: &str) -> Result<usize, IndexValidationError> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    count_sql(connection, &sql)
}

fn count_sql(connection: &Connection, sql: &str) -> Result<usize, IndexValidationError> {
    connection
        .query_row(sql, [], |row| row.get::<_, usize>(0))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

fn count_grouped(
    connection: &Connection,
    sql: &str,
) -> Result<BTreeMap<String, usize>, IndexValidationError> {
    count_grouped_limited(connection, sql)
}

fn count_grouped_limited(
    connection: &Connection,
    sql: &str,
) -> Result<BTreeMap<String, usize>, IndexValidationError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (key, value) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        counts.insert(key, value);
    }
    Ok(counts)
}

fn metadata_value(
    connection: &Connection,
    table: &str,
    key: &str,
) -> Result<Option<String>, IndexValidationError> {
    let sql = format!("SELECT value FROM {table} WHERE key = ?1 LIMIT 1");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    match statement.query_row([key], |row| row.get::<_, String>(0)) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(IndexValidationError::QueryFailed(error.to_string())),
    }
}

fn read_metadata(
    connection: &Connection,
    table: &str,
) -> Result<BTreeMap<String, String>, IndexValidationError> {
    let sql = format!("SELECT key, value FROM {table}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;

    let mut metadata = BTreeMap::new();
    for row in rows {
        let (key, value) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        metadata.insert(key, value);
    }
    Ok(metadata)
}

fn summarize_metadata(metadata: &BTreeMap<String, String>) -> ArtifactMetadataSummary {
    ArtifactMetadataSummary {
        artifact_contract_version: metadata
            .get(artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION)
            .cloned(),
        schema_version: metadata
            .get(artifact_metadata_keys::SCHEMA_VERSION)
            .cloned(),
        source_kind: metadata.get(artifact_metadata_keys::SOURCE_KIND).cloned(),
        source_signature: metadata
            .get(artifact_metadata_keys::SOURCE_SIGNATURE)
            .cloned(),
        source_record_count: metadata
            .get(artifact_metadata_keys::SOURCE_RECORD_COUNT)
            .cloned(),
        content_hash_algorithm: metadata
            .get(artifact_metadata_keys::CONTENT_HASH_ALGORITHM)
            .cloned(),
        embedding_provider_family: metadata
            .get(artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY)
            .cloned(),
        embedding_model_id: metadata
            .get(artifact_metadata_keys::EMBEDDING_MODEL_ID)
            .cloned(),
        embedding_model_revision: metadata
            .get(artifact_metadata_keys::EMBEDDING_MODEL_REVISION)
            .cloned(),
        embedding_tokenizer_id: metadata
            .get(artifact_metadata_keys::EMBEDDING_TOKENIZER_ID)
            .cloned(),
        embedding_pooling: metadata
            .get(artifact_metadata_keys::EMBEDDING_POOLING)
            .cloned(),
        embedding_normalization: metadata
            .get(artifact_metadata_keys::EMBEDDING_NORMALIZATION)
            .cloned(),
        embedding_dimensions: metadata
            .get(artifact_metadata_keys::EMBEDDING_DIMENSIONS)
            .cloned(),
        embedding_dtype: metadata
            .get(artifact_metadata_keys::EMBEDDING_DTYPE)
            .cloned(),
        embedding_distance_metric: metadata
            .get(artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC)
            .cloned(),
        embedding_document_prefix: metadata
            .get(artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX)
            .cloned(),
        embedding_query_prefix: metadata
            .get(artifact_metadata_keys::EMBEDDING_QUERY_PREFIX)
            .cloned(),
        fts_tokenizer: metadata.get(artifact_metadata_keys::FTS_TOKENIZER).cloned(),
        adjacent_manifest_path: metadata
            .get(artifact_metadata_keys::ADJACENT_MANIFEST_PATH)
            .cloned(),
    }
}

fn validate_metadata_values(
    metadata: &BTreeMap<String, String>,
) -> Vec<ArtifactValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    require_value(
        metadata,
        artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
        ARTIFACT_CONTRACT_VERSION,
        ValidationCode::UnsupportedContractVersion,
        ArtifactContractFamily::Contract,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SCHEMA_VERSION,
        ARTIFACT_SCHEMA_VERSION,
        ValidationCode::UnsupportedSchemaVersion,
        ArtifactContractFamily::Schema,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::SOURCE_KIND,
        EXPECTED_SOURCE_KIND,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
        EXPECTED_CONTENT_HASH_ALGORITHM,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_positive_usize(
        metadata,
        artifact_metadata_keys::SOURCE_RECORD_COUNT,
        ValidationCode::InvalidSourceMetadata,
        ArtifactContractFamily::Source,
        &mut diagnostics,
    );
    require_source_signature(metadata, &mut diagnostics);
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
        EXPECTED_EMBEDDING_PROVIDER_FAMILY,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_ID,
        EXPECTED_EMBEDDING_MODEL_ID,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
        EXPECTED_EMBEDDING_MODEL_REVISION,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
        EXPECTED_EMBEDDING_TOKENIZER_ID,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_POOLING,
        EXPECTED_EMBEDDING_POOLING,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_NORMALIZATION,
        EXPECTED_EMBEDDING_NORMALIZATION,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DIMENSIONS,
        EXPECTED_EMBEDDING_DIMENSIONS,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DTYPE,
        EXPECTED_EMBEDDING_DTYPE,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
        EXPECTED_EMBEDDING_DISTANCE_METRIC,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
        EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
        EXPECTED_EMBEDDING_QUERY_PREFIX,
        ValidationCode::EmbeddingMismatch,
        ArtifactContractFamily::Embedding,
        &mut diagnostics,
    );
    require_value(
        metadata,
        artifact_metadata_keys::FTS_TOKENIZER,
        EXPECTED_FTS_TOKENIZER,
        ValidationCode::FtsMismatch,
        ArtifactContractFamily::Fts,
        &mut diagnostics,
    );
    require_manifest_path(metadata, &mut diagnostics);
    diagnostics
}

fn require_value(
    metadata: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
    code: ValidationCode,
    family: ArtifactContractFamily,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual != expected {
        diagnostics.push(ArtifactValidationDiagnostic {
            code,
            family,
            message: format!("metadata key `{key}` has an unsupported value"),
            key: Some(key.to_string()),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_positive_usize(
    metadata: &BTreeMap<String, String>,
    key: &str,
    code: ValidationCode,
    family: ArtifactContractFamily,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    let is_positive = actual
        .parse::<usize>()
        .map(|value| value > 0)
        .unwrap_or(false);
    if !is_positive {
        diagnostics.push(ArtifactValidationDiagnostic {
            code,
            family,
            message: format!("metadata key `{key}` must be a positive integer"),
            key: Some(key.to_string()),
            expected: Some("positive integer".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_source_signature(
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let key = artifact_metadata_keys::SOURCE_SIGNATURE;
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual == "stale" || actual.starts_with("stale:") {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::StaleSourceSignature,
            family: ArtifactContractFamily::Source,
            message: "source signature marks this artifact as stale".to_string(),
            key: Some(key.to_string()),
            expected: Some("current source signature".to_string()),
            actual: Some(actual.to_string()),
        });
    } else if !actual.starts_with("foundry-pf2e:") {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::InvalidSourceMetadata,
            family: ArtifactContractFamily::Source,
            message: "source signature must identify a Foundry PF2E source snapshot".to_string(),
            key: Some(key.to_string()),
            expected: Some("foundry-pf2e:<signature>".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

fn require_manifest_path(
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) {
    let key = artifact_metadata_keys::ADJACENT_MANIFEST_PATH;
    let actual = metadata.get(key).map(String::as_str).unwrap_or_default();
    if actual.trim().is_empty() || actual.contains('\0') || Path::new(actual).is_absolute() {
        diagnostics.push(ArtifactValidationDiagnostic {
            code: ValidationCode::ManifestMismatch,
            family: ArtifactContractFamily::Manifest,
            message: "adjacent manifest path must be a relative artifact path".to_string(),
            key: Some(key.to_string()),
            expected: Some("relative path".to_string()),
            actual: Some(actual.to_string()),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use atlas_domain::{
        ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_EMBEDDING_MODEL_ID,
        ValidationCode, ValidationStatus, artifact_metadata_keys,
    };
    use rusqlite::Connection;

    use super::validate_index;

    #[test]
    fn reports_valid_artifact_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("valid");
        create_contract_database(&path)?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Ok);
        assert_eq!(report.code, ValidationCode::Ok);
        assert_eq!(
            report.artifact_contract_version.as_deref(),
            Some(ARTIFACT_CONTRACT_VERSION)
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_legacy_metadata_without_accepting_it_as_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("legacy");
        let connection = Connection::open(&path)?;
        connection.execute(
            "CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        connection.execute(
            "INSERT INTO metadata (key, value) VALUES ('schema_version', '25')",
            [],
        )?;
        drop(connection);

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::MissingArtifactMetadata);
        assert_eq!(report.legacy_schema_version.as_deref(), Some("25"));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_missing_required_metadata_key() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("missing-key");
        create_contract_database_without(&path, artifact_metadata_keys::EMBEDDING_DTYPE)?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::MissingRequiredMetadata);
        assert_eq!(
            report.missing_keys,
            vec![artifact_metadata_keys::EMBEDDING_DTYPE.to_string()]
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_stale_source_signature() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("stale-source");
        create_contract_database_with_override(
            &path,
            artifact_metadata_keys::SOURCE_SIGNATURE,
            "stale:fixture",
        )?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::StaleSourceSignature);
        assert_eq!(report.diagnostics.len(), 1);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_embedding_mismatch() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("embedding-mismatch");
        create_contract_database_with_override(
            &path,
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            "BAAI/bge-small-en-v1.5",
        )?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::EmbeddingMismatch);
        assert_eq!(
            report.diagnostics[0].key.as_deref(),
            Some(artifact_metadata_keys::EMBEDDING_MODEL_ID)
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_unsupported_schema_version() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("unsupported-schema");
        create_contract_database_with_override(&path, artifact_metadata_keys::SCHEMA_VERSION, "2")?;

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::UnsupportedSchemaVersion);
        fs::remove_file(path)?;
        Ok(())
    }

    fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        connection.execute(
            "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        insert_contract_metadata(&connection, None)?;
        Ok(())
    }

    fn create_contract_database_without(
        path: &PathBuf,
        omitted_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        connection.execute(
            "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        for (key, value) in valid_metadata_entries() {
            if key != omitted_key {
                connection.execute(
                    "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                    [key, value],
                )?;
            }
        }
        Ok(())
    }

    fn create_contract_database_with_override(
        path: &PathBuf,
        override_key: &str,
        override_value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        connection.execute(
            "CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        insert_contract_metadata(&connection, Some((override_key, override_value)))?;
        Ok(())
    }

    fn insert_contract_metadata(
        connection: &Connection,
        override_entry: Option<(&str, &str)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (key, mut value) in valid_metadata_entries() {
            if let Some((override_key, override_value)) = override_entry
                && key == override_key
            {
                value = override_value;
            }
            connection.execute(
                "INSERT INTO artifact_metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )?;
        }
        Ok(())
    }

    fn valid_metadata_entries() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
                ARTIFACT_CONTRACT_VERSION,
            ),
            (
                artifact_metadata_keys::SCHEMA_VERSION,
                ARTIFACT_SCHEMA_VERSION,
            ),
            (artifact_metadata_keys::SOURCE_KIND, "foundry-pf2e"),
            (
                artifact_metadata_keys::SOURCE_SIGNATURE,
                "foundry-pf2e:fixture",
            ),
            (artifact_metadata_keys::SOURCE_RECORD_COUNT, "3"),
            (artifact_metadata_keys::CONTENT_HASH_ALGORITHM, "sha256"),
            (
                artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
                "transformers-js-minilm",
            ),
            (
                artifact_metadata_keys::EMBEDDING_MODEL_ID,
                EXPECTED_EMBEDDING_MODEL_ID,
            ),
            (artifact_metadata_keys::EMBEDDING_MODEL_REVISION, "main"),
            (
                artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
                EXPECTED_EMBEDDING_MODEL_ID,
            ),
            (artifact_metadata_keys::EMBEDDING_POOLING, "mean"),
            (artifact_metadata_keys::EMBEDDING_NORMALIZATION, "l2"),
            (artifact_metadata_keys::EMBEDDING_DIMENSIONS, "384"),
            (artifact_metadata_keys::EMBEDDING_DTYPE, "f32"),
            (artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC, "cosine"),
            (artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX, ""),
            (artifact_metadata_keys::EMBEDDING_QUERY_PREFIX, ""),
            (
                artifact_metadata_keys::FTS_TOKENIZER,
                "unicode61 remove_diacritics 2",
            ),
            (
                artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
                "manifest.json",
            ),
        ]
    }

    fn temp_db_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "atlas-index-{name}-{}-{}.sqlite",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = fs::remove_file(&path);
        path
    }
}
