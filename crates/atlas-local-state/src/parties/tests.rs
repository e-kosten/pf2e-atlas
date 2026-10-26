use super::*;
use crate::{LOCAL_STATE_SCHEMA_SQL, LocalStateError, PARTIES_SCHEMA_SQL};
use rusqlite::Connection;

fn connection() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE record_identity(id INTEGER PRIMARY KEY,key TEXT UNIQUE,last_known_name TEXT,availability TEXT NOT NULL);").unwrap();
    connection.execute_batch(LOCAL_STATE_SCHEMA_SQL).unwrap();
    connection.execute_batch(PARTIES_SCHEMA_SQL).unwrap();
    connection
        .execute_batch(crate::PARTY_LEVELS_SCHEMA_SQL)
        .unwrap();
    connection
}
fn member(name: &str) -> PartyMemberInput {
    PartyMemberInput {
        level: None,
        member_key: None,
        name: name.into(),
    }
}
#[test]
fn failed_roster_save_rolls_back_its_action_even_if_the_caller_commits() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx);
    let original = parties
        .create("Original", &[member("First"), member("Second")])
        .unwrap();
    tx.execute_batch("CREATE TRIGGER reject_member BEFORE INSERT ON party_members WHEN NEW.name='Reject' BEGIN SELECT RAISE(ABORT,'test member failure'); END;").unwrap();
    assert!(
        parties
            .update(
                &original.party_key,
                original.revision,
                "Changed",
                &[member("Added"), member("Reject")]
            )
            .is_err()
    );
    assert!(
        parties
            .create("Failed new party", &[member("Added"), member("Reject")])
            .is_err()
    );
    tx.commit().unwrap();
    assert_eq!(
        PartiesRead::from_connection(&connection).list().unwrap(),
        vec![original]
    );
}
#[test]
fn stale_edits_and_deletions_preserve_newer_party_names_and_members() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx);
    let party = parties.create("Original", &[member("First")]).unwrap();
    let latest = parties
        .update(
            &party.party_key,
            party.revision,
            "Latest",
            &[PartyMemberInput {
                level: None,
                member_key: Some(party.members[0].member_key.clone()),
                name: "Renamed PC".into(),
            }],
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
fn levels_remain_optional_and_invalid_edits_preserve_the_whole_roster() {
    let mut connection = connection();
    let tx = connection.transaction().unwrap();
    let parties = PartiesWrite::from_connection(&tx);
    let party = parties
        .create(
            "Mixed",
            &[
                PartyMemberInput {
                    member_key: None,
                    name: "Same".into(),
                    level: Some(4),
                },
                PartyMemberInput {
                    member_key: None,
                    name: "Same".into(),
                    level: None,
                },
            ],
        )
        .unwrap();
    assert_eq!(
        party.members.iter().map(|m| m.level).collect::<Vec<_>>(),
        vec![Some(4), None]
    );
    for level in [-1, 0, 21, i64::MAX] {
        assert!(
            parties
                .update(
                    &party.party_key,
                    party.revision,
                    "Invalid",
                    &[PartyMemberInput {
                        member_key: Some(party.members[0].member_key.clone()),
                        name: "Changed".into(),
                        level: Some(level)
                    }]
                )
                .is_err()
        );
        assert_eq!(parties.get(&party.party_key).unwrap(), party);
    }
    let changed = parties
        .update(
            &party.party_key,
            party.revision,
            "Mixed",
            &[
                PartyMemberInput {
                    member_key: Some(party.members[1].member_key.clone()),
                    name: "Same".into(),
                    level: Some(20),
                },
                PartyMemberInput {
                    member_key: Some(party.members[0].member_key.clone()),
                    name: "Same".into(),
                    level: None,
                },
            ],
        )
        .unwrap();
    assert_eq!(changed.members[1].member_key, party.members[0].member_key);
    assert_eq!(
        changed.members.iter().map(|m| m.level).collect::<Vec<_>>(),
        vec![Some(20), None]
    );
    assert!(
        parties
            .update(&party.party_key, party.revision, "Stale", &[])
            .is_err()
    );
    tx.commit().unwrap();
    assert_eq!(
        PartiesRead::from_connection(&connection)
            .get(&party.party_key)
            .unwrap(),
        changed
    );
}
