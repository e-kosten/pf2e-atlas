use super::*;
use rusqlite::trace::{TraceEvent, TraceEventCodes};
use std::cell::RefCell;

thread_local! {
    static STATEMENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
fn measured_roster(
    connection: &rusqlite::Connection,
) -> crate::LocalStateResult<(EncounterWithParticipants, usize)> {
    STATEMENTS.with(|statements| statements.borrow_mut().clear());
    connection.trace_v2(
        TraceEventCodes::SQLITE_TRACE_STMT,
        Some(|event| {
            if let TraceEvent::Stmt(_, sql) = event {
                STATEMENTS.with(|statements| statements.borrow_mut().push(sql.into()));
            }
        }),
    );
    let result = EncountersRead::from_connection(connection).get_with_participants("test");
    connection.trace_v2(TraceEventCodes::empty(), None);
    Ok((
        result?.unwrap(),
        STATEMENTS.with(|statements| statements.borrow().len()),
    ))
}

#[test]
fn roster_statement_count_depends_on_collections_instead_of_participant_count()
-> crate::LocalStateResult<()> {
    let store = fixture("roster-statements")?;
    let connection = store.connection()?;
    assert_eq!(measured_roster(&connection)?.1, 2);
    for count in 1..=40 {
        store
            .encounters()
            .add_participant("test", creature(&format!("Actor {count}"), Some(count), 20))?;
        if [1, 2, 8, 40].contains(&count) {
            let (roster, statements) = measured_roster(&connection)?;
            assert_eq!(roster.participants.len(), count as usize);
            assert_eq!(statements, 7, "roster size {count}");
        }
    }
    Ok(())
}

#[test]
fn batched_roster_preserves_child_order_empty_values_and_other_encounter_isolation()
-> crate::LocalStateResult<()> {
    let store = fixture("roster-aggregates")?;
    let one = store
        .encounters()
        .add_participant("test", creature("One", Some(10), 20))?;
    let mut unknown = creature("Unknown", None, 0);
    unknown.max_hp = None;
    unknown.current_hp = None;
    let two = store.encounters().add_participant("test", unknown)?;
    let connection = store.connection()?;
    let owner = atlas_domain::OwnedContentLocator {
        collection: "items".into(),
        identity: atlas_domain::OwnedContentIdentity::Stable("aaaaaaaaaaaaaaaa".into()),
    };
    let ability = ParticipantAbilityState {
        owner: owner.clone(),
        rule_ordinal: Some(0),
        source_fingerprint: "kept-source".into(),
        declaration_identity: None,
        value: AbilityStateValue::Uses { remaining: 0 },
    };
    connection.execute(
        "INSERT INTO encounter_ability_states VALUES(?1,?2,0,?3)",
        rusqlite::params![
            one.participant_key,
            serde_json::to_string(&owner)?,
            super::super::receipt_codec::encode(&ability)?
        ],
    )?;
    let id: i64 = connection.query_row(
        "SELECT id FROM encounter_participants WHERE participant_key=?1",
        [&one.participant_key],
        |row| row.get(0),
    )?;
    for (condition, created) in [(91, "later"), (92, "earlier"), (93, "earlier")] {
        connection.execute("INSERT INTO encounter_participant_conditions(id,participant_id,name,value,created_at,updated_at) VALUES(?1,?2,'Independent',NULL,?3,?3)",rusqlite::params![condition,id,created])?;
    }
    connection.execute(
        "INSERT INTO encounter_persistent_precedence VALUES(?1,'z','[91,92]')",
        [id],
    )?;
    connection.execute(
        "INSERT INTO encounter_persistent_precedence VALUES(?1,'a','[]')",
        [id],
    )?;
    connection.execute(
        "INSERT INTO encounter_spell_resources VALUES(?1,'z',4,0,NULL)",
        [id],
    )?;
    connection.execute(
        "INSERT INTO encounter_spell_resources VALUES(?1,'a',1,1,0)",
        [id],
    )?;
    connection.execute("INSERT INTO encounter_expended_spells VALUES(?1,'z')", [id])?;
    connection.execute("INSERT INTO encounter_expended_spells VALUES(?1,'a')", [id])?;
    store.encounters().create(NewEncounter {
        slug: "other".into(),
        name: "Other".into(),
        description: None,
        note: None,
    })?;
    let other = store
        .encounters()
        .add_participant("other", creature("Other", None, 20))?;
    connection.execute(
        "INSERT INTO encounter_ability_states VALUES(?1,'malformed',-1,'invalid')",
        [&other.participant_key],
    )?;
    assert!(
        EncountersRead::from_connection(&connection)
            .get_with_participants("other")
            .is_err()
    );
    let (roster, statements) = measured_roster(&connection)?;
    assert_eq!(statements, 7);
    assert_eq!(
        roster
            .participants
            .iter()
            .map(|p| p.participant_key.as_str())
            .collect::<Vec<_>>(),
        vec![one.participant_key.as_str(), two.participant_key.as_str()]
    );
    let first = &roster.participants[0];
    assert_eq!(first.ability_states, vec![ability]);
    assert_eq!(
        first
            .conditions
            .iter()
            .map(|c| c.condition_id)
            .collect::<Vec<_>>(),
        vec![92, 93, 91]
    );
    assert!(first.conditions.iter().all(|c| c.value.is_none()));
    assert_eq!(
        first.persistent_precedence,
        vec![
            PersistentSourcePrecedence {
                episode_id: "a".into(),
                source_order: vec![]
            },
            PersistentSourcePrecedence {
                episode_id: "z".into(),
                source_order: vec![91, 92]
            }
        ]
    );
    assert_eq!(first.expended_spells, vec!["a", "z"]);
    assert_eq!(
        first.spell_resources,
        vec![
            SpellResource {
                key: "a".into(),
                capacity: 1,
                remaining: 1,
                undo_remaining: Some(0)
            },
            SpellResource {
                key: "z".into(),
                capacity: 4,
                remaining: 0,
                undo_remaining: None
            }
        ]
    );
    let empty = &roster.participants[1];
    assert!(empty.max_hp.is_none() && empty.current_hp.is_none() && empty.turn_resources.is_none());
    assert!(
        empty.ability_states.is_empty()
            && empty.conditions.is_empty()
            && empty.persistent_precedence.is_empty()
            && empty.expended_spells.is_empty()
            && empty.spell_resources.is_empty()
    );
    assert_eq!(
        super::super::storage::participant(&connection, &one.participant_key)?.unwrap(),
        *first
    );
    Ok(())
}
