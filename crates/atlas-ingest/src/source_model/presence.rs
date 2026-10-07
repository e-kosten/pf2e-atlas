use serde::Serialize;

/// Authored field state before Foundry defaults. Empty, zero and false are values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePresence<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> SourcePresence<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }
}
