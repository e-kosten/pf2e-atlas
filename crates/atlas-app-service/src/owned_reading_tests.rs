use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

const ACTOR: &str = "actors:testCreature1000";
fn request(generation: &str, selection: OwnedReadingSelection) -> OwnedReadingRequest {
    OwnedReadingRequest {
        parent: RecordNavigationView {
            spell_rank: None,
            record_key: ACTOR.into(),
            owners: vec![],
            field: None,
            passage: None,
            source_fingerprint: None,
        },
        artifact_generation: generation.into(),
        selection,
    }
}

#[test]
fn finite_batches_decode_once_and_direct_late_targets_skip_predecessors() {
    let children: Vec<_> = (0..600).map(|i| json!({"type":"action","name":format!("Ability {i}"),"system":{"description":{"value":format!("<p>Owned {i} @UUID[Compendium.pf2e.actions.Item.testAction100000]</p>")}}})).collect();
    let source =
        json!({"_id":"testCreature1000","type":"npc","name":"Large actor","items":children});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![
                    record("actors", "Actor", source.clone()),
                    record(
                        "actions",
                        "Item",
                        json!({"_id":"testAction100000","type":"action","name":"Target"}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = f.worker.record_detail(ACTOR).unwrap();
    let RecordBodyView::Creature(actor) = &detail.presentation.body else {
        panic!("actor")
    };
    let last = actor.activities.value.as_ref().unwrap()[599]
        .navigation
        .clone();
    let first = f
        .worker
        .owned_reading(request(
            &detail.artifact_generation,
            OwnedReadingSelection::Abilities {
                cursor: None,
                target: None,
            },
        ))
        .unwrap();
    let metrics = f.worker.last_read_metrics();
    assert_eq!(
        (
            metrics.source_body_decodes,
            metrics.prepared_content_batches
        ),
        (1, 1)
    );
    assert!(metrics.sql_statements <= 6, "{metrics:?}");
    assert_eq!(
        (first.total, first.start, first.end, first.entries.len()),
        (600, 0, 32, 32)
    );
    assert!(first.previous.is_none());
    assert_eq!(first.next.as_ref().unwrap().offset, 32);
    for entry in &first.entries {
        assert_eq!(entry.relationships.len(), 1);
        assert_eq!(entry.relationships[0].ordinal, 0);
        assert_eq!(
            entry.relationships[0].source.owners,
            entry.navigation.owners
        );
        assert_eq!(
            entry.relationships[0].source_navigation.owners,
            entry.navigation.owners
        );
    }
    let late = f
        .worker
        .owned_reading(request(
            &detail.artifact_generation,
            OwnedReadingSelection::Abilities {
                cursor: None,
                target: Some(last.clone()),
            },
        ))
        .unwrap();
    let metrics = f.worker.last_read_metrics();
    assert_eq!(
        (
            metrics.source_body_decodes,
            metrics.prepared_content_batches
        ),
        (1, 1)
    );
    assert_eq!((late.start, late.end, late.entries.len()), (576, 600, 24));
    assert_eq!(late.entries.last().unwrap().navigation, last);
    assert_eq!(late.previous.unwrap().offset, 544);
    assert!(late.next.is_none());
    for generation in ["old", ""] {
        let error = f
            .worker
            .owned_reading(request(
                generation,
                OwnedReadingSelection::Abilities {
                    cursor: first.next.clone(),
                    target: None,
                },
            ))
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
                "expected_artifact_generation": generation,
                "actual_artifact_generation": detail.artifact_generation,
            }))
        );
    }
    let mut stale = last;
    stale.source_fingerprint = None;
    assert!(
        f.worker
            .owned_reading(request(
                &detail.artifact_generation,
                OwnedReadingSelection::Activity { navigation: stale }
            ))
            .unwrap_err()
            .to_string()
            .contains("snapshot-local")
    );
    assert!(
        f.worker
            .owned_reading(request(
                &detail.artifact_generation,
                OwnedReadingSelection::Abilities {
                    cursor: Some(OwnedReadingCursor {
                        artifact_generation: detail.artifact_generation.clone(),
                        offset: 600
                    }),
                    target: None
                }
            ))
            .is_err()
    );
}

#[test]
fn heavy_reference_fields_are_explicitly_bounded_without_breaking_other_fields() {
    let heavy = "@UUID[Compendium.pf2e.actions.Item.testAction100000] ".repeat(1025);
    let source = json!({"_id":"testCreature1000","type":"npc","name":"Heavy actor","system":{"details":{"publicNotes":heavy}},"items":[{"_id":"heavy00000000000","type":"action","name":"Heavy","system":{"description":{"value":heavy}}},{"_id":"light00000000000","type":"action","name":"Light","system":{"description":{"value":"<p>@UUID[Compendium.pf2e.actions.Item.testAction100000]</p>"}}}]});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![
                    record("actors", "Actor", source.clone()),
                    record(
                        "actions",
                        "Item",
                        json!({"_id":"testAction100000","type":"action","name":"Target"}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = f.worker.record_detail(ACTOR).unwrap();
    assert!(
        detail
            .presentation
            .content
            .iter()
            .any(|field| matches!(&field.body, PreparedFieldBodyView::Bounded { .. }))
    );
    assert!(detail.relationships_truncated);
    let reading = f
        .worker
        .owned_reading(request(
            &detail.artifact_generation,
            OwnedReadingSelection::Abilities {
                cursor: None,
                target: None,
            },
        ))
        .unwrap();
    assert!(matches!(
        reading.entries[0].content[0].body,
        PreparedFieldBodyView::Bounded { .. }
    ));
    assert!(reading.entries[0].relationships.is_empty());
    assert!(matches!(
        reading.entries[1].content[0].body,
        PreparedFieldBodyView::Html { .. }
    ));
    assert_eq!(reading.entries[1].relationships.len(), 1);
}

#[test]
fn requested_strike_content_has_no_baseline_activity_facts_in_response() {
    let f = fixture_worker();
    let detail = f.worker.record_detail(ACTOR).unwrap();
    let RecordBodyView::Creature(actor) = &detail.presentation.body else {
        panic!("actor")
    };
    let strike = actor
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|a| a.kind == ActorActivityKindView::Strike)
        .unwrap();
    let reading = f
        .worker
        .owned_reading(request(
            &detail.artifact_generation,
            OwnedReadingSelection::Activity {
                navigation: strike.navigation.clone(),
            },
        ))
        .unwrap();
    assert_eq!(reading.entries.len(), 1);
    assert!(reading.previous.is_none() && reading.next.is_none());
    assert!(
        serde_json::to_string(&reading)
            .unwrap()
            .contains("Claw prose")
    );
    let entry = serde_json::to_value(&reading.entries[0]).unwrap();
    assert!(entry.get("attack").is_none() && entry.get("presentation").is_none());
}

#[test]
fn attributed_content_refuses_more_than_256_fields_and_missing_cache_has_no_prose_fallback() {
    use atlas_domain::RecordKey;
    use atlas_record::source_content::SourceContentLocator;
    let (retrieval, _artifact) = fixture_retrieval().unwrap();
    let key = RecordKey::parse(ACTOR).unwrap();
    let locator = SourceContentLocator {
        record: key.clone(),
        owners: vec![],
        field: "/system/details/publicNotes".into(),
    };
    assert!(
        retrieval
            .read_attributed_content(&vec![locator; 257])
            .unwrap_err()
            .to_string()
            .contains("256")
    );
    let detail = retrieval
        .get_record(atlas_search::GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })
        .unwrap()
        .unwrap();
    let selections = detail.source.node_at(&[]).unwrap().content_selections();
    let selected: Vec<_> = selections
        .into_iter()
        .filter(|selection| selection.field == "/system/details/publicNotes")
        .collect();
    assert!(!selected.is_empty());
    let error = crate::prepared_content::prepared_fields(
        &key,
        &[],
        &selected,
        &[],
        &[],
        retrieval.artifact_context().audience,
        retrieval.source_fingerprint(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("no prepared outcome"));
}

#[test]
fn table_reading_batches_complete_text_and_checked_result_links() {
    let rows: Vec<_> = (0..34)
        .map(|i| {
            json!({
                "_id":format!("result{i:010}"), "type":"pack", "range":[i+1,i+1],
                "text":format!("<p>Complete result {i}</p>"),
                "documentCollection":"pf2e.items", "documentId":"item000000000000"
            })
        })
        .collect();
    let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "tables",
                        "RollTable",
                        json!({"_id":"table00000000000","name":"Table","formula":"1d100","results":rows}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"item000000000000","name":"Target","type":"equipment"}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = fixture
        .worker
        .record_detail("tables:table00000000000")
        .unwrap();
    let mut request = OwnedReadingRequest {
        parent: detail.selected,
        artifact_generation: detail.artifact_generation,
        selection: OwnedReadingSelection::TableResults { cursor: None },
    };
    let first = fixture.worker.owned_reading(request.clone()).unwrap();
    assert_eq!(first.entries.len(), 32);
    assert_eq!(first.total, 34);
    assert!(!first.entries[0].content.is_empty());
    assert!(first.entries[0].relationships.iter().any(|edge| {
        edge.kind == "TableResult"
            && edge
                .target
                .as_ref()
                .is_some_and(|target| target.record_key == "items:item000000000000")
    }));
    let metrics = fixture.worker.last_read_metrics();
    assert_eq!(
        (
            metrics.source_body_decodes,
            metrics.prepared_content_batches
        ),
        (1, 1)
    );
    request.selection = OwnedReadingSelection::TableResults { cursor: first.next };
    let next = fixture.worker.owned_reading(request.clone()).unwrap();
    assert_eq!(next.entries.len(), 2);
    assert!(next.next.is_none());
    assert_ne!(first.entries[0].navigation, next.entries[0].navigation);
    request.artifact_generation = "stale".into();
    assert!(fixture.worker.owned_reading(request).is_err());
}
