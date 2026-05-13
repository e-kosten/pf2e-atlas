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
    artifact_schema::{REQUIRED_COLUMNS, REQUIRED_TABLES},
};
use rusqlite::{Connection, OpenFlags};
use thiserror::Error;

pub mod inspect;
pub use inspect::{
    IndexInspectionReport, MetricCoverageReport, RecordCoverageReport, RelationshipCoverageReport,
    TaxonomyCoverageReport, TextCoverageReport, VariantCoverageReport, inspect_index,
};

#[derive(Debug, Error)]
pub enum IndexValidationError {
    #[error("index is unavailable: {0}")]
    Unavailable(String),
    #[error("index query failed: {0}")]
    QueryFailed(String),
    #[error("index artifact metadata is invalid: {0}")]
    InvalidArtifact(String),
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
    let diagnostics = if diagnostics.is_empty() {
        validate_artifact_contract(&connection, &metadata)?
    } else {
        diagnostics
    };
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

fn validate_artifact_contract(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
) -> Result<Vec<ArtifactValidationDiagnostic>, IndexValidationError> {
    let mut diagnostics = Vec::new();
    validate_required_tables(connection, &mut diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(diagnostics);
    }

    validate_required_columns(connection, &mut diagnostics)?;
    if !diagnostics.is_empty() {
        return Ok(diagnostics);
    }

    validate_source_record_count(connection, metadata, &mut diagnostics)?;
    validate_foreign_keys(connection, &mut diagnostics)?;
    validate_boolean_columns(connection, &mut diagnostics)?;
    validate_metric_values(connection, &mut diagnostics)?;
    validate_fts_coverage(connection, &mut diagnostics)?;
    validate_relationships(connection, &mut diagnostics)?;
    validate_metric_catalogs(connection, &mut diagnostics)?;
    Ok(diagnostics)
}

fn validate_required_tables(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for table in REQUIRED_TABLES {
        if !table_exists(connection, table)? {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Schema,
                format!("required artifact table `{table}` is missing"),
                Some(format!("table:{table}")),
                Some("present".to_string()),
                Some("missing".to_string()),
            ));
        }
    }
    Ok(())
}

fn validate_required_columns(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (table, columns) in REQUIRED_COLUMNS {
        let present_columns = table_columns(connection, table)?;
        for column in *columns {
            if !present_columns.contains_key(*column) {
                diagnostics.push(contract_diagnostic(
                    ArtifactContractFamily::Schema,
                    format!("required artifact column `{table}.{column}` is missing"),
                    Some(format!("column:{table}.{column}")),
                    Some("present".to_string()),
                    Some("missing".to_string()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_source_record_count(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let expected = metadata
        .get(artifact_metadata_keys::SOURCE_RECORD_COUNT)
        .and_then(|value| value.parse::<usize>().ok());
    let actual = count_rows(connection, "records")?;
    if expected != Some(actual) {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Source,
            "metadata source_record_count must match the records table".to_string(),
            Some(artifact_metadata_keys::SOURCE_RECORD_COUNT.to_string()),
            expected.map(|value| value.to_string()),
            Some(actual.to_string()),
        ));
    }
    Ok(())
}

fn validate_foreign_keys(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let mut statement = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |_| Ok(()))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut count = 0;
    for row in rows {
        row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        count += 1;
    }
    if count > 0 {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Data,
            "artifact contains foreign key violations".to_string(),
            Some("foreign_key_check".to_string()),
            Some("0".to_string()),
            Some(count.to_string()),
        ));
    }
    Ok(())
}

fn validate_boolean_columns(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for check in BOOLEAN_COLUMN_CHECKS {
        let invalid = count_sql(connection, check.sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("boolean column `{}` contains non-boolean values", check.key),
                Some(check.key.to_string()),
                Some("0 or 1".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}

fn validate_metric_values(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (key, sql) in [
        (
            "record_metrics:number_value",
            "SELECT COUNT(*) FROM record_metrics
             WHERE value_type = 'number'
               AND (number_value IS NULL OR text_value IS NOT NULL OR bool_value IS NOT NULL)",
        ),
        (
            "record_metrics:text_value",
            "SELECT COUNT(*) FROM record_metrics
             WHERE value_type = 'text'
               AND (text_value IS NULL OR number_value IS NOT NULL OR bool_value IS NOT NULL)",
        ),
        (
            "record_metrics:bool_value",
            "SELECT COUNT(*) FROM record_metrics
             WHERE value_type = 'boolean'
               AND (bool_value IS NULL OR number_value IS NOT NULL OR text_value IS NOT NULL)",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("metric value shape `{key}` is inconsistent with value_type"),
                Some(key.to_string()),
                Some("exactly one matching value column".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}

fn validate_fts_coverage(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let fts_rows = count_rows(connection, "records_fts")?;
    let default_visible_records = count_sql(
        connection,
        "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
    )?;
    if fts_rows != default_visible_records {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Fts,
            "FTS row count must match default-visible record count".to_string(),
            Some("records_fts:default_visible_count".to_string()),
            Some(default_visible_records.to_string()),
            Some(fts_rows.to_string()),
        ));
    }

    for (key, sql, expected) in [
        (
            "records_fts:orphan_rows",
            "SELECT COUNT(*)
             FROM records_fts f
             LEFT JOIN records r ON r.record_key = f.record_key
             WHERE r.record_key IS NULL",
            "no orphan FTS rows",
        ),
        (
            "records_fts:hidden_rows",
            "SELECT COUNT(*)
             FROM records_fts f
             JOIN records r ON r.record_key = f.record_key
             WHERE r.is_default_visible <> 1",
            "no hidden records in FTS",
        ),
        (
            "records_fts:duplicate_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM records_fts GROUP BY record_key HAVING COUNT(*) > 1
             )",
            "at most one FTS row per record",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Fts,
                format!("FTS coverage check `{key}` failed"),
                Some(key.to_string()),
                Some(expected.to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}

fn validate_relationships(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (key, sql) in RELATIONSHIP_CHECKS {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("relationship check `{key}` failed"),
                Some(key.to_string()),
                Some("0 invalid rows".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}

fn validate_metric_catalogs(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (key, sql) in METRIC_CATALOG_CHECKS {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("metric catalog coverage check `{key}` failed"),
                Some(key.to_string()),
                Some("catalog rows match default-visible metrics".to_string()),
                Some(format!("{invalid} mismatched rows")),
            ));
        }
    }
    Ok(())
}

fn contract_diagnostic(
    family: ArtifactContractFamily,
    message: String,
    key: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
) -> ArtifactValidationDiagnostic {
    ArtifactValidationDiagnostic {
        code: ValidationCode::ArtifactContractViolation,
        family,
        message,
        key,
        expected,
        actual,
    }
}

fn table_columns(
    connection: &Connection,
    table: &str,
) -> Result<BTreeMap<String, String>, IndexValidationError> {
    let sql = format!("PRAGMA table_xinfo({table})");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut columns = BTreeMap::new();
    for row in rows {
        let (name, column_type) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        columns.insert(name, column_type);
    }
    Ok(columns)
}

struct BooleanColumnCheck {
    key: &'static str,
    sql: &'static str,
}

const BOOLEAN_COLUMN_CHECKS: &[BooleanColumnCheck] = &[
    BooleanColumnCheck {
        key: "records.publication_remaster",
        sql: "SELECT COUNT(*) FROM records WHERE publication_remaster NOT IN (0, 1)",
    },
    BooleanColumnCheck {
        key: "records.is_default_visible",
        sql: "SELECT COUNT(*) FROM records WHERE is_default_visible NOT IN (0, 1)",
    },
    BooleanColumnCheck {
        key: "record_metrics.bool_value",
        sql: "SELECT COUNT(*) FROM record_metrics WHERE bool_value IS NOT NULL AND bool_value NOT IN (0, 1)",
    },
    BooleanColumnCheck {
        key: "actor_records.is_complex",
        sql: "SELECT COUNT(*) FROM actor_records WHERE is_complex NOT IN (0, 1)",
    },
    BooleanColumnCheck {
        key: "spell_records.sustained",
        sql: "SELECT COUNT(*) FROM spell_records WHERE sustained NOT IN (0, 1)",
    },
    BooleanColumnCheck {
        key: "spell_records.basic_save",
        sql: "SELECT COUNT(*) FROM spell_records WHERE basic_save NOT IN (0, 1)",
    },
];

const RELATIONSHIP_CHECKS: &[(&str, &str)] = &[
    (
        "records.pack_name",
        "SELECT COUNT(*)
         FROM records r
         LEFT JOIN packs p ON p.name = r.pack_name
         WHERE p.name IS NULL",
    ),
    (
        "record_traits.record_key",
        "SELECT COUNT(*)
         FROM record_traits t
         LEFT JOIN records r ON r.record_key = t.record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "reference_edges.from_record_key",
        "SELECT COUNT(*)
         FROM reference_edges e
         LEFT JOIN records r ON r.record_key = e.from_record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "reference_edges.to_record_key",
        "SELECT COUNT(*)
         FROM reference_edges e
         LEFT JOIN records r ON r.record_key = e.to_record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "record_aliases.canonical_record_key",
        "SELECT COUNT(*)
         FROM record_aliases a
         LEFT JOIN records r ON r.record_key = a.canonical_record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "remaster_links.remaster_record_key",
        "SELECT COUNT(*)
         FROM remaster_links l
         LEFT JOIN records r ON r.record_key = l.remaster_record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "remaster_links.legacy_record_key",
        "SELECT COUNT(*)
         FROM remaster_links l
         LEFT JOIN records r ON r.record_key = l.legacy_record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "record_metrics.record_key",
        "SELECT COUNT(*)
         FROM record_metrics m
         LEFT JOIN records r ON r.record_key = m.record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "actor_records.record_key",
        "SELECT COUNT(*)
         FROM actor_records a
         LEFT JOIN records r ON r.record_key = a.record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "item_records.record_key",
        "SELECT COUNT(*)
         FROM item_records i
         LEFT JOIN records r ON r.record_key = i.record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "spell_records.record_key",
        "SELECT COUNT(*)
         FROM spell_records s
         LEFT JOIN records r ON r.record_key = s.record_key
         WHERE r.record_key IS NULL",
    ),
    (
        "remaster_links.legacy_visibility",
        "SELECT COUNT(*)
         FROM remaster_links l
         JOIN records legacy ON legacy.record_key = l.legacy_record_key
         WHERE legacy.is_default_visible <> 0",
    ),
    (
        "remaster_links.remaster_visibility",
        "SELECT COUNT(*)
         FROM remaster_links l
         JOIN records remaster ON remaster.record_key = l.remaster_record_key
         WHERE remaster.is_default_visible <> 1",
    ),
];

const METRIC_CATALOG_CHECKS: &[(&str, &str)] = &[
    (
        "metric_key_catalog.missing_keys",
        "SELECT COUNT(*)
         FROM (
           SELECT rm.metric_domain, r.record_family, rm.metric_key
           FROM record_metrics rm
           JOIN records r ON r.record_key = rm.record_key
           WHERE r.is_default_visible = 1
           GROUP BY rm.metric_domain, r.record_family, rm.metric_key
           EXCEPT
           SELECT metric_domain, record_family, metric_key FROM metric_key_catalog
         )",
    ),
    (
        "metric_key_catalog.stale_keys",
        "SELECT COUNT(*)
         FROM (
           SELECT metric_domain, record_family, metric_key FROM metric_key_catalog
           EXCEPT
           SELECT rm.metric_domain, r.record_family, rm.metric_key
           FROM record_metrics rm
           JOIN records r ON r.record_key = rm.record_key
           WHERE r.is_default_visible = 1
           GROUP BY rm.metric_domain, r.record_family, rm.metric_key
         )",
    ),
    (
        "metric_value_catalog.missing_values",
        "SELECT COUNT(*)
         FROM (
           SELECT
             rm.metric_domain,
             r.record_family,
             rm.metric_key,
             CASE
               WHEN rm.value_type = 'text' THEN rm.text_value
               WHEN rm.value_type = 'boolean' THEN CAST(rm.bool_value AS TEXT)
               ELSE NULL
             END AS value
           FROM record_metrics rm
           JOIN records r ON r.record_key = rm.record_key
           WHERE r.is_default_visible = 1
             AND rm.value_type IN ('text', 'boolean')
             AND value IS NOT NULL
           GROUP BY rm.metric_domain, r.record_family, rm.metric_key, value
           EXCEPT
           SELECT metric_domain, record_family, metric_key, value FROM metric_value_catalog
         )",
    ),
    (
        "metric_value_catalog.stale_values",
        "SELECT COUNT(*)
         FROM (
           SELECT metric_domain, record_family, metric_key, value FROM metric_value_catalog
           EXCEPT
           SELECT
             rm.metric_domain,
             r.record_family,
             rm.metric_key,
             CASE
               WHEN rm.value_type = 'text' THEN rm.text_value
               WHEN rm.value_type = 'boolean' THEN CAST(rm.bool_value AS TEXT)
               ELSE NULL
             END AS value
           FROM record_metrics rm
           JOIN records r ON r.record_key = rm.record_key
           WHERE r.is_default_visible = 1
             AND rm.value_type IN ('text', 'boolean')
             AND value IS NOT NULL
           GROUP BY rm.metric_domain, r.record_family, rm.metric_key, value
         )",
    ),
];

fn count_rows(connection: &Connection, table: &str) -> Result<usize, IndexValidationError> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    count_sql(connection, &sql)
}

fn count_sql(connection: &Connection, sql: &str) -> Result<usize, IndexValidationError> {
    connection
        .query_row(sql, [], |row| row.get::<_, usize>(0))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
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

    #[test]
    fn reports_missing_required_artifact_table() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("missing-contract-table");
        let connection = Connection::open(&path)?;
        create_minimal_contract_schema(&connection)?;
        insert_contract_metadata(&connection, None)?;
        connection.execute("DROP TABLE item_records", [])?;
        drop(connection);

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(
            report.diagnostics[0].family,
            atlas_domain::ArtifactContractFamily::Schema
        );
        assert_eq!(
            report.diagnostics[0].key.as_deref(),
            Some("table:item_records")
        );
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn reports_fts_rows_for_hidden_records() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_db_path("hidden-fts");
        create_contract_database(&path)?;
        let connection = Connection::open(&path)?;
        connection.execute(
            "UPDATE records SET is_default_visible = 0 WHERE record_key = 'actions:testAction1'",
            [],
        )?;
        drop(connection);

        let report = validate_index(&path)?;

        assert_eq!(report.status, ValidationStatus::Error);
        assert_eq!(report.code, ValidationCode::ArtifactContractViolation);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.family == atlas_domain::ArtifactContractFamily::Fts
                && diagnostic.key.as_deref() == Some("records_fts:hidden_rows")
        }));
        fs::remove_file(path)?;
        Ok(())
    }

    fn create_contract_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        create_minimal_contract_schema(&connection)?;
        insert_contract_metadata(&connection, None)?;
        insert_minimal_contract_rows(&connection)?;
        Ok(())
    }

    fn create_contract_database_without(
        path: &PathBuf,
        omitted_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = Connection::open(path)?;
        create_minimal_contract_schema(&connection)?;
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
        create_minimal_contract_schema(&connection)?;
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

    fn create_minimal_contract_schema(
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute_batch(
            "
            CREATE TABLE artifact_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE packs (
              name TEXT PRIMARY KEY, label TEXT, document_type TEXT, declared_path TEXT,
              resolved_path TEXT, record_count INTEGER
            );
            CREATE TABLE records (
              record_key TEXT PRIMARY KEY, id TEXT, name TEXT, normalized_name TEXT,
              record_family TEXT, pack_name TEXT, pack_label TEXT, foundry_document_type TEXT,
              foundry_record_type TEXT, level INTEGER, rarity TEXT, traits_json TEXT,
              system_category TEXT, system_group TEXT, system_base_item TEXT, system_usage TEXT,
              system_price_json TEXT, system_actions_value INTEGER, system_time_value TEXT,
              system_duration_value TEXT, price_cp INTEGER, activation_time_kind TEXT,
              activation_time_actions INTEGER, activation_time_duration_value INTEGER,
              activation_time_duration_unit TEXT, activation_time_text TEXT, duration_kind TEXT,
              duration_value INTEGER, duration_unit TEXT, duration_text TEXT,
              publication_title TEXT, publication_remaster INTEGER, description_text TEXT,
              blurb_text TEXT, description_snippet TEXT, publication_family TEXT, folder_id TEXT,
              taxonomy_families_json TEXT, variant_group_key TEXT, variant_base_name TEXT,
              variant_label TEXT, variant_axes_json TEXT, variant_confidence REAL,
              variant_source TEXT, source_path TEXT, is_default_visible INTEGER,
              search_text_projection TEXT, raw_json TEXT
            );
            CREATE TABLE record_traits (record_key TEXT, trait TEXT);
            CREATE TABLE reference_edges (
              from_record_key TEXT, to_record_key TEXT, display_text TEXT, reference_text TEXT
            );
            CREATE TABLE record_aliases (
              canonical_record_key TEXT, alias_text TEXT, normalized_alias TEXT,
              source_kind TEXT, source_ref TEXT
            );
            CREATE TABLE remaster_links (
              remaster_record_key TEXT, legacy_record_key TEXT, source_kind TEXT, source_ref TEXT
            );
            CREATE TABLE record_metrics (
              record_key TEXT, metric_domain TEXT, metric_key TEXT, value_type TEXT,
              number_value REAL, text_value TEXT, bool_value INTEGER
            );
            CREATE TABLE metric_key_catalog (
              metric_domain TEXT, record_family TEXT, namespace_prefix TEXT, metric_key TEXT,
              value_type TEXT, catalog_count INTEGER, numeric_min REAL, numeric_max REAL
            );
            CREATE TABLE metric_value_catalog (
              metric_domain TEXT, record_family TEXT, metric_key TEXT, value TEXT,
              catalog_count INTEGER
            );
            CREATE TABLE actor_records (
              record_key TEXT PRIMARY KEY, size TEXT, languages_json TEXT, speed_types_json TEXT,
              senses_json TEXT, immunities_json TEXT, resistances_json TEXT, weaknesses_json TEXT,
              disable_text TEXT, disable_skills_json TEXT, is_complex INTEGER
            );
            CREATE TABLE item_records (
              record_key TEXT PRIMARY KEY, system_category TEXT, system_base_item TEXT,
              system_group TEXT, system_usage TEXT, price_cp INTEGER, bulk_value REAL,
              hands_requirement TEXT, damage_types_json TEXT
            );
            CREATE TABLE spell_records (
              record_key TEXT PRIMARY KEY, traditions_json TEXT, spell_kinds_json TEXT,
              range_text TEXT, range_value REAL, target_text TEXT, area_type TEXT,
              area_value REAL, save_type TEXT, sustained INTEGER, basic_save INTEGER,
              damage_types_json TEXT
            );
            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED, name, search_text_projection
            );
            ",
        )?;
        Ok(())
    }

    fn insert_minimal_contract_rows(
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        connection.execute(
            "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES ('actions', 'Actions', 'Item', 'packs/actions', 'packs/actions', 3)",
            [],
        )?;
        for index in 1..=3 {
            let record_key = format!("actions:testAction{index}");
            let record_id = format!("testAction{index}");
            let name = format!("Test Action {index}");
            let normalized_name = name.to_lowercase();
            let source_path = format!("packs/actions/test-action-{index}.json");
            connection.execute(
                "INSERT INTO records (
                  record_key, id, name, normalized_name, record_family, pack_name, pack_label,
                  foundry_document_type, foundry_record_type, traits_json, publication_remaster,
                  publication_family, taxonomy_families_json, variant_axes_json, variant_source,
                  source_path, is_default_visible, search_text_projection, raw_json
                ) VALUES (?1, ?2, ?3, ?4, 'rule', 'actions', 'Actions', 'Item', 'action',
                  '[]', 0, 'unknown', '[]', '[]', 'none', ?5, 1, ?3, '{}')",
                [
                    record_key.as_str(),
                    record_id.as_str(),
                    name.as_str(),
                    normalized_name.as_str(),
                    source_path.as_str(),
                ],
            )?;
            connection.execute(
                "INSERT INTO records_fts (record_key, name, search_text_projection)
                 VALUES (?1, ?2, ?2)",
                [record_key.as_str(), name.as_str()],
            )?;
        }
        Ok(())
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
