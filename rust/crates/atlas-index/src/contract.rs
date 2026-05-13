use std::collections::BTreeMap;

use atlas_artifact::schema::{REQUIRED_COLUMNS, REQUIRED_TABLES};
use atlas_domain::{
    ArtifactContractFamily, ArtifactValidationDiagnostic, ValidationCode, artifact_metadata_keys,
};
use rusqlite::Connection;

use crate::IndexValidationError;
use crate::sql::{count_rows, count_sql, table_columns, table_exists};

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

pub(crate) fn contract_diagnostic(
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
