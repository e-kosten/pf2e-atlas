use rusqlite::{Connection, Params};

use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError,
    artifact_validation::artifact_validation_diagnostic,
};

mod fields;
mod numeric;
mod samples;
mod values;

pub(super) fn validate_filter_discovery_catalogs(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    fields::validate_field_catalog(connection, diagnostics)?;
    values::validate_value_catalog(connection, diagnostics)?;
    samples::validate_sample_catalog(connection, diagnostics)?;
    numeric::validate_numeric_catalog(connection, diagnostics)?;
    Ok(())
}

pub(super) fn query_count_pair<P: Params>(
    connection: &Connection,
    sql: &str,
    params: P,
) -> Result<(u64, u64), IndexValidationError> {
    connection
        .query_row(sql, params, |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

pub(super) fn push_row_diagnostics(
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
    table: &str,
    missing: u64,
    stale: u64,
    row_kind: &str,
) {
    if missing > 0 {
        diagnostics.push(discovery_diagnostic(
            &format!("{table}.missing_rows"),
            format!("{missing} expected {row_kind} catalog rows are missing"),
        ));
    }
    if stale > 0 {
        diagnostics.push(discovery_diagnostic(
            &format!("{table}.stale_rows"),
            format!("{stale} {row_kind} catalog rows do not match default-visible records"),
        ));
    }
}

pub(super) fn push_duplicate_diagnostic(
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
    table: &str,
    duplicates: usize,
) {
    if duplicates > 0 {
        diagnostics.push(discovery_diagnostic(
            &format!("{table}.duplicate_rows"),
            format!("{duplicates} catalog keys have duplicate rows"),
        ));
    }
}

pub(super) fn discovery_diagnostic(key: &str, message: String) -> ArtifactValidationDiagnostic {
    artifact_validation_diagnostic(
        ArtifactValidationFamily::Data,
        message,
        Some(key.to_string()),
        Some("filter discovery catalogs match default-visible records".to_string()),
        None,
    )
}
