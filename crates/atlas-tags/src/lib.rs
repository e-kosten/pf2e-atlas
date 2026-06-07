#![deny(unsafe_code)]

mod assignment;
mod catalog;
mod id;
mod model;
mod proposal;
mod validation;
mod yaml;

pub use assignment::{
    AssignmentEvidence, AssignmentFile, RecordTagAssignments, TagAssignment, TagFactField,
};
pub use catalog::{TagCatalog, TagCatalogFile};
pub use id::{TagId, TagIdParseError};
pub use model::{
    TagApplicability, TagApplicabilityClause, TagDefinition, TagDisplayGroup, TagDisplaySubgroup,
    TagFactPredicate, TagGuidance, TagPresentation,
};
pub use proposal::{OntologySuggestion, OntologySuggestionFile};
pub use validation::{
    TagValidationError, validate_assignment_file, validate_assignments_against_catalog,
    validate_catalog_file, validate_ontology_suggestion_file,
};
pub use yaml::{
    TagYamlError, assignment_file_from_str, catalog_file_from_str,
    ontology_suggestion_file_from_str,
};
