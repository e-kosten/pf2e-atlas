use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

const ACTOR: &str = "actors:testCreature1000";

#[test]
fn physical_details_do_not_require_an_applicable_usage_field() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "items",
                        "Item",
                        json!({"_id":"shield0000000001","type":"shield","name":"Wovenwood Shield (Minor)","system":{"bulk":{"value":1},"price":{"value":{"gp":85}}}}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"shield0000000002","type":"shield","name":"Wovenwood Shield (Moderate)","system":{"bulk":{"value":1},"price":{"value":{"gp":850}}}}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"shield0000000003","type":"shield","name":"Unavailable Shield"}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"armor00000000001","type":"armor","name":"Armor without usage","system":{"bulk":{"value":2},"price":{"value":{"gp":20}}}}),
                    ),
                    record(
                        "items",
                        "Item",
                        json!({"_id":"bg00000000000000","type":"background","name":"Reference Background","system":{"description":{"value":"<p>Reference prose</p>"}}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    for (key, price, bulk) in [
        ("items:shield0000000001", "85 gp", 1),
        ("items:shield0000000002", "850 gp", 1),
        ("items:armor00000000001", "20 gp", 2),
    ] {
        let detail = f.worker.record_detail(key).unwrap();
        let RecordBodyView::PhysicalReference(physical) = detail.presentation.body else {
            panic!("typed physical price applicability must select physical presentation");
        };
        assert_eq!(
            physical.usage.state,
            atlas_domain::QueryFieldState::NotApplicable
        );
        assert!(physical.usage.value.is_none());
        assert_eq!(physical.price.value.as_deref(), Some(price));
        assert_eq!(physical.bulk.value, Some(bulk.into()));
    }
    let unavailable = f.worker.record_detail("items:shield0000000003").unwrap();
    let RecordBodyView::PhysicalReference(physical) = unavailable.presentation.body else {
        panic!("missing system is an availability state, not a different family");
    };
    assert_eq!(physical.price.state, atlas_domain::QueryFieldState::Missing);
    assert_eq!(physical.bulk.state, atlas_domain::QueryFieldState::Missing);
    let reference = f.worker.record_detail("items:bg00000000000000").unwrap();
    assert_eq!(reference.presentation.body, RecordBodyView::Content);
    assert!(!reference.presentation.content.is_empty());
}

#[test]
fn spell_physical_and_roll_table_details_display_authored_facts() {
    let spell = json!({"_id":"spell00000000000","type":"spell","name":"Authored Spell","system":{"level":{"value":2},"time":{"value":"2"},"range":{"value":"60 feet"},"target":{"value":"one creature"},"duration":{"value":"1 minute","sustained":true},"description":{"value":"<p>Spell text</p>"}}});
    let physical = json!({"_id":"item000000000000","type":"equipment","name":"Authored Equipment","system":{"usage":{"value":"held-in-one-hand"},"bulk":{"value":1},"price":{"value":{"gp":12,"sp":5},"per":2},"description":{"value":"<p>Equipment text</p>"}}});
    let table = json!({"_id":"table00000000000","name":"Authored Table","formula":"1d4","results":[{"_id":"result0000000000","type":"pack","range":[1,2],"weight":2,"documentCollection":"pf2e.items","documentId":"item000000000000","text":"Equipment result"},{"_id":"result0000000001","type":"pack","range":[3,4],"documentCollection":"pf2e.macros","documentId":"macro00000000000","text":"Tool macro"}]});
    let actor = json!({"_id":"actor00000000000","type":"npc","name":"Caster","items":[spell.clone(),physical.clone()]});
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        Ok(open_source_fixture(
            vec![
                record("spells", "Item", spell.clone()),
                record("items", "Item", physical.clone()),
                record("tables", "RollTable", table.clone()),
                record("actors", "Actor", actor.clone()),
                record(
                    "macros",
                    "Macro",
                    json!({"_id":"macro00000000000","type":"script","name":"Tool macro","command":"return;"}),
                ),
            ],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let spell = f.worker.record_detail("spells:spell00000000000").unwrap();
    let RecordBodyView::Spell(spell) = &spell.presentation.body else {
        panic!("spell family");
    };
    assert_eq!(spell.cast.value.as_deref(), Some("2 actions"));
    assert_eq!(spell.range.value.as_deref(), Some("60 feet"));
    assert_eq!(spell.target.value.as_deref(), Some("one creature"));
    assert_eq!(spell.duration.value.as_deref(), Some("1 minute"));
    assert_eq!(spell.sustained.value, Some(true));
    let equipment = f.worker.record_detail("items:item000000000000").unwrap();
    let RecordBodyView::PhysicalReference(equipment) = &equipment.presentation.body else {
        panic!("physical family");
    };
    assert_eq!(equipment.price.value.as_deref(), Some("12 gp, 5 sp"));
    assert_eq!(equipment.bulk.value, Some(1.into()));
    let actor = f.worker.record_detail("actors:actor00000000000").unwrap();
    for activity in actor_body(&actor.presentation)
        .activities
        .value
        .as_ref()
        .unwrap()
    {
        assert!(activity.usage.is_some());
        let detail = f
            .worker
            .record_detail_at(RecordDetailRequest {
                record_key: activity.navigation.record_key.clone(),
                owners: activity.navigation.owners.clone(),
                fields: vec![],
                passage: None,
                source_fingerprint: activity.navigation.source_fingerprint.clone(),
            })
            .unwrap();
        assert_eq!(detail.presentation.identity.title, activity.title);
        match &detail.presentation.body {
            RecordBodyView::Spell(s) => assert_eq!(s.range.value.as_deref(), Some("60 feet")),
            RecordBodyView::PhysicalReference(p) => {
                assert_eq!(p.price.value.as_deref(), Some("12 gp, 5 sp"))
            }
            _ => panic!("selected child family"),
        }
    }
    let table = f.worker.record_detail("tables:table00000000000").unwrap();
    let RecordBodyView::RollTable(table_body) = &table.presentation.body else {
        panic!("table family");
    };
    assert_eq!(table_body.formula.value.as_deref(), Some("1d4"));
    let result = &table.presentation.owned[0];
    let detail = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: result.navigation.record_key.clone(),
            owners: result.navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: result.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    let RecordBodyView::TableResult(result) = &detail.presentation.body else {
        panic!("result family");
    };
    assert_eq!(result.range.value.as_deref(), Some("1–2"));
    assert_eq!(result.weight.value, Some(2.into()));
    assert!(detail.relationships.iter().any(|r| {
        r.kind == "TableResult"
            && r.target
                .as_ref()
                .is_some_and(|t| t.record_key == "items:item000000000000")
    }));
    let result = &table.presentation.owned[1];
    let detail = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: result.navigation.record_key.clone(),
            owners: result.navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: result.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    assert!(detail.relationships.is_empty());
    assert!(
        !serde_json::to_string(&detail)
            .unwrap()
            .contains("macro00000000000")
    );
}
fn actor_body(p: &RecordPresentationView) -> &ActorPresentationView {
    match &p.body {
        RecordBodyView::Creature(a) => a,
        RecordBodyView::Hazard(h) => &h.actor,
        _ => panic!("actor presentation expected"),
    }
}

#[test]
fn default_detail_batches_only_root_content_and_owned_navigation_loads_exact_field() {
    let f = fixture_worker();
    f.worker
        .submit_retrieval(|r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let detail = f.worker.record_detail(ACTOR).unwrap();
    let metrics = f.worker.submit_retrieval(|r| Ok(r.read_metrics())).unwrap();
    assert_eq!(metrics.source_body_decodes, 1);
    assert_eq!(metrics.prepared_content_batches, 1);
    let activity = actor_body(&detail.presentation)
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|a| a.title == "Breath Weapon")
        .unwrap();
    let owned = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: ACTOR.into(),
            owners: activity.navigation.owners.clone(),
            fields: vec!["/system/description/value".into()],
            passage: None,
            source_fingerprint: activity.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    assert_eq!(owned.presentation.identity.title, "Breath Weapon");
    let html = owned
        .presentation
        .content
        .iter()
        .find_map(|f| match &f.body {
            PreparedFieldBodyView::Html { html, .. } => Some(html),
            _ => None,
        })
        .unwrap();
    assert!(html.contains("Ghoul Fever"));
    assert!(!html.contains("Creature prose"));
    let section = atlas_record::source_record::select_prepared_html_sections(html)
        .unwrap()
        .remove(0);
    let address = atlas_domain::SourcePassageAddress::HtmlSection {
        prepared_html_sha256: atlas_search::test_support::hash(html.as_bytes()),
        canonical_text_sha256: atlas_search::test_support::hash(section.text.as_bytes()),
        selection_version: atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION.into(),
        section_ordinal: section.section_ordinal,
        label: section.label,
        chunk_bytes: atlas_domain::SourceByteRange {
            start: 0,
            end: section.text.len(),
        },
    };
    let mut request = RecordDetailRequest {
        record_key: ACTOR.into(),
        owners: activity.navigation.owners.clone(),
        fields: vec!["/system/description/value".into()],
        passage: Some(address),
        source_fingerprint: activity.navigation.source_fingerprint.clone(),
    };
    assert!(f.worker.record_detail_at(request.clone()).is_ok());
    if let Some(atlas_domain::SourcePassageAddress::HtmlSection {
        canonical_text_sha256,
        ..
    }) = &mut request.passage
    {
        *canonical_text_sha256 = "stale".into();
    }
    assert!(
        f.worker
            .record_detail_at(request)
            .unwrap_err()
            .to_string()
            .contains("stale or invalid passage")
    );
    assert!(
        f.worker
            .record_detail_at(RecordDetailRequest {
                record_key: ACTOR.into(),
                owners: activity.navigation.owners.clone(),
                fields: vec!["/system/details/publicNotes".into()],
                passage: None,
                source_fingerprint: None
            })
            .is_err()
    );
    let json = serde_json::to_value(&detail).unwrap().to_string();
    for developer_field in [
        "source_path",
        "content_hash",
        "admission_diagnostics",
        "raw_json",
    ] {
        assert!(!json.contains(developer_field));
    }
}

#[test]
fn large_owned_collection_does_not_eagerly_decode_child_html_and_unstable_navigation_is_bound() {
    let children = (0..600).map(|i|json!({"type":"action","name":format!("Ability {i}"),"system":{"description":{"value":format!("<p>Owned explanation {i}</p>")}}})).collect::<Vec<_>>();
    let source = json!({"_id":"testCreature1000","type":"npc","name":"Large creature","system":{"details":{"publicNotes":"<p>Only root prose</p>"}},"items":children});
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        Ok(open_source_fixture(
            vec![record("actors", "Actor", source.clone())],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    f.worker
        .submit_retrieval(|r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let detail = f.worker.record_detail(ACTOR).unwrap();
    let metrics = f.worker.submit_retrieval(|r| Ok(r.read_metrics())).unwrap();
    assert_eq!(metrics.source_body_decodes, 1);
    assert_eq!(metrics.prepared_content_batches, 1);
    let activities = actor_body(&detail.presentation)
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(activities.len(), 600);
    let navigation = &activities[599].navigation;
    assert!(navigation.source_fingerprint.is_some());
    for fingerprint in [None, Some("different-build".into())] {
        assert!(
            f.worker
                .record_detail_at(RecordDetailRequest {
                    record_key: ACTOR.into(),
                    owners: navigation.owners.clone(),
                    fields: vec![],
                    passage: None,
                    source_fingerprint: fingerprint
                })
                .unwrap_err()
                .to_string()
                .contains("different artifact")
        );
    }
    let selected = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: ACTOR.into(),
            owners: navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: navigation.source_fingerprint.clone(),
        })
        .unwrap();
    assert_eq!(selected.presentation.identity.title, "Ability 599");
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Large owned encounter".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key,
            record_ref: ACTOR.into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let activities = actor_body(d.participants[0].presentation.as_ref().unwrap())
        .activities
        .value
        .as_ref()
        .unwrap();
    assert_eq!(activities.len(), 600);
    assert!(
        activities
            .iter()
            .all(|a| a.navigation.source_fingerprint.is_some())
    );
    let nav = &activities[599].navigation;
    assert!(
        f.worker
            .record_detail_at(RecordDetailRequest {
                record_key: nav.record_key.clone(),
                owners: nav.owners.clone(),
                fields: vec![],
                passage: None,
                source_fingerprint: nav.source_fingerprint.clone()
            })
            .is_ok()
    );
}

#[test]
fn list_results_are_body_free_and_only_product_summaries_are_serialized() {
    let f = fixture_worker();
    f.worker
        .submit_retrieval(|r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let result = f
        .worker
        .list_records(
            None,
            SearchPageRequest {
                number: 1,
                size: 20,
            },
        )
        .unwrap();
    assert_eq!(result.records.len(), 4);
    assert_eq!(
        f.worker
            .submit_retrieval(|r| Ok(r.read_metrics()))
            .unwrap()
            .source_body_decodes,
        0
    );
    let json = serde_json::to_value(result).unwrap().to_string();
    assert!(!json.contains("source_path"));
    assert!(!json.contains("content_hash"));
}

#[test]
fn saved_lists_over_1024_batch_summaries_preserve_missing_snapshots_and_export_order() {
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        let records = (0..1030)
            .map(|i| {
                record(
                    "actions",
                    "Item",
                    json!({"_id":format!("{i:016}"),"type":"action","name":format!("Action {i}")}),
                )
            })
            .collect::<Vec<_>>();
        Ok(open_source_fixture(records, false)?)
    });
    let f = fixture_worker_with_executor(executor);
    let items = (0..1031)
        .map(|i| atlas_local_state::ImportSavedListItem {
            record_key: format!("actions:{i:016}"),
            note: Some(format!("Note {i}")),
            record_title_snapshot: format!("Saved action {i}"),
            record_kind_snapshot: Some("action".into()),
        })
        .collect();
    f.worker
        .local_state_store()
        .unwrap()
        .saved_lists()
        .import(atlas_local_state::ImportSavedList {
            slug: "large-list".into(),
            name: "Large list".into(),
            description: None,
            tags: vec!["reference".into()],
            items,
            replace: false,
        })
        .unwrap();
    f.worker
        .submit_retrieval(|r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let list = f.worker.saved_list("large-list").unwrap();
    let metrics = f.worker.submit_retrieval(|r| Ok(r.read_metrics())).unwrap();
    assert_eq!(list.items.len(), 1031);
    assert_eq!(metrics.source_body_decodes, 0);
    assert_eq!(metrics.summary_batches, 2);
    assert_eq!(list.items[1030].status, SavedListItemStatusView::Unresolved);
    assert_eq!(list.items[1030].snapshot.title, "Saved action 1030");
    let filtered = f
        .worker
        .filter_saved_list(FilterSavedListRequest {
            list_ref: "large-list".into(),
            query: Some("\"Action 1029\"".into()),
            filter: None,
        })
        .unwrap();
    assert_eq!(filtered.items.len(), 1);
    assert_eq!(filtered.items[0].record_key, "actions:0000000000001029");
    let export = f.worker.export_saved_list("large-list").unwrap();
    assert_eq!(export.items[1029].position, 1030);
    assert_eq!(export.items[1030].note.as_deref(), Some("Note 1030"));
    let imported = f
        .worker
        .import_saved_list(ImportSavedListRequest {
            document: export,
            id: Some("copied-list".into()),
            replace: false,
        })
        .unwrap();
    assert_eq!(
        (imported.active_count, imported.unresolved_count),
        (1030, 1)
    );
    assert!(f.worker.delete_saved_list("copied-list").unwrap().deleted);
}

#[test]
fn editor_uses_shared_catalog_validation_and_rejects_unsupported_facet_removal() {
    let f = fixture_worker();
    let predicate:atlas_domain::QueryPredicate=serde_json::from_value(json!({"clause_id":"hp-choice","kind":"compare","field":"actor.hp.maximum","op":"gte","value":50})).unwrap();
    assert!(
        f.worker
            .validate_filter(predicate.clone())
            .unwrap()
            .errors
            .is_empty()
    );
    let editor = f
        .worker
        .discover_filter_editor(DiscoverFilterEditorRequest {
            context: FilterDiscoveryContext::Filtered {
                filter: Some(predicate.clone()),
                text: None,
                mode: RetrievalModeView::Lexical,
            },
            selected_field_ids: vec!["actor.hp.maximum".into()],
        })
        .unwrap();
    let hp = editor
        .groups
        .iter()
        .flat_map(|g| &g.fields)
        .find(|f| f.definition.id == "actor.hp.maximum")
        .unwrap();
    assert_eq!(hp.control, FilterControlView::Numeric);
    let invalid=serde_json::from_value(json!({"clause_id":"invalid-choice","kind":"compare","field":"invented.field","op":"gte","value":50})).unwrap();
    let validation = f.worker.validate_filter(invalid).unwrap();
    assert_eq!(validation.errors.len(), 1);
    assert_eq!(
        validation.errors[0].field.as_deref(),
        Some("invented.field")
    );
    let disjunction:atlas_domain::QueryPredicate=serde_json::from_value(json!({"kind":"any_of","children":[predicate, {"clause_id":"level-choice","kind":"compare","field":"actor.level","op":"gte","value":2}]})).unwrap();
    assert!(
        f.worker
            .validate_filter(disjunction.clone())
            .unwrap()
            .errors
            .is_empty()
    );
    assert!(
        f.worker
            .discover_raw_filter_counts(crate::RawFilterCountsRequest {
                field: "actor.hp.maximum".into(),
                filter: Some(disjunction),
                clause_id: Some("hp-choice".into())
            })
            .is_err()
    );
}

#[test]
fn explicitly_selected_empty_and_null_prose_keep_distinct_availability() {
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![record(
                "actions",
                "Item",
                json!({"_id":"empty00000000000","type":"action","name":"Empty prose","system":{"description":{"value":"","gm":null}}}),
            )],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let d = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: "actions:empty00000000000".into(),
            owners: vec![],
            fields: vec![
                "/system/description/value".into(),
                "/system/description/gm".into(),
            ],
            passage: None,
            source_fingerprint: None,
        })
        .unwrap();
    let fields = d.presentation.content.iter().collect::<Vec<_>>();
    assert!(
        matches!(&fields.iter().find(|f|f.locator.field.ends_with("/value")).unwrap().body,PreparedFieldBodyView::Html{html,controls}if html.is_empty()&&controls.is_empty())
    );
    assert!(matches!(
        &fields
            .iter()
            .find(|f| f.locator.field.ends_with("/gm"))
            .unwrap()
            .body,
        PreparedFieldBodyView::Unavailable {
            state: atlas_domain::QueryFieldState::Null
        }
    ));
}

#[test]
fn rebuilt_snapshot_rejects_navigation_to_a_reordered_unstable_child() {
    use atlas_search::test_support::{input, open_input_fixture};
    let make = |reverse: bool, fingerprint: char| {
        RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
            let mut children = vec![
                json!({"type":"action","name":"First","system":{"description":{"value":"<p>First prose</p>"}}}),
                json!({"type":"action","name":"Second","system":{"description":{"value":"<p>Second prose</p>"}}}),
            ];
            if reverse {
                children.reverse();
            }
            let mut data = input(
                vec![record(
                    "actors",
                    "Actor",
                    json!({"_id":"testCreature1000","type":"npc","name":"Reordered actor","items":children}),
                )],
                false,
            );
            data.context.source_fingerprint = fingerprint.to_string().repeat(64);
            let hash = data.context.sha256()?;
            for record in &mut data.records {
                record.preparation_context_hash = hash.clone();
            }
            Ok(open_input_fixture(data)?)
        })
    };
    let before = fixture_worker_with_executor(make(false, 'a'));
    let detail = before.worker.record_detail(ACTOR).unwrap();
    let nav = actor_body(&detail.presentation)
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|a| a.title == "First")
        .unwrap()
        .navigation
        .clone();
    let after = fixture_worker_with_executor(make(true, 'b'));
    let request = RecordDetailRequest {
        record_key: nav.record_key,
        owners: nav.owners,
        fields: vec![],
        passage: None,
        source_fingerprint: nav.source_fingerprint,
    };
    assert!(
        after
            .worker
            .record_detail_at(request)
            .unwrap_err()
            .to_string()
            .contains("different artifact")
    );
    assert!(after.worker.record_detail(ACTOR).is_ok());
}
