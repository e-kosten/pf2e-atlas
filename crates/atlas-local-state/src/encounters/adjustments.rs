//! Encounter-owned adjustments and their persisted timing state.
use super::{
    ConditionDuration, ConditionDurationUnit, EncountersWrite, TrackerProgress, receipt_codec,
};
use crate::{LocalStateError, LocalStateResult};
use atlas_domain::{AdjustmentModifierType, AdjustmentTarget};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAdjustment {
    pub adjustment_key: String,
    pub target: AdjustmentTarget,
    pub name: String,
    pub amount: i64,
    pub modifier_type: AdjustmentModifierType,
    pub timing: Option<AdjustmentTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdjustmentTiming {
    pub duration: ConditionDuration,
    pub progress: TrackerProgress,
}

impl EncounterAdjustment {
    pub fn valid(&self) -> bool {
        const EXACT: i64 = 9_007_199_254_740_991;
        !self.adjustment_key.is_empty()
            && self.adjustment_key.len() <= 160
            && !self.name.trim().is_empty()
            && self.name.chars().count() <= 120
            && self.amount != 0
            && (-10000..=10000).contains(&self.amount)
            && self.target.is_supported()
            && self.timing.as_ref().is_none_or(|timing| {
                timing
                    .duration
                    .timing
                    .as_ref()
                    .is_some_and(|anchor| !anchor.participant_key.is_empty())
                    && timing.duration.remaining_seconds.is_none()
                    && (1..=EXACT).contains(&timing.duration.amount)
                    && match timing.progress {
                        TrackerProgress::Rounds(n) => {
                            timing.duration.unit == ConditionDurationUnit::Rounds
                                && (1..=EXACT).contains(&n)
                        }
                        TrackerProgress::Seconds(n) => {
                            timing.duration.unit != ConditionDurationUnit::Rounds
                                && (1..=EXACT).contains(&n)
                        }
                    }
            })
    }
}

#[derive(Debug, Clone)]
pub struct AdjustmentChange {
    pub participant_key: String,
    pub adjustment_key: String,
    pub value: Option<EncounterAdjustment>,
}

pub(super) fn read(
    connection: &Connection,
    owner_id: i64,
    participant_key: &str,
) -> LocalStateResult<Vec<EncounterAdjustment>> {
    let mut statement = connection.prepare(
        "SELECT a.payload_json FROM encounter_adjustments a
         JOIN encounter_participants p USING(participant_key) JOIN encounters e ON e.id=p.encounter_id
         WHERE p.participant_key=?1 AND e.owner_id=?2 ORDER BY a.position",
    )?;
    let rows = statement.query_map(params![participant_key, owner_id], |row| {
        row.get::<_, String>(0)
    })?;
    rows.map(|row| receipt_codec::decode(&row?)).collect()
}

pub(super) fn read_for_encounter(
    connection: &Connection,
    owner_id: i64,
    encounter_key: &str,
) -> LocalStateResult<BTreeMap<String, Vec<EncounterAdjustment>>> {
    let mut statement = connection.prepare(
        "SELECT a.participant_key,a.payload_json FROM encounter_adjustments a
         JOIN encounter_participants p USING(participant_key) JOIN encounters e ON e.id=p.encounter_id
         WHERE e.encounter_key=?1 AND e.owner_id=?2 ORDER BY a.participant_key,a.position",
    )?;
    let rows = statement.query_map(params![encounter_key, owner_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut grouped = BTreeMap::<String, Vec<EncounterAdjustment>>::new();
    for row in rows {
        let (key, payload) = row?;
        grouped
            .entry(key)
            .or_default()
            .push(receipt_codec::decode(&payload)?);
    }
    Ok(grouped)
}

pub(super) fn insert(
    connection: &Connection,
    participant_key: &str,
    adjustment: &EncounterAdjustment,
) -> LocalStateResult<()> {
    connection.execute(
        "INSERT INTO encounter_adjustments(participant_key,adjustment_key,payload_json,position)
         SELECT ?1, ?2, ?3, COALESCE(MAX(position), 0) + 1
         FROM encounter_adjustments WHERE participant_key=?1
         ON CONFLICT(participant_key, adjustment_key)
         DO UPDATE SET payload_json=excluded.payload_json",
        params![
            participant_key,
            adjustment.adjustment_key,
            receipt_codec::encode(adjustment)?
        ],
    )?;
    Ok(())
}

impl EncountersWrite<'_> {
    pub(super) fn save_adjustment(
        &self,
        encounter: &str,
        change: &AdjustmentChange,
    ) -> LocalStateResult<()> {
        let updated = self.connection.execute(
            "UPDATE encounter_participants SET revision=revision+1
             WHERE participant_key=?1 AND participant_kind<>'initiative_entry'
             AND encounter_id=(SELECT id FROM encounters WHERE encounter_key=?2 AND owner_id=?3)",
            params![change.participant_key, encounter, self.owner_id],
        )?;
        if updated != 1 {
            return Err(LocalStateError::ParticipantNotFound(
                change.participant_key.clone(),
            ));
        }
        if let Some(value) = &change.value {
            insert(self.connection, &change.participant_key, value)?;
        } else {
            self.connection.execute(
                "DELETE FROM encounter_adjustments WHERE participant_key=?1 AND adjustment_key=?2",
                params![change.participant_key, change.adjustment_key],
            )?;
        }
        Ok(())
    }
}
