//! Developer probe: compare raw source with serialized typed values using graph shapes.
//! This checks preservation, not enum membership, admission, defaults, or Foundry validity.
use crate::source_model::value::SourceValue;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct FidelityGraph(BTreeMap<String, Value>);
impl FidelityGraph {
    pub fn new(graph: &Value) -> Self {
        Self(
            graph["nodes"]
                .as_array()
                .expect("graph nodes")
                .iter()
                .map(|node| (text(node, "id").to_owned(), node.clone()))
                .collect(),
        )
    }
    pub fn compare(&self, reference: &str, raw: &SourceValue, model: &Value) -> Result<(), String> {
        self.value(reference, raw, model, "$", 0)
    }
    fn value(
        &self,
        reference: &str,
        raw: &SourceValue,
        model: &Value,
        path: &str,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 512 {
            return Err(format!("{path}: fidelity traversal limit"));
        }
        let node = self
            .0
            .get(reference)
            .ok_or_else(|| format!("Missing fidelity node {reference}"))?;
        match text(node, "kind") {
            "open" => equal(&serde_json::to_value(raw).unwrap(), model, path),
            "primitive" | "literal" | "template" => {
                let expected = match raw {
                    SourceValue::Null => Value::Null,
                    SourceValue::Boolean(value) => json!(value),
                    SourceValue::Number(value) => json!(value),
                    SourceValue::String(value) => json!(value),
                    _ => return Err(format!("{path}: scalar representation changed")),
                };
                equal(&expected, model, path)
            }
            "union" => {
                for member in array(node, "members") {
                    if self
                        .value(member.as_str().unwrap(), raw, model, path, depth + 1)
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                Err(format!("{path}: no union representation preserves source"))
            }
            "array" | "tuple" => {
                let SourceValue::Array(raw) = raw else {
                    return Err(format!("{path}: array representation changed"));
                };
                let Some(model) = model.as_array() else {
                    return Err(format!("{path}: array representation changed"));
                };
                if raw.len() != model.len() {
                    return Err(format!("{path}: array length changed"));
                }
                if text(node, "kind") == "tuple" && raw.len() != array(node, "elements").len() {
                    return Err(format!("{path}: tuple position count changed"));
                }
                for (index, (raw, model)) in raw.iter().zip(model).enumerate() {
                    let reference = if text(node, "kind") == "array" {
                        text(node, "element")
                    } else {
                        text(&array(node, "elements")[index], "ref")
                    };
                    self.value(
                        reference,
                        raw,
                        model,
                        &format!("{path}[{index}]"),
                        depth + 1,
                    )?;
                }
                Ok(())
            }
            "object" | "intersection" => {
                let SourceValue::Object(raw) = raw else {
                    return Err(format!("{path}: object representation changed"));
                };
                let fields = array(node, "fields");
                let indices = array(node, "indexSignatures");
                if fields.is_empty()
                    && indices
                        .first()
                        .is_some_and(|index| index["key"] == "primitive:string")
                {
                    return self.entries(
                        text(&indices[0], "value"),
                        raw.fields(),
                        model,
                        path,
                        depth,
                    );
                }
                let mut retained = Vec::new();
                let mut indexed = Vec::new();
                let modeled: Vec<_> = fields
                    .iter()
                    .filter(|field| field["forbidden"] != true)
                    .collect();
                for field in &modeled {
                    let name = text(field, "name");
                    let values: Vec<_> =
                        raw.fields().iter().filter(|(key, _)| key == name).collect();
                    if values.len() > 1 {
                        return Err(format!("{path}.{name}: duplicate modeled source field"));
                    }
                    let key = serialized_key(name);
                    let actual = model
                        .get(&key)
                        .ok_or_else(|| format!("{path}.{name}: modeled field lost"))?;
                    match values.first().map(|(_, value)| value) {
                        None => equal(&json!("missing"), actual, &format!("{path}.{name}"))?,
                        Some(SourceValue::Null) => {
                            equal(&json!("null"), actual, &format!("{path}.{name}"))?
                        }
                        Some(raw) => {
                            if actual.as_object().map(|object| object.len()) != Some(1)
                                || actual.get("value").is_none()
                            {
                                return Err(format!("{path}.{name}: presence state changed"));
                            }
                            self.value(
                                text(field, "ref"),
                                raw,
                                &actual["value"],
                                &format!("{path}.{name}"),
                                depth + 1,
                            )?;
                        }
                    }
                }
                for (key, value) in raw.fields() {
                    if modeled.iter().any(|field| text(field, "name") == key) {
                        continue;
                    }
                    if indices.first().is_some_and(|index| {
                        index["key"] == "primitive:string"
                            || index["key"] == "primitive:number"
                                && crate::source_model::parse::is_numeric_key(key)
                    }) && !fields.iter().any(|field| text(field, "name") == key)
                    {
                        indexed.push((key.clone(), value.clone()));
                    } else {
                        retained.push((key.clone(), value.clone()));
                    }
                }
                let expected = json!({ "fields": retained });
                equal(
                    &expected,
                    &model["additional_fields"],
                    &format!("{path}.additional_fields"),
                )?;
                if !indices.is_empty() {
                    self.entries(
                        text(&indices[0], "value"),
                        &indexed,
                        &model["indexed_fields"],
                        path,
                        depth,
                    )?;
                }
                let count = modeled.len() + 1 + usize::from(!indices.is_empty());
                if model.as_object().map(|object| object.len()) != Some(count) {
                    return Err(format!("{path}: unexpected model fields"));
                }
                Ok(())
            }
            kind => Err(format!("{path}: unsupported fidelity shape {kind}")),
        }
    }
    fn entries(
        &self,
        reference: &str,
        raw: &[(String, SourceValue)],
        model: &Value,
        path: &str,
        depth: usize,
    ) -> Result<(), String> {
        let Some(entries) = model["entries"].as_array() else {
            return Err(format!("{path}: missing map entries"));
        };
        if entries.len() != raw.len() || model.as_object().map(|object| object.len()) != Some(1) {
            return Err(format!("{path}: map entries changed"));
        }
        for ((key, raw), actual) in raw.iter().zip(entries) {
            if actual.as_array().map(|pair| pair.len()) != Some(2)
                || actual[0].as_str() != Some(key)
            {
                return Err(format!("{path}: map key/order changed"));
            }
            self.value(
                reference,
                raw,
                &actual[1],
                &format!("{path}[{key:?}]"),
                depth + 1,
            )?;
        }
        Ok(())
    }
}
fn equal(expected: &Value, actual: &Value, path: &str) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!("{path}: value or presence changed"))
    }
}
fn text<'a>(node: &'a Value, key: &str) -> &'a str {
    node[key].as_str().expect("graph string")
}
fn array<'a>(node: &'a Value, key: &str) -> &'a [Value] {
    node[key].as_array().expect("graph array")
}
// Public serialization convention: camelCase becomes snake_case; punctuation,
// numeric-leading and reserved path names retain their original source spelling.
fn serialized_key(source: &str) -> String {
    let mut snake = String::new();
    let mut previous = None;
    for character in source.chars() {
        if character.is_ascii_uppercase()
            && previous
                .is_some_and(|value: char| value.is_ascii_lowercase() || value.is_ascii_digit())
        {
            snake.push('_');
        }
        snake.push(character.to_ascii_lowercase());
        previous = Some(character);
    }
    if snake.starts_with(|character: char| character.is_ascii_digit())
        || ["self", "super", "crate"].contains(&snake.as_str())
        || snake.chars().any(|character| {
            !character.is_ascii_lowercase() && !character.is_ascii_digit() && character != '_'
        })
    {
        source.into()
    } else {
        snake
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_model::{generated, parse::SourceContext, value::parse_source};

    #[test]
    fn generated_authored_inputs_keep_scalar_array_presence_and_strict_array_boundaries() {
        let context = SourceContext::new("fixture", "fixture.json", "$");
        for (source, selector, selectors) in [
            (
                r#"{"selector":"attack","selectors":["strict"]}"#,
                json!({"value":"attack"}),
                json!({"value":["strict"]}),
            ),
            (
                r#"{"selector":["attack","damage"]}"#,
                json!({"value":["attack","damage"]}),
                json!("missing"),
            ),
            (r#"{"selector":[]}"#, json!({"value":[]}), json!("missing")),
            (r#"{"selector":null}"#, json!("null"), json!("missing")),
            (r#"{}"#, json!("missing"), json!("missing")),
        ] {
            let raw = parse_source(source.as_bytes()).unwrap();
            let model = serde_json::to_value(
                generated::parse_authored_arrays(&raw, &context, "$").unwrap(),
            )
            .unwrap();
            assert_eq!(model["selector"], selector);
            assert_eq!(model["selectors"], selectors);
        }
        for source in [
            r#"{"selectors":"strict"}"#,
            r#"{"selector":5}"#,
            r#"{"selector":["attack",5]}"#,
            r#"{"selector":"a","selector":"b"}"#,
        ] {
            let raw = parse_source(source.as_bytes()).unwrap();
            assert!(generated::parse_authored_arrays(&raw, &context, "$").is_err());
        }
    }

    #[test]
    fn generated_nested_iwr_keeps_authored_forms_and_strict_exceptions() {
        let context = SourceContext::new("fixture", "fixture.json", "$");
        for (source, expected) in [
            (
                r#"{"immunities":[{"type":"fire"}]}"#,
                json!({"value":"fire"}),
            ),
            (
                r#"{"immunities":[{"type":["fire","cold"]}]}"#,
                json!({"value":["fire","cold"]}),
            ),
            (r#"{"immunities":[{"type":[]}]}"#, json!({"value":[]})),
            (r#"{"immunities":[{"type":null}]}"#, json!("null")),
            (r#"{"immunities":[{}]}"#, json!("missing")),
        ] {
            let raw = parse_source(source.as_bytes()).unwrap();
            let model =
                serde_json::to_value(generated::parse_authored_form(&raw, &context, "$").unwrap())
                    .unwrap();
            assert_eq!(model["immunities"]["value"][0]["type"], expected);
        }
        for source in [
            r#"{"immunities":[{"type":3}]}"#,
            r#"{"immunities":[{"type":["fire",3]}]}"#,
            r#"{"immunities":[{"type":"fire","exceptions":"cold"}]}"#,
        ] {
            let raw = parse_source(source.as_bytes()).unwrap();
            assert!(generated::parse_authored_form(&raw, &context, "$").is_err());
        }
    }

    #[test]
    fn fidelity_detects_omission_presence_coercion_number_loss_and_additional_order() {
        let graph = FidelityGraph::new(&json!({"nodes":[
            {"id":"string","kind":"primitive","value":"string"},
            {"id":"number","kind":"primitive","value":"number"},
            {"id":"strings","kind":"array","element":"string"},
            {"id":"selector","kind":"union","members":["string","strings"]},
            {"id":"rule","kind":"object","fields":[
                {"name":"selector","ref":"selector","forbidden":false},
                {"name":"diceNumber","ref":"number","forbidden":false},
                {"name":"nullable","ref":"string","forbidden":false},
                {"name":"absent","ref":"string","forbidden":false}],"indexSignatures":[]}
        ]}));
        let raw = parse_source(br#"{"selector":"attack","diceNumber":9007199254740993,"nullable":null,"future":1,"future":2}"#).unwrap();
        let expected = json!({"selector":{"value":"attack"},"dice_number":{"value":9007199254740993u64},
            "nullable":"null","absent":"missing","additional_fields":{"fields":[["future",{"Number":1}],["future",{"Number":2}]]}});
        assert!(graph.compare("rule", &raw, &expected).is_ok());
        let mut lost = expected.clone();
        lost.as_object_mut().unwrap().remove("selector");
        assert!(graph.compare("rule", &raw, &lost).is_err());
        for (key, replacement) in [
            ("selector", json!({"value":["attack"]})),
            ("dice_number", json!({"value":9007199254740992u64})),
            ("nullable", json!("missing")),
            ("absent", json!("null")),
            (
                "additional_fields",
                json!({"fields":[["future",{"Number":2}],["future",{"Number":1}]]}),
            ),
            (
                "additional_fields",
                json!({"fields":[["future",{"Number":2}]]}),
            ),
        ] {
            let mut changed = expected.clone();
            changed[key] = replacement;
            assert!(graph.compare("rule", &raw, &changed).is_err(), "{key}");
        }
    }

    #[test]
    fn fidelity_checks_actual_generated_values_and_source_key_mapping() {
        let graph = FidelityGraph::new(&json!({"nodes":[
            {"id":"string","kind":"primitive","value":"string"},
            {"id":"number","kind":"primitive","value":"number"},
            {"id":"mapped","kind":"object","fields":[
                {"name":"_id","ref":"string","forbidden":false},
                {"name":"greater-darkvision","ref":"number","forbidden":false},
                {"name":"self","ref":"string","forbidden":false},
                {"name":"1st","ref":"string","forbidden":false}],"indexSignatures":[]}
        ]}));
        let raw = parse_source(
            br#"{"_id":"x","greater-darkvision":60,"self":"a","1st":null,"extra":{"a":1,"a":2}}"#,
        )
        .unwrap();
        let context = SourceContext::new("fixture", "fixture.json", "$");
        let parsed = generated::parse_mapped_fields(&raw, &context, "$").unwrap();
        let actual = serde_json::to_value(parsed).unwrap();
        assert!(graph.compare("mapped", &raw, &actual).is_ok());
    }

    #[test]
    fn fidelity_preserves_open_values_and_typed_map_order() {
        let graph = FidelityGraph::new(&json!({"nodes":[
            {"id":"open","kind":"open","domain":"unknown"},
            {"id":"map","kind":"object","fields":[],"indexSignatures":[{"key":"primitive:string","value":"open"}]}
        ]}));
        let raw = parse_source(br#"{"z":null,"a":{"x":1,"x":2}}"#).unwrap();
        let context = SourceContext::new("fixture", "fixture.json", "$");
        let actual =
            serde_json::to_value(generated::parse_unknown_map(&raw, &context, "$").unwrap())
                .unwrap();
        assert!(graph.compare("map", &raw, &actual).is_ok());
        let mut reordered = actual;
        reordered["entries"].as_array_mut().unwrap().reverse();
        assert!(graph.compare("map", &raw, &reordered).is_err());
    }

    #[test]
    fn fidelity_reports_tuple_shape_loss_without_panicking() {
        let graph = FidelityGraph::new(&json!({"nodes":[
            {"id":"string","kind":"primitive","value":"string"},
            {"id":"tuple","kind":"tuple","elements":[{"ref":"string"}]}
        ]}));
        let raw = parse_source(br#"["one","extra"]"#).unwrap();
        assert!(
            graph
                .compare("tuple", &raw, &json!(["one", "extra"]))
                .is_err()
        );
    }
}
