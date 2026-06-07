use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId(String);

impl TagId {
    pub fn parse(value: impl Into<String>) -> Result<Self, TagIdParseError> {
        let value = value.into();
        validate_tag_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TagId {
    type Err = TagIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for TagId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TagId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid tag id `{value}`: {reason}")]
pub struct TagIdParseError {
    value: String,
    reason: &'static str,
}

fn validate_tag_id(value: &str) -> Result<(), TagIdParseError> {
    if value.is_empty() {
        return Err(invalid(value, "tag id must not be empty"));
    }
    let segments = value.split('.').collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err(invalid(value, "tag id must contain a dotted namespace"));
    }
    for segment in segments {
        validate_segment(value, segment)?;
    }
    Ok(())
}

fn validate_segment(value: &str, segment: &str) -> Result<(), TagIdParseError> {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return Err(invalid(value, "tag id segments must not be empty"));
    };
    if !first.is_ascii_lowercase() {
        return Err(invalid(
            value,
            "tag id segments must start with a lowercase ascii letter",
        ));
    }
    if chars.any(|character| {
        !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
    }) {
        return Err(invalid(
            value,
            "tag id segments may contain only lowercase ascii letters, digits, and underscores",
        ));
    }
    Ok(())
}

fn invalid(value: &str, reason: &'static str) -> TagIdParseError {
    TagIdParseError {
        value: value.to_string(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_dotted_lowercase_ids() {
        let id = TagId::parse("problem.counteract_magic").expect("tag id parses");
        assert_eq!(id.as_str(), "problem.counteract_magic");
    }

    #[test]
    fn rejects_ids_without_namespace_or_with_invalid_case() {
        assert!(TagId::parse("counteract_magic").is_err());
        assert!(TagId::parse("Problem.counteract_magic").is_err());
        assert!(TagId::parse("problem.counteract-magic").is_err());
    }
}
