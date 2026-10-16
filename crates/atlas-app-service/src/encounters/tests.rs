use crate::test_support::encounter_fixture_worker;
use atlas_app_model::*;

#[test]
fn source_free_manual_encounter_never_opens_missing_reference_artifact() {
    let worker = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoEmbeddings,
        ),
        atlas_runtime::AtlasRuntimeOptions {
            default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
            path_mode: atlas_runtime::AtlasPathMode::Global,
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(
                    crate::test_support::fixture_local_state_path()
                        .with_extension("missing-artifact"),
                ),
                ..Default::default()
            },
        },
        crate::test_support::fixture_local_state_path(),
    )
    .unwrap();
    assert!(worker.record_detail("actors:testCreature1000").is_err());
    let encounter = worker
        .create_encounter(CreateEncounterRequest {
            name: "Source free".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            display_name: "Manual PC".into(),
            max_hp: Some(0),
            current_hp: Some(0),
            initiative: None,
        })
        .and_then(|receipt| worker.encounter(&receipt.encounter_key))
        .unwrap();
    assert_eq!(detail.artifact_generation, None);
    assert_eq!(
        detail.participants[0].status,
        EncounterParticipantStatusView::Manual
    );
    assert_eq!(detail.participants[0].current_hp, Some(0));
    let mut invalid = worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap()
        .participants
        .remove(0);
    invalid.participant_kind = atlas_local_state::ParticipantKind::Creature;
    invalid.record_key = Some("invalid source address".into());
    let projected = super::projection::encounter_detail_view(
        &worker,
        worker
            .local_state_store()
            .unwrap()
            .encounters()
            .get_with_participants(&encounter.encounter_key)
            .unwrap()
            .unwrap()
            .encounter,
        vec![invalid],
    )
    .unwrap();
    assert!(projected.artifact_generation.is_none());
    assert_eq!(
        projected.participants[0].status,
        EncounterParticipantStatusView::Unresolved
    );
    assert!(projected.participants[0].presentation.is_none());
}

#[test]
fn quantity_add_returns_receipt_when_any_postcommit_retrieval_would_fail() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(1, 16);
    let f = crate::test_support::fixture_worker_with_executor(executor.clone());
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Atomic quantity".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    // The resolved source is the only permitted retrieval; hydration would fail.
    executor.fail_after_submissions(1);
    let receipt = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.slug,
            record_ref: "Test Creature 1".into(),
            quantity: 3,
            initiative: Some(20),
        })
        .unwrap();
    assert_eq!(receipt.encounter_key, encounter.encounter_key);
    let persisted = f
        .worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&receipt.encounter_key)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.participants.len(), 3);
    assert!(
        persisted
            .participants
            .iter()
            .all(|p| p.current_hp == Some(60))
    );
    assert_eq!(
        f.worker
            .encounter(&receipt.encounter_key)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::QueryFailed
    );
}

#[test]
fn derived_edit_receipt_and_stale_guard_do_not_run_postcommit_hydration() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(1, 16);
    let f = crate::test_support::fixture_worker_with_executor(executor.clone());
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Revision guard".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let initial = f
        .worker
        .encounter(&encounter.encounter_key)
        .unwrap()
        .participants
        .remove(0);
    let mut deliberate = update(&initial);
    deliberate.participant_variant = EncounterParticipantVariantView::Elite;
    deliberate.variant_edit = true;
    deliberate.side = EncounterParticipantSideView::Ally;
    executor.fail_after_submissions(1);
    let receipt = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, deliberate)
        .unwrap();
    assert_eq!(receipt.encounter_key, encounter.encounter_key);
    let before = f
        .worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap();
    assert_eq!(before.participants[0].max_hp, Some(80));
    assert_eq!(
        before.participants[0].hp_origin,
        atlas_local_state::ParticipantHpOrigin::DerivedPristine
    );
    let mut stale = update(&initial);
    stale.use_derived_max = true; // Must conflict before attempting unavailable derivation.
    stale.current_hp = Some(1);
    stale.hp_edit = true;
    stale.max_hp = Some(999);
    let error = f
        .worker
        .update_encounter_participant(&encounter.encounter_key, stale)
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::EncounterParticipantConflict);
    assert_eq!(error.retryable, Some(false));
    assert_eq!(
        error.details.unwrap()["actual_revision"],
        serde_json::json!(before.participants[0].revision)
    );
    assert_eq!(
        f.worker
            .local_state_store()
            .unwrap()
            .encounters()
            .get_with_participants(&encounter.encounter_key)
            .unwrap()
            .unwrap(),
        before
    );
}

#[test]
fn invalid_source_and_failed_derivation_leave_local_state_unchanged() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(1, 16);
    let f = crate::test_support::fixture_worker_with_executor(executor.clone());
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Precommit validation".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let before = f
        .worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap();
    assert!(
        f.worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: encounter.encounter_key.clone(),
                record_ref: "Test Action 1".into(),
                quantity: 2,
                initiative: None,
            })
            .is_err()
    );
    assert_eq!(
        f.worker
            .local_state_store()
            .unwrap()
            .encounters()
            .get_with_participants(&encounter.encounter_key)
            .unwrap()
            .unwrap(),
        before
    );
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let participant = f
        .worker
        .encounter(&encounter.encounter_key)
        .unwrap()
        .participants
        .remove(0);
    let before = f
        .worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&encounter.encounter_key)
        .unwrap()
        .unwrap();
    let mut request = update(&participant);
    request.use_derived_max = true;
    executor.fail_after_submissions(0);
    assert_eq!(
        f.worker
            .update_encounter_participant(&encounter.encounter_key, request)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::QueryFailed
    );
    assert_eq!(
        f.worker
            .local_state_store()
            .unwrap()
            .encounters()
            .get_with_participants(&encounter.encounter_key)
            .unwrap()
            .unwrap(),
        before
    );
}

#[test]
fn reference_independent_mutations_commit_with_unavailable_artifact() {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Unavailable reads".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: Some(20),
        })
        .unwrap();
    let initial = f
        .worker
        .encounter(&encounter.encounter_key)
        .unwrap()
        .participants
        .remove(0);
    let worker = crate::AtlasAppService::new(
        crate::service::RetrievalBackend::command(
            crate::service::CommandRetrievalMode::NoEmbeddings,
        ),
        atlas_runtime::AtlasRuntimeOptions {
            default_search_profile: Some(atlas_domain::SearchProfileId::new("bge-base")),
            path_mode: atlas_runtime::AtlasPathMode::Global,
            overrides: atlas_runtime::AtlasPathOverrides {
                index_path: Some(
                    crate::test_support::fixture_local_state_path()
                        .with_extension("missing-artifact"),
                ),
                ..Default::default()
            },
        },
        f.worker.local_state_path.clone(),
    )
    .unwrap();
    assert!(worker.encounter(&encounter.encounter_key).is_err());
    let key = encounter.encounter_key.clone();
    let assert_receipt = |receipt: EncounterMutationView| assert_eq!(receipt.encounter_key, key);
    assert_receipt(
        worker
            .update_encounter(UpdateEncounterRequest {
                encounter_key: key.clone(),
                slug: encounter.slug,
                name: encounter.name,
                description: Some("Updated".into()),
                note: None,
                status: EncounterStatusView::Running,
            })
            .unwrap(),
    );
    assert_receipt(
        worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: key.clone(),
                display_name: "Manual".into(),
                max_hp: Some(10),
                current_hp: Some(10),
                initiative: Some(15),
            })
            .unwrap(),
    );
    let mut edit = update(&initial);
    edit.current_hp = Some(32);
    edit.hp_edit = true;
    assert_receipt(worker.update_encounter_participant(&key, edit).unwrap());
    assert_receipt(
        worker
            .add_encounter_participant_condition(
                &key,
                AddEncounterParticipantConditionRequest {
                    participant_key: initial.participant_key.clone(),
                    condition_ref: None,
                    name: Some("Custom".into()),
                    value: Some(1),
                    source_participant_key: None,
                    duration_rounds: None,
                    note: None,
                },
            )
            .unwrap(),
    );
    let local = worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&key)
        .unwrap()
        .unwrap();
    let condition_id = local.participants[0].conditions[0].condition_id;
    let manual_key = local.participants[1].participant_key.clone();
    assert_receipt(
        worker
            .update_encounter_participant_condition(
                &key,
                &initial.participant_key,
                UpdateEncounterParticipantConditionRequest {
                    condition_id,
                    condition_ref: None,
                    name: "Changed".into(),
                    value: Some(2),
                    source_participant_key: None,
                    duration_rounds: None,
                    note: None,
                },
            )
            .unwrap(),
    );
    assert_receipt(
        worker
            .remove_encounter_participant_condition(&key, &initial.participant_key, condition_id)
            .unwrap(),
    );
    assert_receipt(
        worker
            .reorder_encounter_participant(
                &key,
                ReorderEncounterParticipantRequest {
                    participant_key: manual_key.clone(),
                    target_participant_key: initial.participant_key.clone(),
                    placement: ReorderEncounterParticipantPlacementView::Before,
                },
            )
            .unwrap(),
    );
    assert_receipt(
        worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: key.clone(),
                participant_key: None,
            })
            .unwrap(),
    );
    assert_receipt(
        worker
            .remove_encounter_participant(&key, &manual_key)
            .unwrap(),
    );
    let persisted = worker
        .local_state_store()
        .unwrap()
        .encounters()
        .get_with_participants(&key)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.participants.len(), 1);
    assert_eq!(persisted.participants[0].current_hp, Some(32));
    assert_eq!(
        persisted.encounter.current_turn_participant_key,
        Some(initial.participant_key)
    );
    assert!(worker.encounter(&key).is_err());
}

#[test]
fn list_import_reuses_precommit_summaries_when_further_retrieval_fails() {
    let executor = crate::executor::RetrievalExecutor::from_fixture_workers(1, 16);
    let f = crate::test_support::fixture_worker_with_executor(executor.clone());
    executor.fail_after_submissions(1);
    let imported = f
        .worker
        .import_saved_list(ImportSavedListRequest {
            document: SavedListExportDocumentView {
                format: "pf2e-atlas.saved-list".into(),
                version: 1,
                exported_at: "now".into(),
                list: SavedListExportListView {
                    id: "imported".into(),
                    name: "Imported".into(),
                    description: None,
                    tags: vec![],
                },
                items: ["actors:testCreature1000", "actors:missing00000000"]
                    .into_iter()
                    .enumerate()
                    .map(|(i, key)| SavedListExportItemView {
                        position: i as i64 + 1,
                        record_key: key.into(),
                        record_name: "Snapshot".into(),
                        kind: Some("creature".into()),
                        status: SavedListItemStatusView::Unresolved,
                        note: None,
                        snapshot: SavedListItemSnapshotView {
                            title: "Snapshot".into(),
                            kind: Some("creature".into()),
                        },
                    })
                    .collect(),
            },
            id: None,
            replace: false,
        })
        .unwrap();
    assert_eq!((imported.active_count, imported.unresolved_count), (1, 1));
    let persisted = f
        .worker
        .local_state_store()
        .unwrap()
        .saved_lists()
        .get_with_items("imported")
        .unwrap()
        .unwrap();
    assert_eq!(persisted.items[0].record_title_snapshot, "Test Creature 1");
    assert_eq!(persisted.items[1].record_title_snapshot, "Snapshot");
    assert!(f.worker.saved_list("imported").is_err());
}

fn hazard(p: &EncounterParticipantView) -> &HazardPresentationView {
    let RecordBodyView::Hazard(h) = &p.presentation.as_ref().unwrap().body else {
        panic!("hazard presentation")
    };
    h
}

pub(super) fn update(p: &EncounterParticipantView) -> UpdateEncounterParticipantRequest {
    UpdateEncounterParticipantRequest {
        expected_revision: p.revision,
        hp_edit: false,
        max_hp_edit: false,
        variant_edit: false,
        use_derived_max: false,
        participant_key: p.participant_key.clone(),
        display_name: p.display_name.clone(),
        side: p.side,
        participant_variant: p.participant_variant,
        initiative: p.initiative,
        max_hp: p.max_hp,
        current_hp: p.current_hp,
        temporary_hp: p.temporary_hp,
        defeated: p.defeated,
        hidden: p.hidden,
        note: p.note.clone(),
    }
}
fn actor(p: &EncounterParticipantView) -> &ActorPresentationView {
    match &p.presentation.as_ref().unwrap().body {
        RecordBodyView::Creature(a) => a,
        RecordBodyView::Hazard(h) => &h.actor,
        _ => panic!("actor presentation"),
    }
}
fn authored_arithmetic_baseline(source: serde_json::Value) -> super::mechanics::StatBlockView {
    let input = atlas_search::test_support::record("actors", "Actor", source);
    let key = input.record.key().clone();
    let (retrieval, _artifact) =
        atlas_search::test_support::open_source_fixture(vec![input], false).unwrap();
    let detail = retrieval
        .get_record(atlas_search::GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })
        .unwrap()
        .unwrap();
    let summary = crate::projection::localized_summary_view(&detail.summary, &retrieval);
    let node = atlas_record::source_record::SourceQueryView::new(
        detail.source.source(),
        key.pack().as_str(),
        "",
    )
    .source;
    let presentation = crate::presentation::record_presentation(
        &detail.source,
        node,
        &summary,
        &[],
        vec![],
        &retrieval,
    );
    super::mechanics::arithmetic_baseline(&presentation).unwrap()
}

#[test]
fn repeated_participants_share_authored_baselines_and_load_source_only_on_request() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Shared authored baseline".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let initial = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 3,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let mut change = update(&initial.participants[0]);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    f.worker
        .add_encounter_participant_condition(
            &e.encounter_key,
            AddEncounterParticipantConditionRequest {
                participant_key: initial.participants[0].participant_key.clone(),
                condition_ref: Some("conditionitems:fesd1n5eVhpCSS18".into()),
                name: None,
                value: Some(1),
                source_participant_key: None,
                duration_rounds: None,
                note: None,
            },
        )
        .unwrap();
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Hazard 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    f.worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            r.reset_read_metrics();
            crate::presentation::actor::authored_projection_count(true);
            super::mechanics::baseline_count(true);
            Ok(())
        })
        .unwrap();
    let detail = f.worker.encounter(&e.encounter_key).unwrap();
    assert!(detail.artifact_generation.is_some());
    let (metrics, authored, baselines) = f
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            Ok((
                r.read_metrics(),
                crate::presentation::actor::authored_projection_count(false),
                super::mechanics::baseline_count(false),
            ))
        })
        .unwrap();
    assert_eq!(metrics.source_body_decodes, 2);
    assert_eq!(metrics.prepared_content_batches, 0);
    assert_eq!((authored, baselines), (2, 2));
    assert!(
        detail
            .participants
            .iter()
            .all(|p| p.presentation.as_ref().unwrap().content.is_empty())
    );
    assert_eq!(
        actor(&detail.participants[0]).armor_class.value,
        Some(23.into())
    );
    assert_eq!(
        actor(&detail.participants[1]).armor_class.value,
        Some(22.into())
    );
    assert_eq!(
        actor(&detail.participants[2]).armor_class.value,
        Some(22.into())
    );
    assert_eq!(
        (detail.participants[0].max_hp, detail.participants[1].max_hp),
        (Some(80), Some(60))
    );
    assert_eq!(
        hazard(&detail.participants[3]).hardness.value,
        Some(8.into())
    );
    f.worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            r.reset_read_metrics();
            Ok(())
        })
        .unwrap();
    let source = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: detail.participants[0].record_key.clone().unwrap(),
            owners: vec![],
            fields: vec!["/system/details/publicNotes".into()],
            passage: None,
            source_fingerprint: None,
        })
        .unwrap();
    let metrics = f
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            Ok(r.read_metrics())
        })
        .unwrap();
    assert_eq!(metrics.source_body_decodes, 1);
    assert_eq!(metrics.prepared_content_batches, 1);
    assert_eq!(source.presentation.content.len(), 1);
    let PreparedFieldBodyView::Html { html, .. } = &source.presentation.content[0].body else {
        panic!("prepared HTML")
    };
    assert!(html.contains("Creature prose"));
}

#[test]
fn strike_status_penalties_preserve_strongest_and_suppressed_explanations_in_both_orders() {
    for conditions in [
        [
            ("conditionitems:TBSHQspnbcqxsmjL", 2),
            ("conditionitems:fesd1n5eVhpCSS18", 1),
        ],
        [
            ("conditionitems:fesd1n5eVhpCSS18", 1),
            ("conditionitems:TBSHQspnbcqxsmjL", 2),
        ],
    ] {
        let f = encounter_fixture_worker();
        let e = f
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Strike explanations".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        let detail = f
            .worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: e.encounter_key.clone(),
                record_ref: "Test Creature 1".into(),
                quantity: 1,
                initiative: None,
            })
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .unwrap();
        for (reference, value) in conditions {
            f.worker
                .add_encounter_participant_condition(
                    &e.encounter_key,
                    AddEncounterParticipantConditionRequest {
                        participant_key: detail.participants[0].participant_key.clone(),
                        condition_ref: Some(reference.into()),
                        name: None,
                        value: Some(value),
                        source_participant_key: None,
                        duration_rounds: None,
                        note: None,
                    },
                )
                .unwrap();
        }
        let detail = f.worker.encounter(&e.encounter_key).unwrap();
        let attack = &actor(&detail.participants[0])
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.title == "Claw")
            .unwrap()
            .attack;
        assert_eq!(attack.value, Some(10.into()));
        let explanation = attack.adjustment.as_ref().unwrap();
        assert_eq!(explanation.authored, serde_json::Number::from(12));
        assert_eq!(
            explanation
                .applied
                .iter()
                .map(|m| m.value)
                .collect::<Vec<_>>(),
            vec![-2]
        );
        assert_eq!(
            explanation
                .suppressed
                .iter()
                .map(|m| m.value)
                .collect::<Vec<_>>(),
            vec![-1]
        );
        assert!(
            explanation
                .applied
                .iter()
                .chain(&explanation.suppressed)
                .all(|m| m.modifier_type == StatModifierTypeView::Status && !m.source.is_empty())
        );
    }
}

#[test]
fn composed_strike_keeps_suppressed_only_explanation_without_inventing_an_applied_effect() {
    let f = encounter_fixture_worker();
    let authored = f
        .worker
        .record_detail("actors:testCreature1000")
        .unwrap()
        .presentation;
    let mut workspace = super::mechanics::arithmetic_baseline(&authored).unwrap();
    workspace
        .activities
        .iter_mut()
        .find(|a| a.label == "Claw")
        .unwrap()
        .rolls[0]
        .suppressed_modifiers
        .push(StatModifierView {
            source: "Frightened".into(),
            label: "Frightened".into(),
            modifier_type: StatModifierTypeView::Status,
            value: -1,
        });
    let composed = crate::presentation::compose_participant(authored, &workspace);
    let RecordBodyView::Creature(actor) = composed.body else {
        panic!("creature")
    };
    let attack = &actor
        .activities
        .value
        .as_ref()
        .unwrap()
        .iter()
        .find(|a| a.title == "Claw")
        .unwrap()
        .attack;
    assert_eq!(attack.value, Some(12.into()));
    let explanation = attack.adjustment.as_ref().unwrap();
    assert!(explanation.applied.is_empty());
    assert_eq!(explanation.suppressed.len(), 1);
    assert_eq!(explanation.suppressed[0].source, "Frightened");
    assert_eq!(explanation.suppressed[0].value, -1);
}
#[test]
fn encounter_projection_keeps_semantic_statistics_and_authored_context() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Semantic actor".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key,
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let a = actor(&d.participants[0]);
    assert_eq!(a.armor_class.value, Some(22.into()));
    assert!(a.saves.fortitude.value.is_some());
    assert!(
        a.abilities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .any(|v| v.key == "str")
    );
    assert!(
        a.skills
            .value
            .as_ref()
            .unwrap()
            .iter()
            .any(|v| v.key == "athletics")
    );
    assert_eq!(a.land_speed.value, Some(25.into()));
    assert!(a.runtime.as_ref().unwrap().action_budget.is_some());
    assert!(
        a.runtime
            .as_ref()
            .unwrap()
            .unapplied_effects
            .iter()
            .any(|n| n.label == "Identifying spells")
    );
    assert!(
        a.activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .any(|v| v.title == "Breath Weapon")
    );
}

#[test]
fn derived_capacity_changes_clamp_valid_damage_at_zero_and_preserve_manual_outliers() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP boundaries".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let mut change = update(&d.participants[0]);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    let mut change = update(&p);
    change.current_hp = Some(3);
    change.hp_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(0)));
    for current in [400, -5] {
        let p = f
            .worker
            .encounter(&e.encounter_key)
            .unwrap()
            .participants
            .remove(0);
        let mut change = update(&p);
        change.current_hp = Some(current);
        change.hp_edit = true;
        let p = {
            let participant_key = change.participant_key.clone();
            f.worker
                .update_encounter_participant(&e.encounter_key, change)
                .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
                .map(|detail| {
                    detail
                        .participants
                        .into_iter()
                        .find(|p| p.participant_key == participant_key)
                        .unwrap()
                })
        }
        .unwrap();
        let mut change = update(&p);
        change.participant_variant = EncounterParticipantVariantView::Elite;
        change.variant_edit = true;
        let p = {
            let participant_key = change.participant_key.clone();
            f.worker
                .update_encounter_participant(&e.encounter_key, change)
                .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
                .map(|detail| {
                    detail
                        .participants
                        .into_iter()
                        .find(|p| p.participant_key == participant_key)
                        .unwrap()
                })
        }
        .unwrap();
        assert_eq!(p.current_hp, Some(current));
    }
}

#[test]
fn known_normal_hp_does_not_require_level_and_runtime_level_uses_foundry_bounds() {
    use atlas_local_state::ParticipantVariant;
    let input = atlas_search::test_support::record(
        "actors",
        "Actor",
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":{"adjustment":null,"hp":{"max":60}}}}),
    );
    assert_eq!(
        super::mechanics::effective_max(&input.record, Some(ParticipantVariant::Normal)),
        Some(60)
    );
    assert_eq!(
        super::mechanics::effective_max(&input.record, Some(ParticipantVariant::Elite)),
        None
    );
    for (level, variant, expected) in [
        (i64::MIN, ParticipantVariant::Normal, -1),
        (-5, ParticipantVariant::Elite, 1),
        (0, ParticipantVariant::Elite, 2),
        (1, ParticipantVariant::Weak, -1),
        (i64::MAX, ParticipantVariant::Normal, 100),
        (200, ParticipantVariant::Elite, 101),
        (200, ParticipantVariant::Weak, 99),
    ] {
        assert_eq!(
            super::mechanics::adjusted_npc_level(level, variant),
            expected
        );
    }
}
#[test]
fn derived_hp_follows_pristine_variant_then_preserves_damage_and_explicit_capacity() {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "HP policy".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let p = &detail.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(60)));
    assert_eq!(p.hp_origin, "derived_pristine");
    assert_eq!(p.variant_origin, "inherited_known");
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(80), Some(80)));
    let block = actor(&p);
    assert_eq!(
        block.armor_class.value.clone().unwrap(),
        serde_json::Number::from(24)
    );
    assert_eq!(
        block
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.title == "Claw")
            .unwrap()
            .damage
            .value
            .as_ref()
            .unwrap()[0]
            .formula
            .value
            .as_deref()
            .unwrap(),
        "1d6+4"
    );
    assert!(
        block
            .runtime
            .as_ref()
            .unwrap()
            .unapplied_effects
            .iter()
            .any(|n| n.label == "Identifying spells")
    );
    let mut change = update(&p);
    change.current_hp = Some(55);
    change.hp_edit = true;
    change.temporary_hp = 9;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(p.hp_origin, "derived_edited");
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Weak;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(
        (p.max_hp, p.current_hp, p.temporary_hp),
        (Some(45), Some(20), 9)
    );
    let mut change = update(&p);
    change.max_hp = Some(200);
    change.max_hp_edit = true;
    change.current_hp = Some(-5);
    change.hp_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(p.hp_origin, "explicit");
    let mut change = update(&p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(200), Some(-5)));
    let read = f.worker.encounter(&encounter.encounter_key).unwrap();
    assert_eq!(
        (read.participants[0].max_hp, read.participants[0].current_hp),
        (Some(200), Some(-5))
    );
}

#[test]
fn same_value_hp_edit_is_preserved_and_manual_hp_is_never_clamped() {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Manual HP".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: encounter.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let mut change = update(&detail.participants[0]);
    change.hp_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(p.hp_origin, "derived_edited");
    let mut change = update(&p);
    change.current_hp = Some(400);
    change.hp_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(p.current_hp, Some(400));
    let detail = f
        .worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: encounter.encounter_key,
            display_name: "PC".into(),
            max_hp: Some(10),
            current_hp: Some(-2),
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "PC")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(10), Some(-2)));
    assert_eq!(p.hp_origin, "explicit");
}

#[test]
fn hp_numeric_policy_does_not_coerce_fractional_or_unsigned_out_of_range_values() {
    for n in [
        serde_json::Number::from(u64::MAX),
        serde_json::Number::from_f64(1.5).unwrap(),
    ] {
        assert_eq!(super::mechanics::integer(&n), None);
    }
    assert_eq!(
        super::mechanics::integer(&serde_json::Number::from(i64::MAX)),
        Some(i64::MAX)
    );
    assert_eq!(
        super::mechanics::integer(&serde_json::Number::from_f64(60.0).unwrap()),
        Some(60)
    );
}

#[test]
fn inherited_elite_applies_once_unknown_adjustment_stays_unknown_and_active_hp_preserves_damage() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"elite00000000000","name":"Elite source","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":"elite","hp":{"max":60},"ac":{"value":22}}}}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"unknown000000000","name":"Unknown adjustment","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":"bad","hp":{"max":60}}}}),
                ),
                record(
                    "actors",
                    "Actor",
                    json!({"_id":"fraction00000000","name":"Fractional HP","type":"npc","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60.5}}}}),
                ),
            ],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Source adjustment".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let add = |name: &str| {
        f.worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                encounter_ref: e.encounter_key.clone(),
                record_ref: name.into(),
                quantity: 1,
                initiative: None,
            })
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .unwrap()
    };
    let detail = add("Elite source");
    let p = &detail.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(80), Some(80)));
    assert_eq!(
        p.participant_variant,
        EncounterParticipantVariantView::Elite
    );
    assert_eq!(
        actor(p).armor_class.value.clone().unwrap(),
        serde_json::Number::from(24)
    );
    f.worker
        .update_encounter(UpdateEncounterRequest {
            encounter_key: e.encounter_key.clone(),
            slug: e.slug.clone(),
            name: e.name.clone(),
            description: None,
            note: None,
            status: EncounterStatusView::Running,
        })
        .unwrap();
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Normal;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(60), Some(60)));
    assert_eq!(p.hp_origin, "derived_edited");
    let detail = add("Unknown adjustment");
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "Unknown adjustment")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
    assert_eq!(p.variant_origin, "inherited_unknown");
    assert!(
        actor(p)
            .runtime
            .as_ref()
            .unwrap()
            .unapplied_effects
            .iter()
            .any(|n| n.source == "npc-adjustment")
    );
    let mut unrelated = update(p);
    unrelated.display_name = "Renamed".into();
    let p = {
        let participant_key = unrelated.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, unrelated)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
    assert_eq!(p.variant_origin, "inherited_unknown");
    let mut explicit = update(&p);
    explicit.variant_edit = true;
    let p = {
        let participant_key = explicit.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, explicit)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!(p.max_hp, Some(60));
    assert_eq!(p.current_hp, None);
    assert_eq!(p.variant_origin, "explicit");
    let detail = add("Fractional HP");
    let p = detail
        .participants
        .iter()
        .find(|p| p.display_name == "Fractional HP")
        .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (None, None));
}

#[test]
fn conditions_stack_known_penalties_and_preserve_unapplied_context() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Conditions".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let detail = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let key = detail.participants[0].participant_key.clone();
    for (reference, value) in [
        ("conditionitems:TBSHQspnbcqxsmjL", 2),
        ("conditionitems:fesd1n5eVhpCSS18", 1),
    ] {
        f.worker
            .add_encounter_participant_condition(
                &e.encounter_key,
                AddEncounterParticipantConditionRequest {
                    participant_key: key.clone(),
                    condition_ref: Some(reference.into()),
                    name: None,
                    value: Some(value),
                    source_participant_key: None,
                    duration_rounds: None,
                    note: None,
                },
            )
            .unwrap();
    }
    let detail = f.worker.encounter(&e.encounter_key).unwrap();
    let block = actor(&detail.participants[0]);
    let ac = &block.armor_class;
    assert_eq!(ac.value.clone().unwrap(), serde_json::Number::from(20));
    assert_eq!(ac.adjustment.as_ref().unwrap().suppressed.len(), 1);
    assert_eq!(
        block
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| a.title == "Claw")
            .unwrap()
            .attack
            .value
            .clone()
            .unwrap(),
        serde_json::Number::from(10)
    );
    assert!(
        block
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .any(|a| a.title == "Breath Weapon"
                && a.attack.value.is_none()
                && a.damage.value.is_none())
    );
}

#[test]
fn movement_restrictions_do_not_compound_across_sources_or_depend_on_order() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::{Number, json};
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture([30i64, i64::MAX,4,0].into_iter().enumerate().map(|(i,speed)| record("actors","Actor",json!({"_id":format!("speed{i:011}"),"type":"npc","name":format!("Speed {speed}"),"system":{"attributes":{"adjustment":null,"speed":{"value":speed,"otherSpeeds":[]}}}}))).collect(),false)?)
        },
    ));
    for baseline in [30i64, i64::MAX, 4, 0] {
        for immobilized_order in [None, Some(0), Some(2)] {
            let encounter = f
                .worker
                .create_encounter(CreateEncounterRequest {
                    name: format!("Movement {baseline} {immobilized_order:?}"),
                    description: None,
                    note: None,
                })
                .unwrap()
                .encounter;
            let detail = f
                .worker
                .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                    encounter_ref: encounter.encounter_key.clone(),
                    record_ref: format!("Speed {baseline}"),
                    quantity: 3,
                    initiative: None,
                })
                .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
                .unwrap();
            let participants = detail.participants;
            let mut conditions = vec![
                "conditionitems:D5mg6Tc7Jzrj6ro7",
                "conditionitems:D5mg6Tc7Jzrj6ro7",
            ];
            if let Some(position) = immobilized_order {
                conditions.insert(position, "conditionitems:eIcWbB5o3pP6OIMe");
            }
            for (i, condition) in conditions.iter().enumerate() {
                f.worker
                    .add_encounter_participant_condition(
                        &encounter.encounter_key,
                        AddEncounterParticipantConditionRequest {
                            participant_key: participants[0].participant_key.clone(),
                            condition_ref: Some((*condition).into()),
                            name: None,
                            value: None,
                            source_participant_key: Some(
                                participants[1 + i % 2].participant_key.clone(),
                            ),
                            duration_rounds: None,
                            note: None,
                        },
                    )
                    .unwrap();
            }
            let detail = f.worker.encounter(&encounter.encounter_key).unwrap();
            let participant = &detail.participants[0];
            let block = actor(participant);
            let speed = &block.land_speed;
            let expected = if immobilized_order.is_some() {
                0
            } else if baseline > 0 {
                (baseline - 10).max(5)
            } else {
                baseline
            };
            assert_eq!(
                speed
                    .adjustment
                    .as_ref()
                    .map(|a| a.authored.clone())
                    .or_else(|| speed.value.clone())
                    .unwrap(),
                Number::from(baseline)
            );
            assert_eq!(
                speed.value.clone().unwrap(),
                Number::from(expected),
                "{baseline} {immobilized_order:?}"
            );
            assert_eq!(speed.adjustment.as_ref().unwrap().notes.len(), 1);
            assert_eq!(
                participant
                    .conditions
                    .iter()
                    .filter(
                        |c| c.condition_key.as_deref() == Some("conditionitems:D5mg6Tc7Jzrj6ro7")
                    )
                    .count(),
                2
            );
        }
    }
}

#[test]
fn artifact_rebuild_does_not_reinitialize_capacity_or_current_hp() {
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;
    let before = encounter_fixture_worker();
    let e = before
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Rebuild".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = before
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| before.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let mut change = update(&d.participants[0]);
    change.current_hp = Some(17);
    change.hp_edit = true;
    change.temporary_hp = 4;
    {
        let participant_key = change.participant_key.clone();
        before
            .worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| before.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![record(
                "actors",
                "Actor",
                json!({"_id":"testCreature1000","type":"npc","name":"Updated creature","system":{"details":{"level":{"value":6}},"attributes":{"adjustment":null,"hp":{"max":120},"ac":{"value":24}}}}),
            )],
            false,
        )?)
    });
    let mut after = fixture_worker_with_executor(executor);
    after.worker.local_state_path = before.worker.local_state_path.clone();
    let d = after.worker.encounter(&e.encounter_key).unwrap();
    let p = &d.participants[0];
    assert_eq!(
        (p.max_hp, p.current_hp, p.temporary_hp),
        (Some(60), Some(17), 4)
    );
    assert_eq!(p.hp_origin, "derived_edited");
    assert_eq!(p.record.as_ref().unwrap().title, "Updated creature");
    assert_eq!(
        actor(p).maximum_hp.value.clone().unwrap(),
        serde_json::Number::from(120)
    );
}

#[test]
fn hazard_uses_authored_hp_hardness_and_stealth_without_npc_adjustments() {
    let f = encounter_fixture_worker();
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Hazard".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key.clone(),
            record_ref: "Test Hazard 1".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let p = &d.participants[0];
    assert_eq!((p.max_hp, p.current_hp), (Some(40), Some(40)));
    let block = actor(p);
    assert_eq!(
        hazard(p).hardness.value.clone().unwrap(),
        serde_json::Number::from(8)
    );
    assert_eq!(
        hazard(p).stealth.value.clone().unwrap(),
        serde_json::Number::from(12)
    );
    assert!(
        !block
            .runtime
            .as_ref()
            .unwrap()
            .unapplied_effects
            .iter()
            .any(|n| n.source == "npc-adjustment")
    );
    let mut change = update(p);
    change.participant_variant = EncounterParticipantVariantView::Elite;
    change.variant_edit = true;
    let p = {
        let participant_key = change.participant_key.clone();
        f.worker
            .update_encounter_participant(&e.encounter_key, change)
            .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
            .map(|detail| {
                detail
                    .participants
                    .into_iter()
                    .find(|p| p.participant_key == participant_key)
                    .unwrap()
            })
    }
    .unwrap();
    assert_eq!((p.max_hp, p.current_hp), (Some(40), Some(40)));
    assert_eq!(
        actor(&p).armor_class.value.clone().unwrap(),
        serde_json::Number::from(18)
    );
}

#[test]
fn authored_lore_variants_and_nondamaging_casting_entries_remain_context() {
    let block = authored_arithmetic_baseline(
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[{"_id":"bbbbbbbbbbbbbbbb","type":"lore","name":"Sailing Lore","system":{"mod":{"value":14},"variants":{"storm":{"label":"Storm navigation","options":"Only while navigating a storm"}}}},{"_id":"cccccccccccccccc","type":"spellcastingEntry","name":"Arcane Spells","system":{"description":{"value":"<p>Cast authored spells</p>"}}}]}),
    );
    let lore = block
        .values
        .iter()
        .find(|v| v.label == "Sailing Lore")
        .unwrap();
    assert_eq!(lore.base_value, serde_json::Number::from(14));
    assert!(
        block
            .unapplied_effects
            .iter()
            .any(|n| n.label == "Storm navigation" && n.reason == "Only while navigating a storm")
    );
    let casting = block
        .activities
        .iter()
        .find(|a| a.label == "Arcane Spells")
        .unwrap();
    assert!(casting.rolls.is_empty() && casting.damage.is_empty());
    assert_eq!(casting.navigation.owners.len(), 1);
}

#[test]
fn nondamaging_owned_effects_keep_checked_navigation() {
    let block = authored_arithmetic_baseline(
        serde_json::json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[{"type":"effect","name":"Aura","system":{"description":{"value":"<p>Authored effect</p>"}}}]}),
    );
    let effect = &block.activities[0];
    assert_eq!(effect.label, "Aura");
    assert!(effect.rolls.is_empty() && effect.damage.is_empty());
    assert_eq!(effect.navigation.source_fingerprint, Some("a".repeat(64)));
}

#[test]
fn omitted_adjustment_under_known_attributes_defaults_local_hp_without_rewriting_source() {
    use atlas_local_state::{ParticipantVariant, ParticipantVariantOrigin};
    use atlas_record::source_record::{SourceFieldView, SourceQueryView};
    for (attributes, origin, max) in [
        (
            serde_json::json!({"hp":{"max":20}}),
            ParticipantVariantOrigin::DefaultUnadjusted,
            Some(20),
        ),
        (
            serde_json::json!({"adjustment":"bad","hp":{"max":20}}),
            ParticipantVariantOrigin::InheritedUnknown,
            None,
        ),
        (
            serde_json::Value::Null,
            ParticipantVariantOrigin::InheritedUnknown,
            None,
        ),
    ] {
        let input = atlas_search::test_support::record(
            "actors",
            "Actor",
            serde_json::json!({"_id":"LHHgGSs0ELCR4CYK","type":"npc","name":"Ghoul","system":{"attributes":attributes}}),
        );
        let (variant, actual_origin) = super::mechanics::initial_variant(&input.record);
        assert_eq!(actual_origin, origin);
        assert_eq!(super::mechanics::effective_max(&input.record, variant), max);
        if origin == ParticipantVariantOrigin::DefaultUnadjusted {
            assert_eq!(variant, Some(ParticipantVariant::Normal));
            assert!(matches!(
                SourceQueryView::new(input.record.source(), "actors", "")
                    .actor()
                    .authored_adjustment(),
                SourceFieldView::Missing
            ));
        }
    }
    let executor = crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(atlas_search::test_support::open_source_fixture(
            vec![atlas_search::test_support::record(
                "actors",
                "Actor",
                serde_json::json!({"_id":"LHHgGSs0ELCR4CYK","type":"npc","name":"Ghoul","system":{"attributes":{"hp":{"max":20}}}}),
            )],
            false,
        )?)
    });
    let f = crate::test_support::fixture_worker_with_executor(executor);
    let e = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Default adjustment".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let d = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: e.encounter_key,
            record_ref: "Ghoul".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    assert_eq!(
        (d.participants[0].max_hp, d.participants[0].current_hp),
        (Some(20), Some(20))
    );
    assert_eq!(d.participants[0].variant_origin, "default_unadjusted");
}

#[test]
#[ignore = "requires the pinned source corpus; run explicitly for cutover evidence"]
fn corpus_npc_adjustment_defaults_preserve_typed_ghoul_hp() {
    use atlas_foundry_model::{SourceContext, admit_document_source};
    use atlas_local_state::ParticipantVariantOrigin;
    use atlas_record::source_record::SourceBackedRecord;
    let current = std::env::current_dir().unwrap();
    let source = current
        .ancestors()
        .find_map(|p| {
            let path = p.join("scratch/source-contracts-full/pf2e/packs");
            path.is_dir().then_some(path)
        })
        .expect("pinned corpus path available in checkout ancestor");
    let mut pending = vec![source.clone()];
    let (mut total, mut defaults, mut inherited, mut unknown) = (0, 0, 0, 0);
    let mut ghoul = None;
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        if raw["type"] != "npc" {
            continue;
        }
        let pack = path
            .strip_prefix(&source)
            .unwrap()
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let admitted = admit_document_source(
            "Actor",
            SourceContext::new(pack, path.to_str().unwrap(), "$"),
            &bytes,
        )
        .unwrap()
        .model
        .unwrap();
        let record = SourceBackedRecord::new(pack, admitted).unwrap();
        let (variant, origin) = super::mechanics::initial_variant(&record);
        let max = super::mechanics::effective_max(&record, variant);
        total += 1;
        match origin {
            ParticipantVariantOrigin::DefaultUnadjusted => {
                defaults += 1;
                assert!(max.is_some(), "{}", record.key());
            }
            ParticipantVariantOrigin::InheritedKnown => inherited += 1,
            ParticipantVariantOrigin::InheritedUnknown => unknown += 1,
            ParticipantVariantOrigin::Explicit => panic!("source initialization is never explicit"),
        }
        if record.key().to_string() == "pathfinder-bestiary:LHHgGSs0ELCR4CYK" {
            ghoul = Some((origin, max));
        }
    }
    assert_eq!((total, defaults, inherited, unknown), (5492, 5237, 252, 3));
    assert_eq!(
        ghoul,
        Some((ParticipantVariantOrigin::DefaultUnadjusted, Some(20)))
    );
    eprintln!(
        "NPC defaults: roots={total} default_unadjusted={defaults} inherited={inherited} unknown={unknown}; actual Ghoul HP=20"
    );
}
