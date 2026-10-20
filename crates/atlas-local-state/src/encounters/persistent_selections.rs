//! Saved GM comparison decisions; effective damage remains app-service policy.
use crate::LocalStateResult;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentSourceSelection {
    pub episode_id: String,
    pub selected_source_id: i64,
    pub source_condition_ids: Vec<i64>,
}

pub(super) fn read(
    connection: &Connection,
    participant_key: &str,
) -> LocalStateResult<Vec<PersistentSourceSelection>> {
    let mut statement = connection.prepare("SELECT episode_id, selected_source_id, membership_json FROM encounter_persistent_selections WHERE participant_id=(SELECT id FROM encounter_participants WHERE participant_key=?1) ORDER BY episode_id")?;
    let rows = statement.query_map([participant_key], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    rows.map(|row| {
        let (episode_id, selected_source_id, members) = row?;
        Ok(PersistentSourceSelection {
            episode_id,
            selected_source_id,
            source_condition_ids: serde_json::from_str(&members)?,
        })
    })
    .collect()
}

pub(super) fn write(
    connection: &Connection,
    participant_key: &str,
    selection: &PersistentSourceSelection,
) -> LocalStateResult<()> {
    connection.execute("INSERT INTO encounter_persistent_selections(participant_id,episode_id,selected_source_id,membership_json) SELECT id,?2,?3,?4 FROM encounter_participants WHERE participant_key=?1 ON CONFLICT(participant_id,episode_id) DO UPDATE SET selected_source_id=excluded.selected_source_id,membership_json=excluded.membership_json", params![participant_key,selection.episode_id,selection.selected_source_id,serde_json::to_string(&selection.source_condition_ids)?])?;
    Ok(())
}
