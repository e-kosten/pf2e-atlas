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
        remaining_seconds: None,
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
            remaining_seconds: None,
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
            remaining_rounds: None,
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
            i64::MAX,
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
        remaining_seconds: None,
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
        duration: condition
            .duration
            .clone()
            .map(|d| super::timing::duration_view(d, 6)),
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
            remaining_seconds: None,
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
            participant_variant: None,
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
                    remaining_rounds: None,
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
    assert_eq!(before.value, Some(20.into()));
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
                remaining_seconds: None,
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
    assert!(matches!(
        &outgoing.reminders[..],
        [EncounterTurnReminderView::ConditionRecovery {
            condition_value: Some(2),
            ..
        }]
    ));
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

#[test]
fn frightened_reminder_value_uses_known_effective_sources_without_guessing_unknown_values() {
    let (f, key, a, _) = fixture();
    let frightened =
        super::conditions::modeled_condition_by_rule(super::conditions::ConditionRule::Frightened)
            .unwrap();
    add(&f, &a, Some(frightened.condition_ref), None);
    add(&f, &a, Some(frightened.condition_ref), None);
    set(&f, &key, &a);
    let mut state = snapshot(&f, &key);
    state.participants[0].conditions[1].value = Some(4);
    let value = |state: &atlas_local_state::EncounterWithParticipants| {
        let review = super::timing::turn_review(state);
        match review
            .reminders
            .into_iter()
            .find(|r| matches!(r, EncounterTurnReminderView::ConditionRecovery { .. }))
            .unwrap()
        {
            EncounterTurnReminderView::ConditionRecovery {
                condition_value, ..
            } => condition_value,
            _ => unreachable!(),
        }
    };
    assert_eq!(value(&state), Some(4));
    state.participants[0].conditions[0].value = None;
    assert_eq!(value(&state), None);
}

fn tracker_request(
    f: &FixtureWorker,
    key: &str,
    participant: &str,
    id: &str,
    action: EncounterManualTrackerAction,
) -> UpdateEncounterManualTrackerRequest {
    let view = f.worker.encounter(key).unwrap();
    let p = view
        .participants
        .iter()
        .find(|p| p.participant_key == participant)
        .unwrap();
    UpdateEncounterManualTrackerRequest {
        request_id: id.into(),
        expected_encounter_revision: view.revision,
        expected_participant_revision: p.revision,
        participant_key: participant.into(),
        action,
    }
}
fn minute(anchor: &str) -> ConditionDurationView {
    ConditionDurationView {
        amount: 1,
        unit: ConditionDurationUnit::Minutes,
        timing: Some(ConditionTimingView {
            participant_key: anchor.into(),
            event: ConditionTurnEvent::End,
        }),
        remaining_rounds: None,
    }
}
#[test]
fn manual_trackers_preserve_progress_replay_and_independent_participants() {
    let (f, key, a, b) = fixture();
    let request = tracker_request(
        &f,
        &key,
        &a,
        "breath",
        EncounterManualTrackerAction::Add {
            name: "Breath ready".into(),
            duration: minute(&a),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, request.clone())
        .unwrap();
    f.worker
        .update_encounter_manual_tracker(&key, request)
        .unwrap();
    let saved = snapshot(&f, &key);
    assert_eq!(saved.participants[0].manual_trackers.len(), 1);
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        10
    );
    let tracker = saved.participants[0].manual_trackers[0].tracker_key.clone();
    set(&f, &key, &a);
    for _ in 0..3 {
        f.worker.encounter(&key).unwrap();
        f.worker.encounter_turn_review(&key).unwrap();
    }
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        10
    );
    let advance = AdvanceEncounterTurnRequest {
        request_id: "breath-tick".into(),
        expected_revision: snapshot(&f, &key).encounter.revision,
    };
    f.worker
        .advance_encounter_turn(&key, advance.clone())
        .unwrap();
    f.worker.advance_encounter_turn(&key, advance).unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        9
    );
    let rename = tracker_request(
        &f,
        &key,
        &a,
        "rename",
        EncounterManualTrackerAction::Edit {
            tracker_key: tracker.clone(),
            name: "Fire breath".into(),
            duration: minute(&a),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, rename)
        .unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        9
    );
    assert!(
        snapshot(&f, &key).participants[1]
            .manual_trackers
            .is_empty()
    );
    next(&f, &key); // B end does not tick A's end anchor.
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        9
    );
    for _ in 0..18 {
        next(&f, &key);
    }
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        0
    );
    let view = f.worker.encounter(&key).unwrap();
    assert_eq!(view.participants[0].manual_trackers[0].name, "Fire breath");
    let restart = tracker_request(
        &f,
        &key,
        &a,
        "restart",
        EncounterManualTrackerAction::Restart {
            tracker_key: tracker.clone(),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, restart)
        .unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        10
    );
    let foreign = tracker_request(
        &f,
        &key,
        &b,
        "foreign-tracker",
        EncounterManualTrackerAction::Remove {
            tracker_key: tracker.clone(),
        },
    );
    assert!(
        f.worker
            .update_encounter_manual_tracker(&key, foreign)
            .is_err()
    );
    let remove = tracker_request(
        &f,
        &key,
        &a,
        "remove-tracker",
        EncounterManualTrackerAction::Remove {
            tracker_key: tracker,
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, remove.clone())
        .unwrap();
    f.worker
        .update_encounter_manual_tracker(&key, remove)
        .unwrap();
    assert!(
        snapshot(&f, &key).participants[0]
            .manual_trackers
            .is_empty()
    );
}
#[test]
fn manual_tracker_admission_is_atomic_and_guards_revisions_and_timing() {
    let (f, key, a, _) = fixture();
    let original = snapshot(&f, &key);
    for (i, duration) in [
        ConditionDurationView {
            amount: -1,
            ..minute(&a)
        },
        ConditionDurationView {
            amount: i64::MAX,
            ..minute(&a)
        },
        ConditionDurationView {
            timing: None,
            ..minute(&a)
        },
        minute("foreign"),
    ]
    .into_iter()
    .enumerate()
    {
        let request = tracker_request(
            &f,
            &key,
            &a,
            &format!("invalid-{i}"),
            EncounterManualTrackerAction::Add {
                name: "Breath".into(),
                duration,
            },
        );
        assert!(
            f.worker
                .update_encounter_manual_tracker(&key, request)
                .is_err()
        );
        assert_eq!(snapshot(&f, &key), original);
    }
    let mut request = tracker_request(
        &f,
        &key,
        &a,
        "stale",
        EncounterManualTrackerAction::Add {
            name: "Breath".into(),
            duration: minute(&a),
        },
    );
    request.expected_participant_revision -= 1;
    assert!(
        f.worker
            .update_encounter_manual_tracker(&key, request)
            .is_err()
    );
    assert_eq!(snapshot(&f, &key), original);
}
#[test]
fn configured_nonround_durations_count_exact_ticks_and_preserve_entered_units() {
    let (f, key, a, _) = fixture();
    let duration =
        super::timing::duration_input(Some(minute(&a)), None, &snapshot(&f, &key)).unwrap();
    add(&f, &a, None, duration);
    set(&f, &key, &a);
    for _ in 0..18 {
        next(&f, &key);
    }
    let saved = snapshot(&f, &key);
    let duration = saved.participants[0].conditions[0]
        .duration
        .as_ref()
        .unwrap();
    assert_eq!(
        (duration.amount, duration.unit, duration.remaining_seconds),
        (1, LocalUnit::Minutes, Some(6))
    );
    assert!(matches!(
        &f.worker.encounter_turn_review(&key).unwrap().reminders[..],
        [EncounterTurnReminderView::DurationEnding { .. }]
    ));
    let input = ConditionDurationView {
        remaining_rounds: Some(999),
        ..minute(&a)
    };
    assert_eq!(
        super::timing::duration_input(Some(input), Some(duration), &saved)
            .unwrap()
            .as_ref(),
        Some(duration),
        "metadata edits cannot reset or forge saved progress"
    );
    next(&f, &key);
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .remaining_seconds,
        Some(0)
    );
    for _ in 0..4 {
        next(&f, &key);
    }
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions.len(),
        1,
        "expiry follows explicit-removal condition policy"
    );
    for (unit, expected) in [
        (LocalUnit::Rounds, 1),
        (LocalUnit::Minutes, 60),
        (LocalUnit::Hours, 3600),
        (LocalUnit::Days, 86400),
    ] {
        assert_eq!(super::timing::duration_seconds(1, unit).unwrap(), expected);
    }
}

#[test]
fn tracker_start_timing_and_deleted_anchors_remain_explicit() {
    let (f, key, a, b) = fixture();
    let duration = ConditionDurationView {
        timing: Some(ConditionTimingView {
            participant_key: b.clone(),
            event: ConditionTurnEvent::Start,
        }),
        amount: 2,
        unit: ConditionDurationUnit::Rounds,
        ..minute(&a)
    };
    let request = tracker_request(
        &f,
        &key,
        &a,
        "other-start",
        EncounterManualTrackerAction::Add {
            name: "Other turn".into(),
            duration,
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, request)
        .unwrap();
    set(&f, &key, &a);
    next(&f, &key);
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        1
    );
    f.worker.remove_encounter_participant(&key, &b).unwrap();
    next(&f, &key);
    let view = f.worker.encounter(&key).unwrap();
    let tracker = &view.participants[0].manual_trackers[0];
    assert_eq!(tracker.remaining_rounds, 1);
    assert!(!tracker.timing_available);
    assert_eq!(tracker.timing_label, "Timing participant unavailable");
}

#[test]
fn overnight_expires_short_and_exact_durations_and_preserves_long_progress_then_reset_clears_trackers()
 {
    let (f, key, a, _) = fixture();
    for (id, amount, unit) in [
        ("short", 1, ConditionDurationUnit::Minutes),
        ("exact", 8, ConditionDurationUnit::Hours),
        ("long", 1, ConditionDurationUnit::Days),
    ] {
        let duration = ConditionDurationView {
            amount,
            unit,
            ..minute(&a)
        };
        let request = tracker_request(
            &f,
            &key,
            &a,
            id,
            EncounterManualTrackerAction::Add {
                name: id.into(),
                duration: duration.clone(),
            },
        );
        f.worker
            .update_encounter_manual_tracker(&key, request)
            .unwrap();
        let condition_duration =
            super::timing::duration_input(Some(duration), None, &snapshot(&f, &key)).unwrap();
        add(&f, &a, None, condition_duration);
    }
    let action = EncounterRecoveryAction::RestOvernight {
        healing_mode: OvernightHealingMode::Natural,
        participants: vec![OvernightParticipantInput {
            participant_key: a.clone(),
            healing: Some(0),
        }],
    };
    let rest = ApplyEncounterRecoveryRequest {
        request_id: "countdown-rest".into(),
        expected_encounter_revision: snapshot(&f, &key).encounter.revision,
        action,
    };
    f.worker
        .apply_encounter_recovery(&key, rest.clone())
        .unwrap();
    f.worker.apply_encounter_recovery(&key, rest).unwrap();
    let saved = snapshot(&f, &key);
    let owner = &saved.participants[0];
    for (name, remaining) in [("short", 0), ("exact", 0), ("long", 9600)] {
        assert_eq!(
            super::timing::tracker_rounds(
                &owner
                    .manual_trackers
                    .iter()
                    .find(|t| t.name == name)
                    .unwrap()
                    .progress,
                6
            ),
            remaining
        );
    }
    assert_eq!(owner.conditions.len(), 1);
    let duration = owner.conditions[0].duration.as_ref().unwrap();
    assert_eq!(
        (duration.amount, duration.unit, duration.remaining_seconds),
        (1, LocalUnit::Days, Some(57600))
    );
    let action = EncounterRecoveryAction::ResetParticipant { participant_key: a };
    let preview = f
        .worker
        .preview_encounter_recovery(
            &key,
            PreviewEncounterRecoveryRequest {
                expected_encounter_revision: saved.encounter.revision,
                action: action.clone(),
            },
        )
        .unwrap();
    assert!(
        preview.participants[0]
            .changes
            .iter()
            .any(|c| c == "Restore baseline trackers")
    );
    f.worker
        .apply_encounter_recovery(
            &key,
            ApplyEncounterRecoveryRequest {
                request_id: "countdown-reset".into(),
                expected_encounter_revision: saved.encounter.revision,
                action,
            },
        )
        .unwrap();
    let reset = snapshot(&f, &key);
    assert!(reset.participants[0].manual_trackers.is_empty());
    assert!(reset.participants[0].conditions.is_empty());
}

#[test]
fn legacy_outgoing_and_baseline_payloads_default_missing_tracker_fields() {
    let (f, key, a, _) = fixture();
    add(&f, &a, None, rounds(2, Some(&a), LocalEvent::End));
    let snapshot = snapshot(&f, &key);
    let outgoing = atlas_local_state::SavedOutgoingTurn {
        boundary_key: "old-boundary".into(),
        snapshot: snapshot.clone(),
        next_participant_key: Some(a),
    };
    let mut json = serde_json::to_value(&outgoing).unwrap();
    json["snapshot"]["encounter"]
        .as_object_mut()
        .unwrap()
        .remove("round_duration_seconds");
    for participant in json["snapshot"]["participants"].as_array_mut().unwrap() {
        participant
            .as_object_mut()
            .unwrap()
            .remove("manual_trackers");
        for condition in participant["conditions"].as_array_mut().unwrap() {
            if let Some(duration) = condition
                .get_mut("duration")
                .and_then(serde_json::Value::as_object_mut)
            {
                duration.remove("remaining_rounds");
            }
        }
    }
    let decoded: atlas_local_state::SavedOutgoingTurn = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, outgoing);
    let baseline = atlas_local_state::ParticipantGameplay::from(&snapshot.participants[0]);
    let mut json = serde_json::to_value(&baseline).unwrap();
    json.as_object_mut().unwrap().remove("manual_trackers");
    let decoded: atlas_local_state::ParticipantGameplay = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, baseline);
}

fn set_round_seconds(
    f: &FixtureWorker,
    key: &str,
    seconds: Option<i64>,
) -> crate::AppServiceResult<EncounterMutationView> {
    let view = f.worker.encounter(key).unwrap();
    f.worker.update_encounter(UpdateEncounterRequest {
        encounter_key: view.encounter.encounter_key,
        slug: view.encounter.slug,
        name: view.encounter.name,
        description: view.encounter.description,
        note: view.note,
        status: view.encounter.status,
        round_duration_seconds: seconds,
    })
}
#[test]
fn custom_round_lengths_preserve_elapsed_seconds_and_turn_count_trackers() {
    let (f, key, a, _) = fixture();
    for (name, duration) in [
        ("Elapsed", minute(&a)),
        (
            "Turns",
            ConditionDurationView {
                amount: 3,
                unit: ConditionDurationUnit::Rounds,
                ..minute(&a)
            },
        ),
    ] {
        let request = tracker_request(
            &f,
            &key,
            &a,
            name,
            EncounterManualTrackerAction::Add {
                name: name.into(),
                duration,
            },
        );
        f.worker
            .update_encounter_manual_tracker(&key, request)
            .unwrap();
    }
    let duration =
        super::timing::duration_input(Some(minute(&a)), None, &snapshot(&f, &key)).unwrap();
    add(&f, &a, None, duration);
    set(&f, &key, &a);
    next(&f, &key);
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .remaining_seconds,
        Some(54)
    );
    set_round_seconds(&f, &key, Some(30)).unwrap();
    let view = f.worker.encounter(&key).unwrap();
    assert_eq!(view.encounter.round_duration_seconds, 30);
    assert!(
        view.participants[0]
            .manual_trackers
            .iter()
            .all(|t| t.remaining_rounds == 2)
    );
    assert_eq!(
        view.participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .remaining_rounds,
        Some(2)
    );
    set_round_seconds(&f, &key, None).unwrap();
    assert_eq!(
        f.worker
            .encounter(&key)
            .unwrap()
            .encounter
            .round_duration_seconds,
        30,
        "unrelated metadata writes preserve timing"
    );
    next(&f, &key);
    next(&f, &key);
    let saved = snapshot(&f, &key);
    assert_eq!(
        saved.participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .remaining_seconds,
        Some(24)
    );
    set_round_seconds(&f, &key, Some(7)).unwrap();
    let view = f.worker.encounter(&key).unwrap();
    assert_eq!(
        view.participants[0]
            .manual_trackers
            .iter()
            .find(|t| t.name == "Elapsed")
            .unwrap()
            .remaining_rounds,
        4
    );
    assert_eq!(
        view.participants[0]
            .manual_trackers
            .iter()
            .find(|t| t.name == "Turns")
            .unwrap()
            .remaining_rounds,
        1
    );
    for _ in 0..8 {
        next(&f, &key);
    }
    let view = f.worker.encounter(&key).unwrap();
    assert!(
        view.participants[0]
            .manual_trackers
            .iter()
            .all(|t| t.remaining_rounds == 0)
    );
    assert_eq!(
        snapshot(&f, &key).participants[0].conditions[0]
            .duration
            .as_ref()
            .unwrap()
            .remaining_seconds,
        Some(0)
    );
    let original = snapshot(&f, &key);
    for seconds in [0, -1, 86401] {
        assert!(set_round_seconds(&f, &key, Some(seconds)).is_err());
        assert_eq!(snapshot(&f, &key), original);
    }
}

#[test]
fn migrated_large_counters_remain_exact_when_changing_round_length_or_restarting() {
    let (f, key, a, _) = fixture();
    let request = tracker_request(
        &f,
        &key,
        &a,
        "large",
        EncounterManualTrackerAction::Add {
            name: "Large".into(),
            duration: minute(&a),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, request)
        .unwrap();
    let saved = snapshot(&f, &key);
    let mut tracker = saved.participants[0].manual_trackers[0].clone();
    tracker.duration.amount = 900_719_925_474_099;
    tracker.progress = atlas_local_state::TrackerProgress::Seconds(9_007_199_254_740_990 * 6);
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().apply_condition_event(
                &key,
                "legacy-large",
                &serde_json::json!({"legacy":true}),
                saved.encounter.revision,
                atlas_local_state::PreparedConditionEvent {
                    manual_trackers: vec![atlas_local_state::ManualTrackerChange {
                        participant_key: a.clone(),
                        tracker_key: tracker.tracker_key.clone(),
                        value: Some(tracker),
                    }],
                    ..Default::default()
                },
            )?)
        })
        .unwrap();
    let original = snapshot(&f, &key);
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        9_007_199_254_740_990
    );
    assert!(set_round_seconds(&f, &key, Some(1)).is_err());
    assert_eq!(snapshot(&f, &key), original);
    set_round_seconds(&f, &key, Some(12)).unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0].remaining_rounds,
        4_503_599_627_370_495
    );
}

#[test]
fn renaming_a_spent_large_legacy_tracker_does_not_readmit_or_restart_its_template() {
    let (f, key, a, _) = fixture();
    let request = tracker_request(
        &f,
        &key,
        &a,
        "large-spent",
        EncounterManualTrackerAction::Add {
            name: "Spent".into(),
            duration: minute(&a),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, request)
        .unwrap();
    let saved = snapshot(&f, &key);
    let mut tracker = saved.participants[0].manual_trackers[0].clone();
    tracker.duration.amount = 900_719_925_474_099;
    tracker.progress = atlas_local_state::TrackerProgress::Seconds(0);
    let tracker_key = tracker.tracker_key.clone();
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().apply_condition_event(
                &key,
                "legacy-spent",
                &serde_json::json!({"legacy":true}),
                saved.encounter.revision,
                atlas_local_state::PreparedConditionEvent {
                    manual_trackers: vec![atlas_local_state::ManualTrackerChange {
                        participant_key: a.clone(),
                        tracker_key: tracker.tracker_key.clone(),
                        value: Some(tracker),
                    }],
                    ..Default::default()
                },
            )?)
        })
        .unwrap();
    set_round_seconds(&f, &key, Some(1)).unwrap();
    let duration = f.worker.encounter(&key).unwrap().participants[0].manual_trackers[0]
        .duration
        .clone();
    let request = tracker_request(
        &f,
        &key,
        &a,
        "rename-spent",
        EncounterManualTrackerAction::Edit {
            tracker_key: tracker_key.clone(),
            name: "Renamed".into(),
            duration,
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, request)
        .unwrap();
    let view = f.worker.encounter(&key).unwrap();
    assert_eq!(view.participants[0].manual_trackers[0].remaining_rounds, 0);
    assert_eq!(view.participants[0].manual_trackers[0].name, "Renamed");
    let restart = tracker_request(
        &f,
        &key,
        &a,
        "restart-spent",
        EncounterManualTrackerAction::Restart { tracker_key },
    );
    assert!(
        f.worker
            .update_encounter_manual_tracker(&key, restart)
            .is_err(),
        "restarting still requires an exactly representable projected count"
    );
}

fn add_transition_tracker(
    f: &FixtureWorker,
    key: &str,
    owner: &str,
    name: &str,
    duration: ConditionDurationView,
) -> String {
    let request = tracker_request(
        f,
        key,
        owner,
        name,
        EncounterManualTrackerAction::Add {
            name: name.into(),
            duration,
        },
    );
    f.worker
        .update_encounter_manual_tracker(key, request)
        .unwrap();
    snapshot(f, key)
        .participants
        .iter()
        .find(|p| p.participant_key == owner)
        .unwrap()
        .manual_trackers
        .iter()
        .find(|t| t.name == name)
        .unwrap()
        .tracker_key
        .clone()
}
fn elapsed_rows(reminders: &[EncounterTurnReminderView]) -> Vec<(&str, &str, Option<&str>)> {
    reminders
        .iter()
        .filter_map(|reminder| match reminder {
            EncounterTurnReminderView::TrackerElapsed {
                participant_key,
                tracker_name,
                result,
                ..
            } => Some((
                participant_key.as_str(),
                tracker_name.as_str(),
                result.as_deref(),
            )),
            _ => None,
        })
        .collect()
}
#[test]
fn elapsed_trackers_use_actual_end_and_foreign_start_transitions_without_repeating() {
    let (f, key, a, b) = fixture();
    for (owner, name, anchor, event, amount) in [
        (&a, "own-end", &a, ConditionTurnEvent::End, 1),
        (&a, "foreign-start", &b, ConditionTurnEvent::Start, 1),
        (&b, "foreign-end", &a, ConditionTurnEvent::End, 1),
        (&a, "already-zero", &a, ConditionTurnEvent::End, 0),
    ] {
        add_transition_tracker(
            &f,
            &key,
            owner,
            name,
            ConditionDurationView {
                amount,
                unit: ConditionDurationUnit::Rounds,
                remaining_rounds: None,
                timing: Some(ConditionTimingView {
                    participant_key: anchor.clone(),
                    event,
                }),
            },
        );
    }
    set(&f, &key, &a);
    let unchanged = snapshot(&f, &key);
    for _ in 0..2 {
        let review = f.worker.encounter_turn_review(&key).unwrap();
        assert_eq!(
            std::collections::BTreeSet::from_iter(elapsed_rows(&review.reminders)),
            std::collections::BTreeSet::from([
                (a.as_str(), "own-end", None),
                (a.as_str(), "foreign-start", None),
                (b.as_str(), "foreign-end", None),
            ])
        );
    }
    assert_eq!(
        snapshot(&f, &key),
        unchanged,
        "forecast reads do not tick trackers"
    );
    let request = AdvanceEncounterTurnRequest {
        request_id: "elapsed-boundary".into(),
        expected_revision: unchanged.encounter.revision,
    };
    f.worker
        .advance_encounter_turn(&key, request.clone())
        .unwrap();
    let outgoing = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(outgoing.participant_key, a);
    assert_eq!(elapsed_rows(&outgoing.reminders).len(), 3);
    assert!(elapsed_rows(&f.worker.encounter_turn_review(&key).unwrap().reminders).is_empty());
    let after = snapshot(&f, &key);
    f.worker.advance_encounter_turn(&key, request).unwrap();
    assert_eq!(
        snapshot(&f, &key),
        after,
        "replay cannot tick or create another boundary"
    );
    assert_eq!(
        f.worker.encounter_turn_review(&key).unwrap().outgoing,
        Some(outgoing)
    );
    for _ in 0..4 {
        next(&f, &key);
        assert!(
            f.worker
                .encounter_turn_review(&key)
                .unwrap()
                .outgoing
                .is_none(),
            "existing zero trackers do not keep notifying"
        );
    }
}
#[test]
fn elapsed_tracker_initial_start_retains_a_review_without_spurious_outgoing_damage() {
    let (f, key, a, b) = fixture();
    add_transition_tracker(
        &f,
        &key,
        &b,
        "first-start",
        ConditionDurationView {
            amount: 1,
            unit: ConditionDurationUnit::Rounds,
            remaining_rounds: None,
            timing: Some(ConditionTimingView {
                participant_key: a.clone(),
                event: ConditionTurnEvent::Start,
            }),
        },
    );
    let persistent = super::conditions::modeled_condition_by_rule(
        super::conditions::ConditionRule::PersistentDamage,
    )
    .unwrap();
    f.worker
        .with_write(|session| {
            Ok(session.local_state().encounters().add_condition(
                AddEncounterParticipantCondition {
                    participant_key: a.clone(),
                    condition_key: Some(persistent.condition_ref.into()),
                    name: "Persistent fire".into(),
                    value: None,
                    source_participant_key: None,
                    duration: None,
                    note: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    origin: None,
                    parameters: atlas_local_state::ConditionParameters::PersistentDamage {
                        formula: "1d6".into(),
                        damage_type: "fire".into(),
                        multiplier: atlas_local_state::DamageMultiplier::Normal,
                        recovery: atlas_local_state::PersistentRecovery::Standard,
                        episode_id: "initial-fire".into(),
                    },
                },
            )?)
        })
        .unwrap();
    assert_eq!(
        f.worker.encounter(&key).unwrap().participants[0]
            .persistent_damage_groups
            .len(),
        1
    );
    assert!(
        snapshot(&f, &key)
            .encounter
            .current_turn_participant_key
            .is_none()
    );
    let before_start = snapshot(&f, &key);
    f.worker
        .with_write(|session| {
            Ok(session
                .local_state()
                .encounters()
                .apply_claimed_condition_event(
                    &key,
                    "pre-start-damage-claim",
                    &serde_json::json!({"pre_start":true}),
                    before_start.encounter.revision,
                    atlas_local_state::ConditionEventBoundary {
                        claims: &[atlas_local_state::EncounterBoundaryClaim {
                            participant_key: a.clone(),
                            round_number: before_start.encounter.round_number,
                            kind: atlas_local_state::EncounterBoundaryKind::PersistentDamage,
                            scope_key: String::new(),
                        }],
                        outgoing_boundary_key: None,
                    },
                    Default::default(),
                )?)
        })
        .unwrap();
    next(&f, &key);
    let review = f.worker.encounter_turn_review(&key).unwrap();
    let outgoing = review.outgoing.unwrap();
    assert_eq!(outgoing.participant_key, a);
    assert_eq!(
        elapsed_rows(&outgoing.reminders),
        [(b.as_str(), "first-start", None)]
    );
    assert!(outgoing.persistent_damage_groups.is_empty());
    assert!(!outgoing.damage_applied);
    assert!(
        outgoing
            .reminders
            .iter()
            .all(|r| !matches!(r, EncounterTurnReminderView::PersistentDamage { .. }))
    );
    assert_eq!(snapshot(&f, &key).participants[0].current_hp, Some(20));
    next(&f, &key);
    let outgoing = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(
        outgoing.persistent_damage_groups.len(),
        1,
        "real end-of-turn damage remains available"
    );
    assert!(elapsed_rows(&outgoing.reminders).is_empty());
}
#[test]
fn elapsed_tracker_current_status_distinguishes_restart_removal_and_removed_owner() {
    let (f, key, a, _) = fixture();
    set_round_seconds(&f, &key, Some(7)).unwrap();
    let tracker = add_transition_tracker(&f, &key, &a, "minute-complete", minute(&a));
    set(&f, &key, &a);
    for _ in 0..16 {
        next(&f, &key);
    }
    let saved = snapshot(&f, &key);
    assert_eq!(
        saved.participants[0].manual_trackers[0].progress,
        atlas_local_state::TrackerProgress::Seconds(4)
    );
    assert_eq!(
        elapsed_rows(&f.worker.encounter_turn_review(&key).unwrap().reminders),
        [(a.as_str(), "minute-complete", None)]
    );
    next(&f, &key);
    let outgoing = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    let boundary = outgoing.boundary_key.clone();
    assert_eq!(
        elapsed_rows(&outgoing.reminders),
        [(a.as_str(), "minute-complete", None)]
    );
    let restart = tracker_request(
        &f,
        &key,
        &a,
        "elapsed-restart",
        EncounterManualTrackerAction::Restart {
            tracker_key: tracker.clone(),
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, restart)
        .unwrap();
    let review = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(review.boundary_key, boundary);
    assert_eq!(
        elapsed_rows(&review.reminders),
        [(a.as_str(), "minute-complete", Some("Restarted"))]
    );
    let remove = tracker_request(
        &f,
        &key,
        &a,
        "elapsed-remove",
        EncounterManualTrackerAction::Remove {
            tracker_key: tracker,
        },
    );
    f.worker
        .update_encounter_manual_tracker(&key, remove.clone())
        .unwrap();
    f.worker
        .update_encounter_manual_tracker(&key, remove)
        .unwrap();
    let review = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(review.boundary_key, boundary);
    assert_eq!(
        elapsed_rows(&review.reminders),
        [(a.as_str(), "minute-complete", Some("Removed"))]
    );
    f.worker.remove_encounter_participant(&key, &a).unwrap();
    let review = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(
        elapsed_rows(&review.reminders),
        [(a.as_str(), "minute-complete", Some("Participant removed"))]
    );
}
#[test]
fn elapsed_tracker_single_actor_start_and_end_are_independent_and_legacy_snapshots_have_no_trackers()
 {
    let (f, key, a, b) = fixture();
    f.worker.remove_encounter_participant(&key, &b).unwrap();
    for (name, event) in [
        ("single-start", ConditionTurnEvent::Start),
        ("single-end", ConditionTurnEvent::End),
    ] {
        add_transition_tracker(
            &f,
            &key,
            &a,
            name,
            ConditionDurationView {
                amount: 1,
                unit: ConditionDurationUnit::Rounds,
                remaining_rounds: None,
                timing: Some(ConditionTimingView {
                    participant_key: a.clone(),
                    event,
                }),
            },
        );
    }
    let legacy = atlas_local_state::SavedOutgoingTurn {
        boundary_key: "legacy".into(),
        snapshot: snapshot(&f, &key),
        next_participant_key: Some(a.clone()),
    };
    let mut json = serde_json::to_value(legacy).unwrap();
    for p in json["snapshot"]["participants"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("manual_trackers");
    }
    let decoded: atlas_local_state::SavedOutgoingTurn = serde_json::from_value(json).unwrap();
    assert!(elapsed_rows(&super::timing::turn_review(&decoded.snapshot).reminders).is_empty());
    next(&f, &key);
    let review = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(
        elapsed_rows(&review.reminders),
        [(a.as_str(), "single-start", None)]
    );
    next(&f, &key);
    let review = f
        .worker
        .encounter_turn_review(&key)
        .unwrap()
        .outgoing
        .unwrap();
    assert_eq!(
        elapsed_rows(&review.reminders),
        [(a.as_str(), "single-end", None)]
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
