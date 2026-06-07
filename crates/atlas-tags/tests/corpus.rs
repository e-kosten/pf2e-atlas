use std::path::PathBuf;

use atlas_tags::{
    TagCorpus, TagCorpusLoadError, TagValidationError, load_tag_corpus, validate_tag_corpus,
};

#[test]
fn loads_valid_corpus_from_standard_directory_layout() {
    let corpus = load_tag_corpus(fixture_path("corpus")).expect("corpus loads");

    assert_eq!(corpus.catalog.definitions().len(), 3);
    assert!(
        corpus
            .catalog
            .contains_id(&"problem.counteract_magic".parse().expect("id parses"))
    );
    assert_eq!(corpus.assignments.records.len(), 3);
    assert_eq!(corpus.ontology_suggestions.suggestions.len(), 1);
}

#[test]
fn tag_corpus_associated_loader_uses_same_layout() {
    let corpus = TagCorpus::load_from_dir(fixture_path("corpus")).expect("corpus loads");

    assert!(
        corpus
            .catalog
            .get(&"setting.darklands".parse().expect("id parses"))
            .is_some()
    );
}

#[test]
fn validates_corpus_through_named_validation_entry_point() {
    let corpus = validate_tag_corpus(fixture_path("corpus")).expect("corpus validates");

    assert_eq!(corpus.assignments.records.len(), 3);
}

#[test]
fn rejects_assignments_that_reference_tags_outside_the_merged_catalog() {
    let error = load_tag_corpus(fixture_path("corpus-unknown-assignment"))
        .expect_err("unknown assigned tag is rejected");

    assert!(matches!(
        error,
        TagCorpusLoadError::Validation(TagValidationError::UnknownAssignedTag { .. })
    ));
}

#[test]
fn rejects_duplicate_record_assignments_across_assignment_files() {
    let error = load_tag_corpus(fixture_path("corpus-duplicate-assignment"))
        .expect_err("duplicate record assignment is rejected");

    assert!(matches!(
        error,
        TagCorpusLoadError::Validation(TagValidationError::DuplicateAssignedRecord(_))
    ));
}

#[test]
fn requires_catalog_directory() {
    let error = load_tag_corpus(fixture_path("missing-corpus"))
        .expect_err("missing catalog directory is rejected");

    assert!(matches!(
        error,
        TagCorpusLoadError::MissingCatalogDirectory { .. }
    ));
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}
