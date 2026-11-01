use super::*;

fn advance(
    store: &TestStore,
    key: &str,
    id: &str,
    review: bool,
) -> crate::LocalStateResult<Encounter> {
    let current = store.encounters().get("test")?.unwrap();
    store
        .encounters()
        .advance_turn_once(
            "test",
            id,
            &serde_json::json!({"next":id}),
            current.revision,
            |_| {
                Ok((
                    EncounterTurnUpdate {
                        turn: EncounterTurnState {
                            participant_key: Some(key.into()),
                            round_number: current.round_number + 1,
                            mark_running: true,
                        },
                        manual_trackers: vec![],
                        durations: vec![],
                        start_event: None,
                        start_claims: vec![],
                    },
                    review,
                ))
            },
        )
        .map(|value| value.unwrap())
}

#[test]
fn lifetime_receipts_survive_later_turns_reset_and_rest_with_current_canonical_recovery()
-> crate::LocalStateResult<()> {
    for reset_encounter in [false, true] {
        let store = fixture(if reset_encounter {
            "turn-reset-retry"
        } else {
            "turn-rest-retry"
        })?;
        let participant = store
            .encounters()
            .add_participant("test", creature("Actor", Some(10), 20))?;
        store
            .encounters()
            .set_current_turn("test", Some(&participant.participant_key))?;
        let first = advance(&store, &participant.participant_key, "first", true)?;
        advance(&store, &participant.participant_key, "second", false)?;
        let mut connection = store.connection()?;
        let transaction = connection.transaction()?;
        let domain = EncountersWrite::from_connection(&transaction);
        let current = domain.get("test")?.unwrap();
        let baseline = domain
            .participant_baseline(&participant.participant_key)?
            .unwrap();
        domain.apply_recovery(ApplyEncounterRecovery {
            encounter_key: &current.encounter_key,
            request_id: "reset-or-rest",
            intent: &serde_json::json!({"reset":reset_encounter}),
            expected_revision: current.revision,
            reset_encounter,
            end_turn: true,
            replacements: &[(participant.participant_key.clone(), baseline)],
        })?;
        let saved = domain.get("test")?.unwrap();
        assert!(saved.revision > first.revision);
        assert!(domain.latest_outgoing_turn("test")?.is_none());
        assert_eq!(
            domain.advance_turn_receipt("test", "first", &serde_json::json!({"next":"first"}))?,
            Some(saved.clone())
        );
        let recovered = domain.advance_turn_once(
            "test",
            "first",
            &serde_json::json!({"next":"first"}),
            -1,
            |_| panic!("a lifetime retry must not prepare another turn"),
        )?;
        assert_eq!(recovered, Some(saved));
        assert!(
            domain
                .advance_turn_receipt("test", "first", &serde_json::json!({"different":true}))
                .is_err()
        );
        assert_eq!(
            transaction.query_row("SELECT count(*) FROM encounter_turn_receipts", [], |row| {
                row.get::<_, i64>(0)
            })?,
            2
        );
        assert_eq!(
            transaction.query_row("SELECT count(*) FROM encounter_outgoing_turns", [], |row| {
                row.get::<_, i64>(0)
            })?,
            0
        );
        transaction.commit()?;
    }
    Ok(())
}

#[test]
fn only_current_boundary_is_actionable_even_when_it_has_no_review() -> crate::LocalStateResult<()> {
    let store = fixture("turn-current-only")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("Actor", Some(10), 20))?;
    store
        .encounters()
        .set_current_turn("test", Some(&participant.participant_key))?;
    advance(&store, &participant.participant_key, "first", true)?;
    let connection = store.connection()?;
    let read = EncountersRead::from_connection(&connection);
    let first = read.latest_outgoing_turn("test")?.unwrap();
    advance(&store, &participant.participant_key, "second", false)?;
    assert!(read.outgoing_turn("test", &first.boundary_key)?.is_none());
    assert!(read.latest_outgoing_turn("test")?.is_none());
    let latest: String = connection.query_row(
        "SELECT boundary_key FROM encounter_turn_receipts WHERE request_id='second'",
        [],
        |row| row.get(0),
    )?;
    let current = read.outgoing_turn("test", &latest)?.unwrap();
    assert_eq!(current.boundary_key, latest);
    assert_eq!(
        connection.query_row("SELECT count(*) FROM encounter_outgoing_turns", [], |row| {
            row.get::<_, i64>(0)
        })?,
        1
    );
    store.encounters().remove_participant_and_select_turn(
        "test",
        &participant.participant_key,
        |_| None,
    )?;
    assert!(
        read.outgoing_turn("test", &latest)?.is_some(),
        "participant removal retains frozen reminder context"
    );
    Ok(())
}

#[test]
fn context_failure_rolls_back_new_receipt_and_preserves_previous_boundary()
-> crate::LocalStateResult<()> {
    let store = fixture("turn-context-rollback")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("Actor", Some(10), 20))?;
    store
        .encounters()
        .set_current_turn("test", Some(&participant.participant_key))?;
    advance(&store, &participant.participant_key, "first", true)?;
    let connection = store.connection()?;
    let before = EncountersRead::from_connection(&connection).latest_outgoing_turn("test")?;
    let state = store.encounters().get_with_participants("test")?.unwrap();
    connection.execute_batch("CREATE TRIGGER reject_current_context BEFORE UPDATE ON encounter_outgoing_turns BEGIN SELECT RAISE(ABORT,'context failure'); END;")?;
    assert!(advance(&store, &participant.participant_key, "second", true).is_err());
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        state
    );
    assert_eq!(
        EncountersRead::from_connection(&connection).latest_outgoing_turn("test")?,
        before
    );
    assert_eq!(
        connection.query_row("SELECT count(*) FROM encounter_turn_receipts", [], |row| {
            row.get::<_, i64>(0)
        })?,
        1
    );
    Ok(())
}

#[test]
fn encounter_deletion_removes_lifetime_receipts_and_current_context() -> crate::LocalStateResult<()>
{
    let store = fixture("turn-delete")?;
    let participant = store
        .encounters()
        .add_participant("test", creature("Actor", Some(10), 20))?;
    advance(&store, &participant.participant_key, "first", true)?;
    assert!(store.encounters().delete("test")?);
    let connection = store.connection()?;
    for table in ["encounter_turn_receipts", "encounter_outgoing_turns"] {
        assert_eq!(
            connection.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row
                .get::<_, i64>(0))?,
            0
        );
    }
    Ok(())
}

#[test]
fn tracker_turn_changes_are_atomic_and_lifetime_retries_after_reset_or_rest_do_not_tick()
-> crate::LocalStateResult<()> {
    for reset_encounter in [false, true] {
        let store = fixture(if reset_encounter {
            "tracker-reset-retry"
        } else {
            "tracker-rest-retry"
        })?;
        let actor = store
            .encounters()
            .add_participant("test", creature("Actor", Some(10), 20))?;
        let mut connection = store.connection()?;
        let transaction = connection.transaction()?;
        let domain = EncountersWrite::from_connection(&transaction);
        let tracker = ManualTracker {
            tracker_key: "timer".into(),
            name: "Timer".into(),
            duration: ConditionDuration {
                amount: 1,
                unit: ConditionDurationUnit::Rounds,
                timing: Some(ConditionTiming {
                    participant_key: actor.participant_key.clone(),
                    event: ConditionTurnEvent::Start,
                }),
                remaining_seconds: None,
            },
            progress: TrackerProgress::Rounds(1),
        };
        domain.save_manual_tracker(
            &domain.get("test")?.unwrap().encounter_key,
            &ManualTrackerChange {
                participant_key: actor.participant_key.clone(),
                tracker_key: "timer".into(),
                value: Some(tracker.clone()),
            },
        )?;
        super::super::recovery::capture_baseline(&transaction, &actor.participant_key)?;
        let current = domain.get("test")?.unwrap();
        let mut elapsed = tracker.clone();
        elapsed.progress = TrackerProgress::Rounds(0);
        let intent = serde_json::json!({"next":"first"});
        domain.advance_turn_once("test", "first", &intent, current.revision, |_| {
            Ok((
                EncounterTurnUpdate {
                    turn: EncounterTurnState {
                        participant_key: Some(actor.participant_key.clone()),
                        round_number: 1,
                        mark_running: true,
                    },
                    durations: vec![],
                    start_event: None,
                    start_claims: vec![],
                    manual_trackers: vec![ManualTrackerChange {
                        participant_key: actor.participant_key.clone(),
                        tracker_key: "timer".into(),
                        value: Some(elapsed.clone()),
                    }],
                },
                true,
            ))
        })?;
        assert!(
            domain.latest_outgoing_turn("test")?.is_some(),
            "first-start completion must retain its incoming reminder"
        );
        let after = domain.get_with_participants("test")?.unwrap();
        assert_eq!(
            after.participants[0].manual_trackers[0].progress,
            TrackerProgress::Rounds(0)
        );
        assert!(
            domain
                .advance_turn_once(
                    "test",
                    "bad",
                    &serde_json::json!({"bad":true}),
                    after.encounter.revision,
                    |_| Ok((
                        EncounterTurnUpdate {
                            turn: EncounterTurnState {
                                participant_key: Some(actor.participant_key.clone()),
                                round_number: 2,
                                mark_running: true
                            },
                            durations: vec![],
                            start_event: None,
                            start_claims: vec![],
                            manual_trackers: vec![
                                ManualTrackerChange {
                                    participant_key: actor.participant_key.clone(),
                                    tracker_key: "timer".into(),
                                    value: Some(tracker.clone()),
                                },
                                ManualTrackerChange {
                                    participant_key: "missing".into(),
                                    tracker_key: "timer".into(),
                                    value: Some(tracker.clone()),
                                },
                            ],
                        },
                        true
                    ))
                )
                .is_err()
        );
        assert_eq!(domain.get_with_participants("test")?.unwrap(), after);
        let mut replacement = domain
            .participant_baseline(&actor.participant_key)?
            .unwrap();
        if !reset_encounter {
            replacement.manual_trackers[0].progress = TrackerProgress::Rounds(0);
        }
        domain.apply_recovery(ApplyEncounterRecovery {
            encounter_key: &after.encounter.encounter_key,
            request_id: "recovery",
            intent: &serde_json::json!({"reset":reset_encounter}),
            expected_revision: after.encounter.revision,
            reset_encounter,
            end_turn: true,
            replacements: &[(actor.participant_key.clone(), replacement.clone())],
        })?;
        let recovered = domain.get_with_participants("test")?.unwrap();
        assert_eq!(
            recovered.participants[0].manual_trackers,
            replacement.manual_trackers
        );
        assert!(domain.latest_outgoing_turn("test")?.is_none());
        assert_eq!(
            domain.advance_turn_once("test", "first", &intent, -1, |_| panic!(
                "retry must recover before preparing or ticking timers"
            ))?,
            Some(recovered.encounter.clone())
        );
        assert_eq!(domain.get_with_participants("test")?.unwrap(), recovered);
        assert_eq!(
            transaction.query_row("SELECT count(*) FROM encounter_turn_receipts", [], |r| {
                r.get::<_, usize>(0)
            })?,
            1
        );
        transaction.commit()?;
    }
    Ok(())
}
