use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

#[test]
fn similarity_windows_page_filter_exclude_seed_and_expire() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture((0..4).map(|i| record("items", "Item", json!({"_id":format!("{:016}", i),"name":format!("Record {i}"),"type":"effect","system":{"level":{"value":i},"traits":{"value":[]}}}))).collect(), true)?)
        },
    ));
    let filter =
        serde_json::from_value(json!({"kind":"compare","field":"item.level","op":"gte","value":2}))
            .unwrap();
    let request = OpenResultWindowRequest {
        mode: ResultWindowMode::SimilarRecords {
            seed_record_key: "items:0000000000000000".into(),
            filter: Some(filter),
        },
        page: SearchPageRequest { number: 1, size: 1 },
    };
    let first = f.worker.open_result_window(request.clone()).unwrap();
    let ResultWindowModeSummary::SimilarRecords { seed } = &first.mode else {
        panic!("similar mode");
    };
    assert_eq!(seed.title, "Record 0");
    assert_eq!(first.page.total, 2);
    assert!(first.page.has_more);
    assert!(!first.coverage.as_ref().unwrap().exhaustive);
    assert_ne!(first.rows[0].record.record_key, seed.record_key);
    let second = f
        .worker
        .read_result_window_page(
            first.window_id,
            ReadResultWindowPageRequest {
                page: SearchPageRequest { number: 2, size: 1 },
            },
        )
        .unwrap();
    assert_ne!(
        first.rows[0].record.record_key,
        second.rows[0].record.record_key
    );
    assert!(!second.page.has_more);
    assert!(
        serde_json::to_value(seed)
            .unwrap()
            .get("presentation")
            .is_none()
    );
    for _ in 0..crate::windows::MAX_RESULT_WINDOWS {
        f.worker.open_result_window(request.clone()).unwrap();
    }
    let error = f
        .worker
        .read_result_window_page(
            first.window_id,
            ReadResultWindowPageRequest { page: request.page },
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::WindowExpired);
    let mut invalid = request.clone();
    invalid.page.size = 101;
    assert!(f.worker.open_result_window(invalid).is_err());
    let missing = OpenResultWindowRequest {
        mode: ResultWindowMode::SimilarRecords {
            seed_record_key: "items:missing000000000".into(),
            filter: None,
        },
        page: request.page,
    };
    assert_eq!(
        f.worker
            .open_result_window(missing)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::RecordNotFound
    );
}

#[test]
fn related_availability_is_exact_owner_scoped_generation_checked_and_nonempty() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"actor00000000000","name":"Reader","type":"npc","system":{"details":{"publicNotes":"<p>@UUID[Compendium.pf2e.items.Item.target0000000000]</p>"}},"items":[{"_id":"owned00000000000","type":"action","name":"Local action","system":{"description":{"value":"<p>@UUID[Compendium.pf2e.items.Item.target0000000001]</p>"}}},{"type":"action","name":"Local identity","system":{"description":{"value":"<p>Text</p>"}}}]}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"target0000000000","type":"effect","name":"Target A"}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"target0000000001","type":"effect","name":"Target B"}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"weapon0000000000","type":"weapon","name":"Lesser Weapon","system":{"group":"sword"}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let root = f.worker.record_detail("actors:actor00000000000").unwrap();
    let request = RelatedRecordsRequest {
        selection: root.selected.clone(),
        artifact_generation: root.artifact_generation.clone(),
    };
    let result = f.worker.related_records(request.clone()).unwrap();
    let RelatedCategoryView::Content { content } = result.references else {
        panic!("root references");
    };
    assert_eq!(
        content.outgoing.occurrences.len(),
        1,
        "must not broaden root to child references"
    );
    assert!(content.outgoing.occurrences[0].source.owners.is_empty());
    assert!(matches!(result.counterparts, RelatedCategoryView::Empty));
    let mut owned = request.clone();
    owned.selection.owners = vec![atlas_record::source_content::OwnedContentLocator {
        collection: "/items".into(),
        identity: atlas_record::source_content::OwnedContentIdentity::Stable(
            "owned00000000000".into(),
        ),
    }];
    let result = f.worker.related_records(owned.clone()).unwrap();
    let RelatedCategoryView::Content { content } = result.references else {
        panic!("owned references");
    };
    assert_eq!(content.outgoing.occurrences.len(), 1);
    assert_eq!(
        content.outgoing.occurrences[0].source.owners,
        owned.selection.owners
    );
    assert!(matches!(
        result.counterparts,
        RelatedCategoryView::Unavailable { .. }
    ));
    assert!(matches!(
        result.variants,
        RelatedCategoryView::Unavailable { .. }
    ));
    owned.selection.owners[0].identity =
        atlas_record::source_content::OwnedContentIdentity::SnapshotLocal { index: 1 };
    assert!(
        f.worker.related_records(owned).is_err(),
        "snapshot-local fingerprint is independently required"
    );
    let mut stale = request;
    stale.artifact_generation = "stale-preparation".into();
    assert_eq!(
        f.worker
            .related_records(stale)
            .unwrap_err()
            .into_app_error()
            .recoverable_action,
        Some(AppRecoverableAction::ReloadRecord)
    );
    let empty = f.worker.record_detail("items:target0000000001").unwrap();
    let result = f
        .worker
        .related_records(RelatedRecordsRequest {
            selection: empty.selected,
            artifact_generation: empty.artifact_generation,
        })
        .unwrap();
    assert!(matches!(result.variants, RelatedCategoryView::Empty));
    let unknown = f.worker.record_detail("items:weapon0000000000").unwrap();
    let result = f
        .worker
        .related_records(RelatedRecordsRequest {
            selection: unknown.selected,
            artifact_generation: unknown.artifact_generation,
        })
        .unwrap();
    assert!(matches!(
        result.variants,
        RelatedCategoryView::Unavailable { .. }
    ));
}
