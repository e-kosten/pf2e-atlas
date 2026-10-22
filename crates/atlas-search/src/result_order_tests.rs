use crate::{
    test_support::{input, open_input_fixture, record},
    *,
};
use atlas_domain::{RecordKey, ResultSort};
use serde_json::json;
fn key(i: usize) -> RecordKey {
    RecordKey::parse(&format!("items:{i:016}")).unwrap()
}
fn effect(
    i: usize,
    name: serde_json::Value,
    level: serde_json::Value,
) -> atlas_index::SourceArtifactRecordInput {
    record(
        "items",
        "Item",
        json!({"_id":format!("{i:016}"),"name":name,"type":"effect","system":{"level":level}}),
    )
}
fn attributes() -> [ResultSort; 6] {
    [
        ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Asc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Desc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::Level,
            atlas_domain::SortDirection::Asc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::Level,
            atlas_domain::SortDirection::Desc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::UnitPrice,
            atlas_domain::SortDirection::Asc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::UnitPrice,
            atlas_domain::SortDirection::Desc,
        ),
    ]
}
#[test]
fn sorting_complete_browse_and_lexical_sets_precedes_paging_beyond_key_and_page_budgets() {
    let records = (0..1107)
        .map(|i| {
            effect(
                i,
                json!(format!("Record {i:04}")),
                json!({"value":i as i32-550}),
            )
        })
        .collect();
    let (mut service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let mut browse = Vec::new();
    let mut lexical = Vec::new();
    for page in 1..=12 {
        let requested = SearchPage::new(page, 100).unwrap();
        let result = service
            .list_records(ListRecordsRequest::new(None, requested).with_sort(
                ResultSort::attribute(
                    atlas_domain::SortField::Level,
                    atlas_domain::SortDirection::Desc,
                ),
            ))
            .unwrap();
        assert_eq!(result.total, 1107);
        browse.extend(result.record_keys);
        let mut request = TextSearchRequest::new("Record", requested);
        request.mode = RetrievalMode::Fts;
        request.sort = ResultSort::attribute(
            atlas_domain::SortField::Level,
            atlas_domain::SortDirection::Desc,
        );
        let result = service.search_text(request).unwrap();
        assert!(result.coverage.exhaustive);
        assert_eq!(result.page.total, 1107);
        lexical.extend(result.records.into_iter().map(|r| r.record.key));
    }
    let expected = (0..1107).rev().map(key).collect::<Vec<_>>();
    assert_eq!(browse, expected);
    assert_eq!(lexical, expected);
    let ordered = service
        .order_record_keys(
            &(0..1107).map(key).collect::<Vec<_>>(),
            ResultSort::attribute(
                atlas_domain::SortField::Name,
                atlas_domain::SortDirection::Desc,
            ),
        )
        .unwrap();
    assert_eq!(
        ordered, expected,
        "complete key ordering must not chunk or cap input"
    );
}
#[test]
fn sorting_missing_values_and_numeric_ties_use_stable_names_and_exact_native_prices() {
    let missing = effect(3, json!("Delta missing"), json!({}));
    // No fallback from an unavailable spell rank to other record facts.
    let spell = record(
        "items",
        "Item",
        json!({"_id":"0000000000000005","type":"spell","name":"Echo rank","system":{"level":{"value":2}}}),
    );
    let records = vec![
        effect(0, json!("beta"), json!({"value":0})),
        effect(1, json!("Alpha"), json!({"value":0})),
        effect(2, json!("Negative"), json!({"value":-2})),
        missing,
        spell,
        effect(4, json!("Charlie null"), json!({"value":null})),
        effect(6, json!("Bravo invalid"), json!({"value":"bad"})),
        record(
            "items",
            "Item",
            json!({"_id":"0000000000000007","type":"action","name":"Alpha inapplicable"}),
        ),
        effect(8, serde_json::Value::Null, json!({"value":0})),
    ];
    let (service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let keys = (0..9).map(key).collect::<Vec<_>>();
    assert_eq!(
        service
            .order_record_keys(
                &keys,
                ResultSort::attribute(
                    atlas_domain::SortField::Level,
                    atlas_domain::SortDirection::Asc
                )
            )
            .unwrap(),
        [2, 1, 0, 8, 5, 7, 6, 4, 3].map(key)
    );
    assert_eq!(
        service
            .order_record_keys(
                &keys,
                ResultSort::attribute(
                    atlas_domain::SortField::Level,
                    atlas_domain::SortDirection::Desc
                )
            )
            .unwrap(),
        [5, 1, 0, 8, 2, 7, 6, 4, 3].map(key)
    );
    assert_eq!(
        service
            .order_record_keys(
                &keys,
                ResultSort::attribute(
                    atlas_domain::SortField::Name,
                    atlas_domain::SortDirection::Desc
                )
            )
            .unwrap()
            .last(),
        Some(&key(8))
    );
    let price = |i: usize, name: &str, value: serde_json::Value| {
        record(
            "items",
            "Item",
            json!({"_id":format!("{i:016}"),"type":"equipment","name":name,"system":{"price":value}}),
        )
    };
    let (service, _artifact) = open_input_fixture(input(
        vec![
            price(0, "Zero", json!({"value":{"cp":0}})),
            price(1, "Fraction", json!({"value":{"cp":1},"per":2})),
            price(2, "Big B", json!({"value":{"cp":9007199254740992u64}})),
            price(3, "Big Z", json!({"value":{"cp":9007199254740991u64}})),
            price(4, "Missing", json!({})),
            price(5, "Null", serde_json::Value::Null),
            price(6, "Invalid", json!({"value":{"cp":"bad"}})),
            effect(7, json!("Inapplicable"), json!({"value":1})),
        ],
        false,
    ))
    .unwrap();
    let keys = (0..8).map(key).collect::<Vec<_>>();
    service.reset_read_metrics();
    assert_eq!(
        service
            .order_record_keys(
                &keys,
                ResultSort::attribute(
                    atlas_domain::SortField::UnitPrice,
                    atlas_domain::SortDirection::Asc
                )
            )
            .unwrap(),
        [0, 1, 3, 2, 7, 6, 4, 5].map(key)
    );
    assert_eq!(
        service
            .order_record_keys(
                &keys,
                ResultSort::attribute(
                    atlas_domain::SortField::UnitPrice,
                    atlas_domain::SortDirection::Desc
                )
            )
            .unwrap(),
        [2, 3, 1, 0, 7, 6, 4, 5].map(key)
    );
    let prices = service.read_unit_prices(&keys).unwrap();
    assert_eq!(
        prices
            .iter()
            .find(|p| p.key == key(1))
            .unwrap()
            .copper
            .as_value(),
        Some(&0.5)
    );
    assert_eq!(service.read_metrics().source_body_decodes, 0);
    assert!(
        service
            .order_record_keys(
                &[key(99)],
                ResultSort::attribute(
                    atlas_domain::SortField::Name,
                    atlas_domain::SortDirection::Asc
                )
            )
            .is_err()
    );
}
#[test]
fn sorting_semantic_set_above_two_hundred_preserves_membership_window_and_witnesses() {
    let records = (0..321)
        .map(|i| effect(i, json!(format!("Record {i:04}")), json!({"value":i})))
        .collect();
    let (service, _artifact) = open_input_fixture(input(records, true)).unwrap();
    let read = |sort: ResultSort, page, size| {
        service
            .similar_records(SimilarRecordRequest {
                sort: sort.clone(),
                search_profile: None,
                record_key: &key(0),
                filter: None,
                scope: RecordScope::All,
                page: SearchPage::new(page, size).unwrap(),
                prefer_remaster: true,
            })
            .unwrap()
            .unwrap()
            .results
    };
    let mut baseline = std::collections::BTreeMap::new();
    for page in 1..=4 {
        let result = read(ResultSort::Relevance {}, page, 100);
        assert_eq!(result.page.total, 320);
        assert_eq!(result.coverage.semantic_unit_window, Some(1024));
        for row in result.records {
            baseline.insert(row.record.key, serde_json::to_value(row.matches).unwrap());
        }
    }
    for sort in attributes().into_iter().chain([
        ResultSort::Random { seed: 0 },
        ResultSort::Random { seed: u32::MAX },
        ResultSort::Attributes {
            rules: vec![
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::Level,
                    direction: atlas_domain::SortDirection::Desc,
                },
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::UnitPrice,
                    direction: atlas_domain::SortDirection::Asc,
                },
            ],
        },
    ]) {
        let mut actual = std::collections::BTreeMap::new();
        for page in 1..=7 {
            let result = read(sort.clone(), page, 53);
            assert_eq!(result.page.total, 320);
            assert_eq!(result.coverage.semantic_unit_window, Some(1024));
            assert!(!result.coverage.exhaustive);
            for row in result.records {
                actual.insert(row.record.key, serde_json::to_value(row.matches).unwrap());
            }
        }
        assert_eq!(actual, baseline);
    }
}
#[test]
fn sorting_hybrid_keeps_lexical_members_outside_semantic_window_and_ranking_strategy() {
    let records = (0..1100)
        .map(|i| effect(i, json!(format!("Record {i:04}")), json!({"value":i})))
        .collect();
    let (service, _artifact) = open_input_fixture(input(records, true)).unwrap();
    let query = service.predicate(None).unwrap();
    let profile = atlas_domain::SearchProfileId::new("bge-base");
    let mut vector = vec![0.0; 768];
    vector[0] = 1.0;
    for ranking in [
        atlas_domain::SearchRanking::Score,
        atlas_domain::SearchRanking::Rrf,
    ] {
        let (roots, window) = service
            .rank_candidates(crate::text::CandidateRequest {
                ranking,
                search_profile: &profile,
                text: "Record",
                mode: RetrievalMode::Hybrid,
                query: &query,
                scope: RecordScope::All,
                prefer: true,
                suppress: true,
                vector: Some(&vector),
                exclude: None,
            })
            .unwrap();
        assert_eq!(window, Some(1024));
        assert_eq!(
            roots.len(),
            1100,
            "lexical union must not inherit the semantic unit cap"
        );
        let state = |rows: &[crate::text::RankedRoot]| {
            rows.iter()
                .map(|r| {
                    (
                        r.key.clone(),
                        serde_json::to_value((
                            &r.witnesses,
                            r.lexical_rank,
                            r.similarity,
                            r.score,
                            r.tier,
                        ))
                        .unwrap(),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let original = state(&roots);
        for sort in [
            ResultSort::Random { seed: 0 },
            ResultSort::Random { seed: u32::MAX },
            ResultSort::Attributes {
                rules: vec![
                    atlas_domain::SortRule {
                        field: atlas_domain::SortField::Level,
                        direction: atlas_domain::SortDirection::Desc,
                    },
                    atlas_domain::SortRule {
                        field: atlas_domain::SortField::Name,
                        direction: atlas_domain::SortDirection::Asc,
                    },
                ],
            },
        ] {
            assert_eq!(
                state(&service.order_ranked_roots(roots.clone(), sort).unwrap()),
                original
            );
        }
        let sorted = service
            .order_ranked_roots(
                roots,
                ResultSort::attribute(
                    atlas_domain::SortField::Level,
                    atlas_domain::SortDirection::Desc,
                ),
            )
            .unwrap();
        assert_eq!(
            sorted.iter().map(|r| r.key.clone()).collect::<Vec<_>>(),
            (0..1100).rev().map(key).collect::<Vec<_>>()
        );
    }
}

#[test]
fn sorting_ties_keep_canonical_keys_and_relationship_scopes_do_not_rebind_editions() {
    let (service, _artifact) = open_input_fixture(input(
        vec![
            effect(2, json!("Same"), json!({"value":1})),
            effect(1, json!("same"), json!({"value":1})),
        ],
        false,
    ))
    .unwrap();
    for sort in attributes().into_iter().chain([ResultSort::Attributes {
        rules: vec![
            atlas_domain::SortRule {
                field: atlas_domain::SortField::Level,
                direction: atlas_domain::SortDirection::Desc,
            },
            atlas_domain::SortRule {
                field: atlas_domain::SortField::UnitPrice,
                direction: atlas_domain::SortDirection::Asc,
            },
        ],
    }]) {
        assert_eq!(
            service.order_record_keys(&[key(2), key(1)], sort).unwrap(),
            vec![key(1), key(2)]
        );
    }
    let mut fixture = input(
        vec![
            record(
                "items",
                "Item",
                json!({"_id":"0000000000000000","type":"equipment","name":"Legacy","_stats":{"compendiumSource":"Compendium.pf2e.items.Item.0000000000000002"}}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"0000000000000001","type":"equipment","name":"Replacement"}),
            ),
            record(
                "items",
                "Item",
                json!({"_id":"0000000000000002","type":"armor","name":"Armor"}),
            ),
        ],
        false,
    );
    fixture
        .remaster_pairs
        .push(atlas_index::SourceRemasterPairInput {
            legacy: key(0),
            remaster: key(1),
            evidence: "verified source pair".into(),
        });
    let (service, _artifact) = open_input_fixture(fixture).unwrap();
    let keys = [key(0), key(1)];
    let relationships = [RelationshipConstraint {
        direction: SourceRelationshipDirection::Incoming,
        record: key(2),
        owners: vec![],
        field: None,
        include_contents: false,
    }];
    let filter = serde_json::from_value(
        json!({"kind":"compare","field":"kind","op":"eq","value":"equipment"}),
    )
    .unwrap();
    for sort in attributes().into_iter().chain([
        ResultSort::Random { seed: 0 },
        ResultSort::Random { seed: u32::MAX },
        ResultSort::Attributes {
            rules: vec![
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::Level,
                    direction: atlas_domain::SortDirection::Desc,
                },
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::UnitPrice,
                    direction: atlas_domain::SortDirection::Asc,
                },
            ],
        },
    ]) {
        let result = service
            .list_records(
                ListRecordsRequest::new(Some(&filter), SearchPage::default())
                    .with_sort(sort)
                    .with_scope(RecordScope::Constrained {
                        keys: Some(&keys),
                        relationships: &relationships,
                    }),
            )
            .unwrap();
        assert_eq!(result.record_keys, vec![key(0)]);
    }
}
#[test]
fn sorting_adaptive_semantic_windows_preserves_expansion_for_dense_contents() {
    let spells=(0..1050).map(|i|json!({"_id":format!("{i:016x}"),"name":format!("Spell {i}"),"type":"spell","system":{"description":{"value":format!("<p>Copied spell body {i}.</p>")}}})).collect::<Vec<_>>();
    let actor = record(
        "actors",
        "Actor",
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Dense actor","type":"npc","system":{"details":{"level":{"value":9}}},"items":spells}),
    );
    let mut fixture = input(
        vec![
            effect(0, json!("Seed"), json!({"value":1})),
            actor,
            effect(1, json!("Sparse"), json!({"value":-1})),
        ],
        true,
    );
    let actor_key = RecordKey::parse("actors:bbbbbbbbbbbbbbbb").unwrap();
    for unit in &mut fixture.semantic_units {
        if unit.record == actor_key {
            unit.vector.fill(0.0);
            unit.vector[0] = 1.0;
        }
    }
    let (service, _artifact) = open_input_fixture(fixture).unwrap();
    for sort in [
        ResultSort::Relevance {},
        ResultSort::Random { seed: 0 },
        ResultSort::Random { seed: u32::MAX },
        ResultSort::Attributes {
            rules: vec![
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::Level,
                    direction: atlas_domain::SortDirection::Desc,
                },
                atlas_domain::SortRule {
                    field: atlas_domain::SortField::Name,
                    direction: atlas_domain::SortDirection::Asc,
                },
            ],
        },
        ResultSort::attribute(
            atlas_domain::SortField::Level,
            atlas_domain::SortDirection::Asc,
        ),
        ResultSort::attribute(
            atlas_domain::SortField::Level,
            atlas_domain::SortDirection::Desc,
        ),
    ] {
        let result = service
            .similar_records(SimilarRecordRequest {
                sort: sort.clone(),
                search_profile: None,
                record_key: &key(0),
                filter: None,
                scope: RecordScope::All,
                page: SearchPage::new(1, 100).unwrap(),
                prefer_remaster: true,
            })
            .unwrap()
            .unwrap()
            .results;
        assert_eq!(result.coverage.semantic_unit_window, Some(4096));
        assert_eq!(result.page.total, 2);
        if sort
            == ResultSort::attribute(
                atlas_domain::SortField::Level,
                atlas_domain::SortDirection::Asc,
            )
        {
            assert_eq!(result.records[0].record.key, key(1));
        }
    }
}

#[test]
fn sorting_rule_precedence_continues_unavailable_ties_and_only_reverses_values() {
    use atlas_domain::{SortDirection::*, SortField::*, SortRule};
    let equipment = |i, name, level, cp| {
        record(
            "items",
            "Item",
            json!({"_id":format!("{i:016}"),"type":"equipment","name":name,"system":{"level":level,"price":{"value":{"cp":cp},"per":1}}}),
        )
    };
    let records = vec![
        equipment(0, "Zulu", json!({"value":1}), 2),
        equipment(1, "Alpha", json!({"value":1}), 3),
        equipment(2, "Beta", json!({"value":1}), 3),
        equipment(3, "Delta", json!({}), 1),
        equipment(4, "Gamma", json!({"value":null}), 4),
        equipment(5, "Epsilon", json!({"value":"invalid"}), 4),
        record(
            "items",
            "Item",
            json!({"_id":"0000000000000006","name":"Action","type":"action"}),
        ),
    ];
    let (service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let keys = (0..7).map(key).collect::<Vec<_>>();
    for (direction, expected) in [
        (Asc, vec![2, 1, 0, 4, 5, 3, 6]),
        (Desc, vec![2, 1, 0, 4, 5, 3, 6]),
    ] {
        let sort = ResultSort::Attributes {
            rules: vec![
                SortRule {
                    field: Level,
                    direction,
                },
                SortRule {
                    field: UnitPrice,
                    direction: Desc,
                },
                SortRule {
                    field: Name,
                    direction: Desc,
                },
            ],
        };
        assert_eq!(
            service.order_record_keys(&keys, sort).unwrap(),
            expected.into_iter().map(key).collect::<Vec<_>>()
        );
    }
    // All price values unavailable: level remains an active second rule, including
    // valid negative values. Unavailable level ties fall through to name descending.
    let records = vec![
        effect(0, json!("Alpha"), json!({"value":-2})),
        effect(1, json!("Beta"), json!({"value":0})),
        effect(2, json!("Gamma"), json!({})),
        effect(3, json!("Delta"), json!({"value":null})),
    ];
    let (service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let sort = ResultSort::Attributes {
        rules: vec![
            SortRule {
                field: UnitPrice,
                direction: Asc,
            },
            SortRule {
                field: Level,
                direction: Desc,
            },
            SortRule {
                field: Name,
                direction: Desc,
            },
        ],
    };
    assert_eq!(
        service
            .order_record_keys(&(0..4).map(key).collect::<Vec<_>>(), sort)
            .unwrap(),
        vec![key(1), key(0), key(2), key(3)]
    );
}

#[test]
fn sorting_random_orders_the_complete_eligible_set_before_any_page_and_keeps_scopes() {
    let records = (0..1107)
        .map(|i| effect(i, json!(format!("Record {i}")), json!({"value":i})))
        .collect();
    let (mut service, _artifact) = open_input_fixture(input(records, false)).unwrap();
    let all = (0..1107).map(key).collect::<Vec<_>>();
    let mut seed_orders = Vec::new();
    service.reset_read_metrics();
    for seed in [0, u32::MAX] {
        let sort = ResultSort::Random { seed };
        let expected = service.order_record_keys(&all, sort.clone()).unwrap();
        for size in [17, 100] {
            let mut actual = Vec::new();
            let mut lexical = Vec::new();
            for page in 1..=1107u32.div_ceil(size) {
                let page = SearchPage::new(page, size).unwrap();
                let result = service
                    .list_records(ListRecordsRequest::new(None, page).with_sort(sort.clone()))
                    .unwrap();
                assert_eq!(result.total, 1107);
                actual.extend(result.record_keys);
                let mut request = TextSearchRequest::new("Record", page);
                request.mode = RetrievalMode::Fts;
                request.sort = sort.clone();
                lexical.extend(
                    service
                        .search_text(request)
                        .unwrap()
                        .records
                        .into_iter()
                        .map(|r| r.record.key),
                );
            }
            assert_eq!(actual, expected);
            assert_eq!(lexical, expected);
        }
        let allowed = [key(0), key(2), key(7), key(100)];
        let scoped = service
            .list_records(
                ListRecordsRequest::new(None, SearchPage::new(1, 100).unwrap())
                    .with_scope(RecordScope::Keys(&allowed))
                    .with_sort(sort),
            )
            .unwrap();
        assert_eq!(
            scoped.record_keys,
            expected
                .iter()
                .filter(|key| allowed.contains(key))
                .cloned()
                .collect::<Vec<_>>()
        );
        seed_orders.push(expected);
    }
    assert_ne!(seed_orders[0], seed_orders[1]);
    assert_eq!(service.read_metrics().source_body_decodes, 0);
}
