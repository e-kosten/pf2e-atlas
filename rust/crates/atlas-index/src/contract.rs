use std::collections::BTreeMap;
use std::mem::size_of;

use atlas_artifact::metadata::artifact_metadata_keys;
use atlas_artifact::schema::{
    BOOLEAN_COLUMNS, REQUIRED_COLUMNS, REQUIRED_REFERENCES, REQUIRED_TABLES,
    TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORDS, TABLE_RECORDS_FTS, invalid_boolean_column_sql,
    orphan_reference_sql,
};
use atlas_embedding::EmbeddingUnitKind;
use atlas_record::ContentDocument;
use rusqlite::Connection;

use crate::sql::{count_rows, count_sql, table_columns, table_exists};
use crate::{
    ArtifactContractFamily, ArtifactValidationDiagnostic, IndexValidationError, ValidationCode,
};

pub(crate) fn validate_artifact_contract(
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

    validate_record_counts(connection, metadata, &mut diagnostics)?;
    validate_foreign_keys(connection, &mut diagnostics)?;
    validate_boolean_columns(connection, &mut diagnostics)?;
    validate_metric_values(connection, &mut diagnostics)?;
    validate_content_json(connection, &mut diagnostics)?;
    validate_fts_coverage(connection, &mut diagnostics)?;
    validate_document_embedding_cache(connection, metadata, &mut diagnostics)?;
    validate_relationships(connection, &mut diagnostics)?;
    validate_metric_catalogs(connection, &mut diagnostics)?;
    Ok(diagnostics)
}

fn validate_content_json(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let mut invalid = 0;
    let mut statement = connection
        .prepare(
            "SELECT record_key, description_json, blurb_json
             FROM records
             WHERE description_json IS NOT NULL OR blurb_json IS NOT NULL",
        )
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?
    {
        let description: Option<String> = row
            .get("description_json")
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        let blurb: Option<String> = row
            .get("blurb_json")
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        invalid += usize::from(!content_json_is_valid(description.as_deref()));
        invalid += usize::from(!content_json_is_valid(blurb.as_deref()));
    }

    let mut statement = connection
        .prepare("SELECT content_json FROM record_content")
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?
    {
        let content_json: String = row
            .get("content_json")
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        invalid += usize::from(!content_json_is_valid(Some(&content_json)));
    }

    if invalid > 0 {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Data,
            "content JSON must deserialize as ContentDocument".to_string(),
            Some("content:json".to_string()),
            Some("valid ContentDocument JSON".to_string()),
            Some(format!("{invalid} invalid documents")),
        ));
    }
    Ok(())
}

fn content_json_is_valid(value: Option<&str>) -> bool {
    value
        .map(|value| serde_json::from_str::<ContentDocument>(value).is_ok())
        .unwrap_or(true)
}

fn validate_document_embedding_cache(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let cache_rows = count_rows(connection, TABLE_DOCUMENT_EMBEDDING_CACHE)?;
    if cache_rows == 0 {
        return Ok(());
    }

    let default_visible_records = count_sql(
        connection,
        "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
    )?;
    let parent_units = count_sql(
        connection,
        "SELECT COUNT(*) FROM document_embedding_cache WHERE unit_kind = 'parent'",
    )?;
    if parent_units != default_visible_records {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Embedding,
            "document embedding cache parent unit count must match default-visible record count"
                .to_string(),
            Some("document_embedding_cache:default_visible_count".to_string()),
            Some(default_visible_records.to_string()),
            Some(parent_units.to_string()),
        ));
    }

    let expected_dimensions = metadata
        .get(artifact_metadata_keys::EMBEDDING_DIMENSIONS)
        .and_then(|value| value.parse::<usize>().ok());
    if let Some(expected_dimensions) = expected_dimensions {
        let invalid_dimensions = count_sql(
            connection,
            &format!(
                "SELECT COUNT(*) FROM document_embedding_cache WHERE dimensions <> {expected_dimensions}"
            ),
        )?;
        if invalid_dimensions > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Embedding,
                "document embedding cache dimensions must match embedding metadata".to_string(),
                Some("document_embedding_cache:dimensions".to_string()),
                Some(expected_dimensions.to_string()),
                Some(format!("{invalid_dimensions} invalid rows")),
            ));
        }

        let expected_bytes = expected_dimensions * size_of::<f32>();
        let invalid_blob_bytes = count_sql(
            connection,
            &format!(
                "SELECT COUNT(*) FROM document_embedding_cache WHERE length(vector_blob) <> {expected_bytes}"
            ),
        )?;
        if invalid_blob_bytes > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Embedding,
                "document embedding cache vector blob length must match embedding dimensions"
                    .to_string(),
                Some("document_embedding_cache:vector_blob".to_string()),
                Some(format!("{expected_bytes} bytes")),
                Some(format!("{invalid_blob_bytes} invalid rows")),
            ));
        }
    }

    for (key, sql, expected) in [
        (
            "document_embedding_cache:hidden_rows",
            "SELECT COUNT(*)
             FROM document_embedding_cache e
             JOIN records r ON r.record_key = e.record_key
             WHERE r.is_default_visible <> 1",
            "no hidden records in document embedding cache",
        ),
        (
            "document_embedding_cache:missing_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM records WHERE is_default_visible = 1
               EXCEPT
               SELECT record_key FROM document_embedding_cache WHERE unit_kind = 'parent'
             )",
            "every default-visible record has a parent document embedding",
        ),
        (
            "document_embedding_cache:stale_rows",
            "SELECT COUNT(*)
             FROM (
               SELECT record_key FROM document_embedding_cache WHERE unit_kind = 'parent'
               EXCEPT
               SELECT record_key FROM records WHERE is_default_visible = 1
             )",
            "every parent document embedding row belongs to a default-visible record",
        ),
        (
            "document_embedding_cache:invalid_unit_kind",
            &valid_embedding_unit_kind_sql(),
            "every document embedding row has a known unit kind",
        ),
    ] {
        let invalid = count_sql(connection, sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Embedding,
                format!("document embedding cache coverage check `{key}` failed"),
                Some(key.to_string()),
                Some(expected.to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }

    Ok(())
}

fn valid_embedding_unit_kind_sql() -> String {
    format!(
        "SELECT COUNT(*)
         FROM document_embedding_cache
         WHERE unit_kind NOT IN ('{}', '{}', '{}')",
        EmbeddingUnitKind::Parent.as_str(),
        EmbeddingUnitKind::HeadingSection.as_str(),
        EmbeddingUnitKind::TitledOption.as_str()
    )
}

fn validate_required_tables(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for table in REQUIRED_TABLES {
        let table_name = table.name();
        if !table_exists(connection, table_name)? {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Schema,
                format!("required artifact table `{table_name}` is missing"),
                Some(format!("table:{table_name}")),
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
        let table_name = table.name();
        let present_columns = table_columns(connection, table_name)?;
        for column in *columns {
            let column_name = column.name();
            if !present_columns.contains_key(column_name) {
                diagnostics.push(contract_diagnostic(
                    ArtifactContractFamily::Schema,
                    format!("required artifact column `{table_name}.{column_name}` is missing"),
                    Some(format!("column:{table_name}.{column_name}")),
                    Some("present".to_string()),
                    Some("missing".to_string()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_record_counts(
    connection: &Connection,
    metadata: &BTreeMap<String, String>,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let source_count = metadata
        .get(artifact_metadata_keys::SOURCE_RECORD_COUNT)
        .and_then(|value| value.parse::<usize>().ok());
    let artifact_count = metadata
        .get(artifact_metadata_keys::ARTIFACT_RECORD_COUNT)
        .and_then(|value| value.parse::<usize>().ok());
    let generated_count = metadata
        .get(artifact_metadata_keys::GENERATED_RECORD_COUNT)
        .and_then(|value| value.parse::<usize>().ok());
    let actual_artifact_count = count_rows(connection, TABLE_RECORDS)?;
    if artifact_count != Some(actual_artifact_count) {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Source,
            "metadata artifact_record_count must match the records table".to_string(),
            Some(artifact_metadata_keys::ARTIFACT_RECORD_COUNT.to_string()),
            artifact_count.map(|value| value.to_string()),
            Some(actual_artifact_count.to_string()),
        ));
    }
    if let (Some(source_count), Some(generated_count), Some(artifact_count)) =
        (source_count, generated_count, artifact_count)
        && source_count + generated_count != artifact_count
    {
        diagnostics.push(contract_diagnostic(
            ArtifactContractFamily::Source,
            "metadata source_record_count plus generated_record_count must match artifact_record_count"
                .to_string(),
            Some("record_counts".to_string()),
            Some(artifact_count.to_string()),
            Some((source_count + generated_count).to_string()),
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
    for check in BOOLEAN_COLUMNS {
        let sql = invalid_boolean_column_sql(check);
        let invalid = count_sql(connection, &sql)?;
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
    let fts_rows = count_rows(connection, TABLE_RECORDS_FTS)?;
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
    for reference in REQUIRED_REFERENCES {
        let sql = orphan_reference_sql(reference);
        let invalid = count_sql(connection, &sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("relationship check `{}` failed", reference.key),
                Some(reference.key.to_string()),
                Some("0 invalid rows".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    for (key, sql) in RELATIONSHIP_POLICY_CHECKS {
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

pub(crate) fn contract_diagnostic(
    family: ArtifactContractFamily,
    message: String,
    key: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
) -> ArtifactValidationDiagnostic {
    contract_diagnostic_with_code(
        family,
        message,
        key,
        expected,
        actual,
        ValidationCode::ArtifactContractViolation,
    )
}

pub(crate) fn contract_diagnostic_with_code(
    family: ArtifactContractFamily,
    message: String,
    key: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    code: ValidationCode,
) -> ArtifactValidationDiagnostic {
    ArtifactValidationDiagnostic {
        code,
        family,
        message,
        key,
        expected,
        actual,
    }
}

const RELATIONSHIP_POLICY_CHECKS: &[(&str, &str)] = &[
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
