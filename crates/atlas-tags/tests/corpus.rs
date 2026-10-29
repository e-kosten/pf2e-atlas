use std::path::PathBuf;

use atlas_tags::{
    TagCorpus, TagCorpusLoadError, TagValidationError, load_tag_corpus, validate_tag_corpus,
};

#[test]
fn all_public_loaders_preserve_merged_catalog_and_record_assignments() {
    for (entry_point, result) in [
        ("load_tag_corpus", load_tag_corpus(fixture_path("corpus"))),
        (
            "TagCorpus::load_from_dir",
            TagCorpus::load_from_dir(fixture_path("corpus")),
        ),
        (
            "validate_tag_corpus",
            validate_tag_corpus(fixture_path("corpus")),
        ),
    ] {
        let corpus = result.expect(entry_point);
        assert_eq!(corpus.catalog.definitions().len(), 3, "{entry_point}");
        assert_eq!(corpus.assignments.records.len(), 3, "{entry_point}");
        let ids = corpus
            .catalog
            .ids()
            .map(|id| id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            ids,
            [
                "problem.counteract_magic",
                "setting.darklands",
                "equipment.party_support.defense"
            ]
            .into(),
            "{entry_point}"
        );
        let assigned = corpus
            .assignments
            .records
            .iter()
            .map(|record| {
                (
                    record.record_key.to_string(),
                    record
                        .tags
                        .iter()
                        .map(|tag| tag.tag_id.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            assigned,
            [
                (
                    "equipment-srd:fortress-shield".into(),
                    vec!["equipment.party_support.defense"]
                ),
                ("equipment-srd:plain-clothing".into(), vec![]),
                (
                    "spells-srd:dispel-magic".into(),
                    vec!["problem.counteract_magic"]
                ),
            ]
            .into(),
            "{entry_point}"
        );
        let suggestions = corpus.ontology_suggestions.suggestions;
        assert_eq!(suggestions.len(), 1, "{entry_point}");
        assert_eq!(
            suggestions[0].proposed_id.as_ref().unwrap().as_str(),
            "exploration.environment.aquatic",
            "{entry_point}"
        );
        assert_eq!(
            suggestions[0].triggering_record_key.to_string(),
            "spells-srd:water-breathing",
            "{entry_point}"
        );
    }
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
