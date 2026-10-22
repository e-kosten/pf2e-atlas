use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

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
    owned.selection.owners = vec![atlas_domain::OwnedContentLocator {
        collection: "/items".into(),
        identity: atlas_domain::OwnedContentIdentity::Stable("owned00000000000".into()),
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
        atlas_domain::OwnedContentIdentity::SnapshotLocal { index: 1 };
    assert!(
        f.worker.related_records(owned).is_err(),
        "snapshot-local fingerprint is independently required"
    );
    let mut stale = request;
    stale.artifact_generation = "stale-preparation".into();
    let error = f
        .worker
        .related_records(stale)
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
    assert_eq!(
        error.recoverable_action,
        Some(AppRecoverableAction::ReloadRecord)
    );
    assert_eq!(
        error.details,
        Some(json!({
            "expected_artifact_generation": "stale-preparation",
            "actual_artifact_generation": root.artifact_generation,
        }))
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
