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

    let assigned = assignments
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
        .collect::<Vec<_>>();
    assert_eq!(
        assigned,
        [
            (
                "spells-srd:dispel-magic".into(),
                vec!["problem.counteract_magic"]
            ),
            (
                "spells-srd:water-breathing".into(),
                vec!["exploration.environment.aquatic"]
            ),
            (
                "equipment-srd:spyglass".into(),
                vec!["exploration.reconnaissance.long_range"]
            ),
        ]
    );
}

#[test]
fn reviewed_empty_example_contains_three_intentionally_untagged_records() {
    let reviewed_empty =
        assignment_file_from_str(include_str!("fixtures/reviewed-empty-example.yaml"))
            .expect("reviewed-empty assignments parse");

    assert_eq!(
        reviewed_empty
            .records
            .iter()
            .map(|record| record.record_key.to_string())
            .collect::<Vec<_>>(),
        [
            "spells-srd:generic-flavor-cantrip",
            "equipment-srd:plain-clothing",
            "bestiary:minimal-placeholder",
        ]
    );
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

    let proposed = suggestions
        .suggestions
        .iter()
        .map(|suggestion| {
            (
                suggestion.proposed_id.as_ref().unwrap().as_str(),
                suggestion.triggering_record_key.to_string(),
                suggestion.follow_up_research_needed,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        proposed,
        [
            (
                "exploration.environment.aquatic",
                "spells-srd:water-breathing".into(),
                true
            ),
            (
                "encounter.summoner_support",
                "spells-srd:summoners-precaution".into(),
                true
            ),
            (
                "hazard.countermeasure.noise",
                "hazards:resonant-alarm".into(),
                true
            ),
        ]
    );
}
