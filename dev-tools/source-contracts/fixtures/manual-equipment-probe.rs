//! Validation-only oracle. Compile against PR18's actual atlas-ingest, never the candidate.
use atlas_ingest::item_source::{ItemParentSource, SerializedSourceObject, SerializedSourceValue};
use atlas_ingest::physical_item_source::{
    ItemCarryTypeSource, ItemCoinsSource, ItemEquippedSource, ItemPriceSource,
    PhysicalHitPointsSource, PhysicalUsageSource, parse_physical_item_source,
};
use atlas_ingest::{ActorType, SourceIdentity, SourcePresence, pinned_source_version_metadata};
use serde::Deserialize;
use serde_json::{Value, json};
use std::error::Error;
use std::io::{self, BufRead};

#[derive(Deserialize)]
struct Packet {
    source: String,
    actor: Option<String>,
    ordinal: Option<usize>,
    context: Context,
}
#[derive(Deserialize)]
struct Context {
    record_key: String,
    source_path: String,
}

fn presence<T>(source: &SourcePresence<T>, map: impl FnOnce(&T) -> Value) -> Value {
    match source {
        SourcePresence::Missing => json!("missing"),
        SourcePresence::Null => json!("null"),
        SourcePresence::Value(value) => json!({"value": map(value)}),
    }
}
fn object(source: &SerializedSourceObject) -> Value {
    json!({"fields": source.fields().iter().map(|(k,v)| json!([k, value(v)])).collect::<Vec<_>>()})
}
fn value(source: &SerializedSourceValue) -> Value {
    match source {
        SerializedSourceValue::Null => json!("Null"),
        SerializedSourceValue::Boolean(v) => json!({"Boolean": v}),
        SerializedSourceValue::Number(v) => json!({"Number": v}),
        SerializedSourceValue::String(v) => json!({"String": v}),
        SerializedSourceValue::Array(v) => {
            json!({"Array": v.iter().map(value).collect::<Vec<_>>()})
        }
        SerializedSourceValue::Object(v) => json!({"Object": object(v)}),
    }
}
fn hp(v: &PhysicalHitPointsSource) -> Value {
    json!({"max": presence(&v.max, |v| json!(v)), "value": presence(&v.value, |v| json!(v)),
        "additional_fields": object(&v.additional_fields)})
}
fn coins(v: &ItemCoinsSource) -> Value {
    json!({"cp": presence(&v.cp, |v| json!(v)), "gp": presence(&v.gp, |v| json!(v)),
        "pp": presence(&v.pp, |v| json!(v)), "sp": presence(&v.sp, |v| json!(v)),
        "additional_fields": object(&v.additional_fields)})
}
fn price(v: &ItemPriceSource) -> Value {
    json!({"per": presence(&v.per, |v| json!(v)), "value": presence(&v.value, coins),
        "size_sensitive": presence(&v.size_sensitive, |v| json!(v)),
        "additional_fields": object(&v.additional_fields)})
}
fn equipped(v: &ItemEquippedSource) -> Value {
    json!({"carry_type": presence(&v.carry_type, |v| json!(match v {
        ItemCarryTypeSource::Attached => "attached", ItemCarryTypeSource::Dropped => "dropped",
        ItemCarryTypeSource::Held => "held", ItemCarryTypeSource::Stowed => "stowed",
        ItemCarryTypeSource::Worn => "worn",
    })), "hands_held": presence(&v.hands_held, |v| json!(v)),
        "in_slot": presence(&v.in_slot, |v| json!(v)), "invested": presence(&v.invested, |v| json!(v)),
        "additional_fields": object(&v.additional_fields)})
}
fn usage(v: &PhysicalUsageSource) -> Value {
    json!({"value": presence(&v.value, |v| json!(v)), "additional_fields": object(&v.additional_fields)})
}
// Convert Rust numbers before JavaScript reads the results. Integral 1.0 compares with u8 1.
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
            )
        }
        Value::Array(values) => values.iter_mut().for_each(transport),
        Value::Object(values) => values.values_mut().for_each(transport),
        _ => {}
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    for line in io::stdin().lock().lines() {
        let packet: Packet = serde_json::from_str(&line?)?;
        let parent = match (packet.actor, packet.ordinal) {
            (Some(actor), Some(item_ordinal)) => Some(ItemParentSource {
                actor_type: ActorType::ALL
                    .into_iter()
                    .find(|t| t.as_str() == actor)
                    .ok_or("unknown actor")?,
                item_ordinal,
            }),
            (None, None) => None,
            _ => return Err("inconsistent parent context".into()),
        };
        let identity = SourceIdentity::new(packet.context.record_key, packet.context.source_path);
        let result = match parse_physical_item_source(
            pinned_source_version_metadata(),
            identity,
            parent,
            packet.source.as_bytes(),
        ) {
            Ok(source) => {
                let p = source.source.physical;
                let mut result = json!({"equipped": presence(&p.equipped, equipped),
                    "hp": presence(&p.hp, hp), "price": presence(&p.price, price), "usage": presence(&p.usage, usage)});
                transport(&mut result);
                json!({"ok": true, "value": result})
            }
            Err(error) => json!({"ok": false, "path": error.json_path()}),
        };
        println!("{result}");
    }
    Ok(())
}
