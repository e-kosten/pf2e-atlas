use super::*;
use crate::{CharactersWrite, LocalStateError, test_support::initialize_schema};
use rusqlite::Connection;

fn connection() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    initialize_schema(&connection).unwrap();
    connection
}

fn member(connection: &Connection, owner: i64, name: &str, level: Option<i64>) -> PartyMemberInput {
    PartyMemberInput {
        character_key: CharactersWrite::from_connection(connection, owner)
            .create(name, level, None)
            .unwrap()
            .character_key,
    }
}

#[test]
fn party_members_cannot_be_read_or_replaced_through_another_owner() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    tx.execute(
        "INSERT INTO principals (id, principal_key, name) VALUES (2, 'second', 'Second')",
        [],
    )
    .unwrap();
    let first = PartiesWrite::from_connection(&tx, 1);
    let second = PartiesWrite::from_connection(&tx, 2);
    let foreign_member = member(&tx, 1, "First", None);
    let own_member = member(&tx, 2, "Second", None);
    let other = first
        .create("Party", std::slice::from_ref(&foreign_member))
        .unwrap();
    let own = second.create("Party", &[own_member]).unwrap();

    assert_eq!(second.list().unwrap(), vec![own.clone()]);
    assert!(second.candidates(&other.party_key).unwrap().is_empty());
    assert_eq!(
        second.candidates(" Party ").unwrap()[0].party_key,
        own.party_key
    );
    assert!(matches!(
        second.get(&other.party_key),
        Err(LocalStateError::PartyNotFound(_))
    ));
    assert!(matches!(
        second.update(&other.party_key, other.revision, "Changed", &[]),
        Err(LocalStateError::PartyNotFound(_))
    ));
    assert!(matches!(
        second.delete(&other.party_key, other.revision),
        Err(LocalStateError::PartyNotFound(_))
    ));
    assert!(matches!(
        second.update(&own.party_key, own.revision, "Changed", &[foreign_member]),
        Err(LocalStateError::CharacterNotFound(_))
    ));
    assert_eq!(second.get(&own.party_key).unwrap(), own);
    assert_eq!(first.get(&other.party_key).unwrap(), other);
}

#[test]
fn failed_roster_save_rolls_back_its_action_even_if_the_caller_commits() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx, 1);
    let original = parties
        .create(
            "Original",
            &[
                member(&tx, 1, "First", None),
                member(&tx, 1, "Second", None),
            ],
        )
        .unwrap();
    let replacement = [
        member(&tx, 1, "Added", None),
        member(&tx, 1, "Reject", None),
    ];
    tx.execute_batch("CREATE TRIGGER reject_member BEFORE INSERT ON party_character_memberships WHEN (SELECT name FROM characters WHERE character_key=NEW.character_key)='Reject' BEGIN SELECT RAISE(ABORT,'test member failure'); END;").unwrap();
    assert!(
        parties
            .update(
                &original.party_key,
                original.revision,
                "Changed",
                &replacement
            )
            .is_err()
    );
    assert!(parties.create("Failed new party", &replacement).is_err());
    tx.commit().unwrap();
    assert_eq!(
        PartiesRead::from_connection(&connection, 1).list().unwrap(),
        vec![original]
    );
}

#[test]
fn stale_edits_and_deletions_preserve_newer_party_names_and_members() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx, 1);
    let original_member = member(&tx, 1, "First", None);
    let party = parties
        .create("Original", std::slice::from_ref(&original_member))
        .unwrap();
    let latest = parties
        .update(
            &party.party_key,
            party.revision,
            "Latest",
            &[original_member],
        )
        .unwrap();
    assert!(matches!(
        parties.update(&party.party_key, party.revision, "Stale", &[]),
        Err(LocalStateError::PartyConflict)
    ));
    assert!(matches!(
        parties.delete(&party.party_key, party.revision),
        Err(LocalStateError::PartyConflict)
    ));
    assert_eq!(parties.get(&party.party_key).unwrap(), latest);
}

#[test]
fn memberships_use_live_character_levels_and_preserve_distinct_same_names() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx, 1);
    let characters = CharactersWrite::from_connection(&tx, 1);
    let first = member(&tx, 1, "Same", Some(4));
    let second = member(&tx, 1, "Same", None);
    let party = parties
        .create("Mixed", &[first.clone(), second.clone()])
        .unwrap();
    assert_ne!(first.character_key, second.character_key);
    assert_eq!(
        party.members.iter().map(|m| m.level).collect::<Vec<_>>(),
        vec![Some(4), None]
    );
    for level in [-1, 0, 21, i64::MAX] {
        assert!(
            characters
                .update(&first.character_key, 0, "Changed", Some(level), None)
                .is_err()
        );
        assert_eq!(parties.get(&party.party_key).unwrap(), party);
    }
    characters
        .update(&first.character_key, 0, "Same", None, None)
        .unwrap();
    characters
        .update(&second.character_key, 0, "Same", Some(20), None)
        .unwrap();
    let changed = parties
        .update(
            &party.party_key,
            party.revision,
            "Mixed",
            &[second.clone(), first.clone()],
        )
        .unwrap();
    assert_eq!(changed.members[1].character_key, first.character_key);
    assert_eq!(
        changed.members.iter().map(|m| m.level).collect::<Vec<_>>(),
        vec![Some(20), None]
    );
    assert!(
        parties
            .update(
                &party.party_key,
                changed.revision,
                "Duplicate",
                &[second.clone(), second]
            )
            .is_err()
    );
    assert_eq!(parties.get(&party.party_key).unwrap(), changed);
    tx.commit().unwrap();
    assert_eq!(
        PartiesRead::from_connection(&connection, 1)
            .get(&party.party_key)
            .unwrap(),
        changed
    );
}

#[test]
fn party_identity_queries_do_not_fetch_unrelated_rosters() {
    use rusqlite::trace::{TraceEvent, TraceEventCodes};
    thread_local! { static STATEMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let party = PartiesWrite::from_connection(&tx, 1)
        .create("Heroes", &[member(&tx, 1, "Hero", None)])
        .unwrap();
    tx.commit().unwrap();
    for extra in [0, 32] {
        let tx = connection.transaction().unwrap();
        for n in 0..extra {
            PartiesWrite::from_connection(&tx, 1)
                .create(&format!("Unrelated {n}"), &[member(&tx, 1, "Other", None)])
                .unwrap();
        }
        tx.commit().unwrap();
        STATEMENTS.set(0);
        connection
            .trace_v2(
                TraceEventCodes::SQLITE_TRACE_STMT,
                Some(|event: TraceEvent<'_>| {
                    if let TraceEvent::Stmt(_, _) = event {
                        STATEMENTS.set(STATEMENTS.get() + 1);
                    }
                }),
            )
            .unwrap();
        let reader = PartiesRead::from_connection(&connection, 1);
        let matches = reader.candidates(&party.party_key).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(reader.get(&matches[0].party_key).unwrap(), party);
        connection
            .trace_v2(TraceEventCodes::empty(), None::<fn(TraceEvent<'_>)>)
            .unwrap();
        assert_eq!(
            STATEMENTS.get(),
            3,
            "one identity read, one party read and one roster read"
        );
    }
}
