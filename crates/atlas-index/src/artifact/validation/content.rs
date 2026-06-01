use atlas_record::ContentDocument;
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
        .prepare(
            "SELECT record_key, description_json, blurb_json
             FROM records
             WHERE description_json IS NOT NULL OR blurb_json IS NOT NULL",
        )
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut rows = statement
        .query([])
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    while let Some(row) = rows
        .next()
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?
    {
        let description: Option<String> = row
            .get("description_json")
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        let blurb: Option<String> = row
            .get("blurb_json")
            .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        invalid += usize::from(!content_json_is_valid(description.as_deref()));
        invalid += usize::from(!content_json_is_valid(blurb.as_deref()));
    }

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
            "content JSON must deserialize as ContentDocument".to_string(),
            Some("content:json".to_string()),
            Some("valid ContentDocument JSON".to_string()),
            Some(format!("{invalid} invalid documents")),
        ));
    }
    Ok(())
}

fn content_json_is_valid(value: Option<&str>) -> bool {
    value
        .map(|value| serde_json::from_str::<ContentDocument>(value).is_ok())
        .unwrap_or(true)
}
