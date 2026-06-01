use crate::schema_inventory::TABLE_RECORDS_FTS;
use rusqlite::Connection;

use crate::sql::{count_rows, count_sql};
use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError,
    artifact_validation::artifact_validation_diagnostic,
};

pub(super) fn validate_fts_coverage(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let fts_rows = count_rows(connection, TABLE_RECORDS_FTS)?;
    let default_visible_records = count_sql(
        connection,
        "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
    )?;
    if fts_rows != default_visible_records {
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Fts,
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
            diagnostics.push(artifact_validation_diagnostic(
                ArtifactValidationFamily::Fts,
                format!("FTS coverage check `{key}` failed"),
                Some(key.to_string()),
                Some(expected.to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}
