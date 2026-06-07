use atlas_tags::{
    assignment_file_from_str, catalog_file_from_str, ontology_suggestion_file_from_str,
};

#[test]
fn catalog_example_contains_three_valid_tag_definitions() {
    let catalog = catalog_file_from_str(include_str!("fixtures/catalog-example.yaml"))
        .expect("catalog parses");

    assert_eq!(catalog.tags.len(), 3);
    assert_eq!(catalog.tags[0].id.as_str(), "problem.counteract_magic");
    assert_eq!(catalog.tags[1].id.as_str(), "setting.darklands");
    assert_eq!(
        catalog.tags[2].id.as_str(),
        "equipment.party_support.defense"
    );
}

#[test]
fn assignment_example_contains_three_assigned_records() {
    let assignments = assignment_file_from_str(include_str!("fixtures/assignments-example.yaml"))
        .expect("assignments parse");

    assert_eq!(assignments.records.len(), 3);
    assert!(
        assignments
            .records
            .iter()
            .all(|record| !record.tags.is_empty())
    );
}

#[test]
fn reviewed_empty_example_contains_three_intentionally_untagged_records() {
    let reviewed_empty =
        assignment_file_from_str(include_str!("fixtures/reviewed-empty-example.yaml"))
            .expect("reviewed-empty assignments parse");

    assert_eq!(reviewed_empty.records.len(), 3);
    assert!(
        reviewed_empty
            .records
            .iter()
            .all(|record| record.tags.is_empty())
    );
}

#[test]
fn ontology_suggestions_example_contains_three_valid_suggestions() {
    let suggestions = ontology_suggestion_file_from_str(include_str!(
        "fixtures/ontology-suggestions-example.yaml"
    ))
    .expect("ontology suggestions parse");

    assert_eq!(suggestions.suggestions.len(), 3);
    assert_eq!(
        suggestions.suggestions[0]
            .proposed_id
            .as_ref()
            .expect("suggestion has proposed id")
            .as_str(),
        "exploration.environment.aquatic"
    );
    assert!(
        suggestions
            .suggestions
            .iter()
            .all(|suggestion| suggestion.follow_up_research_needed)
    );
}
