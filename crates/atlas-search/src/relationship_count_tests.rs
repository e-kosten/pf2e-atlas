use super::*;
use atlas_domain::{QueryFacetOutcome, QueryLiteral};

fn available<T>(outcome: QueryFacetOutcome<T>) -> T {
    match outcome {
        QueryFacetOutcome::Available { value } => value,
        QueryFacetOutcome::Unavailable { reason } => {
            panic!("expected available facet, got {reason:?}")
        }
    }
}
fn context<'a>(
    scope: RecordScope<'a>,
    filter: Option<&'a QueryPredicate>,
) -> FilterDiscoveryContext<'a> {
    FilterDiscoveryContext {
        content_scope: atlas_domain::SearchContentScope::All,
        search_profile: None,
        text: None,
        scope,
        prefer_remaster: true,
        filter,
    }
}
fn option_counts(
    service: &mut AtlasRetrievalService,
    scope: RecordScope<'_>,
    filter: Option<&QueryPredicate>,
    field: &str,
    clause: Option<&str>,
    values: &[&str],
) -> Vec<u64> {
    let values = values
        .iter()
        .map(|v| QueryLiteral::String((*v).into()))
        .collect::<Vec<_>>();
    available(
        service
            .discover_filter_value_counts(DiscoverFilterValueCountsRequest {
                field,
                clause_id: clause,
                context: context(scope, filter),
                values: &values,
            })
            .unwrap(),
    )
    .options
    .into_iter()
    .map(|o| o.distinct_roots)
    .collect()
}

#[test]
fn relationship_counts_cover_complete_set_numeric_stats_self_exclusion_and_key_scopes() {
    let target = "target0000000000";
    let second = "target0000000001";
    let mut records = vec![effect(target, "Target", ""), effect(second, "Second", "")];
    for i in 0..125 {
        records.push(record("items","Item",json!({"_id":format!("{i:016}"),"name":format!("Linked {i:03}"),"type":if i==124{"weapon"}else{"effect"},"system":{"level":{"value":i},"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{target}] @UUID[Compendium.pf2e.items.Item.{target}] {}</p>",if i==124{format!("@UUID[Compendium.pf2e.items.Item.{second}]")}else{String::new()})}}})));
    }
    records.push(record("items","Item",json!({"_id":"unlinked00000000","name":"Unlinked","type":"weapon","system":{"level":{"value":999}}})));
    let (mut service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let constraints = [criterion(
        SourceRelationshipDirection::Incoming,
        key("items", target),
        vec![],
        false,
    )];
    let links = link_filter(None, &constraints);
    let scope = RecordScope::All;
    assert_eq!(
        option_counts(
            &mut service,
            scope,
            Some(&links),
            "source.type",
            None,
            &["effect", "weapon"]
        ),
        vec![124, 1]
    );
    let numeric = available(
        service
            .discover_filter_counts(DiscoverFilterCountsRequest {
                field: "level",
                clause_id: None,
                context: context(scope, Some(&links)),
            })
            .unwrap(),
    );
    assert_eq!(
        (numeric.minimum, numeric.maximum),
        (Some(0.into()), Some(124.into()))
    );
    assert_eq!(
        numeric
            .states
            .iter()
            .find(|s| s.state == atlas_domain::QueryFieldState::Value)
            .unwrap()
            .distinct_roots,
        125
    );
    let filter:QueryPredicate=serde_json::from_value(json!({"kind":"all_of","children":[{"clause_id":"type-choice","kind":"compare","field":"source.type","op":"eq","value":"effect"},{"clause_id":"level-choice","kind":"compare","field":"level","op":"gte","value":120}]})).unwrap();
    let filter = link_filter(Some(&filter), &constraints);
    assert_eq!(
        option_counts(
            &mut service,
            scope,
            Some(&filter),
            "source.type",
            Some("type-choice"),
            &["effect", "weapon"]
        ),
        vec![4, 1]
    );
    let values = available(
        service
            .discover_filter_values(DiscoverFilterValuesRequest {
                only_matching: true,
                field: "source.type",
                clause_id: Some("type-choice"),
                context: context(scope, Some(&filter)),
                text: Some("weap"),
                offset: 0,
                limit: 10,
            })
            .unwrap(),
    );
    assert!(
        values
            .options
            .iter()
            .any(|o| o.value == QueryLiteral::String("weapon".into()) && o.distinct_roots == 1)
    );
    let numeric = available(
        service
            .discover_filter_counts(DiscoverFilterCountsRequest {
                field: "level",
                clause_id: Some("level-choice"),
                context: context(scope, Some(&filter)),
            })
            .unwrap(),
    );
    assert_eq!(
        (numeric.minimum, numeric.maximum),
        (Some(0.into()), Some(123.into()))
    );
    let keys = [
        key("items", "0000000000000000"),
        key("items", "0000000000000124"),
        key("items", "unlinked00000000"),
    ];
    let scoped = RecordScope::Keys(&keys);
    assert_eq!(
        option_counts(
            &mut service,
            scoped,
            Some(&links),
            "source.type",
            None,
            &["effect", "weapon"]
        ),
        vec![1, 1]
    );
    assert_eq!(
        option_counts(
            &mut service,
            RecordScope::Keys(&[]),
            Some(&links),
            "source.type",
            None,
            &["effect", "weapon"]
        ),
        vec![0, 0]
    );
    let both = [
        constraints[0].clone(),
        criterion(
            SourceRelationshipDirection::Incoming,
            key("items", second),
            vec![],
            false,
        ),
    ];
    assert_eq!(
        option_counts(
            &mut service,
            RecordScope::All,
            Some(&link_filter(None, &both)),
            "source.type",
            None,
            &["effect", "weapon"]
        ),
        vec![0, 1]
    );
    let vocabulary = service
        .discover_filter_vocabulary(DiscoverFilterVocabularyRequest {
            field: "kind",
            scope,
            text: None,
            offset: 0,
            limit: 20,
            selected_values: &[],
        })
        .unwrap();
    assert!(
        vocabulary
            .options
            .iter()
            .any(|o| o.value == QueryLiteral::String("equipment".into()))
    );
}

#[test]
fn relationship_counts_keep_exact_direction_owner_and_contents_scope() {
    let actor = "cccccccccccccccc";
    let child_a = "dddddddddddddddd";
    let child_b = "eeeeeeeeeeeeeeee";
    let root = record(
        "actors",
        "Actor",
        json!({"_id":actor,"type":"npc","name":"Source","system":{"details":{"publicNotes":"<p>@UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa]</p>"}},"items":[{"_id":child_a,"name":"Child A","type":"action","system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.bbbbbbbbbbbbbbbb]</p>"}}},{"_id":child_b,"name":"Child B","type":"action","system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa]</p>"}}}]}),
    );
    let owned_link = effect(
        "ffffffffffffffff",
        "Owned target link",
        &format!("<p>@UUID[Compendium.pf2e.actors.Actor.{actor}.Item.{child_a}]</p>"),
    );
    let (mut service, _artifact) = open_input_fixture(input(
        vec![
            root,
            owned_link,
            effect("aaaaaaaaaaaaaaaa", "A", ""),
            effect("bbbbbbbbbbbbbbbb", "B", ""),
        ],
        false,
    ))
    .unwrap();
    for (direction, selected, owners, include, expected) in [
        (
            SourceRelationshipDirection::Outgoing,
            key("actors", actor),
            vec![],
            false,
            vec![1, 0],
        ),
        (
            SourceRelationshipDirection::Outgoing,
            key("actors", actor),
            vec![],
            true,
            vec![2, 0],
        ),
        (
            SourceRelationshipDirection::Outgoing,
            key("actors", actor),
            owned(child_a),
            true,
            vec![1, 0],
        ),
        (
            SourceRelationshipDirection::Incoming,
            key("items", "bbbbbbbbbbbbbbbb"),
            vec![],
            false,
            vec![0, 0],
        ),
        (
            SourceRelationshipDirection::Incoming,
            key("items", "bbbbbbbbbbbbbbbb"),
            vec![],
            true,
            vec![0, 1],
        ),
        (
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            vec![],
            true,
            vec![0, 0],
        ),
        (
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            owned(child_a),
            true,
            vec![1, 0],
        ),
        (
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            owned(child_b),
            true,
            vec![0, 0],
        ),
    ] {
        let constraints = [criterion(direction, selected, owners, include)];
        let links = link_filter(None, &constraints);
        let scope = RecordScope::All;
        assert_eq!(
            option_counts(
                &mut service,
                scope,
                Some(&links),
                "source.type",
                None,
                &["effect", "npc"]
            ),
            expected
        );
    }
}

#[test]
fn relationship_scoped_facets_keep_same_child_witness_and_parameter_bindings() {
    let make_actor = |id: &str, links: bool, spells: Vec<serde_json::Value>| {
        record(
            "actors",
            "Actor",
            json!({"_id":id,"name":id,"type":"npc","system":{"details":{"publicNotes":if links{"<p>@UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa] @UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa]</p>"}else{""}}},"items":spells}),
        )
    };
    let spell = |id: &str, rank: i32, tradition: &str| json!({"_id":id,"type":"spell","name":"Spell","system":{"level":{"value":rank},"traits":{"traditions":[tradition]}}});
    let (mut service, _artifact) = open_input_fixture(input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Target", ""),
            make_actor(
                "bbbbbbbbbbbbbbbb",
                true,
                vec![
                    spell("spell00000000000", 1, "arcane"),
                    spell("spell00000000001", 5, "divine"),
                ],
            ),
            make_actor(
                "cccccccccccccccc",
                true,
                vec![
                    spell("spell00000000000", 5, "arcane"),
                    spell("spell00000000001", 5, "arcane"),
                ],
            ),
            make_actor(
                "dddddddddddddddd",
                false,
                vec![spell("spell00000000000", 9, "occult")],
            ),
        ],
        false,
    ))
    .unwrap();
    let constraints = [criterion(
        SourceRelationshipDirection::Incoming,
        key("items", "aaaaaaaaaaaaaaaa"),
        vec![],
        true,
    )];
    let scope = RecordScope::All;
    let filter:QueryPredicate=serde_json::from_value(json!({"kind":"exists","collection":"embedded.spells","scope_id":"spells","predicate":{"kind":"all_of","children":[{"clause_id":"rank-choice","kind":"compare","field":"spell.rank","op":"gte","value":3},{"clause_id":"tradition-choice","kind":"set_match","field":"spell.traditions","op":"includes","values":["arcane"]}]}})).unwrap();
    let filter = link_filter(Some(&filter), &constraints);
    assert_eq!(
        option_counts(
            &mut service,
            scope,
            Some(&filter),
            "embedded.spells[].spell.traditions",
            Some("tradition-choice"),
            &["arcane", "divine", "occult"]
        ),
        vec![1, 1, 0]
    );
    let values = available(
        service
            .discover_filter_values(DiscoverFilterValuesRequest {
                only_matching: true,
                field: "embedded.spells[].spell.traditions",
                clause_id: Some("tradition-choice"),
                context: context(scope, Some(&filter)),
                text: None,
                offset: 0,
                limit: 20,
            })
            .unwrap(),
    );
    assert_eq!(
        values
            .options
            .iter()
            .map(|o| o.distinct_roots)
            .collect::<Vec<_>>(),
        vec![1, 1]
    );
    let numeric = available(
        service
            .discover_filter_counts(DiscoverFilterCountsRequest {
                field: "embedded.spells[].spell.rank",
                clause_id: Some("rank-choice"),
                context: context(scope, Some(&filter)),
            })
            .unwrap(),
    );
    assert_eq!(
        (numeric.minimum, numeric.maximum),
        (Some(1.into()), Some(5.into()))
    );
    let stats = numeric
        .states
        .iter()
        .find(|s| s.state == atlas_domain::QueryFieldState::Value)
        .unwrap();
    assert_eq!((stats.occurrences, stats.distinct_roots), (3, 2));
}

#[test]
fn relationship_option_counts_apply_edition_preference_only_to_matching_edges() {
    let mut fixture = input(
        vec![
            effect("aaaaaaaaaaaaaaaa", "Target", ""),
            effect(
                "bbbbbbbbbbbbbbbb",
                "Legacy",
                "<p>@UUID[Compendium.pf2e.items.Item.aaaaaaaaaaaaaaaa]</p>",
            ),
            effect("cccccccccccccccc", "Remaster", ""),
        ],
        false,
    );
    fixture
        .remaster_pairs
        .push(atlas_index::SourceRemasterPairInput {
            legacy: key("items", "bbbbbbbbbbbbbbbb"),
            remaster: key("items", "cccccccccccccccc"),
            provenance: atlas_record::source_content::SourceContentLocator {
                record: key("items", "bbbbbbbbbbbbbbbb"),
                owners: vec![],
                field: "/system/description/value".into(),
            },
        });
    let (mut service, _artifact) = open_input_fixture(fixture).unwrap();
    let constraints = [criterion(
        SourceRelationshipDirection::Incoming,
        key("items", "aaaaaaaaaaaaaaaa"),
        vec![],
        true,
    )];
    let links = link_filter(None, &constraints);
    assert_eq!(
        option_counts(
            &mut service,
            RecordScope::All,
            Some(&links),
            "source.type",
            None,
            &["effect"]
        ),
        vec![1]
    );
}
