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
        "SELECT member_key, name, level FROM party_members WHERE party_key=?1 ORDER BY position",
    )?;
    party.members = statement
        .query_map([key], |row| {
            Ok(PartyMember {
                member_key: row.get(0)?,
                name: row.get(1)?,
                level: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(Some(party))
}

pub(super) fn list(connection: &Connection, owner_id: i64) -> LocalStateResult<Vec<Party>> {
    let mut statement = connection.prepare(
        "SELECT party_key FROM parties WHERE owner_id=?1 ORDER BY name COLLATE NOCASE, party_key",
    )?;
    let keys = statement
        .query_map([owner_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    keys.into_iter()
        .map(|key| {
            get(connection, owner_id, &key)?.ok_or(crate::LocalStateError::PartyNotFound(key))
        })
        .collect()
}

pub(super) fn replace_members(
    connection: &Connection,
    owned_party_key: &str,
    members: &[PartyMemberInput],
) -> LocalStateResult<()> {
    // Park existing rows outside the new ordering so reordering never deletes a retained member.
    let previous = connection
        .prepare("SELECT member_key,position FROM party_members WHERE party_key=?1")?
        .query_map([owned_party_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let occupied: std::collections::HashSet<_> =
        previous.iter().map(|(_, position)| *position).collect();
    let mut next = members.len() as i64;
    for (key, _) in previous {
        while occupied.contains(&next) {
            next += 1;
        }
        connection.execute(
            "UPDATE party_members SET position=?1 WHERE member_key=?2",
            params![next, key],
        )?;
        next += 1;
    }

    for (position, member) in members.iter().enumerate() {
        if let Some(key) = &member.member_key {
            connection.execute(
                "UPDATE party_members SET position=?1,name=?2,level=?3 WHERE member_key=?4 AND party_key=?5",
                params![position as i64, member.name.trim(), member.level, key, owned_party_key],
            )?;
        } else {
            connection.execute(
                "INSERT INTO party_members(member_key,party_key,position,name,level) VALUES(?1,?2,?3,?4,?5)",
                params![new_key("member"), owned_party_key, position as i64, member.name.trim(), member.level],
            )?;
        }
    }
    connection.execute(
        "DELETE FROM party_members WHERE party_key=?1 AND position>=?2",
        params![owned_party_key, members.len() as i64],
    )?;
    Ok(())
}

pub(super) fn new_key(prefix: &str) -> String {
    format!("{prefix}_{:032x}", rand::random::<u128>())
}
