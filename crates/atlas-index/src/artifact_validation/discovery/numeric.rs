use atlas_discovery::all_discovery_field_definitions;
use atlas_domain::FilterValuePolicy;
use rusqlite::{Connection, params};

use crate::{ArtifactValidationDiagnostic, IndexValidationError, sql::count_sql};

use super::{
    discovery_diagnostic, push_duplicate_diagnostic, push_row_diagnostics, query_count_pair,
};

pub(super) fn validate_numeric_catalog(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_numeric_catalog_uniqueness(connection, diagnostics)?;
    validate_numeric_catalog_coverage(connection, diagnostics)?;
    validate_numeric_catalog_bounds(connection, diagnostics)?;
    Ok(())
}

fn validate_numeric_catalog_uniqueness(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let duplicates = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM (
           SELECT field,
                  COALESCE(record_family, '<global>') AS scope,
                  COALESCE(metric_domain, '<none>') AS metric_domain_key,
                  COALESCE(metric_key, '<none>') AS metric_key_key
           FROM filter_numeric_catalog
           GROUP BY field, scope, metric_domain_key, metric_key_key
           HAVING COUNT(*) > 1
         )",
    )?;
    push_duplicate_diagnostic(diagnostics, "filter_numeric_catalog", duplicates);
    Ok(())
}

fn validate_numeric_catalog_coverage(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let mut missing = 0_u64;
    let mut stale = 0_u64;
    for definition in all_discovery_field_definitions()
        .iter()
        .filter(|definition| definition.value_policy == FilterValuePolicy::NumericStats)
    {
        let (definition_missing, definition_stale) =
            numeric_field_catalog_diff(connection, definition.field, definition.value_sql)?;
        missing += definition_missing;
        stale += definition_stale;
    }
    let (metric_missing, metric_stale) = metric_numeric_catalog_diff(connection)?;
    missing += metric_missing;
    stale += metric_stale;
    push_row_diagnostics(
        diagnostics,
        "filter_numeric_catalog",
        missing,
        stale,
        "numeric stats",
    );
    Ok(())
}

fn numeric_field_catalog_diff(
    connection: &Connection,
    field: &str,
    value_sql: &str,
) -> Result<(u64, u64), IndexValidationError> {
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql}),
              scoped AS (
                SELECT NULL AS record_family, CAST(value AS REAL) AS value
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1 AND value IS NOT NULL
                UNION ALL
                SELECT r.record_family, CAST(value AS REAL) AS value
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1 AND value IS NOT NULL
              ),
              ordered AS (
                SELECT record_family, value,
                       ROW_NUMBER() OVER (PARTITION BY record_family ORDER BY value ASC) AS rn,
                       COUNT(*) OVER (PARTITION BY record_family) AS n
                FROM scoped
              ),
              expected AS (
                SELECT record_family,
                       COUNT(*) AS catalog_count,
                       CASE
                         WHEN record_family IS NULL THEN
                           (SELECT COUNT(*) FROM records WHERE is_default_visible = 1)
                         ELSE
                           (SELECT COUNT(*) FROM records WHERE is_default_visible = 1
                              AND records.record_family = ordered.record_family)
                       END - COUNT(*) AS null_count,
                       MIN(value) AS min,
                       MAX(CASE WHEN rn = CAST(n * 0.05 + 0.999999999 AS INTEGER) THEN value END) AS p05,
                       MAX(CASE WHEN rn = CAST(n * 0.25 + 0.999999999 AS INTEGER) THEN value END) AS p25,
                       MAX(CASE WHEN rn = CAST(n * 0.50 + 0.999999999 AS INTEGER) THEN value END) AS p50,
                       AVG(value) AS mean,
                       MAX(CASE WHEN rn = CAST(n * 0.75 + 0.999999999 AS INTEGER) THEN value END) AS p75,
                       MAX(CASE WHEN rn = CAST(n * 0.95 + 0.999999999 AS INTEGER) THEN value END) AS p95,
                       MAX(value) AS max
                FROM ordered
                GROUP BY record_family
              ),
              actual AS (
                SELECT record_family, catalog_count, null_count, min, p05, p25, p50,
                       mean, p75, p95, max
                FROM filter_numeric_catalog
                WHERE field = ?1 AND metric_domain IS NULL AND metric_key IS NULL
              )
         SELECT
           (SELECT COUNT(*)
            FROM expected e
            LEFT JOIN actual a
              ON ((a.record_family IS NULL AND e.record_family IS NULL)
                  OR a.record_family = e.record_family)
            WHERE a.catalog_count IS NULL),
           (SELECT COUNT(*)
            FROM actual a
            LEFT JOIN expected e
              ON ((a.record_family IS NULL AND e.record_family IS NULL)
                  OR a.record_family = e.record_family)
            WHERE e.catalog_count IS NULL
               OR a.catalog_count <> e.catalog_count
               OR a.null_count <> e.null_count
               OR ABS(a.min - e.min) > 0.000001
               OR ABS(a.p05 - e.p05) > 0.000001
               OR ABS(a.p25 - e.p25) > 0.000001
               OR ABS(a.p50 - e.p50) > 0.000001
               OR ABS(a.mean - e.mean) > 0.000001
               OR ABS(a.p75 - e.p75) > 0.000001
               OR ABS(a.p95 - e.p95) > 0.000001
               OR ABS(a.max - e.max) > 0.000001)"
    );
    query_count_pair(connection, &sql, params![field])
}

fn metric_numeric_catalog_diff(
    connection: &Connection,
) -> Result<(u64, u64), IndexValidationError> {
    let sql = "
        WITH scoped AS (
          SELECT NULL AS record_family, rm.metric_domain, rm.metric_key, rm.number_value AS value
          FROM record_metrics rm
          JOIN records r ON r.record_key = rm.record_key
          WHERE r.is_default_visible = 1
            AND rm.value_type = 'number'
            AND rm.number_value IS NOT NULL
          UNION ALL
          SELECT r.record_family, rm.metric_domain, rm.metric_key, rm.number_value AS value
          FROM record_metrics rm
          JOIN records r ON r.record_key = rm.record_key
          WHERE r.is_default_visible = 1
            AND rm.value_type = 'number'
            AND rm.number_value IS NOT NULL
        ),
        ordered AS (
          SELECT record_family, metric_domain, metric_key, value,
                 ROW_NUMBER() OVER (
                   PARTITION BY record_family, metric_domain, metric_key
                   ORDER BY value ASC
                 ) AS rn,
                 COUNT(*) OVER (
                   PARTITION BY record_family, metric_domain, metric_key
                 ) AS n
          FROM scoped
        ),
        expected AS (
          SELECT record_family, metric_domain, metric_key,
                 COUNT(*) AS catalog_count,
                 CASE
                   WHEN record_family IS NULL THEN
                     (SELECT COUNT(*) FROM records WHERE is_default_visible = 1)
                   ELSE
                     (SELECT COUNT(*) FROM records WHERE is_default_visible = 1
                        AND records.record_family = ordered.record_family)
                 END - COUNT(*) AS null_count,
                 MIN(value) AS min,
                 MAX(CASE WHEN rn = CAST(n * 0.05 + 0.999999999 AS INTEGER) THEN value END) AS p05,
                 MAX(CASE WHEN rn = CAST(n * 0.25 + 0.999999999 AS INTEGER) THEN value END) AS p25,
                 MAX(CASE WHEN rn = CAST(n * 0.50 + 0.999999999 AS INTEGER) THEN value END) AS p50,
                 AVG(value) AS mean,
                 MAX(CASE WHEN rn = CAST(n * 0.75 + 0.999999999 AS INTEGER) THEN value END) AS p75,
                 MAX(CASE WHEN rn = CAST(n * 0.95 + 0.999999999 AS INTEGER) THEN value END) AS p95,
                 MAX(value) AS max
          FROM ordered
          GROUP BY record_family, metric_domain, metric_key
        ),
        actual AS (
          SELECT record_family, metric_domain, metric_key, catalog_count, null_count,
                 min, p05, p25, p50, mean, p75, p95, max
          FROM filter_numeric_catalog
          WHERE field = 'metric'
        )
        SELECT
          (SELECT COUNT(*)
           FROM expected e
           LEFT JOIN actual a
             ON ((a.record_family IS NULL AND e.record_family IS NULL)
                 OR a.record_family = e.record_family)
            AND a.metric_domain = e.metric_domain
            AND a.metric_key = e.metric_key
           WHERE a.metric_key IS NULL),
          (SELECT COUNT(*)
           FROM actual a
           LEFT JOIN expected e
             ON ((a.record_family IS NULL AND e.record_family IS NULL)
                 OR a.record_family = e.record_family)
            AND a.metric_domain = e.metric_domain
            AND a.metric_key = e.metric_key
           WHERE e.metric_key IS NULL
              OR a.catalog_count <> e.catalog_count
              OR a.null_count <> e.null_count
              OR ABS(a.min - e.min) > 0.000001
              OR ABS(a.p05 - e.p05) > 0.000001
              OR ABS(a.p25 - e.p25) > 0.000001
              OR ABS(a.p50 - e.p50) > 0.000001
              OR ABS(a.mean - e.mean) > 0.000001
              OR ABS(a.p75 - e.p75) > 0.000001
              OR ABS(a.p95 - e.p95) > 0.000001
              OR ABS(a.max - e.max) > 0.000001)";
    query_count_pair(connection, sql, [])
}

fn validate_numeric_catalog_bounds(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let invalid = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM filter_numeric_catalog
         WHERE catalog_count > 0
           AND NOT (
             min <= p05 AND p05 <= p25 AND p25 <= p50
             AND p50 <= p75 AND p75 <= p95 AND p95 <= max
           )",
    )?;
    if invalid > 0 {
        diagnostics.push(discovery_diagnostic(
            "filter_numeric_catalog.bounds",
            format!("{invalid} numeric catalog rows have incoherent bounds"),
        ));
    }
    Ok(())
}
