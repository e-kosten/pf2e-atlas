use crate::{
    test_support::{input, open_input_fixture, record},
    *,
};
use atlas_domain::{OwnedContentIdentity, OwnedContentLocator};
use atlas_domain::{QueryPredicate, RecordKey};

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
pub(super) fn criterion(
    direction: SourceRelationshipDirection,
    record: RecordKey,
    owners: Vec<OwnedContentLocator>,
    include_contents: bool,
) -> atlas_domain::QueryRelationship {
    atlas_domain::QueryRelationship {
        direction: match direction {
            SourceRelationshipDirection::Incoming => {
                atlas_domain::QueryRelationshipDirection::Incoming
            }
            SourceRelationshipDirection::Outgoing => {
                atlas_domain::QueryRelationshipDirection::Outgoing
            }
        },
        selection: atlas_domain::QueryRecordSelection {
            record_key: record,
            owners,
            field: None,
            passage: None,
            source_fingerprint: None,
            spell_rank: None,
        },
        include_contents,
    }
}
pub(super) fn link_filter(
    ordinary: Option<&QueryPredicate>,
    links: &[atlas_domain::QueryRelationship],
) -> QueryPredicate {
    let mut children = ordinary.into_iter().cloned().collect::<Vec<_>>();
    children.extend(links.iter().cloned().map(|relationship| {
        QueryPredicate::new(atlas_domain::QueryExpression::Relationship { relationship })
    }));
    let mut filter = QueryPredicate::new(atlas_domain::QueryExpression::AllOf { children });
    atlas_domain::assign_relationship_clause_ids(&mut filter).unwrap();
    filter
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
    let links = link_filter(None, &constraints);
    let scope = RecordScope::All;
    let result = service
        .list_records(
            ListRecordsRequest::new(Some(&links), SearchPage::new(1, 100).unwrap())
                .with_scope(scope),
        )
        .unwrap();
    assert_eq!((result.total, result.records.len()), (125, 100));
    let last = service
        .list_records(
            ListRecordsRequest::new(Some(&links), SearchPage::new(2, 100).unwrap())
                .with_scope(scope),
        )
        .unwrap();
    assert_eq!(last.records.len(), 25);
    let filter: QueryPredicate = serde_json::from_value(
        json!({"kind":"compare","field":"kind","op":"eq","value":"equipment"}),
    )
    .unwrap();
    let filter = link_filter(Some(&filter), &constraints);
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
            relationship: &constraints[0],
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
    let scoped = RecordScope::Keys(&scoped_keys);
    assert_eq!(
        service
            .list_records(
                ListRecordsRequest::new(Some(&links), SearchPage::default()).with_scope(scoped)
            )
            .unwrap()
            .total,
        1
    );
    let similar = service
        .similar_records(SimilarRecordRequest {
            content_scope: atlas_domain::SearchContentScope::All,
            sort: atlas_domain::ResultSort::Relevance {},
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
            .list_records(ListRecordsRequest::new(
                Some(&link_filter(None, &all)),
                SearchPage::default()
            ))
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
    let query = |relationship: atlas_domain::QueryRelationship| {
        service
            .list_records(ListRecordsRequest::new(
                Some(&link_filter(None, &[relationship])),
                SearchPage::default(),
            ))
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
    let mut selected_text = criterion(
        SourceRelationshipDirection::Incoming,
        key("actors", actor),
        owned(child_a),
        true,
    );
    selected_text.selection.field = Some("/system/description/value".into());
    assert_eq!(
        query(selected_text),
        vec![key("items", "ffffffffffffffff")],
        "incoming target owner remains exact when navigation retains its selected prose field"
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
            relationship: &constraint,
            result: &result,
            offset: 0,
            limit: 100,
        })
        .unwrap();
    let second = service
        .index
        .relationship_occurrence_page(RelationshipOccurrencePageRequest {
            relationship: &constraint,
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
        .list_records(ListRecordsRequest::new(
            Some(&link_filter(None, &constraints)),
            SearchPage::default(),
        ))
        .unwrap();
    assert_eq!(result.record_keys, vec![key("items", "bbbbbbbbbbbbbbbb")]);
}

#[path = "relationship_count_tests.rs"]
mod counts;

#[test]
fn boolean_relationship_proofs_follow_successful_sql_branches_and_polarity() {
    let target = "aaaaaaaaaaaaaaaa";
    let source = key("items", "bbbbbbbbbbbbbbbb");
    let (service, _artifact) = open_input_fixture(input(
        vec![
            effect(target, "Target", ""),
            effect(
                "bbbbbbbbbbbbbbbb",
                "Source",
                &format!("<p>@UUID[Compendium.pf2e.items.Item.{target}]</p>"),
            ),
            effect("cccccccccccccccc", "Unlinked", ""),
        ],
        false,
    ))
    .unwrap();
    let a = format!(r#"links_to("items:{target}")"#);
    for (text, expected_roots, expected_proofs) in [
        (
            format!("({a} && source.type == 'weapon') || {a}"),
            1,
            vec![1],
        ),
        (format!("!({a} && source.type == 'weapon')"), 3, vec![]),
        (format!("!!({a})"), 1, vec![0]),
        (format!("!({a})"), 2, vec![]),
        (format!("{a} || level > 100"), 1, vec![0]),
        (format!("{a} && !(level > 100)"), 0, vec![]),
        (format!("({a} && !(level > 100)) || {a}"), 1, vec![1]),
    ] {
        let filter = service.parse_where(&text).unwrap().predicate().clone();
        let result = service
            .list_records(ListRecordsRequest::new(
                Some(&filter),
                SearchPage::default(),
            ))
            .unwrap();
        assert_eq!(result.total, expected_roots, "{text}");
        let clauses = atlas_domain::relationship_clauses(&filter).unwrap();
        let expected_proofs = expected_proofs
            .into_iter()
            .map(|i| clauses[i].0)
            .collect::<Vec<_>>();
        let proofs = service
            .relationship_proofs(&filter, std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            proofs.iter().map(|(_, id)| id.as_str()).collect::<Vec<_>>(),
            expected_proofs,
            "{text}"
        );
    }
}

#[test]
fn relationship_admission_checks_negated_and_short_circuited_selections() {
    let (service, _artifact) =
        open_input_fixture(input(vec![effect("aaaaaaaaaaaaaaaa", "Target", "")], false)).unwrap();
    for expression in [
        "!links_to('items:missing000000000')",
        "source.type == 'weapon' && links_to('items:missing000000000')",
        "source.type == 'effect' || links_to('items:missing000000000')",
    ] {
        // Parsing remains artifact-free in the index; retrieval validates every source.
        let parsed = atlas_index::parse_where(expression)
            .unwrap()
            .predicate()
            .clone();
        assert!(service.validate_filter(&parsed).is_err(), "{expression}");
        assert!(
            service
                .list_records(ListRecordsRequest::new(
                    Some(&parsed),
                    SearchPage::default()
                ))
                .is_err()
        );
    }
    let expression = "inventory.items.exists(i, links_to('items:aaaaaaaaaaaaaaaa'))";
    assert!(
        atlas_index::parse_where(expression).is_err(),
        "links must not imply authored-child matching"
    );
}
