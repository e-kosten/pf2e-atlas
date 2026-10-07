//! Contributor fidelity probe: compare typed slice values with raw ordered source projections.
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};

use atlas_ingest::{SourceContext, SourceObject, SourceValue, parse_item_source_slice};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Packet {
    source: String,
    family: String,
    context: Context,
}
#[derive(Deserialize, serde::Serialize)]
struct Context {
    record_key: String,
    source_path: String,
    json_path: String,
}
fn member<'a>(object: &'a SourceObject, name: &str) -> Option<&'a SourceValue> {
    object
        .fields()
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}
fn object(value: &SourceValue) -> &SourceObject {
    let SourceValue::Object(object) = value else {
        panic!("oracle expected object after successful parsing")
    };
    object
}
fn scalar(value: &SourceValue) -> Value {
    match value {
        SourceValue::Null => Value::Null,
        SourceValue::Boolean(value) => json!(value),
        SourceValue::Number(value) => json!(value),
        SourceValue::String(value) => json!(value),
        SourceValue::Array(values) => Value::Array(values.iter().map(scalar).collect()),
        SourceValue::Object(_) => panic!("selected scalar field changed shape"),
    }
}
fn presence(value: Option<&SourceValue>, project: impl FnOnce(&SourceValue) -> Value) -> Value {
    match value {
        None => json!("missing"),
        Some(SourceValue::Null) => json!("null"),
        Some(value) => json!({"value":project(value)}),
    }
}
fn remaining(object: &SourceObject, known: &[&str]) -> Value {
    json!({"fields":object.fields().iter().filter(|(key,_)| !known.contains(&key.as_str())).collect::<Vec<_>>()})
}
fn indexed(
    object: &SourceObject,
    known: &[&str],
    project: impl Fn(&SourceValue) -> Value,
) -> Value {
    json!({"entries":object.fields().iter().filter(|(key,_)| !known.contains(&key.as_str()))
        .map(|(key,value)|json!([key,project(value)])).collect::<Vec<_>>()})
}
fn open(value: &SourceValue) -> Value {
    serde_json::to_value(value).expect("source value serializes")
}
fn unknown_map(value: &SourceValue) -> Value {
    indexed(object(value), &[], open)
}
fn selection_value(value: &SourceValue) -> Value {
    match value {
        SourceValue::String(_) | SourceValue::Number(_) => scalar(value),
        SourceValue::Array(_) | SourceValue::Object(_) => open(value),
        _ => panic!("oracle called only after accepted rule selection"),
    }
}
fn fields(object: &SourceObject, names: &[&str]) -> Value {
    let mut result = serde_json::Map::new();
    for name in names {
        result.insert(snake(name), presence(member(object, name), scalar));
    }
    result.insert("additional_fields".into(), remaining(object, names));
    Value::Object(result)
}
fn snake(name: &str) -> String {
    name.chars()
        .flat_map(|c| {
            if c.is_uppercase() {
                vec!['_', c.to_ascii_lowercase()]
            } else {
                vec![c]
            }
        })
        .collect()
}
fn expected(
    packet: &Packet,
    root_fields: &BTreeMap<String, Vec<String>>,
) -> Result<Value, Box<dyn Error>> {
    let raw: SourceValue = serde_json::from_slice(packet.source.as_bytes())?;
    let envelope = object(&raw);
    let system = object(member(envelope, "system").ok_or("missing system")?);
    let traits = root_fields
        .get(&packet.family)
        .ok_or("missing family root")?
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    Ok(json!({
        "family":packet.family,
        "description":presence(member(system,"description"),|v|fields(object(v), &["gm","value"])),
        "publication":presence(member(system,"publication"),|v|fields(object(v), &["authors","license","remaster","title"])),
        "traits":presence(member(system,"traits"),|v|fields(object(v), &traits)),
        "flags":presence(member(envelope,"flags"),|v|{
            let flags = object(v);
            json!({"pf2e":presence(member(flags,"pf2e"),|v|{
                let pf2e = object(v);
                json!({"item_grants":presence(member(pf2e,"itemGrants"),|v|indexed(object(v), &[], |value|fields(object(value), &["id","nested","onDelete"]))),
                    "granted_by":presence(member(pf2e,"grantedBy"),|v|fields(object(v), &["id","onDelete"])),
                    "rules_selections":presence(member(pf2e,"rulesSelections"),|v|indexed(object(v), &[], selection_value)),
                    "indexed_fields":indexed(pf2e,&["itemGrants","grantedBy","rulesSelections"],open),
                    "additional_fields":{"fields":[]}})
            }),"indexed_fields":indexed(flags,&["pf2e"],unknown_map),"additional_fields":{"fields":[]}})
        }),
        "system_fields":remaining(system,&["description","publication","traits"]),
        "envelope_fields":remaining(envelope,&["type","system","flags"])
    }))
}
fn main() -> Result<(), Box<dyn Error>> {
    let snapshot: Value = serde_json::from_str(include_str!(
        "../../../scripts/source-contracts/snapshots/items/traits.json"
    ))?;
    let root_fields = snapshot["roots"]
        .as_array()
        .ok_or("missing roots")?
        .iter()
        .filter_map(|root| {
            let family = root["family"].as_str()?;
            let fields = root["fields"]
                .as_array()?
                .iter()
                .filter(|field| field["forbidden"] == false)
                .map(|field| field["name"].as_str().unwrap().to_owned())
                .collect();
            Some((family.to_owned(), fields))
        })
        .collect::<BTreeMap<_, _>>();
    let mut records = 0;
    let mut accepted = 0;
    let mut equal = 0;
    let mut roots = 0;
    let mut families = BTreeMap::<String, u64>::new();
    let mut differences = Vec::new();
    let mut difference_count = 0;
    for line in io::stdin().lock().lines() {
        let packet: Packet = serde_json::from_str(&line?)?;
        records += 1;
        if packet.context.json_path == "$" {
            roots += 1;
        }
        *families.entry(packet.family.clone()).or_default() += 1;
        let failure = match parse_item_source_slice(
            SourceContext::new(
                &packet.context.record_key,
                &packet.context.source_path,
                &packet.context.json_path,
            ),
            packet.source.as_bytes(),
        ) {
            Ok(value) => {
                accepted += 1;
                let actual = serde_json::to_value(value)?;
                let wanted = expected(&packet, &root_fields)?;
                if actual == wanted {
                    equal += 1;
                    None
                } else {
                    Some(json!({"context":packet.context,"expected":wanted,"actual":actual}))
                }
            }
            Err(error) => Some(json!({"context":packet.context,"error":error})),
        };
        if let Some(failure) = failure {
            difference_count += 1;
            if differences.len() < 20 {
                differences.push(failure);
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"records":records,"root":roots,"embedded":records-roots,"accepted":accepted,"equalValues":equal,"families":families,"differenceCount":difference_count,"firstDifferences":differences})
        )?
    );
    if difference_count > 0 {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_oracle_covers_complete_flags_and_preserves_open_member_values() {
        for flags in [
            r#"{"pf2e":{"itemGrants":{"x":{"id":"child","onDelete":"detach","nested":null,"future":1}},
                "grantedBy":{"id":"parent","onDelete":"cascade","future":2},
                "rulesSelections":{"number":18446744073709551615,"string":"18446744073709551615","array":[null,1],"object":{"x":1,"x":2}},
                "future":{"x":1,"x":2}},"module":{"null":null,"boolean":false,"payload":[1,{"x":1,"x":2}]}}"#,
            r#"{"pf2e":{"grantedBy":null,"itemGrants":null,"rulesSelections":null,"future":null},"module":{}}"#,
        ] {
            let packet = Packet {
                source: format!(r#"{{"type":"feat","system":{{}},"flags":{flags}}}"#),
                family: "feat".into(),
                context: Context {
                    record_key: "fixture:id".into(),
                    source_path: "fixture.json".into(),
                    json_path: "$".into(),
                },
            };
            let parsed = parse_item_source_slice(
                SourceContext::new("fixture:id", "fixture.json", "$"),
                packet.source.as_bytes(),
            )
            .unwrap();
            let wanted = expected(&packet, &BTreeMap::from([("feat".into(), vec![])])).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), wanted);
        }
    }
}
