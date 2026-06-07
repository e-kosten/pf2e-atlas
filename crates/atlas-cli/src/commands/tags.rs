use std::process::ExitCode;

use atlas_tags::{AssignmentFile, OntologySuggestionFile, TagCorpus, validate_tag_corpus};
use serde::Serialize;

use crate::commands::tags::args::TagsValidateOptions;
use crate::output::write_json_data;

pub(crate) mod args;

pub(crate) fn run_tags_validate(options: TagsValidateOptions) -> Result<ExitCode, String> {
    match validate_tag_corpus(&options.path) {
        Ok(corpus) => {
            let report = TagCorpusValidationReport::from_corpus(&corpus);
            if options.json {
                write_json_data(&report)?;
            } else {
                println!("Tag corpus valid");
                println!("Path: {}", options.path.display());
                println!("Catalog tags: {}", report.catalog_tags);
                println!("Assignment records: {}", report.assignment_records);
                println!("Assigned tags: {}", report.assigned_tags);
                println!("Reviewed empty records: {}", report.reviewed_empty_records);
                println!("Ontology suggestions: {}", report.ontology_suggestions);
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            let message = error.to_string();
            if options.json {
                let report = TagCorpusValidationReport::invalid(message);
                write_json_data(&report)?;
            } else {
                eprintln!("Tag corpus invalid: {message}");
            }
            Ok(ExitCode::from(3))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TagCorpusValidationReport {
    valid: bool,
    catalog_tags: usize,
    assignment_records: usize,
    assigned_tags: usize,
    reviewed_empty_records: usize,
    ontology_suggestions: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<String>,
}

impl TagCorpusValidationReport {
    fn from_corpus(corpus: &TagCorpus) -> Self {
        Self {
            valid: true,
            catalog_tags: corpus.catalog.definitions().len(),
            assignment_records: corpus.assignments.records.len(),
            assigned_tags: assigned_tag_count(&corpus.assignments),
            reviewed_empty_records: reviewed_empty_record_count(&corpus.assignments),
            ontology_suggestions: ontology_suggestion_count(&corpus.ontology_suggestions),
            errors: Vec::new(),
        }
    }

    fn invalid(error: String) -> Self {
        Self {
            valid: false,
            catalog_tags: 0,
            assignment_records: 0,
            assigned_tags: 0,
            reviewed_empty_records: 0,
            ontology_suggestions: 0,
            errors: vec![error],
        }
    }
}

fn assigned_tag_count(assignments: &AssignmentFile) -> usize {
    assignments
        .records
        .iter()
        .map(|record| record.tags.len())
        .sum()
}

fn reviewed_empty_record_count(assignments: &AssignmentFile) -> usize {
    assignments
        .records
        .iter()
        .filter(|record| record.tags.is_empty())
        .count()
}

fn ontology_suggestion_count(suggestions: &OntologySuggestionFile) -> usize {
    suggestions.suggestions.len()
}
