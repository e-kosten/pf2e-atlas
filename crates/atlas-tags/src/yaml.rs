use thiserror::Error;

use crate::{AssignmentFile, OntologySuggestionFile, TagCatalogFile, TagValidationError};

pub fn catalog_file_from_str(input: &str) -> Result<TagCatalogFile, TagYamlError> {
    let file = yaml_serde::from_str(input).map_err(TagYamlError::Parse)?;
    crate::validate_catalog_file(&file)?;
    Ok(file)
}

pub fn assignment_file_from_str(input: &str) -> Result<AssignmentFile, TagYamlError> {
    let file = yaml_serde::from_str(input).map_err(TagYamlError::Parse)?;
    crate::validate_assignment_file(&file)?;
    Ok(file)
}

pub fn ontology_suggestion_file_from_str(
    input: &str,
) -> Result<OntologySuggestionFile, TagYamlError> {
    let file = yaml_serde::from_str(input).map_err(TagYamlError::Parse)?;
    crate::validate_ontology_suggestion_file(&file)?;
    Ok(file)
}

#[derive(Debug, Error)]
pub enum TagYamlError {
    #[error("failed to parse tag YAML: {0}")]
    Parse(yaml_serde::Error),
    #[error(transparent)]
    Validation(#[from] TagValidationError),
}
