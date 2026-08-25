use std::collections::BTreeMap;

use serde_json::Value;
use serde_json::{Map, Number};

use super::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity, SourcePresence};

/// Complete serialized JSON tree retained by the typed source envelope.
///
/// This is intentionally not a semantic product model. Later source DTO slices
/// promote approved fields into explicit structs instead of querying this tree
/// through fallback JSON pointers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SerializedSourceValue {
    Null,
    Boolean(bool),
    Number(Number),
    String(String),
    Array(Vec<SerializedSourceValue>),
    Object(SerializedSourceObject),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SerializedSourceObject {
    fields: BTreeMap<String, SerializedSourceValue>,
}

impl SerializedSourceObject {
    fn from_map(map: &Map<String, Value>) -> Self {
        Self {
            fields: map
                .iter()
                .map(|(key, value)| (key.clone(), SerializedSourceValue::from_json(value)))
                .collect(),
        }
    }

    #[cfg(test)]
    pub(super) fn contains_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }
}

impl SerializedSourceValue {
    fn from_json(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Boolean(*value),
            Value::Number(value) => Self::Number(value.clone()),
            Value::String(value) => Self::String(value.clone()),
            Value::Array(values) => Self::Array(
                values
                    .iter()
                    .map(SerializedSourceValue::from_json)
                    .collect(),
            ),
            Value::Object(map) => Self::Object(SerializedSourceObject::from_map(map)),
        }
    }
}

/// Original parsed JSON retained solely for provenance and deliberate audits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSourceJson(Value);

impl RawSourceJson {
    pub(crate) fn new(value: Value) -> Self {
        Self(value)
    }

    pub(crate) fn for_audit(&self) -> &Value {
        &self.0
    }
}

pub(crate) fn required_object<'a>(
    map: &'a Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<&'a Map<String, Value>, SourceDiagnostic> {
    let Some(value) = map.get(key) else {
        return Err(SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            "object",
            "missing",
        ));
    };
    value.as_object().ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            "object",
            actual_shape(value),
        )
    })
}

pub(crate) fn required_string(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<String, SourceDiagnostic> {
    let Some(value) = map.get(key) else {
        return Err(SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            "string",
            "missing",
        ));
    };
    value.as_str().map(str::to_owned).ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            "string",
            actual_shape(value),
        )
    })
}

pub(crate) fn optional_string(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<SourcePresence<String>, SourceDiagnostic> {
    optional_member(map, key, identity, json_path, "string", |value| {
        value.as_str().map(str::to_owned)
    })
}

pub(crate) fn optional_integer(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<SourcePresence<i64>, SourceDiagnostic> {
    optional_member(map, key, identity, json_path, "integer", Value::as_i64)
}

pub(crate) fn optional_object(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<SourcePresence<SerializedSourceObject>, SourceDiagnostic> {
    optional_member(map, key, identity, json_path, "object", |value| {
        value.as_object().map(SerializedSourceObject::from_map)
    })
}

pub(crate) fn optional_array_of_objects(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
) -> Result<SourcePresence<Vec<SerializedSourceObject>>, SourceDiagnostic> {
    optional_member(map, key, identity, json_path, "array of objects", |value| {
        value.as_array().and_then(|values| {
            values
                .iter()
                .map(|value| value.as_object().map(SerializedSourceObject::from_map))
                .collect()
        })
    })
}

fn optional_member<T>(
    map: &Map<String, Value>,
    key: &str,
    identity: &SourceIdentity,
    json_path: &str,
    expected_shape: &str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Result<SourcePresence<T>, SourceDiagnostic> {
    let Some(value) = map.get(key) else {
        return Ok(SourcePresence::Missing);
    };
    if value.is_null() {
        return Ok(SourcePresence::Null);
    }
    parse(value).map(SourcePresence::Value).ok_or_else(|| {
        SourceDiagnostic::new(
            SourceDiagnosticKind::MalformedShape,
            identity,
            json_path,
            expected_shape,
            actual_shape(value),
        )
    })
}

pub(crate) fn actual_shape(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub(crate) fn serialized_object(map: &Map<String, Value>) -> SerializedSourceObject {
    SerializedSourceObject::from_map(map)
}
