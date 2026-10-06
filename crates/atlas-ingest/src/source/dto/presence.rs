use serde::{Deserialize, Serialize};

/// Presence of a serialized source field before Foundry applies defaults.
///
/// `Missing`, `Null`, zero, false, empty strings, and empty collections remain
/// distinct. This is deliberately not represented as `Option<T>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePresence<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> SourcePresence<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }
}
