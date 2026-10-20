use crate::{LocalStateResult, Party, PartyMember, PartyMemberInput};
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn get(connection: &Connection, key: &str) -> LocalStateResult<Option<Party>> {
    let party = connection
        .query_row(
            "SELECT party_key, name, revision FROM parties WHERE owner_id=1 AND party_key=?1",
            [key],
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
        "SELECT member_key, name FROM party_members WHERE party_key=?1 ORDER BY position",
    )?;
    party.members = statement
        .query_map([key], |row| {
            Ok(PartyMember {
                member_key: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Some(party))
}

pub(super) fn list(connection: &Connection) -> LocalStateResult<Vec<Party>> {
    let mut statement = connection.prepare(
        "SELECT party_key FROM parties WHERE owner_id=1 ORDER BY name COLLATE NOCASE, party_key",
    )?;
    let keys = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    keys.into_iter()
        .map(|key| get(connection, &key)?.ok_or(crate::LocalStateError::PartyNotFound(key)))
        .collect()
}

pub(super) fn replace_members(
    connection: &Connection,
    key: &str,
    members: &[PartyMemberInput],
) -> LocalStateResult<()> {
    connection.execute("DELETE FROM party_members WHERE party_key=?1", [key])?;
    for (position, member) in members.iter().enumerate() {
        let member_key = member
            .member_key
            .clone()
            .unwrap_or_else(|| new_key("member"));
        connection.execute(
            "INSERT INTO party_members(member_key,party_key,position,name) VALUES(?1,?2,?3,?4)",
            params![member_key, key, position as i64, member.name.trim()],
        )?;
    }
    Ok(())
}

pub(super) fn new_key(prefix: &str) -> String {
    format!("{prefix}_{:032x}", rand::random::<u128>())
}
