use rusqlite::Connection;

use crate::sql::count_sql;
use crate::{
    ArtifactContractFamily, ArtifactValidationDiagnostic, IndexValidationError,
    contract::contract_diagnostic,
};

pub(super) fn validate_metric_values(
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

pub(super) fn validate_metric_catalogs(
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
