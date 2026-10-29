use crate::{
    ReadExecutionContext, executor::RetrievalExecutor, test_support::fixture_worker_with_executor,
};
use atlas_app_model::*;
use atlas_domain::ResultSort;
use atlas_search::test_support::{input, open_input_fixture, record};
use serde_json::json;
fn fixture() -> crate::test_support::FixtureWorker {
    fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        let equipment = |id: &str, name: &str, cp: i64, per: i64| {
            record(
                "items",
                "Item",
                json!({"_id":id,"type":"equipment","name":name,"system":{"level":{"value":1},"price":{"value":{"cp":cp},"per":per}}}),
            )
        };
        let mut fixture = input(
            vec![
                equipment("0000000000000000", "Item Expensive", 100, 1),
                equipment("0000000000000001", "Item Cheap", 1, 2),
                equipment("0000000000000002", "Item Legacy", 25, 1),
                equipment("0000000000000003", "Item Remaster", 50, 1),
            ],
            false,
        );
        fixture.remaster_pairs.push(serde_json::from_value(json!({"legacy":"items:0000000000000002","remaster":"items:0000000000000003","evidence":"verified source pair"})).unwrap());
        Ok(open_input_fixture(fixture)?)
    }))
}
fn search(sort: Option<ResultSort>, criteria: SearchCriteria) -> SearchResultsRequest {
    SearchResultsRequest {
        sort,

        ranking: None,
        search_profile: None,
        criteria,
        page: SearchPageRequest {
            number: 1,
            size: 100,
        },
        expected_artifact_generation: None,
    }
}
#[test]
fn sorting_search_resolves_defaults_rejects_before_inference_and_reads_price_labels_without_bodies()
{
    let f = fixture();
    let execution = ReadExecutionContext::uncancellable();
    let automatic = f
        .worker
        .search_page(
            search(None, SearchCriteria::ListRecords { filter: None }),
            &execution,
        )
        .unwrap();
    assert_eq!(
        automatic.sort,
        ResultSort::attribute(
            atlas_domain::SortField::Name,
            atlas_domain::SortDirection::Asc
        )
    );
    assert!(
        automatic
            .rows
            .iter()
            .all(|r| r.record.unit_price_label.is_none())
    );
    f.worker
        .with_retrieval(&execution, |r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let ordered = f
        .worker
        .search_page(
            search(
                Some(ResultSort::attribute(
                    atlas_domain::SortField::UnitPrice,
                    atlas_domain::SortDirection::Asc,
                )),
                SearchCriteria::ListRecords { filter: None },
            ),
            &execution,
        )
        .unwrap();
    assert_eq!(
        ordered.sort,
        ResultSort::attribute(
            atlas_domain::SortField::UnitPrice,
            atlas_domain::SortDirection::Asc
        )
    );
    assert_eq!(ordered.rows[0].record.title, "Item Cheap");
    assert_eq!(
        ordered.rows[0].record.unit_price_label.as_deref(),
        Some("0.005 gp each")
    );
    assert_eq!(
        ordered
            .rows
            .last()
            .unwrap()
            .record
            .unit_price_label
            .as_deref(),
        Some("1 gp each")
    );
    let metrics = f
        .worker
        .with_retrieval(&execution, |r| Ok(r.read_metrics()))
        .unwrap();
    assert_eq!(metrics.source_body_decodes, 0);
    let mut invalid = search(
        Some(ResultSort::SavedOrder {}),
        SearchCriteria::TextSearch {
            search_content: None,
            query: "query requiring unavailable native inference".into(),
            filter: None,
            mode: RetrievalModeView::Semantic,
        },
    );
    assert_eq!(
        f.worker
            .search_page(invalid.clone(), &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    invalid.sort = Some(ResultSort::Relevance {});
    invalid.criteria = SearchCriteria::ListRecords { filter: None };
    assert_eq!(
        f.worker
            .search_page(invalid, &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    let mut stale = search(
        Some(ResultSort::attribute(
            atlas_domain::SortField::UnitPrice,
            atlas_domain::SortDirection::Desc,
        )),
        SearchCriteria::TextSearch {
            search_content: None,
            query: "query requiring unavailable native inference".into(),
            filter: None,
            mode: RetrievalModeView::Semantic,
        },
    );
    stale.expected_artifact_generation = Some("stale".into());
    assert_eq!(
        f.worker
            .search_page(stale, &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::ArtifactGenerationChanged
    );
}
#[test]
fn sorting_saved_lists_is_temporary_and_retains_unavailable_tails_and_explicit_editions() {
    let f = fixture();
    let keys = [
        "items:0000000000000000",
        "items:gone000000000001",
        "items:0000000000000001",
        "items:gone000000000002",
        "items:0000000000000002",
        "items:0000000000000003",
    ];
    f.worker
        .with_write(|write| {
            Ok(write
                .local_state()
                .saved_lists()
                .import(atlas_local_state::ImportSavedList {
                    slug: "ordered".into(),
                    name: "Ordered".into(),
                    description: None,
                    items: keys
                        .iter()
                        .map(|key| atlas_local_state::ImportSavedListItem {
                            record_key: (*key).into(),
                            record_title_snapshot: format!("Snapshot {key}"),
                            record_kind_snapshot: None,
                            note: Some("Keep note".into()),
                        })
                        .collect(),
                    tags: vec![],
                    replace: false,
                })?)
        })
        .unwrap();
    let before = f.worker.saved_list("ordered").unwrap();
    assert_eq!(before.sort, ResultSort::SavedOrder {});
    assert_eq!(
        before
            .items
            .iter()
            .map(|i| i.record_key.as_str())
            .collect::<Vec<_>>(),
        keys
    );
    let sorted = f
        .worker
        .filter_saved_list(FilterSavedListRequest {
            sort: Some(ResultSort::attribute(
                atlas_domain::SortField::UnitPrice,
                atlas_domain::SortDirection::Asc,
            )),
            list_ref: "ordered".into(),
            query: None,
            filter: None,
        })
        .unwrap();
    assert_eq!(
        sorted
            .items
            .iter()
            .map(|i| i.record_key.as_str())
            .collect::<Vec<_>>(),
        [keys[2], keys[4], keys[5], keys[0], keys[1], keys[3]]
    );
    assert_eq!(
        sorted.items[0]
            .record
            .as_ref()
            .unwrap()
            .unit_price_label
            .as_deref(),
        Some("0.005 gp each")
    );
    for seed in [0, u32::MAX] {
        let request = FilterSavedListRequest {
            sort: Some(ResultSort::Random { seed }),
            list_ref: "ordered".into(),
            query: None,
            filter: None,
        };
        let shuffled = f.worker.filter_saved_list(request.clone()).unwrap();
        assert_eq!(shuffled, f.worker.filter_saved_list(request).unwrap());
        assert_eq!(shuffled.items.len(), keys.len());
        assert_eq!(shuffled.items[4].record_key, keys[1]);
        assert_eq!(shuffled.items[5].record_key, keys[3]);
        let mut active = shuffled.items[..4]
            .iter()
            .map(|item| item.record_key.as_str())
            .collect::<Vec<_>>();
        active.sort_unstable();
        assert_eq!(active, [keys[0], keys[2], keys[4], keys[5]]);
        assert!(shuffled.items.iter().all(|i| {
            i.record
                .as_ref()
                .is_none_or(|r| r.unit_price_label.is_none())
        }));
    }
    let filtered = f
        .worker
        .filter_saved_list(FilterSavedListRequest {
            sort: None,
            list_ref: "ordered".into(),
            query: Some("Item".into()),
            filter: None,
        })
        .unwrap();
    assert_eq!(filtered.sort, ResultSort::SavedOrder {});
    assert_eq!(
        filtered
            .items
            .iter()
            .map(|i| i.record_key.as_str())
            .collect::<Vec<_>>(),
        [keys[0], keys[2], keys[5]]
    );
    assert_eq!(
        f.worker.saved_list("ordered").unwrap(),
        before,
        "view sorting must not mutate positions, notes, snapshots or saved order"
    );
    assert_eq!(
        f.worker
            .filter_saved_list(FilterSavedListRequest {
                sort: Some(ResultSort::Relevance {}),
                list_ref: "ordered".into(),
                query: None,
                filter: None
            })
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
}

#[test]
fn sorting_multirule_labels_use_any_price_rule_and_invalid_rules_fail_before_inference() {
    use atlas_domain::{SortDirection::*, SortField::*, SortRule};
    let fixture = fixture();
    let execution = ReadExecutionContext::uncancellable();
    fixture
        .worker
        .with_retrieval(&execution, |r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    for fields in [vec![Level, UnitPrice], vec![Level, Name, UnitPrice]] {
        let sort = ResultSort::Attributes {
            rules: fields
                .into_iter()
                .map(|field| SortRule {
                    field,
                    direction: Asc,
                })
                .collect(),
        };
        let result = fixture
            .worker
            .search_page(
                search(
                    Some(sort.clone()),
                    SearchCriteria::ListRecords { filter: None },
                ),
                &execution,
            )
            .unwrap();
        assert_eq!(result.sort, sort);
        assert!(
            result
                .rows
                .iter()
                .all(|r| r.record.unit_price_label.is_some())
        );
    }
    assert_eq!(
        fixture
            .worker
            .with_retrieval(&execution, |r| Ok(r.read_metrics()))
            .unwrap()
            .source_body_decodes,
        0
    );
    let duplicate = SortRule {
        field: Name,
        direction: Asc,
    };
    for rules in [vec![], vec![duplicate, duplicate]] {
        let request = search(
            Some(ResultSort::Attributes { rules }),
            SearchCriteria::TextSearch {
                search_content: None,
                query: "would require unavailable inference".into(),
                filter: None,
                mode: RetrievalModeView::Semantic,
            },
        );
        assert_eq!(
            fixture
                .worker
                .search_page(request, &execution)
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::InvalidRequest
        );
    }
}

#[test]
fn sorting_trait_modes_reject_provided_order_outside_similar() {
    let fixture = fixture();
    let execution = ReadExecutionContext::uncancellable();
    for kind in [
        TraitAssociationKind::Members,
        TraitAssociationKind::References,
    ] {
        let request = TraitRecordsRequest {
            sort: Some(ResultSort::Random { seed: 0 }),
            trait_key: "trait:fire".into(),
            kind,
            page: SearchPageRequest {
                number: 1,
                size: 20,
            },
            expected_artifact_generation: "not-consumed-for-invalid-order".into(),
            search_profile: None,
            ranking: None,
        };
        assert_eq!(
            fixture
                .worker
                .trait_records(request, &execution)
                .unwrap_err()
                .into_app_error()
                .code,
            AppErrorCode::InvalidRequest
        );
    }
    let request = TraitRecordsRequest {
        sort: Some(ResultSort::Attributes { rules: vec![] }),
        trait_key: "trait:fire".into(),
        kind: TraitAssociationKind::Similar,
        page: SearchPageRequest {
            number: 1,
            size: 20,
        },
        expected_artifact_generation: "not-consumed-for-invalid-order".into(),
        search_profile: None,
        ranking: None,
    };
    assert_eq!(
        fixture
            .worker
            .trait_records(request, &execution)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
}
