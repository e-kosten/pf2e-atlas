use crate::test_support::{FixtureWorker, encounter_fixture_worker};
use atlas_app_model::*;
use atlas_local_state::{
    AddEncounterParticipantCondition, ConditionDuration, ConditionDurationUnit as LocalUnit,
    ConditionTiming, ConditionTurnEvent as LocalEvent,
};

fn fixture() -> (FixtureWorker, String, String, String) {
    let f = encounter_fixture_worker();
    let key = f
        .worker
        .create_encounter(CreateEncounterRequest {
            name: "Timing".into(),
            description: None,
            note: None,
        })
        .unwrap()
        .encounter
        .encounter_key;
    for (name, initiative) in [("A", 20), ("B", 10)] {
        f.worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: key.clone(),
                display_name: name.into(),
                max_hp: Some(20),
                current_hp: Some(20),
                initiative: Some(initiative),
            })
            .unwrap();
    }
    let snapshot = f
        .worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&key)?)
        })
        .unwrap()
        .unwrap();
    let a = snapshot.participants[0].participant_key.clone();
    let b = snapshot.participants[1].participant_key.clone();
    (f, key, a, b)
}
fn add(
    f: &FixtureWorker,
    owner: &str,
    key: Option<&str>,
    duration: Option<ConditionDuration>,
) -> i64 {
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().add_condition(
                AddEncounterParticipantCondition {
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    participant_key: owner.into(),
                    condition_key: key.map(Into::into),
                    name: "Same display name".into(),
                    value: Some(2),
                    source_participant_key: None,
                    duration,
                    note: Some("table note".into()),
                },
            )?)
        })
        .unwrap()
        .condition_id
}
fn rounds(amount: i64, anchor: Option<&str>, event: LocalEvent) -> Option<ConditionDuration> {
    Some(ConditionDuration {
        amount,
        unit: LocalUnit::Rounds,
        timing: anchor.map(|key| ConditionTiming {
            participant_key: key.into(),
            event,
        }),
    })
}
fn snapshot(f: &FixtureWorker, key: &str) -> atlas_local_state::EncounterWithParticipants {
    f.worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(key)?)
        })
        .unwrap()
        .unwrap()
}
fn amounts(f: &FixtureWorker, key: &str) -> Vec<i64> {
    snapshot(f, key)
        .participants
        .iter()
        .flat_map(|p| &p.conditions)
        .filter_map(|c| c.duration.as_ref().map(|d| d.amount))
        .collect()
}
fn next(f: &FixtureWorker, key: &str) {
    let review = f.worker.encounter_turn_review(key).unwrap();
    f.worker
        .advance_encounter_turn(
            key,
            AdvanceEncounterTurnRequest {
                request_id: format!("next-{}", review.expected_revision),
                expected_revision: review.expected_revision,
            },
        )
        .unwrap();
}
fn set(f: &FixtureWorker, key: &str, participant: &str) {
    f.worker
        .set_encounter_turn(SetEncounterTurnRequest {
            encounter_ref: key.into(),
            participant_key: participant.into(),
        })
        .unwrap();
}

#[test]
fn explicit_end_old_and_start_new_tick_once_and_full_wrap_is_not_global_tick() {
    let (f, key, a, b) = fixture();
    add(&f, &a, None, rounds(2, Some(&a), LocalEvent::End));
    add(&f, &a, None, rounds(2, Some(&b), LocalEvent::Start));
    add(&f, &a, None, rounds(2, Some(&a), LocalEvent::Start));
    add(&f, &a, None, rounds(2, Some(&b), LocalEvent::End));
    set(&f, &key, &a);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    assert_eq!(review.next_participant_key.as_deref(), Some(b.as_str()));
    assert!(review.reminders.is_empty());
    assert_eq!(amounts(&f, &key), [2, 2, 2, 2]);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [1, 1, 2, 2]);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [1, 1, 1, 1]);
    assert_eq!(snapshot(&f, &key).encounter.round_number, 2);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [0, 0, 1, 1]);
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0].value,
        Some(2)
    );
}

#[test]
fn upcoming_start_and_end_expiry_are_forecast_without_mutation_and_zero_is_retained() {
    let (f, key, a, b) = fixture();
    let end = add(&f, &a, None, rounds(1, Some(&a), LocalEvent::End));
    let start = add(&f, &a, None, rounds(1, Some(&b), LocalEvent::Start));
    set(&f, &key, &a);
    let before = snapshot(&f, &key);
    for _ in 0..3 {
        let review = f.worker.encounter_turn_review(&key).unwrap();
        assert!(
            matches!(&review.reminders[..],[EncounterTurnReminderView::DurationEnding{condition_id:x,timing:ConditionTimingView{event:ConditionTurnEvent::End,..},..},EncounterTurnReminderView::DurationEnding{condition_id:y,timing:ConditionTimingView{event:ConditionTurnEvent::Start,..},..}] if *x==end && *y==start)
        );
    }
    assert_eq!(snapshot(&f, &key), before);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [0, 0]);
    next(&f, &key);
    assert!(f.worker.encounter_turn_review(&key).unwrap().reminders.iter().any(|r|matches!(r,EncounterTurnReminderView::DurationEnded{condition_id,..} if *condition_id==end)));
    assert_eq!(snapshot(&f, &key).participants[0].conditions.len(), 2);
}

#[test]
fn remove_now_is_durable_even_without_continue_and_invalidates_old_review() {
    let (f, key, a, _) = fixture();
    let id = add(&f, &a, None, rounds(1, Some(&a), LocalEvent::End));
    set(&f, &key, &a);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    f.worker
        .remove_encounter_participant_condition(
            &key,
            &a,
            id,
            atlas_app_model::RemoveEncounterParticipantConditionRequest {
                request_id: "fixture-remove".into(),
                expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
                intent: atlas_app_model::ConditionRemovalIntentView::RemoveEffect,
            },
        )
        .unwrap();
    let error = f
        .worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: format!("next-{}", review.expected_revision),
                expected_revision: review.expected_revision,
            },
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::EncounterTurnConflict);
    assert_eq!(error.retryable, Some(false));
    assert!(snapshot(&f, &key).participants[0].conditions.is_empty());
    assert_eq!(
        snapshot(&f, &key).encounter.current_turn_participant_key,
        Some(a)
    );
}

#[test]
fn unresolved_and_deleted_anchors_do_not_tick_or_get_guessed_from_source() {
    let (f, key, a, b) = fixture();
    add(&f, &a, None, rounds(3, None, LocalEvent::End));
    add(
        &f,
        &a,
        None,
        rounds(3, Some("deleted-participant"), LocalEvent::Start),
    );
    add(&f, &a, None, rounds(3, Some(&b), LocalEvent::Start));
    // Existing manual Set and participant removal are explicitly not timing events.
    set(&f, &key, &a);
    f.worker.remove_encounter_participant(&key, &b).unwrap();
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [3, 3, 3]);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    assert_eq!(
        review
            .reminders
            .iter()
            .filter(|r| matches!(r, EncounterTurnReminderView::TimingUnavailable { .. }))
            .count(),
        3
    );
}

#[test]
fn canonical_persistent_only_owner_end_and_nonround_reminders_never_apply_hp_or_decay() {
    let (f, key, a, b) = fixture();
    let persistent = add(&f, &a, Some("conditionitems:lDVqvLKA6eF3Df60"), None);
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().add_condition(
                AddEncounterParticipantCondition {
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    participant_key: a.clone(),
                    condition_key: None,
                    name: "Persistent Damage".into(),
                    value: None,
                    source_participant_key: None,
                    duration: None,
                    note: None,
                },
            )?)
        })
        .unwrap();
    add(
        &f,
        &a,
        None,
        Some(ConditionDuration {
            amount: 5,
            unit: LocalUnit::Minutes,
            timing: None,
        }),
    );
    set(&f, &key, &b);
    assert!(
        f.worker
            .encounter_turn_review(&key)
            .unwrap()
            .reminders
            .is_empty()
    );
    next(&f, &key);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    assert!(
        matches!(&review.reminders[..],[EncounterTurnReminderView::PersistentDamage{condition_id,..},EncounterTurnReminderView::DurationReminder{duration:ConditionDurationView{amount:5,unit:ConditionDurationUnit::Minutes,..},..}] if *condition_id==persistent)
    );
    next(&f, &key);
    let saved = snapshot(&f, &key);
    assert_eq!(saved.participants[0].current_hp, Some(20));
    assert_eq!(saved.participants[0].conditions.len(), 3);
    assert_eq!(saved.participants[0].conditions[0].value, Some(2));
    assert_eq!(amounts(&f, &key), [5]);
}

#[test]
fn single_actor_has_one_end_one_start_and_no_eligible_actor_has_no_tick() {
    let (f, key, a, b) = fixture();
    f.worker.remove_encounter_participant(&key, &b).unwrap();
    add(&f, &a, None, rounds(2, Some(&a), LocalEvent::End));
    add(&f, &a, None, rounds(2, Some(&a), LocalEvent::Start));
    // Initial entry has START only.
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [2, 1]);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [1, 0]);
    assert_eq!(snapshot(&f, &key).encounter.round_number, 2);
    let mut row = f
        .worker
        .with_state(|session| {
            Ok(session
                .local_state()
                .encounters()
                .get_with_participants(&key)?)
        })
        .unwrap()
        .unwrap()
        .participants
        .remove(0);
    row.defeated = true;
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().update_participant(
                atlas_local_state::UpdateEncounterParticipant {
                    expected_revision: row.revision,
                    participant_key: row.participant_key,
                    hp_origin: row.hp_origin,
                    variant_origin: row.variant_origin,
                    display_name: row.display_name,
                    side: row.side,
                    participant_variant: row.participant_variant,
                    initiative: None,
                    max_hp: row.max_hp,
                    current_hp: row.current_hp,
                    temporary_hp: row.temporary_hp,
                    defeated: true,
                    hazard_disabled: row.hazard_disabled,
                    hidden: row.hidden,
                    note: row.note,
                },
            )?)
        })
        .unwrap();
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [1, 0]);
    assert_eq!(
        snapshot(&f, &key).encounter.current_turn_participant_key,
        None
    );
}

#[test]
fn new_duration_requires_valid_amount_unit_anchor_and_legacy_unchanged_is_preserved() {
    let (f, key, a, b) = fixture();
    let snapshot = snapshot(&f, &key);
    let input = |amount, unit, timing| {
        Some(ConditionDurationView {
            amount,
            unit,
            timing,
        })
    };
    for bad in [
        input(-1, ConditionDurationUnit::Minutes, None),
        input(2, ConditionDurationUnit::Rounds, None),
        input(
            2,
            ConditionDurationUnit::Rounds,
            Some(ConditionTimingView {
                participant_key: "other-encounter".into(),
                event: ConditionTurnEvent::End,
            }),
        ),
        input(
            2,
            ConditionDurationUnit::Days,
            Some(ConditionTimingView {
                participant_key: a.clone(),
                event: ConditionTurnEvent::Start,
            }),
        ),
    ] {
        let error = super::timing::duration_input(bad, None, &snapshot)
            .unwrap_err()
            .into_app_error();
        assert_eq!(error.code, AppErrorCode::InvalidRequest);
    }
    let legacy = ConditionDuration {
        amount: 3,
        unit: LocalUnit::Rounds,
        timing: None,
    };
    assert_eq!(
        super::timing::duration_input(
            input(3, ConditionDurationUnit::Rounds, None),
            Some(&legacy),
            &snapshot
        )
        .unwrap(),
        Some(legacy)
    );
    let valid = super::timing::duration_input(
        input(
            2,
            ConditionDurationUnit::Rounds,
            Some(ConditionTimingView {
                participant_key: b.clone(),
                event: ConditionTurnEvent::Start,
            }),
        ),
        None,
        &snapshot,
    )
    .unwrap()
    .unwrap();
    assert_eq!(valid.timing.unwrap().participant_key, b);
}

#[test]
fn manual_set_and_reorder_never_tick_and_any_mutation_invalidates_review() {
    let (f, key, a, b) = fixture();
    add(&f, &a, None, rounds(4, Some(&a), LocalEvent::End));
    set(&f, &key, &a);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    f.worker
        .reorder_encounter_participant(
            &key,
            ReorderEncounterParticipantRequest {
                participant_key: b.clone(),
                target_participant_key: a.clone(),
                placement: ReorderEncounterParticipantPlacementView::Before,
            },
        )
        .unwrap();
    assert_eq!(amounts(&f, &key), [4]);
    let error = f
        .worker
        .advance_encounter_turn(
            &key,
            AdvanceEncounterTurnRequest {
                request_id: format!("next-{}", review.expected_revision),
                expected_revision: review.expected_revision,
            },
        )
        .unwrap_err()
        .into_app_error();
    assert_eq!(error.code, AppErrorCode::EncounterTurnConflict);
    set(&f, &key, &b);
    set(&f, &key, &a);
    assert_eq!(amounts(&f, &key), [4]);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    f.worker
        .apply_encounter_hp_action(
            &key,
            ApplyEncounterHpActionRequest {
                request_id: crate::test_support::next_request_id(),
                context: None,
                participant_key: a.clone(),
                expected_revision: snapshot(&f, &key)
                    .participants
                    .iter()
                    .find(|p| p.participant_key == a)
                    .unwrap()
                    .revision,
                action: EncounterHpAction::Damage { amount: "1".into() },
            },
        )
        .unwrap();
    assert_eq!(
        f.worker
            .advance_encounter_turn(
                &key,
                AdvanceEncounterTurnRequest {
                    request_id: format!("next-{}", review.expected_revision),
                    expected_revision: review.expected_revision
                }
            )
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterTurnConflict
    );
    assert_eq!(amounts(&f, &key), [4]);
}

fn condition_edit(
    condition: &atlas_local_state::EncounterParticipantCondition,
    revision: i64,
) -> UpdateEncounterParticipantConditionRequest {
    UpdateEncounterParticipantConditionRequest {
        request_id: "fixture-timing_tests-470".into(),
        event_intent: Default::default(),
        decisions: Default::default(),
        scope: Default::default(),
        extra_action: Default::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        expected_encounter_revision: revision,
        condition_id: condition.condition_id,
        condition_ref: condition.condition_key.clone(),
        name: condition.name.clone(),
        value: condition.value,
        source_participant_key: condition.source_participant_key.clone(),
        duration: condition.duration.clone().map(super::timing::duration_view),
        note: Some("edited note".into()),
    }
}

#[test]
fn stale_notes_cannot_restore_countdown_and_fresh_note_source_edits_preserve_remaining() {
    let (f, key, a, b) = fixture();
    add(&f, &a, None, rounds(3, Some(&a), LocalEvent::End));
    set(&f, &key, &a);
    let before = snapshot(&f, &key);
    let draft = condition_edit(
        &before.participants[0].conditions[0],
        before.encounter.revision,
    );
    next(&f, &key);
    let after = snapshot(&f, &key);
    assert_eq!(amounts(&f, &key), [2]);
    assert_eq!(
        f.worker
            .update_encounter_participant_condition(&key, &a, draft)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::EncounterTurnConflict
    );
    assert_eq!(snapshot(&f, &key), after);
    let mut fresh = condition_edit(
        &after.participants[0].conditions[0],
        after.encounter.revision,
    );
    fresh.source_participant_key = Some(b);
    f.worker
        .update_encounter_participant_condition(&key, &a, fresh)
        .unwrap();
    assert_eq!(amounts(&f, &key), [2]);
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0]
            .note
            .as_deref(),
        Some("edited note")
    );
}

#[test]
fn unrelated_legacy_notes_edit_preserves_unresolved_rounds_but_changed_duration_needs_anchor() {
    let (f, key, a, _) = fixture();
    add(&f, &a, None, rounds(4, None, LocalEvent::Start));
    let before = snapshot(&f, &key);
    f.worker
        .update_encounter_participant_condition(
            &key,
            &a,
            condition_edit(
                &before.participants[0].conditions[0],
                before.encounter.revision,
            ),
        )
        .unwrap();
    let after = snapshot(&f, &key);
    assert_eq!(amounts(&f, &key), [4]);
    assert_eq!(
        after.participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .timing,
        None
    );
    let mut changed = condition_edit(
        &after.participants[0].conditions[0],
        after.encounter.revision,
    );
    changed.duration.as_mut().unwrap().amount = 5;
    assert_eq!(
        f.worker
            .update_encounter_participant_condition(&key, &a, changed)
            .unwrap_err()
            .into_app_error()
            .code,
        AppErrorCode::InvalidRequest
    );
    assert_eq!(snapshot(&f, &key), after);
}

#[test]
fn zero_nonround_duration_is_only_a_manual_reminder() {
    let (f, key, a, _) = fixture();
    add(
        &f,
        &a,
        None,
        Some(ConditionDuration {
            amount: 0,
            unit: LocalUnit::Hours,
            timing: None,
        }),
    );
    set(&f, &key, &a);
    assert!(matches!(
        &f.worker.encounter_turn_review(&key).unwrap().reminders[..],
        [EncounterTurnReminderView::DurationReminder {
            duration: ConditionDurationView {
                amount: 0,
                unit: ConditionDurationUnit::Hours,
                ..
            },
            ..
        }]
    ));
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [0]);
}

#[test]
fn expired_rounds_do_not_disable_modeled_condition_effects() {
    let (f, key, _, _) = fixture();
    f.worker
        .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
            encounter_ref: key.clone(),
            record_ref: "Test Creature 1".into(),
            quantity: 1,
            initiative: Some(30),
        })
        .unwrap();
    let owner = snapshot(&f, &key).participants[0].participant_key.clone();
    f.worker
        .add_encounter_participant_condition(
            &key,
            AddEncounterParticipantConditionRequest {
                request_id: format!(
                    "fixture-timing_tests-609-{}",
                    f.worker.encounter(&key).unwrap().revision
                ),
                event_intent: Default::default(),
                decisions: Default::default(),
                expected_encounter_revision: f.worker.encounter(&key).unwrap().revision,
                scope: Default::default(),
                extra_action: Default::default(),
                immunity_override: false,
                parameters: Default::default(),
                origin: None,
                participant_key: owner.clone(),
                condition_ref: Some("conditionitems:TBSHQspnbcqxsmjL".into()),
                name: None,
                value: Some(2),
                source_participant_key: None,
                duration: Some(ConditionDurationView {
                    amount: 1,
                    unit: ConditionDurationUnit::Rounds,
                    timing: Some(ConditionTimingView {
                        participant_key: owner.clone(),
                        event: ConditionTurnEvent::End,
                    }),
                }),
                note: None,
            },
        )
        .unwrap();
    set(&f, &key, &owner);
    let armor_class = |p: EncounterParticipantView| match p.presentation.unwrap().body {
        RecordBodyView::Creature(actor) => actor.armor_class,
        _ => panic!("creature presentation"),
    };
    let before = armor_class(
        f.worker
            .encounter(&key)
            .unwrap()
            .participants
            .into_iter()
            .find(|p| p.participant_key == owner)
            .unwrap(),
    );
    next(&f, &key);
    let participant = f
        .worker
        .encounter(&key)
        .unwrap()
        .participants
        .into_iter()
        .find(|p| p.participant_key == owner)
        .unwrap();
    assert_eq!(
        participant.conditions[0].duration.as_ref().unwrap().amount,
        0
    );
    assert_eq!(participant.conditions[0].value, Some(2));
    assert_eq!(armor_class(participant), before);
}

#[test]
fn present_inactive_anchor_waits_until_its_actual_boundary() {
    let (f, key, a, b) = fixture();
    add(&f, &a, None, rounds(3, Some(&b), LocalEvent::Start));
    add(&f, &a, None, rounds(3, Some(&b), LocalEvent::End));
    let row = snapshot(&f, &key).participants.remove(1);
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().update_participant(
                atlas_local_state::UpdateEncounterParticipant {
                    expected_revision: row.revision,
                    participant_key: row.participant_key,
                    hp_origin: row.hp_origin,
                    variant_origin: row.variant_origin,
                    display_name: row.display_name,
                    side: row.side,
                    participant_variant: row.participant_variant,
                    initiative: None,
                    max_hp: row.max_hp,
                    current_hp: row.current_hp,
                    temporary_hp: row.temporary_hp,
                    defeated: true,
                    hazard_disabled: row.hazard_disabled,
                    hidden: row.hidden,
                    note: row.note,
                },
            )?)
        })
        .unwrap();
    set(&f, &key, &a);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [3, 3]);
    // Manually choosing B is not an event. Leaving its actual current turn has END.
    set(&f, &key, &b);
    assert_eq!(amounts(&f, &key), [3, 3]);
    next(&f, &key);
    assert_eq!(amounts(&f, &key), [3, 2]);
}

#[test]
fn turn_order_skips_unrolled_participants_and_explains_missing_scores() {
    let (f, key, a, b) = fixture();
    f.worker
        .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "C".into(),
            max_hp: Some(10),
            current_hp: Some(10),
            initiative: None,
        })
        .unwrap();
    let mut state = snapshot(&f, &key);
    state.encounter.current_turn_participant_key = Some(a.clone());
    assert_eq!(
        super::turns::next_turn(&state).participant_key.as_deref(),
        Some(b.as_str())
    );
    state.participants[1].initiative = None;
    assert_eq!(
        super::turns::next_turn(&state).participant_key.as_deref(),
        Some(a.as_str())
    );
    for participant in &mut state.participants {
        participant.initiative = None;
    }
    assert_eq!(super::turns::next_turn(&state).participant_key, None);
    assert_eq!(
        super::timing::turn_review(&state).next_turn_unavailable_reason,
        Some(EncounterNextTurnUnavailableReason::NoInitiativeParticipants)
    );
    state.participants[0].initiative = Some(20);
    state.participants[0].defeated = true;
    state.participants[1].initiative = Some(10);
    state.participants[1].hazard_disabled = true;
    state.participants[2].initiative = Some(5);
    state.participants[2].mortality = atlas_local_state::MortalityState::Dead;
    assert_eq!(super::turns::next_turn(&state).participant_key, None);
    assert_eq!(
        super::timing::turn_review(&state).next_turn_unavailable_reason,
        Some(EncounterNextTurnUnavailableReason::NoActiveParticipants)
    );
}

#[test]
fn latest_turn_replaces_old_reminders_and_retries_never_advance_twice() {
    let (f, key, a, b) = fixture();
    let frightened =
        super::conditions::modeled_condition_by_rule(super::conditions::ConditionRule::Frightened)
            .unwrap();
    add(
        &f,
        &a,
        Some(frightened.condition_ref),
        rounds(3, Some(&a), LocalEvent::End),
    );
    set(&f, &key, &a);
    let request = AdvanceEncounterTurnRequest {
        request_id: "boundary-once".into(),
        expected_revision: snapshot(&f, &key).encounter.revision,
    };
    f.worker
        .advance_encounter_turn(&key, request.clone())
        .unwrap();
    assert_eq!(
        snapshot(&f, &key)
            .encounter
            .current_turn_participant_key
            .as_deref(),
        Some(b.as_str())
    );
    assert_eq!(amounts(&f, &key), [2]);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    let boundary = review.outgoing.clone().unwrap();
    assert_eq!(boundary.participant_key, a);
    let before = snapshot(&f, &key).encounter.revision;
    assert_eq!(
        f.worker.encounter_turn_review(&key).unwrap().outgoing,
        review.outgoing,
        "closing/reopening is a read"
    );
    assert_eq!(snapshot(&f, &key).encounter.revision, before);
    next(&f, &key); // wrap to round 2; original outgoing work remains round 1.
    let later = snapshot(&f, &key);
    f.worker
        .advance_encounter_turn(&key, request.clone())
        .unwrap();
    assert_eq!(
        snapshot(&f, &key),
        later,
        "retry after later turns cannot advance/tick again"
    );
    assert!(
        f.worker
            .advance_encounter_turn(
                &key,
                AdvanceEncounterTurnRequest {
                    request_id: "rapid-second".into(),
                    ..request
                }
            )
            .is_err()
    );
    let before_old_action = snapshot(&f, &key);
    assert!(
        f.worker
            .apply_condition_lifecycle(
                &key,
                ApplyConditionLifecycleRequest {
                    boundary_key: Some(boundary.boundary_key),
                    request_id: "old-end".into(),
                    expected_encounter_revision: before_old_action.encounter.revision,
                    participant_key: a,
                    action: ConditionLifecycleActionView::FrightenedEndTurn,
                }
            )
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), before_old_action);
    assert!(
        f.worker
            .encounter_turn_review(&key)
            .unwrap()
            .outgoing
            .is_none(),
        "a quiet new turn does not expose an older reminder"
    );
}

#[test]
fn completed_reminders_are_canonical_results_until_the_next_boundary() {
    let (f, key, a, _) = fixture();
    let ids = [
        add(&f, &a, None, rounds(1, Some(&a), LocalEvent::End)),
        add(&f, &a, None, rounds(0, Some(&a), LocalEvent::End)),
        add(&f, &a, None, rounds(3, None, LocalEvent::End)),
        add(
            &f,
            &a,
            None,
            Some(ConditionDuration {
                amount: 3,
                unit: LocalUnit::Minutes,
                timing: None,
            }),
        ),
    ];
    set(&f, &key, &a);
    next(&f, &key);
    let before = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(before.reminders.len(), 4);
    // A rejected stale removal cannot turn an instruction into a success row.
    assert!(
        f.worker
            .remove_encounter_participant_condition(
                &key,
                &a,
                ids[0],
                RemoveEncounterParticipantConditionRequest {
                    request_id: "stale-reminder-remove".into(),
                    expected_encounter_revision: 0,
                    intent: ConditionRemovalIntentView::RemoveEffect,
                }
            )
            .is_err()
    );
    assert_eq!(
        f.worker
            .encounter_turn_review(&key)
            .unwrap()
            .outgoing
            .unwrap(),
        before
    );
    for id in ids {
        f.worker
            .remove_encounter_participant_condition(
                &key,
                &a,
                id,
                RemoveEncounterParticipantConditionRequest {
                    request_id: format!("reminder-remove-{id}"),
                    expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                    intent: ConditionRemovalIntentView::RemoveEffect,
                },
            )
            .unwrap();
        let review = f
            .worker
            .encounter_turn_review(&key)
            .unwrap()
            .outgoing
            .unwrap();
        assert!(review.reminders.iter().any(|r| matches!(r, EncounterTurnReminderView::Resolved { condition_id, result, .. } if *condition_id == id && result == "Removed")));
    }
    let snapshot_before_read = snapshot(&f, &key);
    let retained = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(retained.boundary_key, before.boundary_key);
    assert_eq!(retained.reminders.len(), 4);
    assert!(
        retained
            .reminders
            .iter()
            .all(|r| matches!(r, EncounterTurnReminderView::Resolved { .. }))
    );
    assert_eq!(
        snapshot(&f, &key),
        snapshot_before_read,
        "reading completed rows has no gameplay effect"
    );
    next(&f, &key);
    assert!(
        f.worker
            .encounter_turn_review(&key)
            .unwrap()
            .outgoing
            .is_none()
    );
}

#[test]
fn recorded_frightened_recovery_remains_a_result_without_claiming_the_effect_ended() {
    let (f, key, a, _) = fixture();
    let frightened =
        super::conditions::modeled_condition_by_rule(super::conditions::ConditionRule::Frightened)
            .unwrap();
    add(&f, &a, Some(frightened.condition_ref), None);
    set(&f, &key, &a);
    next(&f, &key);
    let outgoing = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    f.worker
        .apply_condition_lifecycle(
            &key,
            ApplyConditionLifecycleRequest {
                boundary_key: Some(outgoing.boundary_key.clone()),
                request_id: "retained-frightened-recovery".into(),
                expected_encounter_revision: snapshot(&f, &key).encounter.revision,
                participant_key: a.clone(),
                action: ConditionLifecycleActionView::FrightenedEndTurn,
            },
        )
        .unwrap();
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0].value,
        Some(1)
    );
    let result = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(result.boundary_key, outgoing.boundary_key);
    assert!(
        matches!(&result.reminders[..], [EncounterTurnReminderView::Resolved { result, .. }] if result == "End-of-turn recovery recorded")
    );
}
