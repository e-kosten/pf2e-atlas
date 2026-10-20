use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A vocabulary identity, separate from a Foundry document's pack and ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[ts(type = "string")]
pub struct TraitKey(String);

impl TraitKey {
    /// Accept an exact identifier, including authored values outside the pinned catalog.
    pub fn new(identifier: impl Into<String>) -> Result<Self, TraitKeyParseError> {
        let identifier = identifier.into();
        if identifier.is_empty() {
            return Err(TraitKeyParseError::EmptyIdentifier);
        }
        if identifier.contains(':')
            || identifier
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(TraitKeyParseError::InvalidIdentifier);
        }
        Ok(Self(identifier))
    }

    pub fn identifier(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TraitKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "trait:{}", self.0)
    }
}

impl FromStr for TraitKey {
    type Err = TraitKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let identifier = value
            .strip_prefix("trait:")
            .ok_or(TraitKeyParseError::InvalidNamespace)?;
        Self::new(identifier)
    }
}

impl Serialize for TraitKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TraitKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TraitKeyParseError {
    #[error("trait key must use trait:<identifier> syntax")]
    InvalidNamespace,
    #[error("trait identifier must not be empty")]
    EmptyIdentifier,
    #[error("trait identifier must not contain colons, whitespace or control characters")]
    InvalidIdentifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_exact_variants_and_unknown_identifiers() {
        for identifier in ["deadly-d8", "deadly-d12", "versatile-p", "custom.Trait"] {
            let key = TraitKey::new(identifier).unwrap();
            assert_eq!(key.identifier(), identifier);
            assert_eq!(key.to_string().parse::<TraitKey>().unwrap(), key);
            let json = serde_json::to_string(&key).unwrap();
            assert_eq!(json, format!("\"trait:{identifier}\""));
            assert_eq!(serde_json::from_str::<TraitKey>(&json).unwrap(), key);
        }
        assert_ne!(TraitKey::new("deadly-d8"), TraitKey::new("deadly-d12"));
    }

    #[test]
    fn rejects_invalid_identity_and_noncanonical_boundary_values() {
        for identifier in ["", "has space", "a:b", "tab\t", "control\0"] {
            assert!(TraitKey::new(identifier).is_err(), "{identifier:?}");
        }
        for value in ["agile", "Trait:agile", "spell:agile", "trait:", "trait:a:b"] {
            assert!(value.parse::<TraitKey>().is_err(), "{value:?}");
            assert!(serde_json::from_str::<TraitKey>(&format!("\"{value}\"")).is_err());
        }
    }
}
