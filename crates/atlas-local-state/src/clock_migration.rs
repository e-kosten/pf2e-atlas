//! Convert the preceding six-second countdown payload format without losing progress.
use atlas_storage::{StorageError, StorageResult};
use rusqlite::{Connection, params};
use serde_json::{Value, json};

pub fn backfill_encounter_clock(connection: &Connection) -> StorageResult<()> {
    for table in [
        "encounter_manual_trackers",
        "encounter_participant_baselines",
        "encounter_outgoing_turns",
    ] {
        let mut statement =
            connection.prepare(&format!("SELECT rowid,payload_json FROM {table}"))?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, payload) = row?;
            let mut value: Value = serde_json::from_str(&payload).map_err(|e| {
                StorageError::Migration(format!("Invalid saved countdown payload: {e}"))
            })?;
            convert(&mut value)?;
            let payload = serde_json::to_string(&value)
                .map_err(|e| StorageError::Migration(e.to_string()))?;
            connection.execute(
                &format!("UPDATE {table} SET payload_json=?1 WHERE rowid=?2"),
                params![payload, id],
            )?;
        }
    }
    Ok(())
}
fn convert(value: &mut Value) -> StorageResult<()> {
    match value {
        Value::Array(values) => {
            for value in values {
                convert(value)?;
            }
        }
        Value::Object(fields) => {
            if fields.contains_key("tracker_key") {
                let rounds = fields
                    .remove("remaining_rounds")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        StorageError::Migration("Tracker countdown is unavailable".into())
                    })?;
                let unit = fields
                    .get("duration")
                    .and_then(|v| v.get("unit"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        StorageError::Migration("Tracker duration is unavailable".into())
                    })?;
                let progress = match unit {
                    "rounds" => json!({"unit":"rounds","remaining":rounds}),
                    "minutes" | "hours" | "days" => {
                        json!({"unit":"seconds","remaining":seconds(rounds)?})
                    }
                    _ => {
                        return Err(StorageError::Migration(
                            "Unknown tracker duration unit".into(),
                        ));
                    }
                };
                fields.insert("progress".into(), progress);
            } else if fields.contains_key("amount") && fields.contains_key("unit") {
                if let Some(rounds) = fields.remove("remaining_rounds").and_then(|v| v.as_i64()) {
                    fields.insert("remaining_seconds".into(), json!(seconds(rounds)?));
                }
            }
            for value in fields.values_mut() {
                convert(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn seconds(rounds: i64) -> StorageResult<i64> {
    rounds.checked_mul(6).filter(|v| *v >= 0).ok_or_else(|| {
        StorageError::Migration("Saved countdown is outside the supported range".into())
    })
}
