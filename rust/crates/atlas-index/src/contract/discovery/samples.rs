use atlas_discovery::all_discovery_field_definitions;
use atlas_domain::FilterValuePolicy;
use rusqlite::Connection;

use crate::{ArtifactValidationDiagnostic, IndexValidationError};

use crate::sql::count_sql;

use super::{push_duplicate_diagnostic, push_row_diagnostics, values::value_catalog_diff};

const SAMPLE_LIMIT: u64 = 100;

pub(super) fn validate_sample_catalog(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    validate_sample_catalog_uniqueness(connection, diagnostics)?;
    let mut missing = 0_u64;
    let mut stale = 0_u64;
    for definition in all_discovery_field_definitions()
        .iter()
        .filter(|definition| definition.value_policy == FilterValuePolicy::Sample)
    {
        let (definition_missing, definition_stale) = value_catalog_diff(
            connection,
            "filter_sample_catalog",
            definition.field,
            definition.value_sql,
            Some(SAMPLE_LIMIT),
        )?;
        missing += definition_missing;
        stale += definition_stale;
    }
    push_row_diagnostics(
        diagnostics,
        "filter_sample_catalog",
        missing,
        stale,
        "sample",
    );
    Ok(())
}

fn validate_sample_catalog_uniqueness(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let duplicates = count_sql(
        connection,
        "SELECT COUNT(*)
         FROM (
           SELECT field, COALESCE(record_family, '<global>') AS scope, value
           FROM filter_sample_catalog
           GROUP BY field, scope, value
           HAVING COUNT(*) > 1
         )",
    )?;
    push_duplicate_diagnostic(diagnostics, "filter_sample_catalog", duplicates);
    Ok(())
}
