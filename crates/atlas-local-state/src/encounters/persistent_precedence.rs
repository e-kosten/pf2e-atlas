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
    participant_key: &str,
) -> LocalStateResult<Vec<PersistentSourcePrecedence>> {
    let mut statement = connection.prepare("SELECT episode_id,source_order_json FROM encounter_persistent_precedence WHERE participant_id=(SELECT id FROM encounter_participants WHERE participant_key=?1) ORDER BY episode_id")?;
    let rows = statement.query_map([participant_key], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.map(|row| {
        let (episode_id, order) = row?;
        Ok(PersistentSourcePrecedence {
            episode_id,
            source_order: serde_json::from_str(&order)?,
        })
    })
    .collect()
}

pub(super) fn write(
    connection: &Connection,
    participant_key: &str,
    precedence: &PersistentSourcePrecedence,
) -> LocalStateResult<()> {
    connection.execute("INSERT INTO encounter_persistent_precedence(participant_id,episode_id,source_order_json) SELECT id,?2,?3 FROM encounter_participants WHERE participant_key=?1 ON CONFLICT(participant_id,episode_id) DO UPDATE SET source_order_json=excluded.source_order_json", params![participant_key,precedence.episode_id,serde_json::to_string(&precedence.source_order)?])?;
    Ok(())
}
