use std::collections::BTreeMap;

use atlas_artifact::metadata::artifact_metadata_keys;
use atlas_artifact::schema::{
    TABLE_RECORDS, boolean_columns, invalid_boolean_column_sql, required_columns, required_tables,
};
use rusqlite::Connection;

use crate::sql::{count_rows, count_sql, table_columns, table_exists};
use crate::{
    ArtifactContractFamily, ArtifactValidationDiagnostic, IndexValidationError,
    contract::contract_diagnostic,
};

pub(super) fn validate_required_tables(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for table in required_tables() {
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

pub(super) fn validate_required_columns(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for (table, columns) in required_columns() {
        let table_name = table.name();
        let present_columns = table_columns(connection, table_name)?;
        for column in columns {
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

pub(super) fn validate_record_counts(
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

pub(super) fn validate_foreign_keys(
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

pub(super) fn validate_boolean_columns(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    for check in boolean_columns() {
        let sql = invalid_boolean_column_sql(&check);
        let invalid = count_sql(connection, &sql)?;
        if invalid > 0 {
            diagnostics.push(contract_diagnostic(
                ArtifactContractFamily::Data,
                format!("boolean column `{}` contains non-boolean values", check.key),
                Some(check.key.clone()),
                Some("0 or 1".to_string()),
                Some(format!("{invalid} invalid rows")),
            ));
        }
    }
    Ok(())
}
