//! Contributor comparison probe: JSONL source packets on stdin, typed results on stdout.
use std::error::Error;
use std::io::{self, BufRead};

use atlas_ingest::{SourceContext, parse_equipment_source_slice};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Packet {
    source: String,
    context: Context,
}
#[derive(Deserialize)]
struct Context {
    record_key: String,
    source_path: String,
    json_path: String,
}

// Comparison transport keeps large integers out of JavaScript number conversion.
fn transport(value: &mut Value) {
    match value {
        Value::Number(number) => {
            *value = Value::String(
                number
                    .as_u64()
                    .map(|n| n.to_string())
                    .or_else(|| number.as_i64().map(|n| n.to_string()))
                    .or_else(|| number.as_f64().map(|n| n.to_string()))
                    .unwrap_or_else(|| number.to_string()),
            );
        }
        Value::Array(values) => values.iter_mut().for_each(transport),
        Value::Object(values) => values.values_mut().for_each(transport),
        _ => {}
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    for line in io::stdin().lock().lines() {
        let packet: Packet = serde_json::from_str(&line?)?;
        let context = SourceContext::new(
            packet.context.record_key,
            packet.context.source_path,
            packet.context.json_path,
        );
        let result = match parse_equipment_source_slice(context, packet.source.as_bytes()) {
            Ok(source) => {
                let mut value = serde_json::to_value(source.system)?;
                // Comparison is bounded to the selected components and their unknown members.
                value
                    .as_object_mut()
                    .ok_or("source fields must serialize as object")?
                    .remove("additional_fields");
                transport(&mut value);
                json!({ "ok": true, "value": value })
            }
            Err(error) => json!({ "ok": false, "path": error.json_path }),
        };
        println!("{result}");
    }
    Ok(())
}
