//! Immutable backfill for initiative order, saved indexes and event claims.
use atlas_storage::{StorageError, StorageResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const SAVED_STATE_CLOSEOUT_SCHEMA_VERSION: u32 = 39;

pub fn backfill_saved_state_closeout(connection: &Connection) -> StorageResult<()> {
    if connection.is_autocommit() {
        return Err(StorageError::Migration(
            "saved-state closeout requires a storage-owned transaction".into(),
        ));
    }
    let orphan: Option<(i64, String)> = connection
        .query_row(
            "SELECT b.encounter_id,b.request_id FROM encounter_boundary_claims b
         WHERE NOT EXISTS(SELECT 1 FROM encounter_command_receipts r
                          WHERE r.encounter_id=b.encounter_id AND r.family='event'
                            AND r.request_id=b.request_id)
         ORDER BY b.encounter_id,b.request_id LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((encounter, request)) = orphan {
        return Err(StorageError::Migration(format!(
            "encounter {encounter} has a boundary claim without its event receipt {request}"
        )));
    }
    connection.execute_batch(include_str!("saved_state_closeout.sql"))?;
    repair_frozen_orders(connection)
}

fn repair_frozen_orders(connection: &Connection) -> StorageResult<()> {
    let mut statement = connection.prepare(
        "SELECT encounter_id FROM encounter_outgoing_turns t
         WHERE CASE WHEN json_valid(t.payload_json) THEN
           json_extract(t.payload_json,'$.version')=1
           AND json_type(t.payload_json,'$.payload.snapshot.participants')='array'
           AND EXISTS(
             SELECT 1 FROM json_each(t.payload_json,'$.payload.snapshot.participants') p
             WHERE CASE WHEN p.type='object' THEN json_type(p.value,'$.initiative_order')='integer'
               AND json_extract(p.value,'$.initiative_order')<=0 END
           )
         END ORDER BY encounter_id",
    )?;
    let encounters = statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for encounter in encounters {
        let encoded: String = connection.query_row(
            "SELECT payload_json FROM encounter_outgoing_turns WHERE encounter_id=?",
            [encounter],
            |row| row.get(0),
        )?;
        let invalid = |reason: &str| {
            StorageError::Migration(format!(
                "cannot repair initiative order in the frozen turn for encounter {encounter}: {reason}"
            ))
        };
        let mut payload: Value =
            serde_json::from_str(&encoded).map_err(|_| invalid("invalid payload"))?;
        let participants = payload
            .pointer_mut("/payload/snapshot/participants")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| invalid("invalid roster"))?;
        let mut buckets = BTreeMap::new();
        let mut affected = BTreeSet::new();
        for (index, participant) in participants.iter().enumerate() {
            let initiative = match participant.get("initiative") {
                Some(Value::Null) => None,
                Some(value) => Some(
                    value
                        .as_i64()
                        .ok_or_else(|| invalid("invalid initiative"))?,
                ),
                None => return Err(invalid("missing initiative")),
            };
            let order = participant
                .get("initiative_order")
                .and_then(Value::as_i64)
                .ok_or_else(|| invalid("invalid initiative order"))?;
            let position = participant
                .get("position")
                .and_then(Value::as_i64)
                .ok_or_else(|| invalid("invalid roster position"))?;
            let key = participant
                .get("participant_key")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("invalid participant key"))?;
            if order <= 0 {
                affected.insert(initiative);
            }
            buckets
                .entry(initiative)
                .or_insert_with(Vec::new)
                .push((order, position, key, index));
        }
        let mut replacements = Vec::new();
        for initiative in affected {
            let entries = buckets
                .get_mut(&initiative)
                .ok_or_else(|| invalid("missing initiative group"))?;
            entries.sort_unstable();
            replacements.extend(
                entries
                    .iter()
                    .enumerate()
                    .map(|(rank, (_, _, _, index))| (*index, rank + 1)),
            );
        }
        for (index, rank) in replacements {
            participants[index]["initiative_order"] = Value::from(rank);
        }
        let encoded =
            serde_json::to_string(&payload).map_err(|_| invalid("invalid repaired payload"))?;
        connection.execute(
            "UPDATE encounter_outgoing_turns SET payload_json=? WHERE encounter_id=?",
            params![encoded, encounter],
        )?;
    }
    Ok(())
}
