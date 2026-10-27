use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_app_model::*;
use atlas_search::test_support::{input, open_input_fixture, open_source_fixture, record};
use serde_json::json;

#[test]
fn selected_reference_preserves_local_documents_and_composes_current_participant_facts() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Local caster","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60}}},"items":[
        {"_id":"eeeeeeeeeeeeeeee","name":"Staff casting","type":"spellcastingEntry","system":{"description":{"value":""},"spelldc":{"dc":33,"value":23},"prepared":{"value":"spontaneous"},"tradition":{"value":"arcane"}}},
        {"_id":"ssssssssssssssss","name":"Local Fireball","type":"spell","system":{"description":{"value":"<p>Customized spell prose.</p>"},"level":{"value":3},"location":{"value":"eeeeeeeeeeeeeeee","heightenedLevel":5},"range":{"value":"500 feet"},"traits":{"value":[]},"damage":{"local":{"formula":"10d6","type":"fire"}}}},
        {"_id":"wwwwwwwwwwwwwwww","name":"Customized staff","type":"weapon","_stats":{"compendiumSource":"Compendium.pf2e.equipment.Item.oooooooooooooooo"},"system":{"description":{"value":"<p>Customized equipment prose.</p>"},"usage":{"value":"held-in-one-hand"},"price":{"value":{"gp":900}},"bulk":{"value":1},"runes":{"potency":1,"striking":1,"property":[]}}},
        {"_id":"cccccccccccccccc","name":"Local claw","type":"melee","system":{"bonus":{"value":12},"damageRolls":{"main":{"damage":"1d6+4","damageType":"slashing"}},"description":{"value":"<p>Local claw prose.</p>"}}}
    ]});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![
                    record("actors", "Actor", source.clone()),
                    record(
                        "equipment",
                        "Item",
                        json!({"_id":"oooooooooooooooo","name":"Catalog staff","type":"weapon","system":{"description":{"value":"<p>Catalog prose.</p>"},"price":{"value":{"gp":800}}}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Selected references".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let current = f
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            participant_variant: None,
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| f.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let generation = current.artifact_generation.unwrap();
    let participant = &current.participants[0];
    let request = |id: &str| EncounterReferenceRequest {
        condition_id: None,
        artifact_generation: generation.clone(),
        selection: RecordDetailRequest {
            spell_rank: None,
            record_key: "actors:aaaaaaaaaaaaaaaa".into(),
            owners: vec![atlas_domain::OwnedContentLocator {
                collection: "/items".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable(id.into()),
            }],
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        },
    };
    let selected_request = request("wwwwwwwwwwwwwwww");
    let selected_encounter = encounter.slug.clone();
    let selected_participant = participant.participant_key.clone();
    let (physical, metrics) = f
        .worker
        .with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| {
                workspace.retrieval().reset_read_metrics();
                let physical = super::reference::read_encounter_reference(
                    workspace,
                    &selected_encounter,
                    &selected_participant,
                    selected_request,
                )?;
                Ok((physical, workspace.retrieval().read_metrics()))
            },
        )
        .unwrap();
    assert_eq!(
        (
            metrics.source_body_decodes,
            metrics.prepared_content_batches
        ),
        (1, 1)
    );
    assert_eq!(physical.presentation.identity.title, "Customized staff");
    assert!(matches!(
        physical.presentation.body,
        RecordBodyView::PhysicalReference(_)
    ));
    let serialized = serde_json::to_string(&physical).unwrap();
    assert!(serialized.contains("Customized equipment prose"));
    assert!(!serialized.contains("Catalog prose"));
    let origin = physical
        .relationships
        .iter()
        .find(|r| r.kind == "CompendiumProvenance")
        .unwrap();
    assert_eq!(
        origin.target.as_ref().unwrap().record_key,
        "equipment:oooooooooooooooo"
    );
    assert!(origin.target.as_ref().unwrap().owners.is_empty());
    let mut update = super::tests::update(participant);
    update.participant_variant = EncounterParticipantVariantView::Elite;
    update.variant_edit = true;
    {
        let participant_key = update.participant_key.clone();
        f.worker
            .update_encounter_participant(&encounter.slug, update)
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
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                request_id: format!(
                    "fixture-reference_tests-130-{}",
                    f.worker.encounter(&encounter.slug).unwrap().revision
                ),
                event_intent: Default::default(),
                decisions: Default::default(),
                expected_encounter_revision: f.worker.encounter(&encounter.slug).unwrap().revision,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                participant_key: participant.participant_key.clone(),
                condition_ref: Some("conditionitems:TBSHQspnbcqxsmjL".into()),
                name: None,
                value: Some(1),
                source_participant_key: None,
                duration: None,
                note: None,
            },
        )
        .unwrap();
    let updated = f.worker.encounter(&encounter.slug).unwrap();
    let RecordBodyView::Creature(actor) =
        &updated.participants[0].presentation.as_ref().unwrap().body
    else {
        panic!("actor")
    };
    let activities = actor.activities.value.as_ref().unwrap();
    for (id, index) in [
        ("eeeeeeeeeeeeeeee", 0),
        ("ssssssssssssssss", 1),
        ("cccccccccccccccc", 3),
    ] {
        let selected = f
            .worker
            .encounter_reference(&encounter.slug, &participant.participant_key, request(id))
            .unwrap();
        assert_eq!(selected.selected.owners, request(id).selection.owners);
        match selected.presentation.body {
            RecordBodyView::Activity(entry) => {
                assert_eq!(entry.attack, activities[index].attack);
                if entry.kind == ActorActivityKindView::Strike {
                    assert_eq!(entry.attack.value, Some(13.into()));
                    assert!(entry.attack.adjustment.is_some());
                } else {
                    let spells = entry.casting_context.unwrap().spells.value.unwrap();
                    assert_eq!(spells.len(), 1);
                    assert_eq!(spells[0], activities[1]);
                }
            }
            RecordBodyView::Spell(spell) => {
                assert_eq!(
                    spell.damage.value.unwrap()[0].formula.value.as_deref(),
                    Some("(10d6)+4")
                );
                let notes = spell.runtime_notes.unwrap();
                assert_eq!(notes, activities[index].notes);
            }
            _ => panic!("selected family"),
        }
    }
    let mut stale = request("eeeeeeeeeeeeeeee");
    stale.artifact_generation = "old-generation".into();
    let error = f
        .worker
        .encounter_reference(&encounter.slug, &participant.participant_key, stale)
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
            "expected_artifact_generation": "old-generation",
            "actual_artifact_generation": generation,
        }))
    );
    let mut foreign = request("eeeeeeeeeeeeeeee");
    foreign.selection.record_key = "equipment:oooooooooooooooo".into();
    assert!(
        f.worker
            .encounter_reference(&encounter.slug, &participant.participant_key, foreign)
            .unwrap_err()
            .to_string()
            .contains("does not belong")
    );
    assert!(
        f.worker
            .encounter_reference(
                &encounter.slug,
                "missing-participant",
                request("eeeeeeeeeeeeeeee")
            )
            .is_err()
    );
    assert!(
        f.worker
            .encounter_reference(
                &encounter.slug,
                &participant.participant_key,
                request("missing-owner")
            )
            .is_err()
    );
    let root = f
        .worker
        .encounter_reference(
            &encounter.slug,
            &participant.participant_key,
            EncounterReferenceRequest {
                condition_id: None,
                artifact_generation: generation,
                selection: RecordDetailRequest {
                    spell_rank: None,
                    record_key: "actors:aaaaaaaaaaaaaaaa".into(),
                    owners: vec![],
                    fields: vec![],
                    passage: None,
                    source_fingerprint: None,
                },
            },
        )
        .unwrap();
    assert_eq!(
        root.presentation.body,
        updated.participants[0].presentation.as_ref().unwrap().body
    );
}

#[test]
fn restarted_encounter_reference_requires_reload_when_only_full_build_context_changes() {
    let service = |revision: &'static str| {
        fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                let mut built = input(
                    vec![record(
                        "actors",
                        "Actor",
                        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Stable actor","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60}}},"items":[{"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Stable ability","system":{"description":{"value":"<p>Unchanged ability prose.</p>"}}}]}),
                    )],
                    false,
                );
                built.context.source_revision = Some(revision.into());
                let generation = built.context.preparation_sha256()?;
                for record in &mut built.records {
                    record.preparation_context_hash = generation.clone();
                }
                Ok(open_input_fixture(built)?)
            },
        ))
    };
    let a = service("revision-a");
    let encounter = a
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Generation recovery".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    let current = a
        .worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            participant_variant: None,
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
            quantity: 1,
            initiative: None,
        })
        .and_then(|receipt| a.worker.encounter(&receipt.encounter_key))
        .unwrap();
    let generation_a = current.artifact_generation.unwrap();
    let participant_key = current.participants[0].participant_key.clone();
    let request_a = EncounterReferenceRequest {
        condition_id: None,
        artifact_generation: generation_a.clone(),
        selection: RecordDetailRequest {
            spell_rank: None,
            record_key: "actors:aaaaaaaaaaaaaaaa".into(),
            owners: vec![atlas_domain::OwnedContentLocator {
                collection: "/items".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into()),
            }],
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        },
    };
    let selected_a = a
        .worker
        .encounter_reference(&encounter.slug, &participant_key, request_a.clone())
        .unwrap();
    assert_eq!(selected_a.presentation.identity.title, "Stable ability");
    let fingerprint_a = a
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            Ok(r.source_fingerprint().to_owned())
        })
        .unwrap();
    let mut updated = input(
        vec![record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Stable actor","system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60}}},"items":[{"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Stable ability","system":{"description":{"value":"<p>Unchanged ability prose.</p>"}}}]}),
        )],
        false,
    );
    updated.context.source_revision = Some("revision-b".into());
    let hash = updated.context.preparation_sha256().unwrap();
    for record in &mut updated.records {
        record.preparation_context_hash = hash.clone();
    }
    crate::test_support::publish_input(a.worker.workspace_path(), &updated).unwrap();
    let b = &a;
    let (generation_b, fingerprint_b) = b
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            r.reset_read_metrics();
            Ok((r.artifact_generation()?, r.source_fingerprint().to_owned()))
        })
        .unwrap();
    assert_ne!(generation_a, generation_b);
    assert_eq!(fingerprint_a, fingerprint_b);

    let lookup_encounter = encounter.slug.clone();
    let lookup_participant = participant_key.clone();
    let lookup_request = request_a.clone();
    let (error, metrics) = b
        .worker
        .with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| {
                let error = super::reference::read_encounter_reference(
                    workspace,
                    &lookup_encounter,
                    &lookup_participant,
                    lookup_request,
                )
                .unwrap_err()
                .into_app_error();
                Ok((error, workspace.retrieval().read_metrics()))
            },
        )
        .unwrap();
    assert_eq!(error.code, AppErrorCode::ArtifactGenerationChanged);
    assert_eq!(
        error.details,
        Some(json!({
            "expected_artifact_generation": generation_a,
            "actual_artifact_generation": generation_b,
        }))
    );
    assert_eq!(error.retryable, Some(false));
    assert_eq!(
        error.recoverable_action,
        Some(AppRecoverableAction::ReloadRecord)
    );
    assert_eq!(
        metrics.source_body_decodes, 0,
        "generation conflict is rejected before selecting the owned document",
    );

    let refreshed = b.worker.encounter(&encounter.slug).unwrap();
    let mut request_b = request_a;
    request_b.artifact_generation = refreshed.artifact_generation.unwrap();
    let selected_b = b
        .worker
        .encounter_reference(&encounter.slug, &participant_key, request_b.clone())
        .unwrap();
    assert_eq!(selected_b.selected.owners, request_b.selection.owners);
    assert_eq!(selected_b.presentation.identity.title, "Stable ability");

    request_b.selection.owners[0].identity =
        atlas_domain::OwnedContentIdentity::SnapshotLocal { index: 0 };
    request_b.selection.source_fingerprint = Some("old-source-fingerprint".into());
    let navigation_error = b
        .worker
        .encounter_reference(&encounter.slug, &participant_key, request_b)
        .unwrap_err()
        .into_app_error();
    assert_eq!(navigation_error.code, AppErrorCode::InvalidRequest);
    assert!(
        navigation_error
            .message
            .contains("snapshot-local navigation")
    );
    assert_eq!(navigation_error.recoverable_action, None);
}

#[test]
fn contextual_reference_uses_current_actor_level_and_effective_condition_value() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"contextActor0001","name":"Context creature","type":"npc","system":{"details":{"level":{"value":7},"publicNotes":"<p>@Damage[(ceil(@actor.level/2))d6[untyped]]</p>"},"attributes":{"adjustment":null,"ac":{"value":22},"hp":{"max":60}},"traits":{"value":[],"rarity":"common"}}}),
                    ),
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"contextUnknown01","name":"Unknown adjustment","type":"npc","system":{"details":{"level":{"value":7},"publicNotes":"<p>@Damage[(ceil(@actor.level/2))d6[untyped]]</p>"},"attributes":{"adjustment":"unsupported","ac":{"value":22},"hp":{"max":60}}}}),
                    ),
                    record(
                        "conditionitems",
                        "Item",
                        json!({"_id":"e1XGnhKNSQIm5IXg","name":"Stupefied","type":"condition","system":{"value":{"value":1,"isValued":true},"description":{"value":"<p>@Check[flat|showDC:all|dc:resolve(5+@item.badge.value)] with a DC equal to 5 + your stupefied value.</p>"}}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Context formulas".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter;
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            participant_variant: None,
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:contextActor0001".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let current = f.worker.encounter(&encounter.slug).unwrap();
    let participant = &current.participants[0];
    let request = EncounterReferenceRequest {
        condition_id: None,
        artifact_generation: current.artifact_generation.clone().unwrap(),
        selection: RecordDetailRequest {
            record_key: "actors:contextActor0001".into(),
            owners: vec![],
            fields: vec![],
            passage: None,
            spell_rank: None,
            source_fingerprint: None,
        },
    };
    let selected = f
        .worker
        .encounter_reference(
            &encounter.slug,
            &participant.participant_key,
            request.clone(),
        )
        .unwrap();
    let get_roll = |view: &RecordDetailView| {
        view.presentation
            .content
            .iter()
            .find_map(|f| match &f.body {
                PreparedFieldBodyView::Html { controls, .. } => {
                    controls.iter().find_map(|c| match &c.control {
                        ContentControlKindView::Damage { roll, .. } => roll.clone(),
                        _ => None,
                    })
                }
                _ => None,
            })
            .unwrap()
    };
    let old = get_roll(&selected);
    assert!(old.encounter.is_some());
    assert!(f.worker.roll_content(old.clone()).is_ok());
    let mut update = super::tests::update(participant);
    update.participant_variant = EncounterParticipantVariantView::Weak;
    update.variant_edit = true;
    f.worker
        .update_encounter_participant(&encounter.slug, update)
        .unwrap();
    assert!(
        f.worker.roll_content(old).is_err(),
        "changed participant revision cannot reuse its old offer"
    );
    let updated = f
        .worker
        .encounter_reference(
            &encounter.slug,
            &participant.participant_key,
            request.clone(),
        )
        .unwrap();
    let outcome = f.worker.roll_content(get_roll(&updated)).unwrap();
    assert!((3..=18).contains(&outcome.components[0].outcome.total));
    let current = f.worker.encounter(&encounter.slug).unwrap();
    f.worker
        .add_encounter_participant_condition(
            &encounter.slug,
            AddEncounterParticipantConditionRequest {
                request_id: "context-stupefied".into(),
                event_intent: Default::default(),
                decisions: Default::default(),
                expected_encounter_revision: current.revision,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                participant_key: participant.participant_key.clone(),
                condition_ref: Some("conditionitems:e1XGnhKNSQIm5IXg".into()),
                name: None,
                value: Some(2),
                source_participant_key: None,
                duration: None,
                note: None,
            },
        )
        .unwrap();
    let current = f.worker.encounter(&encounter.slug).unwrap();
    let condition = &current.participants[0].conditions[0];
    let mut request = request;
    request.condition_id = Some(condition.condition_id);
    request.selection.record_key = condition.condition_key.clone().unwrap();
    let selected = f
        .worker
        .encounter_reference(
            &encounter.slug,
            &participant.participant_key,
            request.clone(),
        )
        .unwrap();
    let text = selected
        .presentation
        .content
        .iter()
        .find_map(|f| match &f.body {
            PreparedFieldBodyView::Html { html, .. } => Some(html),
            _ => None,
        })
        .unwrap();
    assert!(text.contains("DC 7 flat check"));
    assert!(!text.contains("your stupefied value"));
    request.condition_id = Some(condition.condition_id + 999);
    assert!(
        f.worker
            .encounter_reference(&encounter.slug, &participant.participant_key, request)
            .is_err()
    );
    let generic = f
        .worker
        .record_detail("conditionitems:e1XGnhKNSQIm5IXg")
        .unwrap();
    let generic = serde_json::to_string(&generic.presentation.content).unwrap();
    assert!(generic.contains("5 + your stupefied value"));
    assert!(!generic.contains("DC 6"));
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            participant_variant: None,
            encounter_ref: encounter.slug.clone(),
            record_ref: "actors:contextUnknown01".into(),
            quantity: 1,
            initiative: None,
        })
        .unwrap();
    let current = f.worker.encounter(&encounter.slug).unwrap();
    let unknown = current
        .participants
        .iter()
        .find(|p| p.record_key.as_deref() == Some("actors:contextUnknown01"))
        .unwrap();
    let unknown = f
        .worker
        .encounter_reference(
            &encounter.slug,
            &unknown.participant_key,
            EncounterReferenceRequest {
                condition_id: None,
                artifact_generation: current.artifact_generation.unwrap(),
                selection: RecordDetailRequest {
                    record_key: "actors:contextUnknown01".into(),
                    owners: vec![],
                    fields: vec![],
                    passage: None,
                    spell_rank: None,
                    source_fingerprint: None,
                },
            },
        )
        .unwrap();
    let unknown = serde_json::to_string(&unknown.presentation.content).unwrap();
    assert!(
        unknown.contains("the creature’s level"),
        "unknown effective level never falls back to its unadjusted source"
    );
    assert!(!unknown.contains(">4d6"));
}
