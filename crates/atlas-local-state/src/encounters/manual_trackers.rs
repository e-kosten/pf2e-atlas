//! Saved manual countdowns. Conversion and timing policy belong to app-service.
use super::{ConditionDuration, EncountersWrite, receipt_codec};
use crate::{LocalStateError, LocalStateResult};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualTracker {
    pub tracker_key: String,
    pub name: String,
    pub duration: ConditionDuration,
    pub progress: TrackerProgress,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "unit", content = "remaining", rename_all = "snake_case")]
pub enum TrackerProgress {
    Rounds(i64),
    Seconds(i64),
}
#[derive(Debug, Clone)]
pub struct ManualTrackerChange {
    pub participant_key: String,
    pub tracker_key: String,
    pub value: Option<ManualTracker>,
}
pub(super) fn read(
    connection: &Connection,
    owner_id: i64,
    key: &str,
) -> LocalStateResult<Vec<ManualTracker>> {
    let mut statement = connection.prepare("SELECT t.payload_json FROM encounter_manual_trackers t JOIN encounter_participants p USING(participant_key) JOIN encounters e ON e.id=p.encounter_id WHERE p.participant_key=?1 AND e.owner_id=?2 ORDER BY t.position")?;
    let rows = statement.query_map(params![key, owner_id], |row| row.get::<_, String>(0))?;
    rows.map(|row| receipt_codec::decode(&row?)).collect()
}
pub(super) fn read_for_encounter(
    connection: &Connection,
    owner_id: i64,
    encounter: &str,
) -> LocalStateResult<std::collections::BTreeMap<String, Vec<ManualTracker>>> {
    let mut statement = connection.prepare("SELECT t.participant_key,t.payload_json FROM encounter_manual_trackers t JOIN encounter_participants p ON p.participant_key=t.participant_key WHERE p.encounter_id=(SELECT id FROM encounters WHERE encounter_key=?1 AND owner_id=?2) ORDER BY t.participant_key,t.position")?;
    let rows = statement.query_map(params![encounter, owner_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut grouped = std::collections::BTreeMap::<String, Vec<ManualTracker>>::new();
    for row in rows {
        let (key, payload) = row?;
        grouped
            .entry(key)
            .or_default()
            .push(receipt_codec::decode(&payload)?);
    }
    Ok(grouped)
}
impl EncountersWrite<'_> {
    pub(super) fn save_manual_tracker(
        &self,
        encounter: &str,
        change: &ManualTrackerChange,
    ) -> LocalStateResult<()> {
        let changed = self.connection.execute("UPDATE encounter_participants SET revision=revision+1 WHERE participant_key=?1 AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?2 AND owner_id=?3)",params![change.participant_key,encounter,self.owner_id])?;
        if changed != 1 {
            return Err(LocalStateError::ParticipantNotFound(
                change.participant_key.clone(),
            ));
        }
        match &change.value {
            Some(value) => {
                self.connection.execute(
                    "INSERT INTO encounter_manual_trackers(participant_key, tracker_key, payload_json, position)
                     SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1
                     FROM encounter_manual_trackers WHERE participant_key = ?1
                     ON CONFLICT(participant_key, tracker_key)
                     DO UPDATE SET payload_json = excluded.payload_json",
                    params![change.participant_key, change.tracker_key, receipt_codec::encode(value)?],
                )?;
            }
            None => {
                self.connection.execute("DELETE FROM encounter_manual_trackers WHERE participant_key=?1 AND tracker_key=?2",params![change.participant_key,change.tracker_key])?;
            }
        }
        Ok(())
    }
}
