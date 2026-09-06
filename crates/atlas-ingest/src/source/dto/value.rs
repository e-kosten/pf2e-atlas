use std::collections::BTreeSet;
use std::fmt;

use atlas_record::UnsupportedSourceShape;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
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
pub(crate) struct SerializedSourceObject {
    fields: Vec<(String, SerializedSourceValue)>,
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

    pub(crate) fn from_json(value: &Value) -> Option<Self> {
        value.as_object().map(Self::from_map)
    }

    pub(crate) fn fields(&self) -> &[(String, SerializedSourceValue)] {
        &self.fields
    }

    pub(crate) fn from_fields(fields: Vec<(String, SerializedSourceValue)>) -> Self {
        Self { fields }
    }

    pub(crate) fn member(&self, key: &str) -> SerializedSourceMember<'_> {
        let values = self
            .fields
            .iter()
            .filter_map(|(candidate, value)| (candidate == key).then_some(value))
            .collect::<Vec<_>>();
        match values.as_slice() {
            [] => SerializedSourceMember::Missing,
            [SerializedSourceValue::Null] => SerializedSourceMember::Null,
            [value] => SerializedSourceMember::Value(value),
            _ => SerializedSourceMember::Duplicate(values),
        }
    }

    pub(crate) fn members(&self, key: &str) -> Vec<&SerializedSourceValue> {
        self.fields
            .iter()
            .filter_map(|(candidate, value)| (candidate == key).then_some(value))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn compact_json(&self) -> String {
        SerializedSourceValue::Object(self.clone()).compact_json()
    }

    #[cfg(test)]
    pub(crate) fn to_legacy_json_rejecting_duplicates(
        &self,
    ) -> Result<Value, LegacyProjectionError> {
        let mut reject = |_: &str, _: &str| LegacyDuplicateDisposition::Reject;
        SerializedSourceValue::Object(self.clone()).to_legacy_json("", &mut reject)
    }

    pub(crate) fn to_legacy_json(
        &self,
        duplicate_policy: &mut impl FnMut(&str, &str) -> LegacyDuplicateDisposition,
    ) -> Result<Value, LegacyProjectionError> {
        SerializedSourceValue::Object(self.clone()).to_legacy_json("", duplicate_policy)
    }

    #[cfg(test)]
    pub(super) fn contains_field(&self, field: &str) -> bool {
        self.fields.iter().any(|(key, _)| key == field)
    }
}

impl std::ops::Deref for SerializedSourceObject {
    type Target = [(String, SerializedSourceValue)];

    fn deref(&self) -> &Self::Target {
        self.fields()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SerializedSourceMember<'a> {
    Missing,
    Null,
    Value(&'a SerializedSourceValue),
    Duplicate(Vec<&'a SerializedSourceValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegacyDuplicateDisposition {
    Reject,
    OmitAfterFamilyRetention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LegacyProjectionError {
    path: String,
    values: String,
}

impl fmt::Display for LegacyProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate source member at {} cannot enter the legacy JSON view: {}",
            self.path, self.values
        )
    }
}

impl std::error::Error for LegacyProjectionError {}

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

    pub(crate) fn object(&self) -> Option<&SerializedSourceObject> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn shape(&self) -> UnsupportedSourceShape {
        match self {
            Self::Null => UnsupportedSourceShape::Null,
            Self::Boolean(_) => UnsupportedSourceShape::Boolean,
            Self::Number(_) => UnsupportedSourceShape::Number,
            Self::String(_) => UnsupportedSourceShape::String,
            Self::Array(_) => UnsupportedSourceShape::Array,
            Self::Object(_) => UnsupportedSourceShape::Object,
        }
    }

    pub(crate) fn compact_json(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::String(value) => Value::String(value.clone()).to_string(),
            Self::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::compact_json)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Object(object) => format!(
                "{{{}}}",
                object
                    .fields
                    .iter()
                    .map(|(key, value)| format!(
                        "{}:{}",
                        Value::String(key.clone()),
                        value.compact_json()
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }

    pub(crate) fn to_legacy_json_rejecting_duplicates(
        &self,
    ) -> Result<Value, LegacyProjectionError> {
        let mut reject = |_: &str, _: &str| LegacyDuplicateDisposition::Reject;
        self.to_legacy_json("", &mut reject)
    }

    fn to_legacy_json(
        &self,
        path: &str,
        duplicate_policy: &mut impl FnMut(&str, &str) -> LegacyDuplicateDisposition,
    ) -> Result<Value, LegacyProjectionError> {
        match self {
            Self::Null => Ok(Value::Null),
            Self::Boolean(value) => Ok(Value::Bool(*value)),
            Self::Number(value) => Ok(Value::Number(value.clone())),
            Self::String(value) => Ok(Value::String(value.clone())),
            Self::Array(values) => values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    value.to_legacy_json(&format!("{path}/{index}"), duplicate_policy)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            Self::Object(object) => {
                let mut seen = BTreeSet::new();
                let mut output = Map::new();
                for (key, _) in &object.fields {
                    if !seen.insert(key.as_str()) {
                        continue;
                    }
                    let values = object.members(key);
                    let selected = if values.len() == 1 {
                        Some(values[0])
                    } else {
                        match duplicate_policy(path, key) {
                            LegacyDuplicateDisposition::OmitAfterFamilyRetention => {
                                for value in &values {
                                    value.to_legacy_json(
                                        &format!("{path}/{}", escape_pointer(key)),
                                        duplicate_policy,
                                    )?;
                                }
                                None
                            }
                            LegacyDuplicateDisposition::Reject => {
                                return Err(LegacyProjectionError {
                                    path: format!("{path}/{}", escape_pointer(key)),
                                    values: format!(
                                        "[{}]",
                                        values
                                            .iter()
                                            .map(|value| value.compact_json())
                                            .collect::<Vec<_>>()
                                            .join(",")
                                    ),
                                });
                            }
                        }
                    };
                    let Some(selected) = selected else {
                        continue;
                    };
                    output.insert(
                        key.clone(),
                        selected.to_legacy_json(
                            &format!("{path}/{}", escape_pointer(key)),
                            duplicate_policy,
                        )?,
                    );
                }
                Ok(Value::Object(output))
            }
        }
    }
}

impl<'de> Deserialize<'de> for SerializedSourceValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SerializedSourceValueVisitor)
    }
}

struct SerializedSourceValueVisitor;

impl<'de> Visitor<'de> for SerializedSourceValueVisitor {
    type Value = SerializedSourceValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::Boolean(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(SerializedSourceValue::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(SerializedSourceValue::String(value))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element()? {
            values.push(value);
        }
        Ok(SerializedSourceValue::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut fields = Vec::new();
        while let Some((key, value)) = map.next_entry()? {
            fields.push((key, value));
        }
        Ok(SerializedSourceValue::Object(SerializedSourceObject {
            fields,
        }))
    }
}

pub(crate) fn parse_serialized_source_object(
    serialized: &[u8],
) -> Result<SerializedSourceObject, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(serialized);
    let root = SerializedSourceValue::deserialize(&mut deserializer)
        .map_err(|error| format!("invalid source JSON: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("source contains trailing JSON: {error}"))?;
    match root {
        SerializedSourceValue::Object(object) => Ok(object),
        other => Err(format!(
            "source document root must be an object, observed {:?}",
            other.shape()
        )),
    }
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

pub(crate) fn serialized_members<'a>(
    object: &'a [(String, SerializedSourceValue)],
    key: &str,
) -> Vec<&'a SerializedSourceValue> {
    object
        .iter()
        .filter_map(|(candidate, value)| (candidate == key).then_some(value))
        .collect()
}

pub(crate) fn serialized_member<'a>(
    object: &'a [(String, SerializedSourceValue)],
    key: &str,
) -> Option<&'a SerializedSourceValue> {
    let values = serialized_members(object, key);
    (values.len() == 1).then_some(values[0])
}

pub(crate) fn serialized_member_order(
    object: &[(String, SerializedSourceValue)],
    key: &str,
) -> Option<u32> {
    let mut matches = object
        .iter()
        .enumerate()
        .filter(|(_, (candidate, _))| candidate == key);
    let (order, _) = matches.next()?;
    matches.next().is_none().then_some(order as u32)
}

pub(crate) fn serialized_unique_string<'a>(
    object: &'a [(String, SerializedSourceValue)],
    key: &str,
) -> Option<&'a str> {
    match serialized_members(object, key).as_slice() {
        [SerializedSourceValue::String(value)] => Some(value),
        _ => None,
    }
}

pub(crate) fn serialized_unique_object<'a>(
    object: &'a [(String, SerializedSourceValue)],
    key: &str,
) -> Option<&'a [(String, SerializedSourceValue)]> {
    match serialized_members(object, key).as_slice() {
        [SerializedSourceValue::Object(value)] => Some(value.fields()),
        _ => None,
    }
}

pub(crate) fn required_serialized_object<'a>(
    value: &'a SerializedSourceValue,
    identity: &SourceIdentity,
    path: &str,
) -> Result<&'a [(String, SerializedSourceValue)], SourceDiagnostic> {
    value
        .object()
        .map(SerializedSourceObject::fields)
        .ok_or_else(|| {
            SourceDiagnostic::new(
                SourceDiagnosticKind::MalformedShape,
                identity,
                path,
                "object",
                format!("{:?}", value.shape()),
            )
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lossless_tree_preserves_nested_members_values_arrays_and_scalar_presence() {
        let source = parse_serialized_source_object(
            br#"{"type":"spell","nested":{"zeta":{"value":1},"alpha":{"value":2},"zeta":{"value":3}},"array":["same","same"],"null":null,"emptyObject":{},"emptyArray":[],"zero":0,"flag":false}"#,
        )
        .expect("lossless source");
        let SerializedSourceMember::Value(SerializedSourceValue::Object(nested)) =
            source.member("nested")
        else {
            panic!("nested object")
        };
        assert_eq!(
            nested
                .fields()
                .iter()
                .map(|(key, value)| (key.as_str(), value.compact_json()))
                .collect::<Vec<_>>(),
            [
                ("zeta", r#"{"value":1}"#.to_string()),
                ("alpha", r#"{"value":2}"#.to_string()),
                ("zeta", r#"{"value":3}"#.to_string()),
            ]
        );
        let SerializedSourceMember::Value(SerializedSourceValue::Array(array)) =
            source.member("array")
        else {
            panic!("repeated array")
        };
        assert_eq!(
            array
                .iter()
                .map(SerializedSourceValue::string)
                .collect::<Vec<_>>(),
            [Some("same"), Some("same")]
        );
        assert!(matches!(
            source.member("missing"),
            SerializedSourceMember::Missing
        ));
        assert!(matches!(
            source.member("null"),
            SerializedSourceMember::Null
        ));
        assert!(matches!(
            source.member("emptyObject"),
            SerializedSourceMember::Value(SerializedSourceValue::Object(value))
                if value.fields().is_empty()
        ));
        assert!(matches!(
            source.member("emptyArray"),
            SerializedSourceMember::Value(SerializedSourceValue::Array(value))
                if value.is_empty()
        ));
        assert!(matches!(
            source.member("zero"),
            SerializedSourceMember::Value(SerializedSourceValue::Number(value))
                if value.as_i64() == Some(0)
        ));
        assert!(matches!(
            source.member("flag"),
            SerializedSourceMember::Value(SerializedSourceValue::Boolean(false))
        ));
        assert_eq!(
            source.compact_json(),
            r#"{"type":"spell","nested":{"zeta":{"value":1},"alpha":{"value":2},"zeta":{"value":3}},"array":["same","same"],"null":null,"emptyObject":{},"emptyArray":[],"zero":0,"flag":false}"#
        );
    }

    #[test]
    fn duplicate_dispatch_discriminator_is_explicit_and_cannot_enter_legacy_json() {
        let source = parse_serialized_source_object(
            br#"{"_id":"one","name":"Duplicate Type","type":"spell","type":"hazard","system":{}}"#,
        )
        .expect("lossless source");
        let SerializedSourceMember::Duplicate(values) = source.member("type") else {
            panic!("duplicate discriminator")
        };
        assert_eq!(
            values
                .iter()
                .map(|value| value.compact_json())
                .collect::<Vec<_>>(),
            [r#""spell""#, r#""hazard""#]
        );
        let error = source
            .to_legacy_json_rejecting_duplicates()
            .expect_err("duplicate discriminator must be rejected");
        assert!(error.to_string().contains("/type"));
    }

    #[test]
    fn compact_json_preserves_json_escaping_and_duplicate_member_order() {
        let source = parse_serialized_source_object(
            br#"{"line\nkey":"quote\" slash\\ control\u0001 snowman \u2603","line\nkey":false}"#,
        )
        .expect("lossless source");

        assert_eq!(
            source.compact_json(),
            r#"{"line\nkey":"quote\" slash\\ control\u0001 snowman ☃","line\nkey":false}"#
        );
    }
}
