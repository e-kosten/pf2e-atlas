use atlas_record::RichDocument;
use rusqlite::Connection;

use crate::{
    ArtifactValidationDiagnostic, ArtifactValidationFamily, IndexValidationError,
    artifact::validation::artifact_validation_diagnostic,
};

pub(super) fn validate_content_json(
    connection: &Connection,
    diagnostics: &mut Vec<ArtifactValidationDiagnostic>,
) -> Result<(), IndexValidationError> {
    let mut invalid = 0;
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
        diagnostics.push(artifact_validation_diagnostic(
            ArtifactValidationFamily::Data,
            "content JSON must deserialize as RichDocument".to_string(),
            Some("content:json".to_string()),
            Some("valid RichDocument JSON".to_string()),
            Some(format!("{invalid} invalid documents")),
        ));
    }
    Ok(())
}

fn content_json_is_valid(value: Option<&str>) -> bool {
    value
        .map(|value| serde_json::from_str::<RichDocument>(value).is_ok())
        .unwrap_or(true)
}
