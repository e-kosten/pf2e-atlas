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
