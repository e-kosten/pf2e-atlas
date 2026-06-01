use rusqlite::{Connection, params};

use crate::discovery::definitions::{DiscoveryFieldDefinition, all_definitions};
use crate::{ArtifactValidationDiagnostic, IndexValidationError, sql::count_sql};

use super::{discovery_diagnostic, push_duplicate_diagnostic};

pub(super) fn validate_field_catalog(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_field_catalog_uniqueness(connection, diagnostics)?;
    let mut missing = 0_u64;
    let mut stale = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM filter_field_catalog
         WHERE field <> 'metric'",
    )?;
    let mut mismatched = 0_u64;
    for definition in all_definitions() {
        let value_sql = definition.value_sql();
        let global_expected = field_observation_count(connection, &value_sql, None)?;
        if global_expected > 0 {
            missing += missing_field_row(connection, definition.field, None)?;
            stale = stale.saturating_sub(1);
            mismatched += mismatched_field_row(connection, definition, &value_sql, None)?;
        }
        for family in definition.applicable_families {
            let family_expected = field_observation_count(connection, &value_sql, Some(*family))?;
            if family_expected > 0 {
                missing += missing_field_row(connection, definition.field, Some(*family))?;
                stale = stale.saturating_sub(1);
                mismatched +=
                    mismatched_field_row(connection, definition, &value_sql, Some(*family))?;
            }
        }
    }
    if missing > 0 {
        diagnostics.push(discovery_diagnostic(
            "filter_field_catalog.missing_rows",
            format!("{missing} supported filter field catalog rows are missing"),
        ));
    }
    if stale > 0 {
        diagnostics.push(discovery_diagnostic(
            "filter_field_catalog.stale_rows",
            format!("{stale} filter field catalog rows do not match supported fields"),
        ));
    }
    if mismatched > 0 {
        diagnostics.push(discovery_diagnostic(
            "filter_field_catalog.mismatched_rows",
            format!("{mismatched} filter field catalog rows have stale metadata or stats"),
        ));
    }
    Ok(())
}

fn validate_field_catalog_uniqueness(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let duplicates = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM (
           SELECT field, COALESCE(record_family, '<global>') AS scope
           FROM filter_field_catalog
           GROUP BY field, scope
           HAVING COUNT(*) > 1
         )",
    )?;
    push_duplicate_diagnostic(diagnostics, "filter_field_catalog", duplicates);
    Ok(())
}

fn field_observation_count(
    connection: &Connection,
    value_sql: &str,
    family: Option<&str>,
) -> Result<u64, IndexValidationError> {
    let family_predicate = if family.is_some() {
        "AND r.record_family = ?1"
    } else {
        ""
    };
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql})
         SELECT COUNT(*)
         FROM field_values fv
         JOIN records r ON r.record_key = fv.record_key
         WHERE r.is_default_visible = 1
           {family_predicate}
           AND value IS NOT NULL
           AND CAST(value AS TEXT) <> ''"
    );
    match family {
        Some(family) => connection
            .query_row(&sql, params![family], |row| row.get(0))
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string())),
        None => connection
            .query_row(&sql, [], |row| row.get(0))
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string())),
    }
}

fn missing_field_row(
    connection: &Connection,
    field: &str,
    family: Option<&str>,
) -> Result<u64, IndexValidationError> {
    let count = match family {
        Some(family) => connection.query_row(
            "SELECT COUNT(*)
             FROM filter_field_catalog
             WHERE field = ?1 AND record_family = ?2",
            params![field, family],
            |row| row.get::<_, u64>(0),
        ),
        None => connection.query_row(
            "SELECT COUNT(*)
             FROM filter_field_catalog
             WHERE field = ?1 AND record_family IS NULL",
            params![field],
            |row| row.get::<_, u64>(0),
        ),
    }
    .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    Ok(u64::from(count == 0))
}

fn mismatched_field_row(
    connection: &Connection,
    definition: &DiscoveryFieldDefinition,
    value_sql: &str,
    family: Option<&str>,
) -> Result<u64, IndexValidationError> {
    let stats = expected_field_stats(connection, value_sql, family)?;
    let operators_json = serde_json::to_string(definition.operators)
        .map_err(|error| IndexValidationError::InvalidArtifact(error.to_string()))?;
    let cli_flags_json = serde_json::to_string(definition.cli_flags)
        .map_err(|error| IndexValidationError::InvalidArtifact(error.to_string()))?;
    let applicable_families_json = serde_json::to_string(definition.applicable_families)
        .map_err(|error| IndexValidationError::InvalidArtifact(error.to_string()))?;
    let count = match family {
        Some(family) => connection.query_row(
            "SELECT COUNT(*)
             FROM filter_field_catalog
             WHERE field = ?1 AND record_family = ?2
               AND (
                 field_type <> ?3 OR field_group <> ?4 OR value_policy <> ?5
                 OR operators_json <> ?6 OR cli_flags_json <> ?7
                 OR applicable_families_json <> ?8
                 OR value_count <> ?9 OR matching_record_count <> ?10
                 OR null_count <> ?11 OR distinct_count <> ?12
                 OR singleton_count <> ?13
                 OR ABS(singleton_ratio - ?14) > 0.000000001
                 OR ABS(observation_singleton_ratio - ?15) > 0.000000001
                 OR policy_reason <> ?16
               )",
            params![
                definition.field,
                family,
                serde_json_string(definition.field_type)?,
                serde_json_string(definition.group)?,
                serde_json_string(definition.value_policy)?,
                operators_json,
                cli_flags_json,
                applicable_families_json,
                stats.value_count,
                stats.matching_record_count,
                stats.null_count,
                stats.distinct_count,
                stats.singleton_count,
                ratio(stats.singleton_count, stats.distinct_count),
                ratio(stats.singleton_count, stats.value_count),
                definition.policy_reason,
            ],
            |row| row.get::<_, u64>(0),
        ),
        None => connection.query_row(
            "SELECT COUNT(*)
             FROM filter_field_catalog
             WHERE field = ?1 AND record_family IS NULL
               AND (
                 field_type <> ?2 OR field_group <> ?3 OR value_policy <> ?4
                 OR operators_json <> ?5 OR cli_flags_json <> ?6
                 OR applicable_families_json <> ?7
                 OR value_count <> ?8 OR matching_record_count <> ?9
                 OR null_count <> ?10 OR distinct_count <> ?11
                 OR singleton_count <> ?12
                 OR ABS(singleton_ratio - ?13) > 0.000000001
                 OR ABS(observation_singleton_ratio - ?14) > 0.000000001
                 OR policy_reason <> ?15
               )",
            params![
                definition.field,
                serde_json_string(definition.field_type)?,
                serde_json_string(definition.group)?,
                serde_json_string(definition.value_policy)?,
                operators_json,
                cli_flags_json,
                applicable_families_json,
                stats.value_count,
                stats.matching_record_count,
                stats.null_count,
                stats.distinct_count,
                stats.singleton_count,
                ratio(stats.singleton_count, stats.distinct_count),
                ratio(stats.singleton_count, stats.value_count),
                definition.policy_reason,
            ],
            |row| row.get::<_, u64>(0),
        ),
    }
    .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    Ok(count)
}

#[derive(Debug, Clone, Copy)]
struct FieldStats {
    value_count: u64,
    matching_record_count: u64,
    null_count: u64,
    distinct_count: u64,
    singleton_count: u64,
}

fn expected_field_stats(
    connection: &Connection,
    value_sql: &str,
    family: Option<&str>,
) -> Result<FieldStats, IndexValidationError> {
    let family_predicate = if family.is_some() {
        "AND r.record_family = ?1"
    } else {
        ""
    };
    let sql = format!(
        "WITH field_values(record_key, value) AS ({value_sql}),
              counts AS (
                SELECT value, COUNT(*) AS value_count
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
                  {family_predicate}
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
                GROUP BY value
              ),
              observed AS (
                SELECT COUNT(DISTINCT fv.record_key) AS observed_record_count
                FROM field_values fv
                JOIN records r ON r.record_key = fv.record_key
                WHERE r.is_default_visible = 1
                  {family_predicate}
                  AND value IS NOT NULL
                  AND CAST(value AS TEXT) <> ''
              ),
              matching AS (
                SELECT COUNT(*) AS matching_record_count
                FROM records r
                WHERE r.is_default_visible = 1
                  {family_predicate}
              )
         SELECT COALESCE(SUM(value_count), 0) AS value_count,
                (SELECT matching_record_count FROM matching) AS matching_record_count,
                (SELECT matching_record_count FROM matching)
                  - (SELECT observed_record_count FROM observed) AS null_count,
                COUNT(*) AS distinct_count,
                COALESCE(SUM(CASE WHEN value_count = 1 THEN 1 ELSE 0 END), 0) AS singleton_count
         FROM counts"
    );
    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(FieldStats {
            value_count: row.get("value_count")?,
            matching_record_count: row.get("matching_record_count")?,
            null_count: row.get("null_count")?,
            distinct_count: row.get("distinct_count")?,
            singleton_count: row.get("singleton_count")?,
        })
    };
    match family {
        Some(family) => connection
            .query_row(&sql, params![family], map_row)
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string())),
        None => connection
            .query_row(&sql, [], map_row)
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string())),
    }
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn serde_json_string<T: serde::Serialize>(value: T) -> Result<String, IndexValidationError> {
    let value = serde_json::to_value(value)
        .map_err(|error| IndexValidationError::InvalidArtifact(error.to_string()))?;
    value.as_str().map(str::to_string).ok_or_else(|| {
        IndexValidationError::InvalidArtifact(
            "discovery field metadata did not serialize as a string".to_string(),
        )
    })
}
