//! Saved GM precedence within a persistent-damage episode.
use crate::LocalStateResult;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentSourcePrecedence {
    pub episode_id: String,
    /// Highest precedence first. Absent sources are ignored by the rules owner.
    pub source_order: Vec<i64>,
}

pub(super) fn read(
    connection: &Connection,
    owner_id: i64,
    participant_key: &str,
) -> LocalStateResult<Vec<PersistentSourcePrecedence>> {
    let mut statement = connection.prepare("SELECT episode_id,source_order_json FROM encounter_persistent_precedence WHERE participant_id=(SELECT p.id FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id WHERE p.participant_key=?1 AND e.owner_id=?2) ORDER BY episode_id")?;
    let rows = statement.query_map(params![participant_key, owner_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.map(|row| {
        let (episode_id, order) = row?;
        decode(episode_id, &order)
    })
    .collect()
}

pub(super) fn read_for_encounter(
    connection: &Connection,
    owner_id: i64,
    encounter: &str,
) -> LocalStateResult<std::collections::BTreeMap<String, Vec<PersistentSourcePrecedence>>> {
    let mut statement = connection.prepare("SELECT p.participant_key,r.episode_id,r.source_order_json FROM encounter_participants p JOIN encounter_persistent_precedence r ON r.participant_id=p.id WHERE p.encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1 AND owner_id=?2) ORDER BY p.participant_key,r.episode_id")?;
    let rows = statement.query_map(params![encounter, owner_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut grouped = std::collections::BTreeMap::<String, Vec<PersistentSourcePrecedence>>::new();
    for row in rows {
        let (key, episode_id, order) = row?;
        grouped
            .entry(key)
            .or_default()
            .push(decode(episode_id, &order)?);
    }
    Ok(grouped)
}

fn decode(episode_id: String, order: &str) -> LocalStateResult<PersistentSourcePrecedence> {
    Ok(PersistentSourcePrecedence {
        episode_id,
        source_order: serde_json::from_str(order)?,
    })
}

pub(super) fn write(
    connection: &Connection,
    owner_id: i64,
    encounter_key: &str,
    participant_key: &str,
    precedence: &PersistentSourcePrecedence,
) -> LocalStateResult<()> {
    connection.execute("INSERT INTO encounter_persistent_precedence(participant_id,episode_id,source_order_json) SELECT p.id,?2,?3 FROM encounter_participants p JOIN encounters e ON e.id=p.encounter_id WHERE p.participant_key=?1 AND e.encounter_key=?4 AND e.owner_id=?5 ON CONFLICT(participant_id,episode_id) DO UPDATE SET source_order_json=excluded.source_order_json", params![participant_key,precedence.episode_id,serde_json::to_string(&precedence.source_order)?,encounter_key,owner_id])?;
    Ok(())
}
