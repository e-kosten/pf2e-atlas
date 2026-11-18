use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;

fn fixture() -> (FixtureWorker, String, String) {
    let f = encounter_fixture_worker();
    let encounter = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "GM adjustments".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    f.worker
        .add_encounter_record_participant(
            &encounter,
            AddEncounterRecordParticipantRequest {
                creature_scenario: None,
                participant_variant: None,
                record_ref: "actors:testCreature1000".into(),
                quantity: 1,
                initiative: Some(20),
            },
        )
        .unwrap();
    let detail = f.worker.encounter(&encounter).unwrap();
    (f, encounter, detail.participants[0].participant_key.clone())
}

fn request(
    f: &FixtureWorker,
    encounter: &str,
    id: &str,
    action: EncounterAdjustmentAction,
) -> EncounterAdjustmentRequest {
    let saved = f.worker.encounter(encounter).unwrap();
    EncounterAdjustmentRequest {
        request_id: id.into(),
        expected_encounter_revision: saved.revision,
        expected_participant_revision: saved.participants[0].revision,
        expected_reference_revision: None,
        action,
    }
}

fn adjustment(target: AdjustmentTarget, amount: i64) -> EncounterAdjustmentInput {
    EncounterAdjustmentInput {
        target,
        amount,
        name: "Table ruling".into(),
        modifier_type: AdjustmentModifierType::Stacks,
        timing: None,
    }
}

fn apply(
    f: &FixtureWorker,
    encounter: &str,
    actor: &str,
    mut request: EncounterAdjustmentRequest,
) -> (EncounterAdjustmentPreviewView, EncounterAdjustmentRequest) {
    let preview = f
        .worker
        .preview_encounter_adjustment(encounter, actor, request.clone())
        .unwrap();
    request.expected_reference_revision = preview.reference_revision;
    f.worker
        .update_encounter_adjustment(encounter, actor, request.clone())
        .unwrap();
    (preview, request)
}

#[test]
fn gm_adjustment_recomputes_ac_and_initiative_and_replays_without_duplication() {
    let (f, encounter, actor) = fixture();
    let ac = |detail: EncounterDetailView| {
        let RecordBodyView::Creature(record) = detail.participants[0]
            .presentation
            .as_ref()
            .unwrap()
            .body
            .clone()
        else {
            panic!("expected creature");
        };
        record.armor_class
    };
    for (id, before, after) in [("first", 22, 20), ("second", 20, 18), ("third", 18, 16)] {
        let (preview, saved) = apply(
            &f,
            &encounter,
            &actor,
            request(
                &f,
                &encounter,
                id,
                EncounterAdjustmentAction::Add {
                    value: adjustment(AdjustmentTarget::ArmorClass, -2),
                },
            ),
        );
        assert_eq!(
            preview.rows[0]
                .before
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(before)
        );
        assert_eq!(
            preview.rows[0]
                .after
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(after)
        );
        f.worker
            .update_encounter_adjustment(&encounter, &actor, saved)
            .unwrap();
        let fact = ac(f.worker.encounter(&encounter).unwrap());
        assert_eq!(
            fact.value.as_ref().and_then(serde_json::Number::as_i64),
            Some(after)
        );
        assert_eq!(
            fact.adjustment.unwrap().applied.len(),
            (22 - after) as usize / 2
        );
    }

    apply(
        &f,
        &encounter,
        &actor,
        request(
            &f,
            &encounter,
            "perception",
            EncounterAdjustmentAction::Add {
                value: adjustment(AdjustmentTarget::Perception, 2),
            },
        ),
    );
    let initiative = f.worker.encounter_initiative_options(&encounter).unwrap();
    assert_eq!(
        initiative.participants[0]
            .statistics
            .iter()
            .find(|stat| stat.key == "perception")
            .unwrap()
            .modifier,
        15
    );

    apply(
        &f,
        &encounter,
        &actor,
        request(
            &f,
            &encounter,
            "end",
            EncounterAdjustmentAction::End {
                adjustment_key: "adjustment_second".into(),
            },
        ),
    );
    assert_eq!(
        ac(f.worker.encounter(&encounter).unwrap())
            .value
            .unwrap()
            .as_i64(),
        Some(18)
    );
}

#[test]
fn timed_gm_adjustment_expires_once_and_standalone_import_remaps_its_anchor() {
    let (f, encounter, actor) = fixture();
    let mut value = adjustment(AdjustmentTarget::ArmorClass, -2);
    value.timing = Some(EncounterAdjustmentTimingInput {
        duration: ConditionDurationView {
            remaining_rounds: None,
            amount: 1,
            unit: ConditionDurationUnit::Rounds,
            timing: Some(ConditionTimingView {
                participant_key: actor.clone(),
                event: ConditionTurnEvent::Start,
            }),
        },
        remaining: None,
    });
    apply(
        &f,
        &encounter,
        &actor,
        request(
            &f,
            &encounter,
            "timer",
            EncounterAdjustmentAction::Add { value },
        ),
    );
    let exported = f.worker.export_encounter(&encounter).unwrap();
    let mut document: serde_json::Value = serde_json::from_slice(&exported).unwrap();
    assert_eq!(document["version"], 2);
    document["version"] = 1.into();
    let mislabeled = serde_json::to_vec(&document).unwrap();
    assert!(f.worker.import_encounter(&mislabeled[..], None).is_err());

    let new = f
        .worker
        .import_encounter(&exported[..], Some("restored-gm-adjustments".into()))
        .unwrap();
    let restored = f.worker.encounter(&new.encounter_key).unwrap();
    let participant = &restored.participants[0];
    let entries = &participant.gm_adjustments.as_ref().unwrap().entries;
    let timing = entries[0].timing.as_ref().unwrap();
    assert_ne!(participant.participant_key, actor);
    assert_eq!(
        timing.duration.timing.as_ref().unwrap().participant_key,
        participant.participant_key
    );
    assert_eq!(timing.remaining, 1);

    let request = AdvanceEncounterTurnRequest {
        request_id: "expire".into(),
        expected_revision: restored.revision,
    };
    f.worker
        .advance_encounter_turn(&new.encounter_key, request.clone())
        .unwrap();
    f.worker
        .advance_encounter_turn(&new.encounter_key, request)
        .unwrap();
    let detail = f.worker.encounter(&new.encounter_key).unwrap();
    assert!(
        detail.participants[0]
            .gm_adjustments
            .as_ref()
            .unwrap()
            .entries
            .is_empty()
    );
    let review = f.worker.encounter_turn_review(&new.encounter_key).unwrap();
    assert_eq!(review.incoming.unwrap().reminders.iter().filter(|item| matches!(item, EncounterTurnReminderView::AdjustmentExpired { adjustment_name, .. } if adjustment_name == "Table ruling")).count(), 1);
}

#[test]
fn overnight_gm_adjustment_advances_exact_seconds_and_keeps_manual_changes() {
    let (f, encounter, actor) = fixture();
    for (id, duration) in [("overnight", Some(24)), ("manual", None)] {
        let mut value = adjustment(AdjustmentTarget::ArmorClass, -2);
        value.timing = duration.map(|amount| EncounterAdjustmentTimingInput {
            duration: ConditionDurationView {
                amount,
                unit: ConditionDurationUnit::Hours,
                timing: Some(ConditionTimingView {
                    participant_key: actor.clone(),
                    event: ConditionTurnEvent::End,
                }),
                remaining_rounds: None,
            },
            remaining: None,
        });
        apply(
            &f,
            &encounter,
            &actor,
            request(&f, &encounter, id, EncounterAdjustmentAction::Add { value }),
        );
    }
    for (index, remaining) in [Some(16 * 3600), Some(8 * 3600), None]
        .into_iter()
        .enumerate()
    {
        let snapshot = f.worker.encounter(&encounter).unwrap();
        let action = EncounterRecoveryAction::RestOvernight {
            healing_mode: OvernightHealingMode::Natural,
            participants: vec![OvernightParticipantInput {
                participant_key: actor.clone(),
                healing: Some(0),
            }],
        };
        let preview = f
            .worker
            .preview_encounter_recovery(
                &encounter,
                PreviewEncounterRecoveryRequest {
                    expected_encounter_revision: snapshot.revision,
                    action: action.clone(),
                },
            )
            .unwrap();
        assert_eq!(
            preview.participants[0]
                .changes
                .iter()
                .any(|change| change == "Table ruling expired"),
            index == 2
        );
        f.worker
            .apply_encounter_recovery(
                &encounter,
                ApplyEncounterRecoveryRequest {
                    expected_encounter_revision: snapshot.revision,
                    action,
                },
            )
            .unwrap();
        let restored = f.worker.encounter(&encounter).unwrap();
        let adjustments = &restored.participants[0]
            .gm_adjustments
            .as_ref()
            .unwrap()
            .entries;
        assert_eq!(
            adjustments
                .iter()
                .find_map(|entry| entry.timing.as_ref().map(|v| v.remaining)),
            remaining
        );
        assert!(
            adjustments
                .iter()
                .any(|entry| entry.adjustment_key == "adjustment_manual")
        );
    }
}

#[test]
fn gm_item_adjustments_compete_by_sign_and_restore_suppressed_contributions() {
    let (f, encounter, actor) = fixture();
    let mut before = 22;
    for (id, modifier_type, amount, after) in [
        ("item-bonus", AdjustmentModifierType::Item, 2, 24),
        ("smaller-item-bonus", AdjustmentModifierType::Item, 1, 24),
        ("item-penalty", AdjustmentModifierType::Item, -1, 23),
        (
            "stronger-item-penalty",
            AdjustmentModifierType::Item,
            -3,
            21,
        ),
        ("status-bonus", AdjustmentModifierType::Status, 2, 23),
    ] {
        let mut value = adjustment(AdjustmentTarget::ArmorClass, amount);
        value.modifier_type = modifier_type;
        let (preview, _) = apply(
            &f,
            &encounter,
            &actor,
            request(&f, &encounter, id, EncounterAdjustmentAction::Add { value }),
        );
        assert_eq!(
            preview.rows[0]
                .before
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(before)
        );
        assert_eq!(
            preview.rows[0]
                .after
                .as_ref()
                .and_then(serde_json::Number::as_i64),
            Some(after)
        );
        before = after;
    }
    let detail = f.worker.encounter(&encounter).unwrap();
    let participant = &detail.participants[0];
    let RecordBodyView::Creature(creature) = &participant.presentation.as_ref().unwrap().body
    else {
        panic!("expected creature");
    };
    let calculation = creature.armor_class.adjustment.as_ref().unwrap();
    assert_eq!(calculation.authored.as_i64(), Some(22));
    assert_eq!(
        calculation
            .applied
            .iter()
            .filter(|modifier| modifier.modifier_type == StatModifierTypeView::Item)
            .count(),
        2
    );
    assert_eq!(
        calculation
            .suppressed
            .iter()
            .filter(|modifier| modifier.modifier_type == StatModifierTypeView::Item)
            .count(),
        2
    );
    let entries = &participant.gm_adjustments.as_ref().unwrap().entries;
    assert_eq!(
        serde_json::to_value(&entries[0]).unwrap()["modifier_type"],
        "item"
    );
    let (preview, _) = apply(
        &f,
        &encounter,
        &actor,
        request(
            &f,
            &encounter,
            "end-item-penalty",
            EncounterAdjustmentAction::End {
                adjustment_key: entries[3].adjustment_key.clone(),
            },
        ),
    );
    assert_eq!(
        preview.rows[0]
            .before
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(23)
    );
    assert_eq!(
        preview.rows[0]
            .after
            .as_ref()
            .and_then(serde_json::Number::as_i64),
        Some(25)
    );
}
