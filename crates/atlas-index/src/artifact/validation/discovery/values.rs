use atlas_domain::FilterValuePolicy;
use rusqlite::{Connection, params};

use crate::discovery::definitions::all_definitions;
use crate::{ArtifactValidationDiagnostic, IndexValidationError, sql::count_sql};

use super::{push_duplicate_diagnostic, push_row_diagnostics, query_count_pair};

pub(super) fn validate_value_catalog(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_value_catalog_uniqueness(connection, diagnostics)?;
    let mut missing = 0_u64;
    let mut stale = 0_u64;
    for definition in all_definitions()
        .iter()
        .filter(|definition| definition.value_policy == FilterValuePolicy::Enumerable)
    {
        let value_sql = definition.value_sql();
        let (definition_missing, definition_stale) = value_catalog_diff(
            connection,
            "filter_value_catalog",
            definition.field,
            &value_sql,
            None,
        )?;
        missing += definition_missing;
        stale += definition_stale;
    }
    push_row_diagnostics(
        diagnostics,
        "filter_value_catalog",
        missing,
        stale,
        "enumerable value",
    );
    Ok(())
}

fn validate_value_catalog_uniqueness(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let duplicates = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM (
           SELECT field, COALESCE(record_family, '<global>') AS scope, value
           FROM filter_value_catalog
           GROUP BY field, scope, value
           HAVING COUNT(*) > 1
         )",
    )?;
    push_duplicate_diagnostic(diagnostics, "filter_value_catalog", duplicates);
    Ok(())
}

pub(super) fn value_catalog_diff(
    connection: &Connection,
    table: &str,
    field: &str,
    value_sql: &str,
    limit_per_scope: Option<u64>,
) -> Result<(u64, u64), IndexValidationError> {
    let limit_predicate = limit_per_scope
        .map(|limit| format!("WHERE sample_rank <= {limit}"))
        .unwrap_or_default();
    let actual_rank_column = if limit_per_scope.is_some() {
        "sample_rank"
    } else {
        "NULL AS sample_rank"
    };
    let expected_rank_column = if limit_per_scope.is_some() {
        "sample_rank"
    } else {
        "NULL AS sample_rank"
    };
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql}),
              counted AS (
                SELECT NULL AS record_family, value, COUNT(*) AS catalog_count
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
                GROUP BY value
                UNION ALL
                SELECT r.record_family, value, COUNT(*) AS catalog_count
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
                GROUP BY r.record_family, value
              ),
              expected AS (
                SELECT record_family, value, catalog_count, {expected_rank_column}
                FROM (
                  SELECT record_family, value, catalog_count,
                         ROW_NUMBER() OVER (
                           PARTITION BY record_family
                           ORDER BY catalog_count DESC, value ASC
                         ) AS sample_rank
                  FROM counted
                )
                {limit_predicate}
              ),
              actual AS (
                SELECT record_family, value, catalog_count, {actual_rank_column}
                FROM {table}
                WHERE field = ?1
              )
         SELECT
           (SELECT COUNT(*)
            FROM expected e
            LEFT JOIN actual a
              ON ((a.record_family IS NULL AND e.record_family IS NULL)
                  OR a.record_family = e.record_family)
             AND a.value = e.value
            WHERE a.value IS NULL),
           (SELECT COUNT(*)
            FROM actual a
            LEFT JOIN expected e
             ON ((a.record_family IS NULL AND e.record_family IS NULL)
                  OR a.record_family = e.record_family)
             AND a.value = e.value
            WHERE e.value IS NULL
               OR a.catalog_count <> e.catalog_count
               OR COALESCE(a.sample_rank, 0) <> COALESCE(e.sample_rank, 0))"
    );
    query_count_pair(connection, &sql, params![field])
}
