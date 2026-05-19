use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackName(String);

impl PackName {
    pub fn new(value: impl Into<String>) -> Result<Self, RecordKeyParseError> {
        let value = value.into();
        validate_part("pack", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for PackName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PackName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordId(String);

impl RecordId {
    pub fn new(value: impl Into<String>) -> Result<Self, RecordKeyParseError> {
        let value = value.into();
        validate_part("record id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for RecordId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RecordId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordKey {
    pack: PackName,
    id: RecordId,
}

impl RecordKey {
    pub fn new(pack: PackName, id: RecordId) -> Self {
        Self { pack, id }
    }

    pub fn parse(value: &str) -> Result<Self, RecordKeyParseError> {
        value.parse()
    }

    pub fn pack(&self) -> &PackName {
        &self.pack
    }

    pub fn id(&self) -> &RecordId {
        &self.id
    }
}

impl fmt::Display for RecordKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.pack, self.id)
    }
}

impl FromStr for RecordKey {
    type Err = RecordKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (pack, id) = value
            .split_once(':')
            .ok_or(RecordKeyParseError::MissingSeparator)?;
        if id.contains(':') {
            return Err(RecordKeyParseError::TooManySeparators);
        }

        Ok(Self {
            pack: PackName::new(pack.to_string())?,
            id: RecordId::new(id.to_string())?,
        })
    }
}

impl Serialize for RecordKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RecordKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordKeyParseError {
    MissingSeparator,
    TooManySeparators,
    EmptyPart { part: &'static str },
    Whitespace { part: &'static str },
}

impl fmt::Display for RecordKeyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => formatter.write_str("record key must use pack:id syntax"),
            Self::TooManySeparators => {
                formatter.write_str("record key must contain exactly one ':' separator")
            }
            Self::EmptyPart { part } => write!(formatter, "record key {part} must not be empty"),
            Self::Whitespace { part } => {
                write!(formatter, "record key {part} must not contain whitespace")
            }
        }
    }
}

impl std::error::Error for RecordKeyParseError {}

fn validate_part(part: &'static str, value: &str) -> Result<(), RecordKeyParseError> {
    if value.is_empty() {
        return Err(RecordKeyParseError::EmptyPart { part });
    }
    if value.chars().any(char::is_whitespace) {
        return Err(RecordKeyParseError::Whitespace { part });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_record_key_into_pack_and_id() {
        let key = RecordKey::parse("spells-srd:abc123").expect("record key should parse");

        assert_eq!(key.pack().as_str(), "spells-srd");
        assert_eq!(key.id().as_str(), "abc123");
        assert_eq!(key.to_string(), "spells-srd:abc123");
    }

    #[test]
    fn rejects_malformed_record_keys() {
        assert_eq!(
            RecordKey::parse("abc").expect_err("missing separator should fail"),
            RecordKeyParseError::MissingSeparator
        );
        assert_eq!(
            RecordKey::parse(":abc").expect_err("empty pack should fail"),
            RecordKeyParseError::EmptyPart { part: "pack" }
        );
        assert_eq!(
            RecordKey::parse("pack:").expect_err("empty id should fail"),
            RecordKeyParseError::EmptyPart { part: "record id" }
        );
        assert_eq!(
            RecordKey::parse("pack:id:extra").expect_err("extra separator should fail"),
            RecordKeyParseError::TooManySeparators
        );
    }

    #[test]
    fn serializes_as_boundary_string() {
        let key = RecordKey::parse("pack:id").expect("record key should parse");
        let json = serde_json::to_string(&key).expect("record key should serialize");
        assert_eq!(json, "\"pack:id\"");

        let decoded: RecordKey =
            serde_json::from_str(&json).expect("record key should deserialize");
        assert_eq!(decoded, key);
    }

    #[test]
    fn serializes_identifier_parts_as_boundary_strings() {
        let pack = PackName::new("pack").expect("pack name should parse");
        let id = RecordId::new("id").expect("record id should parse");

        assert_eq!(
            serde_json::to_string(&pack).expect("pack name should serialize"),
            "\"pack\""
        );
        assert_eq!(
            serde_json::to_string(&id).expect("record id should serialize"),
            "\"id\""
        );
    }
}
