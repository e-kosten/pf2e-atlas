use serde::{Deserialize, Serialize};

use super::parse::SourceDiagnostic;
use super::value::SourceValue;

/// A present field whose authored representation is unavailable to typed consumers.
/// Multiple values retain duplicate structural members without selecting a winner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFieldRejection {
    pub json_path: String,
    pub values: Vec<SourceValue>,
    pub diagnostic: SourceDiagnostic,
}

/// Authored field state before Foundry defaults. Empty, zero and false are values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePresence<T> {
    Missing,
    Null,
    Value(T),
    Invalid(Box<SourceFieldRejection>),
}

impl<T> SourcePresence<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null | Self::Invalid(_) => None,
        }
    }
}
