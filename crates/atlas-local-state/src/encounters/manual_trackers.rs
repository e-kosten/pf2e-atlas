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
    pub remaining_rounds: i64,
}
#[derive(Debug, Clone)]
pub struct ManualTrackerChange {
    pub participant_key: String,
    pub tracker_key: String,
    pub value: Option<ManualTracker>,
}
pub(super) fn read(connection: &Connection, key: &str) -> LocalStateResult<Vec<ManualTracker>> {
    let mut statement = connection.prepare("SELECT payload_json FROM encounter_manual_trackers WHERE participant_key=?1 ORDER BY tracker_key")?;
    let rows = statement.query_map([key], |row| row.get::<_, String>(0))?;
    rows.map(|row| receipt_codec::decode(&row?)).collect()
}
impl EncountersWrite<'_> {
    pub(super) fn save_manual_tracker(
        &self,
        encounter: &str,
        change: &ManualTrackerChange,
    ) -> LocalStateResult<()> {
        let changed = self.connection.execute("UPDATE encounter_participants SET revision=revision+1 WHERE participant_key=?1 AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?2)",params![change.participant_key,encounter])?;
        if changed != 1 {
            return Err(LocalStateError::ParticipantNotFound(
                change.participant_key.clone(),
            ));
        }
        match &change.value {
            Some(value) => {
                self.connection.execute("INSERT INTO encounter_manual_trackers(participant_key,tracker_key,payload_json) VALUES(?1,?2,?3) ON CONFLICT(participant_key,tracker_key) DO UPDATE SET payload_json=excluded.payload_json",params![change.participant_key,change.tracker_key,receipt_codec::encode(value)?])?;
            }
            None => {
                self.connection.execute("DELETE FROM encounter_manual_trackers WHERE participant_key=?1 AND tracker_key=?2",params![change.participant_key,change.tracker_key])?;
            }
        }
        Ok(())
    }
}
