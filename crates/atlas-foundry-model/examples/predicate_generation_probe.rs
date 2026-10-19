//! Contributor value-fidelity probe; upstream discrepancies remain nonzero exits.
#[path = "support/model_inspection.rs"]
mod inspection;

use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead};

use atlas_foundry_model::{
    SourceContext, SourceValue, parse_predicate_input, parse_predicate_statements,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Packet {
    source: String,
    shape: String,
    field: String,
    declaration: String,
    context: Context,
}

#[derive(Deserialize, serde::Serialize)]
struct Context {
    record_key: String,
    source_path: String,
    json_path: String,
}

// Raw value projection; structural union selection is verified separately by fixtures.
fn expected(value: &SourceValue) -> Result<Value, Box<dyn Error>> {
    Ok(match value {
        SourceValue::String(value) => json!(value),
        SourceValue::Number(value) => json!(value),
        SourceValue::Array(values) => {
            Value::Array(values.iter().map(expected).collect::<Result<_, _>>()?)
        }
        SourceValue::Object(object) => {
            let operators = [
                "and", "or", "nand", "nor", "xor", "not", "iff", "eq", "gt", "gte", "lt", "lte",
            ];
            let conditional = ["if", "then"]
                .iter()
                .all(|name| object.fields().iter().any(|(key, _)| key == name));
            let known = if conditional {
                vec!["if", "then"]
            } else {
                operators
                    .into_iter()
                    .filter(|name| object.fields().iter().any(|(key, _)| key == name))
                    .collect()
            };
            let fields = object
                .fields()
                .iter()
                .filter(|(name, _)| known.contains(&name.as_str()))
                .collect::<Vec<_>>();
            let mut result = serde_json::Map::new();
            for (name, value) in fields {
                result.insert(name.clone(), json!({"value":expected(value)?}));
            }
            result.insert("additional_fields".into(),json!({"fields":object.fields().iter().filter(|(name,_)| !known.contains(&name.as_str())).collect::<Vec<_>>()}));
            Value::Object(result)
        }
        _ => return Err("raw projection expected a typed predicate value".into()),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut records = 0;
    let mut accepted = 0;
    let mut equal = 0;
    let mut rejected = 0;
    let mut value_differences = 0;
    let mut fields = BTreeMap::<String, u64>::new();
    let mut shapes = BTreeMap::<String, u64>::new();
    let mut declarations = BTreeMap::<String, u64>::new();
    let mut first_differences = Vec::new();
    for line in io::stdin().lock().lines() {
        let packet: Packet = serde_json::from_str(&line?)?;
        records += 1;
        *fields.entry(packet.field).or_default() += 1;
        *shapes.entry(packet.shape.clone()).or_default() += 1;
        *declarations.entry(packet.declaration).or_default() += 1;
        let context = SourceContext::new(
            &packet.context.record_key,
            &packet.context.source_path,
            &packet.context.json_path,
        );
        let parsed = match packet.shape.as_str() {
            "array" => parse_predicate_statements(context, packet.source.as_bytes())
                .map(inspection::to_value),
            "input" => {
                parse_predicate_input(context, packet.source.as_bytes()).map(inspection::to_value)
            }
            _ => return Err("unknown packet input shape".into()),
        };
        let difference = match parsed {
            Ok(value) => {
                accepted += 1;
                let actual = value?;
                let raw: SourceValue =
                    atlas_foundry_model::parse_source_value(packet.source.as_bytes())?;
                let wanted = expected(&raw)?;
                if actual == wanted {
                    equal += 1;
                    None
                } else {
                    value_differences += 1;
                    Some(json!({"context":packet.context,"expected":wanted,"actual":actual}))
                }
            }
            Err(error) => {
                rejected += 1;
                Some(json!({"context":packet.context,"error":error}))
            }
        };
        if let Some(difference) = difference
            && first_differences.len() < 20
        {
            first_differences.push(difference);
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"records":records,"accepted":accepted,"equalValues":equal,
        "rejected":rejected,"valueDifferences":value_differences,"fields":fields,"shapes":shapes,"declarations":declarations,
        "firstDifferences":first_differences})
        )?
    );
    if rejected > 0 || value_differences > 0 {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::inspection;
    use super::*;
    #[test]
    fn additional_partial_operator_keys_remain_additional_data_in_the_oracle() {
        let source = br#"{"not":"a","then":"b","then":"c"}"#;
        let raw: SourceValue = atlas_foundry_model::parse_source_value(source).unwrap();
        let parsed = atlas_foundry_model::parse_predicate_statement(
            SourceContext::new("test", "test", "$"),
            source,
        )
        .unwrap();
        assert_eq!(
            inspection::to_value(parsed).unwrap(),
            expected(&raw).unwrap()
        );
    }
    #[test]
    fn fidelity_comparison_distinguishes_numbers_from_strings() {
        let raw: SourceValue =
            atlas_foundry_model::parse_source_value(br#"{"eq":["level",18446744073709551615]}"#)
                .unwrap();
        let wanted = expected(&raw).unwrap();
        let mut changed = wanted.clone();
        changed["eq"]["value"][1] = json!("18446744073709551615");
        assert_ne!(changed, wanted);
        assert_eq!(wanted["eq"]["value"][1].as_u64(), Some(u64::MAX));
    }
}
