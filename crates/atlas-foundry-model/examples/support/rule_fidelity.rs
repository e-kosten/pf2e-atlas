//! Developer probe: compare raw source with serialized typed values using graph shapes.
//! This checks preservation, not enum membership, admission, defaults, or Foundry validity.
use crate::source_model::value::SourceValue;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct FidelityGraph(BTreeMap<String, Value>, bool);
impl FidelityGraph {
    pub fn new(graph: &Value) -> Self {
        Self(
            graph["nodes"]
                .as_array()
                .expect("graph nodes")
                .iter()
                .map(|node| (text(node, "id").to_owned(), node.clone()))
                .collect(),
            false,
        )
    }
    #[allow(
        dead_code,
        reason = "Admission is checked by the maintained portfolio probe; strict fixture probes share this oracle."
    )]
    pub fn for_admission(graph: &Value) -> Self {
        let mut graph = Self::new(graph);
        graph.1 = true;
        graph
    }
    pub fn compare(&self, reference: &str, raw: &SourceValue, model: &Value) -> Result<(), String> {
        self.value(reference, raw, model, "$", 0)
    }
    #[allow(
        dead_code,
        reason = "Only the admission probe validates retained fields with embedded source paths."
    )]
    pub fn compare_at(
        &self,
        reference: &str,
        raw: &SourceValue,
        model: &Value,
        path: &str,
    ) -> Result<(), String> {
        self.value(reference, raw, model, path, 0)
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
        // Generated value unions now carry explicit snapshot identity. The
        // independent declaration oracle still compares their authored payload.
        let model = if text(node, "kind") == "union"
            && model.get("$variant").is_some_and(Value::is_string)
        {
            model
                .get("$value")
                .ok_or_else(|| format!("{path}: missing union payload"))?
        } else {
            model
        };
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
                    let key = serialized_key(name);
                    let actual = model
                        .get(&key)
                        .ok_or_else(|| format!("{path}.{name}: modeled field lost"))?;
                    if self.1
                        && let Some(invalid) = actual.get("invalid")
                    {
                        if values.is_empty()
                            || actual.as_object().map(|object| object.len()) != Some(1)
                        {
                            return Err(format!(
                                "{path}.{name}: invalid state without authored field"
                            ));
                        }
                        equal(
                            &json!(values.iter().map(|(_, value)| value).collect::<Vec<_>>()),
                            &invalid["values"],
                            &format!("{path}.{name}: retained invalid values"),
                        )?;
                        equal(
                            &json!(format!("{path}.{name}")),
                            &invalid["json_path"],
                            path,
                        )?;
                        if !invalid["diagnostic"]["expected"].is_string() {
                            return Err(format!("{path}.{name}: missing invalid-field diagnostic"));
                        }
                        continue;
                    }
                    if values.len() > 1 {
                        return Err(format!("{path}.{name}: duplicate modeled source field"));
                    }
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
