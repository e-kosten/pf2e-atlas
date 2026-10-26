use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

#[test]
fn declared_grants_appear_in_explore_and_reference_search_without_duplicate_results() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "backgrounds",
                        "Item",
                        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Eagle Hunter","type":"background","system":{
                            "description":{"value":"<p>Gain @UUID[Compendium.pf2e.feats-srd.Item.bbbbbbbbbbbbbbbb] for eagles only.</p>"},
                            "rules":[{"key":"GrantItem","uuid":"Compendium.pf2e.feats-srd.Item.bbbbbbbbbbbbbbbb"}],
                            "items":{"train":{"name":"Train Animal","level":1,"uuid":"Compendium.pf2e.feats-srd.Item.cccccccccccccccc"}}
                        }}),
                    ),
                    record(
                        "feats-srd",
                        "Item",
                        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Pet","type":"feat"}),
                    ),
                    record(
                        "feats-srd",
                        "Item",
                        json!({"_id":"cccccccccccccccc","name":"Train Animal","type":"feat"}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = f
        .worker
        .record_detail("backgrounds:aaaaaaaaaaaaaaaa")
        .unwrap();
    let grants = detail.presentation.grants.as_ref().unwrap();
    assert_eq!(grants.entries.len(), 1, "Pet is already linked in prose");
    assert_eq!(grants.entries[0].title, "Train Animal");
    assert_eq!(
        grants.entries[0].navigation.record_key,
        "feats-srd:cccccccccccccccc"
    );
    assert_eq!(grants.unavailable_count, 0);
    assert!(!grants.truncated);
    let related = f
        .worker
        .related_records(RelatedRecordsRequest {
            selection: detail.selected,
            artifact_generation: detail.artifact_generation.clone(),
        })
        .unwrap();
    let RelatedCategoryView::Content { content } = related.references else {
        panic!("declared references");
    };
    assert_eq!(content.outgoing.occurrences.len(), 3);
    assert_eq!(
        content
            .outgoing
            .occurrences
            .iter()
            .filter(|o| o.kind == "DeclaredGrant")
            .count(),
        2
    );
    let page = f
        .worker
        .search_page(
            SearchResultsRequest {
                sort: None,
                ranking: None,
                search_profile: None,
                criteria: SearchCriteria::ListRecords {
                    filter: content.outgoing.filter,
                },
                page: SearchPageRequest { number: 1, size: 5 },
                expected_artifact_generation: Some(detail.artifact_generation),
            },
            &crate::ReadExecutionContext::uncancellable(),
        )
        .unwrap();
    assert_eq!(page.page.total, 2);
    let mut titles = page
        .rows
        .iter()
        .map(|r| r.record.title.as_str())
        .collect::<Vec<_>>();
    titles.sort_unstable();
    assert_eq!(titles, ["Pet", "Train Animal"]);
    let target = f
        .worker
        .record_detail("feats-srd:bbbbbbbbbbbbbbbb")
        .unwrap();
    let incoming = f
        .worker
        .related_records(RelatedRecordsRequest {
            selection: target.selected,
            artifact_generation: target.artifact_generation,
        })
        .unwrap();
    let RelatedCategoryView::Content { content } = incoming.references else {
        panic!("incoming declared references");
    };
    assert_eq!(content.backlinks.occurrences.len(), 2);
    assert!(
        content
            .backlinks
            .occurrences
            .iter()
            .all(|o| o.source_navigation.record_key == "backgrounds:aaaaaaaaaaaaaaaa")
    );
}

#[test]
fn recipient_grants_are_uncapped_but_do_not_appear_on_actors_or_partial_readings() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            let rules = (0..7).map(|n| json!({"key":"GrantItem","uuid":format!("Compendium.pf2e.feats-srd.Item.granted{n:09}")})).collect::<Vec<_>>();
            let ability = json!({"_id":"ability000000000","name":"Granting ability","type":"action","system":{
                "description":{"value":"<p>Grants benefits to the recipient.</p>"},"rules":rules
            }});
            let mut records = vec![
                record("actions", "Item", ability.clone()),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"actor00000000000","name":"Internal grants","type":"npc","items":[ability]}),
                ),
            ];
            // The seventh exact reference is unavailable; the first six all remain visible.
            records.extend((0..6).map(|n| record("feats-srd", "Item", json!({"_id":format!("granted{n:09}"),"name":format!("Benefit {n}"),"type":"feat"}))));
            Ok(open_source_fixture(records, false)?)
        },
    ));
    let detail = f.worker.record_detail("actions:ability000000000").unwrap();
    let grants = detail.presentation.grants.unwrap();
    assert_eq!(grants.entries.len(), 6);
    assert_eq!(grants.unavailable_count, 1);
    assert!(!grants.truncated);
    assert!(grants.entries.iter().all(|entry| entry.record.is_some()));
    assert!(
        f.worker
            .record_detail("actors:actor00000000000")
            .unwrap()
            .presentation
            .grants
            .is_none()
    );
    let mut request = RecordDetailRequest {
        record_key: "actors:actor00000000000".into(),
        owners: vec![atlas_domain::OwnedContentLocator {
            collection: "/items".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("ability000000000".into()),
        }],
        fields: vec![],
        spell_rank: None,
        passage: None,
        source_fingerprint: None,
    };
    assert!(
        f.worker
            .record_detail_at(request.clone())
            .unwrap()
            .presentation
            .grants
            .is_none()
    );
    request.record_key = "actions:ability000000000".into();
    request.owners.clear();
    request.fields = vec!["/system/description/value".into()];
    assert!(
        f.worker
            .record_detail_at(request)
            .unwrap()
            .presentation
            .grants
            .is_none()
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
