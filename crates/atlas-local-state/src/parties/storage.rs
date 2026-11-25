use crate::{LocalStateResult, Party, PartyMember, PartyMemberInput};
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn get(
    connection: &Connection,
    owner_id: i64,
    key: &str,
) -> LocalStateResult<Option<Party>> {
    let party = connection
        .query_row(
            "SELECT party_key, name, revision FROM parties WHERE owner_id=?2 AND party_key=?1",
            params![key, owner_id],
            |row| {
                Ok(Party {
                    party_key: row.get(0)?,
                    name: row.get(1)?,
                    revision: row.get(2)?,
                    members: vec![],
                })
            },
        )
        .optional()?;
    let Some(mut party) = party else {
        return Ok(None);
    };
    let mut statement = connection.prepare(
        "SELECT c.character_key,c.name,c.level,c.revision,c.reference_payload IS NOT NULL FROM party_character_memberships m JOIN characters c USING(character_key) WHERE m.party_key=?1 AND c.owner_id=?2 ORDER BY m.position",
    )?;
    party.members = statement
        .query_map(params![key, owner_id], |row| {
            Ok(PartyMember {
                character_key: row.get(0)?,
                revision: row.get(3)?,
                imported: row.get(4)?,
                name: row.get(1)?,
                level: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Some(party))
}

pub(super) fn candidates(
    connection: &Connection,
    owner_id: i64,
    reference: &str,
) -> LocalStateResult<Vec<crate::CharacterParty>> {
    let identity = |row: &rusqlite::Row<'_>| {
        Ok(crate::CharacterParty {
            party_key: row.get(0)?,
            name: row.get(1)?,
        })
    };
    if let Some(party) = connection
        .query_row(
            "SELECT party_key, name FROM parties WHERE owner_id=?1 AND party_key=?2",
            params![owner_id, reference],
            identity,
        )
        .optional()?
    {
        return Ok(vec![party]);
    }
    let mut statement = connection.prepare(
        "SELECT party_key, name FROM parties WHERE owner_id=?1 AND name=?2 ORDER BY party_key",
    )?;
    Ok(statement
        .query_map(params![owner_id, reference.trim()], identity)?
        .collect::<Result<_, _>>()?)
}

pub(super) fn directory(
    connection: &Connection,
    owner_id: i64,
) -> LocalStateResult<Vec<crate::PartySummary>> {
    let mut statement = connection.prepare(
        "SELECT p.party_key,p.name,p.revision,count(m.character_key) FROM parties p
         LEFT JOIN party_character_memberships m USING(party_key)
         WHERE p.owner_id=?1 GROUP BY p.party_key ORDER BY p.name COLLATE NOCASE,p.party_key",
    )?;
    Ok(statement
        .query_map([owner_id], |row| {
            Ok(crate::PartySummary {
                party_key: row.get(0)?,
                name: row.get(1)?,
                revision: row.get(2)?,
                member_count: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?)
}

pub(super) fn list(connection: &Connection, owner_id: i64) -> LocalStateResult<Vec<Party>> {
    let mut statement = connection.prepare(
        "SELECT p.party_key,p.name,p.revision,c.character_key,c.name,c.level,c.revision,c.reference_payload IS NOT NULL
         FROM parties p LEFT JOIN party_character_memberships m USING(party_key)
         LEFT JOIN characters c ON c.character_key=m.character_key AND c.owner_id=p.owner_id
         WHERE p.owner_id=?1 ORDER BY p.name COLLATE NOCASE,p.party_key,m.position",
    )?;
    let mut rows = statement.query([owner_id])?;
    let mut parties = Vec::<Party>::new();
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let party = match parties.last_mut() {
            Some(party) if party.party_key == key => party,
            _ => parties.push_mut(Party {
                party_key: key,
                name: row.get(1)?,
                revision: row.get(2)?,
                members: vec![],
            }),
        };
        if let Some(character_key) = row.get::<_, Option<String>>(3)? {
            party.members.push(PartyMember {
                character_key,
                name: row.get(4)?,
                level: row.get(5)?,
                revision: row.get(6)?,
                imported: row.get(7)?,
            });
        }
    }
    Ok(parties)
}

pub(super) fn replace_members(
    connection: &Connection,
    owned_party_key: &str,
    members: &[PartyMemberInput],
) -> LocalStateResult<()> {
    let owner: i64 = connection.query_row(
        "SELECT owner_id FROM parties WHERE party_key=?1",
        [owned_party_key],
        |r| r.get(0),
    )?;
    connection.execute(
        "DELETE FROM party_character_memberships WHERE party_key=?1",
        [owned_party_key],
    )?;
    for (position, member) in members.iter().enumerate() {
        let inserted = connection.execute(
            "INSERT INTO party_character_memberships(party_key,character_key,position)
             SELECT ?1,character_key,?3 FROM characters WHERE character_key=?2 AND owner_id=?4",
            params![
                owned_party_key,
                member.character_key,
                position as i64,
                owner
            ],
        )?;
        if inserted == 0 {
            return Err(crate::LocalStateError::CharacterNotFound(
                member.character_key.clone(),
            ));
        }
    }
    Ok(())
}

pub(super) fn new_key(prefix: &str) -> String {
    format!("{prefix}_{:032x}", rand::random::<u128>())
}
