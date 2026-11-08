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
    let read = EncountersRead::from_connection(&connection, 1);
    let first = read.latest_outgoing_turn("test")?.unwrap();
    advance(&store, &participant.participant_key, "second", false)?;
    assert!(read.outgoing_turn("test", &first.boundary_key)?.is_none());
    assert!(read.latest_outgoing_turn("test")?.is_none());
    let latest: String = connection.query_row(
        "SELECT boundary_key FROM encounter_command_receipts WHERE family='turn' AND request_id='second'",
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
    let before = EncountersRead::from_connection(&connection, 1).latest_outgoing_turn("test")?;
    let state = store.encounters().get_with_participants("test")?.unwrap();
    connection.execute_batch("CREATE TRIGGER reject_current_context BEFORE UPDATE ON encounter_outgoing_turns BEGIN SELECT RAISE(ABORT,'context failure'); END;")?;
    assert!(advance(&store, &participant.participant_key, "second", true).is_err());
    assert_eq!(
        store.encounters().get_with_participants("test")?.unwrap(),
        state
    );
    assert_eq!(
        EncountersRead::from_connection(&connection, 1).latest_outgoing_turn("test")?,
        before
    );
    assert_eq!(
        connection.query_row(
            "SELECT count(*) FROM encounter_command_receipts WHERE family='turn'",
            [],
            |row| { row.get::<_, i64>(0) }
        )?,
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
    for table in ["encounter_command_receipts", "encounter_outgoing_turns"] {
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
        let domain = EncountersWrite::from_connection(&transaction, 1);
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
        super::super::recovery::capture_baseline(&transaction, 1, &actor.participant_key)?;
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
        domain.advance_turn_once(
            "test",
            "second",
            &serde_json::json!({"next":"second"}),
            after.encounter.revision,
            |_| {
                Ok((
                    EncounterTurnUpdate {
                        turn: EncounterTurnState {
                            participant_key: Some(actor.participant_key.clone()),
                            round_number: 2,
                            mark_running: true,
                        },
                        durations: vec![],
                        start_event: None,
                        start_claims: vec![],
                        manual_trackers: vec![],
                    },
                    false,
                ))
            },
        )?;
        let current = domain.get("test")?.unwrap();
        let mut replacement = domain
            .participant_baseline(&actor.participant_key)?
            .unwrap();
        if !reset_encounter {
            replacement.manual_trackers[0].progress = TrackerProgress::Rounds(0);
        }
        domain.apply_recovery(ApplyEncounterRecovery {
            encounter_key: &after.encounter.encounter_key,
            expected_revision: current.revision,
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
        assert!(recovered.encounter.revision > after.encounter.revision);
        assert_eq!(
            domain.advance_turn_receipt("test", "first", &intent)?,
            Some(recovered.encounter.clone())
        );
        assert!(
            domain
                .advance_turn_receipt("test", "first", &serde_json::json!({"different":true}))
                .is_err()
        );
        assert_eq!(
            domain.advance_turn_once("test", "first", &intent, -1, |_| panic!(
                "retry must recover before preparing or ticking timers"
            ))?,
            Some(recovered.encounter.clone())
        );
        assert_eq!(domain.get_with_participants("test")?.unwrap(), recovered);
        assert_eq!(
            transaction.query_row(
                "SELECT count(*) FROM encounter_command_receipts WHERE family='turn'",
                [],
                |r| { r.get::<_, usize>(0) }
            )?,
            2
        );
        assert_eq!(
            transaction.query_row("SELECT count(*) FROM encounter_outgoing_turns", [], |r| {
                r.get::<_, usize>(0)
            })?,
            0
        );
        transaction.commit()?;
    }
    Ok(())
}

#[test]
fn receipt_recovery_scopes_shared_slugs_and_request_ids_to_the_owner() -> crate::LocalStateResult<()>
{
    let store = fixture("receipt-owner-scope")?;
    let mut connection = store.connection()?;
    connection.execute_batch(
        "UPDATE principals SET principal_key='other' WHERE id=1;
         INSERT INTO principals VALUES(7, 'local', 'Local owner');",
    )?;
    let transaction = connection.transaction()?;
    let other = EncountersWrite::from_connection(&transaction, 1);
    let local = EncountersWrite::from_connection(&transaction, 7);
    let foreign = other.get("test")?.unwrap();
    let owned = local.create(NewEncounter {
        slug: foreign.slug.clone(),
        name: "Local encounter".into(),
        description: None,
        note: None,
    })?;
    let request_id = "shared-request";
    let foreign_intent = serde_json::json!({"owner":"other"});
    let local_intent = serde_json::json!({"owner":"local"});
    let advance = |domain: &EncountersWrite<'_>, intent: &serde_json::Value| {
        let actor = domain.add_participant("test", creature("Actor", Some(10), 20))?;
        let current = domain.get("test")?.unwrap();
        domain.advance_turn_once("test", request_id, intent, current.revision, |_| {
            Ok((
                EncounterTurnUpdate {
                    turn: EncounterTurnState {
                        participant_key: Some(actor.participant_key),
                        round_number: 2,
                        mark_running: true,
                    },
                    manual_trackers: vec![],
                    durations: vec![],
                    start_event: None,
                    start_claims: vec![],
                },
                true,
            ))
        })
    };
    let advanced_other = advance(&other, &foreign_intent)?.unwrap();
    let foreign_boundary = other.latest_outgoing_turn("test")?.unwrap().boundary_key;
    let foreign_event = other.apply_condition_event(
        &foreign.encounter_key,
        request_id,
        &foreign_intent,
        advanced_other.revision,
        PreparedConditionEvent::default(),
    )?;

    assert!(
        local
            .advance_turn_receipt("test", request_id, &local_intent)?
            .is_none()
    );
    assert!(
        local
            .advance_turn_receipt(&foreign.encounter_key, request_id, &local_intent)?
            .is_none()
    );
    assert!(
        local
            .condition_event_receipt(&foreign.encounter_key, request_id, &local_intent)?
            .is_none()
    );
    assert!(
        local
            .latest_outgoing_turn(&foreign.encounter_key)?
            .is_none()
    );
    assert!(local.outgoing_turn("test", &foreign_boundary)?.is_none());

    let advanced_local = advance(&local, &local_intent)?.unwrap();
    let local_event = local.apply_condition_event(
        &owned.encounter_key,
        request_id,
        &local_intent,
        advanced_local.revision,
        PreparedConditionEvent::default(),
    )?;
    assert_eq!(
        local
            .advance_turn_receipt("test", request_id, &local_intent)?
            .unwrap()
            .encounter_key,
        owned.encounter_key
    );
    assert_eq!(
        other
            .advance_turn_receipt("test", request_id, &foreign_intent)?
            .unwrap()
            .encounter_key,
        foreign.encounter_key
    );
    assert_eq!(
        local.apply_condition_event_with(
            &owned.encounter_key,
            request_id,
            &local_intent,
            owned.revision,
            || {
                panic!("a recovered event must not be evaluated again");
            }
        )?,
        local_event
    );
    assert_eq!(
        other.condition_event_receipt(&foreign.encounter_key, request_id, &foreign_intent)?,
        Some(foreign_event)
    );
    assert_eq!(
        local
            .latest_outgoing_turn("test")?
            .unwrap()
            .snapshot
            .encounter
            .encounter_key,
        owned.encounter_key
    );
    Ok(())
}

#[test]
fn a_turn_callback_cannot_tick_a_condition_in_another_encounter() -> crate::LocalStateResult<()> {
    let store = fixture("turn-condition-owner-scope")?;
    let actor = store
        .encounters()
        .add_participant("test", creature("Actor", Some(10), 20))?;
    let mut connection = store.connection()?;
    connection.execute(
        "INSERT INTO principals VALUES(2, 'other', 'Other owner')",
        [],
    )?;
    let transaction = connection.transaction()?;
    let local = EncountersWrite::from_connection(&transaction, 1);
    let other = EncountersWrite::from_connection(&transaction, 2);
    let foreign = other.create(NewEncounter {
        slug: "test".into(),
        name: "Other encounter".into(),
        description: None,
        note: None,
    })?;
    let foreign_actor =
        other.add_participant(&foreign.encounter_key, creature("Foreign", None, 20))?;
    let condition = other.add_condition(AddEncounterParticipantCondition {
        participant_key: foreign_actor.participant_key,
        condition_key: None,
        name: "Timed".into(),
        value: None,
        source_participant_key: None,
        scope: ConditionScope::default(),
        extra_action: ConditionExtraAction::default(),
        immunity_override: false,
        parameters: Default::default(),
        origin: None,
        duration: Some(ConditionDuration {
            amount: 2,
            unit: ConditionDurationUnit::Rounds,
            timing: None,
            remaining_seconds: None,
        }),
        note: None,
    })?;
    let condition_id = condition.condition_id;
    let before = local.get("test")?.unwrap();
    let foreign_before = other.get_with_participants("test")?.unwrap();
    let intent = serde_json::json!({"condition":condition_id});
    let result =
        local.advance_turn_once("test", "wrong-condition", &intent, before.revision, |_| {
            Ok((
                EncounterTurnUpdate {
                    turn: EncounterTurnState {
                        participant_key: Some(actor.participant_key),
                        round_number: 2,
                        mark_running: true,
                    },
                    durations: vec![(condition_id, 1)],
                    manual_trackers: vec![],
                    start_event: None,
                    start_claims: vec![],
                },
                true,
            ))
        });
    assert!(matches!(
        result,
        Err(crate::LocalStateError::InvalidRollRequest(_))
    ));
    assert_eq!(local.get("test")?, Some(before));
    assert_eq!(other.get_with_participants("test")?, Some(foreign_before));
    assert!(
        local
            .advance_turn_receipt("test", "wrong-condition", &intent)?
            .is_none()
    );
    assert!(local.latest_outgoing_turn("test")?.is_none());
    Ok(())
}
