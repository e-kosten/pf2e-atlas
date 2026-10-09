//! Ordered JSON members, recovered from the earlier source DTO reader.
use std::fmt;

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceValue {
    Null,
    Boolean(bool),
    Number(Number),
    String(String),
    Array(Vec<SourceValue>),
    Object(SourceObject),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObject {
    pub(crate) fields: Vec<(String, SourceValue)>,
}

impl SourceObject {
    /// Authored order and repeated names are retained, including unknown fields.
    pub fn fields(&self) -> &[(String, SourceValue)] {
        &self.fields
    }
}

// Authored JSON is a distinct input format from the tagged typed snapshot.
struct AuthoredValue(SourceValue);

impl<'de> Deserialize<'de> for AuthoredValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SourceValueVisitor).map(Self)
    }
}

struct SourceValueVisitor;

impl<'de> Visitor<'de> for SourceValueVisitor {
    type Value = SourceValue;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(SourceValue::Null)
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(SourceValue::Boolean(value))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(SourceValue::Number(value.into()))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(SourceValue::Number(value.into()))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(SourceValue::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(SourceValue::String(value.to_string()))
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(SourceValue::String(value))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(AuthoredValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(SourceValue::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut fields = Vec::new();
        while let Some((key, AuthoredValue(value))) = map.next_entry()? {
            fields.push((key, value));
        }
        Ok(SourceValue::Object(SourceObject { fields }))
    }
}

pub fn parse_source(bytes: &[u8]) -> Result<SourceValue, serde_json::Error> {
    let mut reader = serde_json::Deserializer::from_slice(bytes);
    let AuthoredValue(value) = AuthoredValue::deserialize(&mut reader)?;
    reader.end()?;
    Ok(value)
}
