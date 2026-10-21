use crate::{
    test_support::{input, open_input_fixture, record},
    *,
};
use atlas_domain::{QueryPredicate, RecordKey};
use atlas_record::source_content::{OwnedContentIdentity, OwnedContentLocator};
use serde_json::json;
fn key(pack: &str, id: &str) -> RecordKey {
    RecordKey::parse(&format!("{pack}:{id}")).unwrap()
}
fn owned(id: &str) -> Vec<OwnedContentLocator> {
    vec![OwnedContentLocator {
        collection: "/items".into(),
        identity: OwnedContentIdentity::Stable(id.into()),
    }]
}
fn effect(id: &str, name: &str, html: &str) -> atlas_index::SourceArtifactRecordInput {
    record(
        "items",
        "Item",
        json!({"_id":id,"type":"effect","name":name,"system":{"description":{"value":html}}}),
    )
}
fn criterion(
    direction: SourceRelationshipDirection,
    record: RecordKey,
    owners: Vec<OwnedContentLocator>,
    include_contents: bool,
) -> RelationshipConstraint {
    RelationshipConstraint {
        direction,
        record,
        owners,
        field: None,
        include_contents,
    }
}
#[test]
fn relationship_roots_are_exhaustive_distinct_filtered_and_conjunctive_before_retrieval() {
    let target = "target0000000000";
    let second = "target0000000001";
    let mut records = vec![effect(target, "Target", ""), effect(second, "Second", "")];
    for i in 0..125 {
        let id = format!("{i:016}");
        let html = format!(
            "<p>@UUID[Compendium.pf2e.items.Item.{target}] @UUID[Compendium.pf2e.items.Item.{target}] {}</p>",
            if i == 124 {
                format!("@UUID[Compendium.pf2e.items.Item.{second}]")
            } else {
                String::new()
            }
        );
        if i == 124 {
            records.push(record("items","Item",json!({"_id":id,"type":"weapon","name":format!("Linked {i:03}"),"system":{"description":{"value":html}}})));
        } else {
            records.push(effect(&id, &format!("Linked {i:03}"), &html));
        }
    }
    let (mut service, _artifact) = open_input_fixture(input(records, true)).unwrap();
    let constraints = vec![criterion(
        SourceRelationshipDirection::Incoming,
        key("items", target),
        vec![],
        false,
    )];
    let scope = RecordScope::Constrained {
        keys: None,
        relationships: &constraints,
    };
    let result = service
        .list_records(
            ListRecordsRequest::new(None, SearchPage::new(1, 100).unwrap()).with_scope(scope),
        )
        .unwrap();
    assert_eq!((result.total, result.records.len()), (125, 100));
    let last = service
        .list_records(
            ListRecordsRequest::new(None, SearchPage::new(2, 100).unwrap()).with_scope(scope),
        )
        .unwrap();
    assert_eq!(last.records.len(), 25);
    let filter: QueryPredicate = serde_json::from_value(
        json!({"kind":"compare","field":"kind","op":"eq","value":"equipment"}),
    )
    .unwrap();
    let filtered = service
        .list_records(
            ListRecordsRequest::new(Some(&filter), SearchPage::default()).with_scope(scope),
        )
        .unwrap();
    assert_eq!(
        filtered.records.len(),
        1,
        "kind present beyond old occurrence cap must remain eligible"
    );
    let evidence = service
        .index
        .relationship_occurrence_page(RelationshipOccurrencePageRequest {
            constraint: &constraints[0],
            result: &key("items", "0000000000000000"),
            offset: 1,
            limit: 1,
        })
        .unwrap();
    assert_eq!((evidence.total, evidence.occurrences.len()), (2, 1));
    let mut text = TextSearchRequest::new("Linked", SearchPage::default());
    text.mode = RetrievalMode::Fts;
    text.scope = scope;
    text.filter = Some(&filter);
    assert_eq!(service.search_text(text).unwrap().page.total, 1);
    let scoped_keys = vec![key("items", "0000000000000124")];
    let scoped = RecordScope::Constrained {
        keys: Some(&scoped_keys),
        relationships: &constraints,
    };
    assert_eq!(
        service
            .list_records(ListRecordsRequest::new(None, SearchPage::default()).with_scope(scoped))
            .unwrap()
            .total,
        1
    );
    let similar = service
        .similar_records(SimilarRecordRequest {
            search_profile: None,
            record_key: &key("items", target),
            filter: Some(&filter),
            scope,
            page: SearchPage::default(),
            prefer_remaster: true,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        similar.results.page.total, 1,
        "relationship and ordinary eligibility must precede KNN"
    );
    let all = vec![
        constraints[0].clone(),
        criterion(
            SourceRelationshipDirection::Incoming,
            key("items", second),
            vec![],
            false,
        ),
    ];
    assert_eq!(
        service
            .list_records(
                ListRecordsRequest::new(None, SearchPage::default()).with_scope(
                    RecordScope::Constrained {
                        keys: None,
                        relationships: &all
                    }
                )
            )
            .unwrap()
            .total,
        1
    );
}
#[test]
fn relationship_source_contents_and_exact_destinations_never_include_siblings_or_parents() {
    let a = "aaaaaaaaaaaaaaaa";
    let b = "bbbbbbbbbbbbbbbb";
    let actor = "cccccccccccccccc";
    let child_a = "dddddddddddddddd";
    let child_b = "eeeeeeeeeeeeeeee";
    let root = record(
        "actors",
        "Actor",
        json!({"_id":actor,"type":"npc","name":"Source","system":{"details":{"publicNotes":format!("<p>@UUID[Compendium.pf2e.items.Item.{a}]</p>")}},"items":[{"_id":child_a,"name":"Child A","type":"action","system":{"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{b}]</p>")}}},{"_id":child_b,"name":"Child B","type":"action","system":{"description":{"value":format!("<p>@UUID[Compendium.pf2e.items.Item.{a}]</p>")}}}]}),
    );
    let link = effect(
        "ffffffffffffffff",
        "Owned target link",
        &format!("<p>@UUID[Compendium.pf2e.actors.Actor.{actor}.Item.{child_a}]</p>"),
    );
    let (service, _artifact) = open_input_fixture(input(
        vec![root, link, effect(a, "A", ""), effect(b, "B", "")],
        false,
    ))
    .unwrap();
    let query = |constraint: RelationshipConstraint| {
        service
            .list_records(
                ListRecordsRequest::new(None, SearchPage::default()).with_scope(
                    RecordScope::Constrained {
                        keys: None,
                        relationships: &[constraint],
                    },
                ),
            )
            .unwrap()
            .record_keys
    };
    assert_eq!(
        query(criterion(
            SourceRelationshipDirection::Outgoing,
            key("actors", actor),
            vec![],
            false
        )),
        vec![key("items", a)]
    );
    assert_eq!(
        query(criterion(
            SourceRelationshipDirection::Outgoing,
            key("actors", actor),
            vec![],
            true
        )),
        vec![key("items", a), key("items", b)]
    );
    for include in [false, true] {
        assert_eq!(
            query(criterion(
                SourceRelationshipDirection::Outgoing,
                key("actors", actor),
                owned(child_a),
                include
            )),
            vec![key("items", b)]
        );
    }
    assert!(
        query(criterion(
            SourceRelationshipDirection::Incoming,
            key("items", b),
            vec![],
            false
        ))
        .is_empty()
    );
    assert_eq!(
        query(criterion(
            SourceRelationshipDirection::Incoming,
            key("items", b),
            vec![],
            true
        )),
        vec![key("actors", actor)]
    );
    assert!(
        query(criterion(
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            vec![],
            true
        ))
        .is_empty()
    );
    assert_eq!(
        query(criterion(
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            owned(child_a),
            true
        )),
        vec![key("items", "ffffffffffffffff")]
    );
    assert!(
        query(criterion(
            SourceRelationshipDirection::Incoming,
            key("actors", actor),
            owned(child_b),
            true
        ))
        .is_empty()
    );
}

#[test]
fn occurrence_paging_has_complete_total_beyond_preview_budget() {
    let target = "aaaaaaaaaaaaaaaa";
    let html = format!(
        "<p>{}</p>",
        format!("@UUID[Compendium.pf2e.items.Item.{target}] ").repeat(125)
    );
    let (service, _artifact) = open_input_fixture(input(
        vec![
            effect(target, "Target", ""),
            effect("bbbbbbbbbbbbbbbb", "Source", &html),
        ],
        false,
    ))
    .unwrap();
    let constraint = criterion(
        SourceRelationshipDirection::Incoming,
        key("items", target),
        vec![],
        false,
    );
    let result = key("items", "bbbbbbbbbbbbbbbb");
    let first = service
        .index
        .relationship_occurrence_page(RelationshipOccurrencePageRequest {
            constraint: &constraint,
            result: &result,
            offset: 0,
            limit: 100,
        })
        .unwrap();
    let second = service
        .index
        .relationship_occurrence_page(RelationshipOccurrencePageRequest {
            constraint: &constraint,
            result: &result,
            offset: 100,
            limit: 100,
        })
        .unwrap();
    assert_eq!(
        (
            first.total,
            first.occurrences.len(),
            second.total,
            second.occurrences.len()
        ),
        (125, 100, 125, 25)
    );
    assert_ne!(
        first.occurrences.last().unwrap().ordinal,
        second.occurrences.first().unwrap().ordinal
    );
}
#[test]
fn remaster_preference_does_not_rebind_an_exact_reference() {
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
            evidence: "source relationship".into(),
        });
    let (service, _artifact) = open_input_fixture(fixture).unwrap();
    let constraints = [criterion(
        SourceRelationshipDirection::Incoming,
        key("items", "aaaaaaaaaaaaaaaa"),
        vec![],
        true,
    )];
    let result = service
        .list_records(
            ListRecordsRequest::new(None, SearchPage::default()).with_scope(
                RecordScope::Constrained {
                    keys: None,
                    relationships: &constraints,
                },
            ),
        )
        .unwrap();
    assert_eq!(result.record_keys, vec![key("items", "bbbbbbbbbbbbbbbb")]);
}

#[test]
fn relationship_scoped_discovery_reports_unavailable_and_keeps_count_free_vocabulary() {
    let (mut service, _artifact) =
        open_input_fixture(input(vec![effect("aaaaaaaaaaaaaaaa", "Target", "")], false)).unwrap();
    let constraints = [criterion(
        SourceRelationshipDirection::Incoming,
        key("items", "aaaaaaaaaaaaaaaa"),
        vec![],
        true,
    )];
    let scope = RecordScope::Constrained {
        keys: None,
        relationships: &constraints,
    };
    let context = FilterDiscoveryContext {
        search_profile: None,
        text: None,
        mode: RetrievalMode::Fts,
        scope,
        prefer_remaster: true,
        filter: None,
    };
    assert!(matches!(
        service
            .discover_filter_counts(DiscoverFilterCountsRequest {
                field: "kind",
                clause_id: None,
                context
            })
            .unwrap(),
        atlas_domain::QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::RelationshipContext
        }
    ));
    assert!(matches!(
        service
            .discover_filter_value_counts(DiscoverFilterValueCountsRequest {
                field: "kind",
                clause_id: None,
                context,
                values: &[]
            })
            .unwrap(),
        atlas_domain::QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::RelationshipContext
        }
    ));
    assert!(matches!(
        service
            .discover_filter_values(DiscoverFilterValuesRequest {
                field: "kind",
                clause_id: None,
                context,
                only_matching: true,
                text: None,
                offset: 0,
                limit: 20
            })
            .unwrap(),
        atlas_domain::QueryFacetOutcome::Unavailable {
            reason: atlas_domain::FacetUnavailableReason::RelationshipContext
        }
    ));
    assert!(
        !service
            .discover_filter_vocabulary(DiscoverFilterVocabularyRequest {
                field: "kind",
                scope,
                text: None,
                offset: 0,
                limit: 20,
                selected_values: &[]
            })
            .unwrap()
            .options
            .is_empty()
    );
}
